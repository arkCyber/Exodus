//! Chat Storage - SQLite-based persistent storage for chat messages
//!
//! This module provides SQLite-based persistent storage for:
//! - Friend information and direct messages
//! - Group chat messages
//! - Sequence numbers for message reliability
//! - Message receipts

use rusqlite::{Connection, params, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Chat storage configuration
#[derive(Debug, Clone)]
pub struct ChatStorageConfig {
    pub storage_dir: PathBuf,
}

impl Default for ChatStorageConfig {
    fn default() -> Self {
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_chat_storage");
        Self { storage_dir }
    }
}

/// Chat storage manager - Single database for all users
pub struct ChatStorage {
    config: ChatStorageConfig,
    // Single SQLite connection for all users
    db: Arc<Mutex<Connection>>,
}

impl ChatStorage {
    pub fn new(config: ChatStorageConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;

        let db_path = config.storage_dir.join("exodus_chat.db");
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrency (PRAGMA returns a row — use query_row)
        let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;

        // Create tables with user_id fields for data separation
        Self::create_tables(&conn)?;

        Ok(Self {
            config,
            db: Arc::new(Mutex::new(conn)),
        })
    }

    fn create_tables(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        // Friends table - per-user friend lists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS friends (
                user_id TEXT NOT NULL,
                friend_id TEXT NOT NULL,
                name TEXT NOT NULL,
                avatar_url TEXT,
                status TEXT,
                last_seen INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (user_id, friend_id)
            )",
            [],
        )?;

        // Direct messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS direct_messages (
                message_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,  -- Owner of this message
                chat_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                receiver_id TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                attachments TEXT,
                reply_to TEXT,
                sequence INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                integrity_hash TEXT,
                is_edited INTEGER NOT NULL DEFAULT 0,
                edited_at INTEGER,
                direction TEXT NOT NULL  -- 'sent' or 'received'
            )",
            [],
        )?;

        // Group messages table - shared across users but indexed by user_id
        conn.execute(
            "CREATE TABLE IF NOT EXISTS group_messages (
                message_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,  -- User who has this message
                group_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                sender_name TEXT NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                attachments TEXT,
                reply_to TEXT,
                mentions TEXT,
                sequence INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                integrity_hash TEXT,
                is_edited INTEGER NOT NULL DEFAULT 0,
                edited_at INTEGER
            )",
            [],
        )?;

        // Sequences table - per-user sequence tracking
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sequences (
                user_id TEXT NOT NULL,
                target_id TEXT NOT NULL,  -- For direct: sender_id, For group: group_id
                sequence_type TEXT NOT NULL,  -- 'direct' or 'group'
                last_sequence INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (user_id, target_id, sequence_type)
            )",
            [],
        )?;

        // Message receipts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS message_receipts (
                receipt_id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                user_id TEXT NOT NULL,  -- User who received
                receiver_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                received_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_friends_user_id ON friends(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_direct_messages_user_id ON direct_messages(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_direct_messages_chat_id ON direct_messages(chat_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_direct_messages_timestamp ON direct_messages(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_group_messages_user_id ON group_messages(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_group_messages_group_id ON group_messages(group_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_group_messages_timestamp ON group_messages(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_group_messages_sequence ON group_messages(sequence)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_message_receipts_message_id ON message_receipts(message_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_message_receipts_user_id ON message_receipts(user_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_sequences_user_id ON sequences(user_id)", [])?;

        Ok(())
    }

    /// Save friend information
    pub fn save_friend(&self, user_id: &str, friend: Friend) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let now: i64 = Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO friends
             (user_id, friend_id, name, avatar_url, status, last_seen, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                user_id,
                friend.friend_id,
                friend.name,
                friend.avatar_url,
                friend.status,
                friend.last_seen,
                friend.created_at.unwrap_or(now),
                now,
            ],
        )?;

        Ok(())
    }

    /// Get friend information
    pub fn get_friend(&self, user_id: &str, friend_id: &str) -> Result<Option<Friend>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT friend_id, name, avatar_url, status, last_seen, created_at, updated_at
             FROM friends
             WHERE user_id = ?1 AND friend_id = ?2"
        )?;

        let friend = stmt.query_row(params![user_id, friend_id], |row| {
            Ok(Friend {
                friend_id: row.get(0)?,
                name: row.get(1)?,
                avatar_url: row.get(2)?,
                status: row.get(3)?,
                last_seen: row.get(4)?,
                created_at: Some(row.get(5)?),
                updated_at: row.get(6)?,
            })
        }).ok();

        Ok(friend)
    }

    /// Get all friends for a user
    pub fn get_friends(&self, user_id: &str) -> Result<Vec<Friend>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT friend_id, name, avatar_url, status, last_seen, created_at, updated_at
             FROM friends
             WHERE user_id = ?1"
        )?;

        let friends = stmt.query_map(params![user_id], |row| {
            Ok(Friend {
                friend_id: row.get(0)?,
                name: row.get(1)?,
                avatar_url: row.get(2)?,
                status: row.get(3)?,
                last_seen: row.get(4)?,
                created_at: Some(row.get(5)?),
                updated_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(friends)
    }

    /// Save direct message
    pub fn save_direct_message(&self, user_id: &str, message: DirectMessage) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let attachments_json = serde_json::to_string(&message.attachments)?;

        conn.execute(
            "INSERT OR REPLACE INTO direct_messages
             (message_id, user_id, chat_id, sender_id, receiver_id, content, message_type,
              attachments, reply_to, sequence, timestamp, integrity_hash, is_edited, edited_at, direction)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                message.message_id,
                user_id,
                message.chat_id,
                message.sender_id,
                message.receiver_id,
                message.content,
                message.message_type,
                attachments_json,
                message.reply_to,
                message.sequence,
                message.timestamp,
                message.integrity_hash,
                if message.is_edited { 1 } else { 0 },
                message.edited_at,
                message.direction,
            ],
        )?;

        Ok(())
    }

    /// Save group message
    pub fn save_group_message(&self, user_id: &str, message: GroupMessage) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let attachments_json = serde_json::to_string(&message.attachments)?;
        let mentions_json = serde_json::to_string(&message.mentions)?;

        conn.execute(
            "INSERT OR REPLACE INTO group_messages
             (message_id, user_id, group_id, sender_id, sender_name, content, message_type,
              attachments, reply_to, mentions, sequence, timestamp, integrity_hash, is_edited, edited_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                message.message_id,
                user_id,
                message.group_id,
                message.sender_id,
                message.sender_name,
                message.content,
                message.message_type,
                attachments_json,
                message.reply_to,
                mentions_json,
                message.sequence,
                message.timestamp,
                message.integrity_hash,
                if message.is_edited { 1 } else { 0 },
                message.edited_at,
            ],
        )?;

        Ok(())
    }

    /// Update sequence number
    pub fn update_sequence(&self, user_id: &str, target_id: &str, sequence_type: &str, sequence: u32) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let now: i64 = Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO sequences (user_id, target_id, sequence_type, last_sequence, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, target_id, sequence_type, sequence, now],
        )?;

        Ok(())
    }

    /// Get sequence number
    pub fn get_sequence(&self, user_id: &str, target_id: &str, sequence_type: &str) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT last_sequence
             FROM sequences
             WHERE user_id = ?1 AND target_id = ?2 AND sequence_type = ?3"
        )?;

        let seq = stmt.query_row(params![user_id, target_id, sequence_type], |row| {
            row.get::<_, u32>(0)
        }).ok();

        Ok(seq)
    }

    /// Get direct messages for a user in a chat
    pub fn get_direct_messages(&self, user_id: &str, chat_id: &str) -> Result<Vec<DirectMessage>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT message_id, chat_id, sender_id, receiver_id, content, message_type,
                    attachments, reply_to, sequence, timestamp, integrity_hash, is_edited, edited_at, direction
             FROM direct_messages
             WHERE user_id = ?1 AND chat_id = ?2
             ORDER BY sequence ASC"
        )?;

        let messages = stmt.query_map(params![user_id, chat_id], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: Vec<MessageAttachment> = serde_json::from_str(&attachments_json).unwrap_or_default();

            Ok(DirectMessage {
                message_id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                receiver_id: row.get(3)?,
                content: row.get(4)?,
                message_type: row.get(5)?,
                attachments,
                reply_to: row.get(7)?,
                sequence: row.get(8)?,
                timestamp: row.get(9)?,
                integrity_hash: row.get(10)?,
                is_edited: row.get::<_, i32>(11)? == 1,
                edited_at: row.get(12)?,
                direction: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Get direct messages by sequence range
    pub fn get_direct_messages_by_sequence(&self, user_id: &str, chat_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<DirectMessage>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT message_id, chat_id, sender_id, receiver_id, content, message_type,
                    attachments, reply_to, sequence, timestamp, integrity_hash, is_edited, edited_at, direction
             FROM direct_messages
             WHERE user_id = ?1 AND chat_id = ?2 AND sequence >= ?3 AND sequence <= ?4
             ORDER BY sequence ASC"
        )?;

        let messages = stmt.query_map(params![user_id, chat_id, start_seq, end_seq], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: Vec<MessageAttachment> = serde_json::from_str(&attachments_json).unwrap_or_default();

            Ok(DirectMessage {
                message_id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                receiver_id: row.get(3)?,
                content: row.get(4)?,
                message_type: row.get(5)?,
                attachments,
                reply_to: row.get(7)?,
                sequence: row.get(8)?,
                timestamp: row.get(9)?,
                integrity_hash: row.get(10)?,
                is_edited: row.get::<_, i32>(11)? == 1,
                edited_at: row.get(12)?,
                direction: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Get group messages by sequence range
    pub fn get_group_messages_by_sequence(
        &self,
        user_id: &str,
        group_id: &str,
        start_seq: u32,
        end_seq: u32,
    ) -> Result<Vec<GroupMessage>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT message_id, group_id, sender_id, sender_name, content, message_type,
                    attachments, reply_to, mentions, sequence, timestamp, integrity_hash, is_edited, edited_at
             FROM group_messages
             WHERE user_id = ?1 AND group_id = ?2 AND sequence >= ?3 AND sequence <= ?4
             ORDER BY sequence ASC"
        )?;

        let messages = stmt.query_map(params![user_id, group_id, start_seq, end_seq], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: Vec<MessageAttachment> = serde_json::from_str(&attachments_json).unwrap_or_default();

            let mentions_json: String = row.get(8)?;
            let mentions: Vec<String> = serde_json::from_str(&mentions_json).unwrap_or_default();

            Ok(GroupMessage {
                message_id: row.get(0)?,
                group_id: row.get(1)?,
                sender_id: row.get(2)?,
                sender_name: row.get(3)?,
                content: row.get(4)?,
                message_type: row.get(5)?,
                attachments,
                reply_to: row.get(7)?,
                mentions,
                sequence: row.get(9)?,
                timestamp: row.get(10)?,
                integrity_hash: row.get(11)?,
                is_edited: row.get::<_, i32>(12)? == 1,
                edited_at: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Get group messages for a user in a group
    pub fn get_group_messages(&self, user_id: &str, group_id: &str) -> Result<Vec<GroupMessage>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT message_id, group_id, sender_id, sender_name, content, message_type,
                    attachments, reply_to, mentions, sequence, timestamp, integrity_hash, is_edited, edited_at
             FROM group_messages
             WHERE user_id = ?1 AND group_id = ?2
             ORDER BY sequence ASC"
        )?;

        let messages = stmt.query_map(params![user_id, group_id], |row| {
            let attachments_json: String = row.get(6)?;
            let attachments: Vec<MessageAttachment> = serde_json::from_str(&attachments_json).unwrap_or_default();

            let mentions_json: String = row.get(8)?;
            let mentions: Vec<String> = serde_json::from_str(&mentions_json).unwrap_or_default();

            Ok(GroupMessage {
                message_id: row.get(0)?,
                group_id: row.get(1)?,
                sender_id: row.get(2)?,
                sender_name: row.get(3)?,
                content: row.get(4)?,
                message_type: row.get(5)?,
                attachments,
                reply_to: row.get(7)?,
                mentions,
                sequence: row.get(9)?,
                timestamp: row.get(10)?,
                integrity_hash: row.get(11)?,
                is_edited: row.get::<_, i32>(12)? == 1,
                edited_at: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Save message receipt
    pub fn save_receipt(&self, user_id: &str, receipt: MessageReceipt) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO message_receipts
             (receipt_id, message_id, user_id, receiver_id, sequence, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                receipt.receipt_id,
                receipt.message_id,
                user_id,
                receipt.receiver_id,
                receipt.sequence,
                receipt.received_at,
            ],
        )?;

        Ok(())
    }

    /// Delete all data for a user
    pub fn delete_user_data(&self, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute("DELETE FROM friends WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM direct_messages WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM group_messages WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM sequences WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM message_receipts WHERE user_id = ?1", params![user_id])?;

        Ok(())
    }

    /// Get message receipts
    pub fn get_message_receipts(&self, user_id: &str, message_id: &str) -> Result<Vec<MessageReceipt>, Box<dyn std::error::Error>> {
        let conn = self.db.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT receipt_id, message_id, receiver_id, sequence, received_at
             FROM message_receipts
             WHERE user_id = ?1 AND message_id = ?2"
        )?;

        let receipts = stmt.query_map(params![user_id, message_id], |row| {
            Ok(MessageReceipt {
                receipt_id: row.get(0)?,
                message_id: row.get(1)?,
                receiver_id: row.get(2)?,
                sequence: row.get(3)?,
                received_at: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(receipts)
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub friend_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub last_seen: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub message_id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub message_type: String,
    pub attachments: Vec<MessageAttachment>,
    pub reply_to: Option<String>,
    pub sequence: u32,
    pub timestamp: i64,
    pub integrity_hash: Option<String>,
    pub is_edited: bool,
    pub edited_at: Option<i64>,
    pub direction: String, // 'sent' or 'received'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub message_id: String,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub attachments: Vec<MessageAttachment>,
    pub reply_to: Option<String>,
    pub mentions: Vec<String>,
    pub sequence: u32,
    pub timestamp: i64,
    pub integrity_hash: Option<String>,
    pub is_edited: bool,
    pub edited_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub attachment_id: String,
    pub file_type: String,
    pub blob_hash: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    pub receipt_id: String,
    pub message_id: String,
    pub receiver_id: String,
    pub sequence: u32,
    pub received_at: i64,
}
