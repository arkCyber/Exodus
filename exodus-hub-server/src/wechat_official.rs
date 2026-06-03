//! WeChat Official Account Module
//!
//! Management for WeChat official accounts and their message handling.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, debug};
use uuid::Uuid;

/// Official account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialAccount {
    pub account_id: String,
    pub app_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_sync: DateTime<Utc>,
}

/// Official account message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialMessage {
    pub message_id: String,
    pub account_id: String,
    pub message_type: String, // text, image, video, article, etc.
    pub content: String,
    pub media_url: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

/// WeChat official account service
pub struct WeChatOfficialService {
    conn: Connection,
}

impl WeChatOfficialService {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)
            .context("Failed to open WeChat official account database")?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS official_accounts (
                account_id TEXT PRIMARY KEY,
                app_id TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                avatar_url TEXT,
                description TEXT,
                created_at TEXT NOT NULL,
                last_sync TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS official_messages (
                message_id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                message_type TEXT NOT NULL,
                content TEXT NOT NULL,
                media_url TEXT,
                timestamp TEXT NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (account_id) REFERENCES official_accounts(account_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS account_subscribers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                subscribed_at TEXT NOT NULL,
                UNIQUE(account_id, user_id),
                FOREIGN KEY (account_id) REFERENCES official_accounts(account_id)
            )",
            [],
        )?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_official_messages_account_id ON official_messages(account_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_official_messages_timestamp ON official_messages(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_account_subscribers_account_id ON account_subscribers(account_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_account_subscribers_user_id ON account_subscribers(user_id)", [])?;

        info!("WeChat official account service initialized");

        Ok(Self { conn })
    }

    /// Add official account
    pub fn add_account(&mut self, app_id: &str, name: &str, avatar_url: Option<String>, description: Option<String>) -> Result<OfficialAccount> {
        debug!("Adding official account: app_id={}, name={}", app_id, name);
        let account_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO official_accounts (account_id, app_id, name, avatar_url, description, created_at, last_sync)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![account_id, app_id, name, avatar_url, description, now, now],
        )?;

        info!("Official account added: {}", account_id);
        Ok(OfficialAccount {
            account_id,
            app_id: app_id.to_string(),
            name: name.to_string(),
            avatar_url,
            description,
            created_at: Utc::now(),
            last_sync: Utc::now(),
        })
    }

    /// Get account by app_id
    pub fn get_account_by_app_id(&self, app_id: &str) -> Result<Option<OfficialAccount>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, app_id, name, avatar_url, description, created_at, last_sync
             FROM official_accounts WHERE app_id = ?1"
        )?;

        let account = stmt.query_row(params![app_id], |row| {
            let created_at_str: String = row.get(5)?;
            let last_sync_str: String = row.get(6)?;
            Ok(OfficialAccount {
                account_id: row.get(0)?,
                app_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                description: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                last_sync: DateTime::parse_from_rfc3339(&last_sync_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        }).ok();

        Ok(account)
    }

    /// Get all accounts
    pub fn get_all_accounts(&self) -> Result<Vec<OfficialAccount>> {
        let mut stmt = self.conn.prepare(
            "SELECT account_id, app_id, name, avatar_url, description, created_at, last_sync
             FROM official_accounts ORDER BY created_at DESC"
        )?;

        let accounts = stmt.query_map([], |row| {
            let created_at_str: String = row.get(5)?;
            let last_sync_str: String = row.get(6)?;
            Ok(OfficialAccount {
                account_id: row.get(0)?,
                app_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                description: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                last_sync: DateTime::parse_from_rfc3339(&last_sync_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(accounts)
    }

    /// Store official message
    pub fn store_message(&mut self, message: OfficialMessage) -> Result<()> {
        debug!("Storing official message for account: {}", message.account_id);
        self.conn.execute(
            "INSERT OR REPLACE INTO official_messages
             (message_id, account_id, message_type, content, media_url, timestamp, read)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                message.message_id,
                message.account_id,
                message.message_type,
                message.content,
                message.media_url,
                message.timestamp.to_rfc3339(),
                if message.read { 1 } else { 0 },
            ],
        )?;
        debug!("Official message stored: {}", message.message_id);
        Ok(())
    }

    /// Get messages for an account
    pub fn get_account_messages(&self, account_id: &str, limit: usize) -> Result<Vec<OfficialMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT message_id, account_id, message_type, content, media_url, timestamp, read
             FROM official_messages
             WHERE account_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;

        let messages = stmt.query_map(params![account_id, limit], |row| {
            let timestamp_str: String = row.get(5)?;
            Ok(OfficialMessage {
                message_id: row.get(0)?,
                account_id: row.get(1)?,
                message_type: row.get(2)?,
                content: row.get(3)?,
                media_url: row.get(4)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                read: row.get::<_, i32>(6)? == 1,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Subscribe user to account
    pub fn subscribe_user(&mut self, account_id: &str, user_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO account_subscribers (account_id, user_id, subscribed_at)
             VALUES (?1, ?2, ?3)",
            params![account_id, user_id, now],
        )?;
        Ok(())
    }

    /// Unsubscribe user from account
    pub fn unsubscribe_user(&mut self, account_id: &str, user_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM account_subscribers WHERE account_id = ?1 AND user_id = ?2",
            params![account_id, user_id],
        )?;
        Ok(())
    }

    /// Get subscribers for an account
    pub fn get_subscribers(&self, account_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id FROM account_subscribers WHERE account_id = ?1"
        )?;

        let users = stmt.query_map(params![account_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    /// Mark message as read
    pub fn mark_message_read(&mut self, message_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE official_messages SET read = 1 WHERE message_id = ?1",
            params![message_id],
        )?;
        Ok(())
    }

    /// Update last sync time
    pub fn update_sync_time(&mut self, account_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE official_accounts SET last_sync = ?1 WHERE account_id = ?2",
            params![now, account_id],
        )?;
        Ok(())
    }
}
