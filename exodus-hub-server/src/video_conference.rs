//! Video Conference Module
//!
//! Management for video conferencing rooms and participants.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, debug};
use uuid::Uuid;

/// Video conference room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConferenceRoom {
    pub room_id: String,
    pub name: String,
    pub host_id: String,
    pub max_participants: u32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: String, // pending, active, ended
}

/// Room participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub participant_id: String,
    pub room_id: String,
    pub user_id: String,
    pub display_name: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub role: String, // host, participant
    pub audio_enabled: bool,
    pub video_enabled: bool,
}

/// Video conference service
pub struct VideoConferenceService {
    conn: Connection,
}

impl VideoConferenceService {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)
            .context("Failed to open video conference database")?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conference_rooms (
                room_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host_id TEXT NOT NULL,
                max_participants INTEGER NOT NULL DEFAULT 10,
                created_at TEXT NOT NULL,
                started_at TEXT,
                ended_at TEXT,
                status TEXT NOT NULL DEFAULT 'pending'
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS room_participants (
                participant_id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                left_at TEXT,
                role TEXT NOT NULL DEFAULT 'participant',
                audio_enabled INTEGER NOT NULL DEFAULT 1,
                video_enabled INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (room_id) REFERENCES conference_rooms(room_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS room_invites (
                invite_id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL,
                invited_by TEXT NOT NULL,
                invitee_id TEXT,
                invite_code TEXT,
                expires_at TEXT,
                used INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (room_id) REFERENCES conference_rooms(room_id)
            )",
            [],
        )?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_conference_rooms_host_id ON conference_rooms(host_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_conference_rooms_status ON conference_rooms(status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_room_participants_room_id ON room_participants(room_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_room_participants_user_id ON room_participants(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_room_invites_room_id ON room_invites(room_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_room_invites_code ON room_invites(invite_code)", [])?;

        info!("Video conference service initialized");

        Ok(Self { conn })
    }

    /// Create a new conference room
    pub fn create_room(&mut self, name: &str, host_id: &str, max_participants: u32) -> Result<ConferenceRoom> {
        debug!("Creating conference room: name={}, host_id={}, max_participants={}", name, host_id, max_participants);
        let room_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO conference_rooms (room_id, name, host_id, max_participants, created_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![room_id, name, host_id, max_participants, now, "pending"],
        )?;

        info!("Conference room created: {}", room_id);
        Ok(ConferenceRoom {
            room_id: room_id.clone(),
            name: name.to_string(),
            host_id: host_id.to_string(),
            max_participants,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            status: "pending".to_string(),
        })
    }

    /// Start a conference room
    pub fn start_room(&mut self, room_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conference_rooms SET started_at = ?1, status = 'active' WHERE room_id = ?2",
            params![now, room_id],
        )?;
        Ok(())
    }

    /// End a conference room
    pub fn end_room(&mut self, room_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conference_rooms SET ended_at = ?1, status = 'ended' WHERE room_id = ?2",
            params![now, room_id],
        )?;
        Ok(())
    }

    /// Get room by ID
    pub fn get_room(&self, room_id: &str) -> Result<Option<ConferenceRoom>> {
        let mut stmt = self.conn.prepare(
            "SELECT room_id, name, host_id, max_participants, created_at, started_at, ended_at, status
             FROM conference_rooms WHERE room_id = ?1"
        )?;

        let room = stmt.query_row(params![room_id], |row| {
            let created_at_str: String = row.get(4)?;
            let started_at_str: Option<String> = row.get(5)?;
            let ended_at_str: Option<String> = row.get(6)?;
            Ok(ConferenceRoom {
                room_id: row.get(0)?,
                name: row.get(1)?,
                host_id: row.get(2)?,
                max_participants: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                started_at: started_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()),
                ended_at: ended_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()),
                status: row.get(7)?,
            })
        }).ok();

        Ok(room)
    }

    /// Get rooms for a user
    pub fn get_user_rooms(&self, user_id: &str) -> Result<Vec<ConferenceRoom>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.room_id, r.name, r.host_id, r.max_participants, r.created_at, r.started_at, r.ended_at, r.status
             FROM conference_rooms r
             WHERE r.host_id = ?1 OR r.room_id IN (SELECT room_id FROM room_participants WHERE user_id = ?1)
             ORDER BY r.created_at DESC"
        )?;

        let rooms = stmt.query_map(params![user_id, user_id], |row| {
            let created_at_str: String = row.get(4)?;
            let started_at_str: Option<String> = row.get(5)?;
            let ended_at_str: Option<String> = row.get(6)?;
            Ok(ConferenceRoom {
                room_id: row.get(0)?,
                name: row.get(1)?,
                host_id: row.get(2)?,
                max_participants: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                started_at: started_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()),
                ended_at: ended_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()),
                status: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(rooms)
    }

    /// Join a room
    pub fn join_room(&mut self, room_id: &str, user_id: &str, display_name: &str, role: &str) -> Result<Participant> {
        debug!("User {} joining room {} as {}", user_id, room_id, role);
        let participant_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO room_participants (participant_id, room_id, user_id, display_name, joined_at, role)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![participant_id, room_id, user_id, display_name, now, role],
        )?;

        info!("Participant joined: {} in room {}", participant_id, room_id);
        Ok(Participant {
            participant_id: participant_id.clone(),
            room_id: room_id.to_string(),
            user_id: user_id.to_string(),
            display_name: display_name.to_string(),
            joined_at: Utc::now(),
            left_at: None,
            role: role.to_string(),
            audio_enabled: true,
            video_enabled: true,
        })
    }

    /// Leave a room
    pub fn leave_room(&mut self, room_id: &str, user_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE room_participants SET left_at = ?1 WHERE room_id = ?2 AND user_id = ?3",
            params![now, room_id, user_id],
        )?;
        Ok(())
    }

    /// Get participants in a room
    pub fn get_participants(&self, room_id: &str) -> Result<Vec<Participant>> {
        let mut stmt = self.conn.prepare(
            "SELECT participant_id, room_id, user_id, display_name, joined_at, left_at, role, audio_enabled, video_enabled
             FROM room_participants
             WHERE room_id = ?1 AND left_at IS NULL
             ORDER BY joined_at ASC"
        )?;

        let participants = stmt.query_map(params![room_id], |row| {
            let joined_at_str: String = row.get(4)?;
            let left_at_str: Option<String> = row.get(5)?;
            Ok(Participant {
                participant_id: row.get(0)?,
                room_id: row.get(1)?,
                user_id: row.get(2)?,
                display_name: row.get(3)?,
                joined_at: DateTime::parse_from_rfc3339(&joined_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                left_at: left_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()),
                role: row.get(6)?,
                audio_enabled: row.get::<_, i32>(7)? == 1,
                video_enabled: row.get::<_, i32>(8)? == 1,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(participants)
    }

    /// Update participant media state
    pub fn update_media_state(&mut self, participant_id: &str, audio_enabled: bool, video_enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE room_participants SET audio_enabled = ?1, video_enabled = ?2 WHERE participant_id = ?3",
            params![if audio_enabled { 1 } else { 0 }, if video_enabled { 1 } else { 0 }, participant_id],
        )?;
        Ok(())
    }

    /// Create invite for a room
    pub fn create_invite(&mut self, room_id: &str, invited_by: &str, invitee_id: Option<&str>, expires_hours: u32) -> Result<String> {
        let invite_id = Uuid::new_v4().to_string();
        let invite_code: String = (0..8).map(|_| rand::random::<char>().to_ascii_uppercase()).collect();
        let now = Utc::now().to_rfc3339();
        let expires_at = (Utc::now() + chrono::Duration::hours(expires_hours as i64)).to_rfc3339();

        self.conn.execute(
            "INSERT INTO room_invites (invite_id, room_id, invited_by, invitee_id, invite_code, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![invite_id, room_id, invited_by, invitee_id, invite_code, expires_at, now],
        )?;

        Ok(invite_code)
    }

    /// Validate invite code
    pub fn validate_invite(&self, invite_code: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT room_id, expires_at, used FROM room_invites WHERE invite_code = ?1"
        )?;

        let result = stmt.query_row(params![invite_code], |row| {
            let room_id: String = row.get(0)?;
            let expires_at: String = row.get(1)?;
            let used: i32 = row.get(2)?;

            if used == 1 {
                Ok(None)
            } else if DateTime::parse_from_rfc3339(&expires_at).map(|dt| dt.with_timezone(&Utc)).unwrap_or_default() < Utc::now() {
                Ok(None)
            } else {
                Ok(Some(room_id))
            }
        }).ok();

        Ok(result.flatten())
    }

    /// Mark invite as used
    pub fn mark_invite_used(&mut self, invite_code: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE room_invites SET used = 1 WHERE invite_code = ?1",
            params![invite_code],
        )?;
        Ok(())
    }
}
