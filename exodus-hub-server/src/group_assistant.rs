//! Group Assistant Module
//!
//! Cloud-based group assistant for managing group messages, auto-replies, and message receipts.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, debug};
use uuid::Uuid;

/// Group assistant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAssistantConfig {
    pub db_path: PathBuf,
}

impl Default for GroupAssistantConfig {
    fn default() -> Self {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("group_assistant.db");
        Self { db_path }
    }
}

/// Group assistant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAssistant {
    pub assistant_id: String,
    pub group_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

/// Assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub message_id: String,
    pub group_id: String,
    pub sender_id: String,
    pub content: String,
    pub message_type: String,
    pub sequence: u32,
    pub timestamp: DateTime<Utc>,
    pub integrity_hash: String,
}

/// Group assistant service
pub struct GroupAssistantService {
    conn: Connection,
}

impl GroupAssistantService {
    pub fn new(config: GroupAssistantConfig) -> Result<Self> {
        let conn = Connection::open(&config.db_path)
            .context("Failed to open group assistant database")?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS assistants (
                assistant_id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_active TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS assistant_messages (
                message_id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                integrity_hash TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS assistant_sequences (
                group_id TEXT PRIMARY KEY,
                last_sequence INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS assistant_receipts (
                receipt_id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                receiver_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                received_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_assistant_messages_group_id ON assistant_messages(group_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_assistant_messages_sequence ON assistant_messages(sequence)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_assistant_receipts_message_id ON assistant_receipts(message_id)", [])?;

        info!("Group assistant service initialized");

        Ok(Self { conn })
    }

    /// Create or get group assistant for a group
    pub fn create_or_get_assistant(&mut self, group_id: &str, name: &str) -> Result<GroupAssistant> {
        debug!("Creating or getting assistant for group: {}", group_id);
        let now = Utc::now().to_rfc3339();

        // Try to get existing assistant
        let assistant = {
            let mut stmt = self.conn.prepare("SELECT assistant_id, group_id, name, created_at, last_active FROM assistants WHERE group_id = ?1")?;
            let result = stmt.query_row(params![group_id], |row| {
                let created_at_str: String = row.get(3)?;
                let last_active_str: String = row.get(4)?;
                Ok(GroupAssistant {
                    assistant_id: row.get(0)?,
                    group_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                    last_active: DateTime::parse_from_rfc3339(&last_active_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                })
            }).ok();
            drop(stmt);
            result
        };

        if let Some(assistant) = assistant {
            // Update last active time
            self.conn.execute(
                "UPDATE assistants SET last_active = ?1 WHERE group_id = ?2",
                params![now, group_id],
            )?;
            debug!("Updated last active time for existing assistant: {}", assistant.assistant_id);
            return Ok(assistant);
        }

        // Create new assistant - use transaction
        let tx = self.conn.transaction()?;
        let assistant_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO assistants (assistant_id, group_id, name, created_at, last_active) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![assistant_id, group_id, name, now, now],
        )?;

        // Initialize sequence tracking
        tx.execute(
            "INSERT OR IGNORE INTO assistant_sequences (group_id, last_sequence, updated_at) VALUES (?1, 0, ?2)",
            params![group_id, now],
        )?;
        tx.commit()?;

        info!("Created new assistant for group: {}", group_id);
        Ok(GroupAssistant {
            assistant_id,
            group_id: group_id.to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
            last_active: Utc::now(),
        })
    }

    /// Store message in assistant
    pub fn store_message(&mut self, message: AssistantMessage) -> Result<()> {
        debug!("Storing message for group: {}, sequence: {}", message.group_id, message.sequence);

        // Use transaction for message storage and sequence update
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO assistant_messages
             (message_id, group_id, sender_id, content, message_type, sequence, timestamp, integrity_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                message.message_id,
                message.group_id,
                message.sender_id,
                message.content,
                message.message_type,
                message.sequence,
                message.timestamp.to_rfc3339(),
                message.integrity_hash,
            ],
        )?;

        // Update sequence tracking
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT OR REPLACE INTO assistant_sequences (group_id, last_sequence, updated_at) VALUES (?1, ?2, ?3)",
            params![message.group_id, message.sequence, now],
        )?;
        tx.commit()?;

        debug!("Message stored successfully: {}", message.message_id);
        Ok(())
    }

    /// Get messages by sequence range
    pub fn get_messages(&self, group_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<AssistantMessage>> {
        debug!("Fetching messages for group: {}, sequence range: {}-{}", group_id, start_seq, end_seq);
        let mut stmt = self.conn.prepare(
            "SELECT message_id, group_id, sender_id, content, message_type, sequence, timestamp, integrity_hash
             FROM assistant_messages
             WHERE group_id = ?1 AND sequence >= ?2 AND sequence <= ?3
             ORDER BY sequence ASC"
        )?;

        let messages = stmt.query_map(params![group_id, start_seq, end_seq], |row| {
            let timestamp_str: String = row.get(6)?;
            Ok(AssistantMessage {
                message_id: row.get(0)?,
                group_id: row.get(1)?,
                sender_id: row.get(2)?,
                content: row.get(3)?,
                message_type: row.get(4)?,
                sequence: row.get(5)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                integrity_hash: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        debug!("Retrieved {} messages", messages.len());
        Ok(messages)
    }

    /// Track sequence for a group
    pub fn track_sequence(&mut self, group_id: &str, sequence: u32) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO assistant_sequences (group_id, last_sequence, updated_at) VALUES (?1, ?2, ?3)",
            params![group_id, sequence, now],
        )?;
        Ok(())
    }

    /// Get last sequence for a group
    pub fn get_last_sequence(&self, group_id: &str) -> Result<Option<u32>> {
        let mut stmt = self.conn.prepare("SELECT last_sequence FROM assistant_sequences WHERE group_id = ?1")?;
        let seq = stmt.query_row(params![group_id], |row| row.get::<_, u32>(0)).ok();
        Ok(seq)
    }

    /// Record message receipt
    pub fn record_receipt(&mut self, receipt_id: &str, message_id: &str, receiver_id: &str, sequence: u32) -> Result<()> {
        debug!("Recording receipt for message: {} by receiver: {}", message_id, receiver_id);
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO assistant_receipts (receipt_id, message_id, receiver_id, sequence, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![receipt_id, message_id, receiver_id, sequence, now],
        )?;
        debug!("Receipt recorded: {}", receipt_id);
        Ok(())
    }

    /// Get receipts for a message
    pub fn get_receipts(&self, message_id: &str) -> Result<Vec<(String, u32, DateTime<Utc>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT receiver_id, sequence, received_at FROM assistant_receipts WHERE message_id = ?1"
        )?;

        let receipts = stmt.query_map(params![message_id], |row| {
            let received_at_str: String = row.get(2)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                DateTime::parse_from_rfc3339(&received_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            ))
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(receipts)
    }

    /// Handle missing message request
    pub fn handle_missing_request(&self, _requester_id: &str, group_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<AssistantMessage>> {
        self.get_messages(group_id, start_seq, end_seq)
    }

    /// Update assistant last active time
    pub fn update_activity(&mut self, group_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE assistants SET last_active = ?1 WHERE group_id = ?2",
            params![now, group_id],
        )?;
        Ok(())
    }
}
