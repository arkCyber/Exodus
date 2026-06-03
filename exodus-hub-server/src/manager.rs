//! IM Manager - Message storage and management
//!
//! Provides SQLite-based storage for instant messaging with cyclic sequence numbers.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, debug};
use uuid::Uuid;
use rand::Rng;
use sha2::{Sha256, Digest};
use zstd;
use bincode;

/// Maximum sequence number before cycling back to 1
const MAX_SEQUENCE: u32 = 9999;

/// Generate a 12-digit numeric user ID
pub fn generate_12digit_id() -> String {
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen_range(100_000_000_000..=999_999_999_999);
    id.to_string()
}

/// Generate integrity hash for a message
pub fn generate_integrity_hash(
    sender_id: &str,
    receiver_id: Option<&str>,
    content: &str,
    sequence: u32,
    timestamp: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sender_id.as_bytes());
    if let Some(rid) = receiver_id {
        hasher.update(rid.as_bytes());
    }
    hasher.update(content.as_bytes());
    hasher.update(sequence.to_string().as_bytes());
    hasher.update(timestamp.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
}

/// Chat type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatType {
    OneOnOne,
    Group,
}

/// Conversation/chat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub chat_type: ChatType,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u32,
    pub last_sequence: u32,
}

/// Group member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub conversation_id: String,
    pub user_id: String,
    pub joined_at: DateTime<Utc>,
    pub role: String,
}

/// Message information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub receiver_id: Option<String>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: Option<u32>,
    pub reply_to: Option<String>,
    pub integrity_hash: Option<String>,
}

/// Message receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    pub id: String,
    pub message_id: String,
    pub receiver_id: String,
    pub sequence: u32,
    pub received_at: DateTime<Utc>,
}

/// User's sequence tracking (per sender)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSequence {
    pub id: String,
    pub user_id: String,
    pub sender_id: String,
    pub last_sequence: u32,
    pub updated_at: DateTime<Utc>,
}

/// Sender's sequence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderSequence {
    pub id: String,
    pub sender_id: String,
    pub last_sequence: u32,
    pub updated_at: DateTime<Utc>,
}

/// Pending message for offline users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMessage {
    pub id: String,
    pub message_id: String,
    pub receiver_id: String,
    pub conversation_id: String,
    pub created_at: DateTime<Utc>,
}

/// Sync request for historical messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub user_id: String,
    pub conversation_id: String,
    pub after_sequence: Option<u32>,
    pub limit: usize,
}

/// Sync response with compressed messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
    pub last_sequence: u32,
}

/// IM Manager
pub struct ImManager {
    #[allow(dead_code)]
    db_path: PathBuf,
    conn: Arc<TokioMutex<Connection>>,
}

impl ImManager {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        debug!("Initializing IM manager with database: {:?}", db_path);
        
        let conn = Connection::open(&db_path)
            .context("Failed to open IM database")?;
        
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;")
            .context("Failed to configure database settings")?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_seen TEXT
            )",
            [],
        ).context("Failed to create users table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                chat_type TEXT NOT NULL,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                message_count INTEGER NOT NULL DEFAULT 0,
                last_sequence INTEGER NOT NULL DEFAULT 0
            )",
            [],
        ).context("Failed to create conversations table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS group_members (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(conversation_id, user_id)
            )",
            [],
        ).context("Failed to create group_members table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                receiver_id TEXT,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                sequence INTEGER,
                reply_to TEXT,
                integrity_hash TEXT,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(id),
                FOREIGN KEY (reply_to) REFERENCES messages(id)
            )",
            [],
        ).context("Failed to create messages table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sender_sequences (
                id TEXT PRIMARY KEY,
                sender_id TEXT NOT NULL,
                last_sequence INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(sender_id)
            )",
            [],
        ).context("Failed to create sender_sequences table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pending_messages (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                receiver_id TEXT NOT NULL,
                conversation_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            )",
            [],
        ).context("Failed to create pending_messages table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_sequences (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                last_sequence INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
                UNIQUE(user_id, sender_id)
            )",
            [],
        ).context("Failed to create user_sequences table")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS message_receipts (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                receiver_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                received_at TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (receiver_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            [],
        ).context("Failed to create message_receipts table")?;
        
        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)", [])
            .context("Failed to create username index")?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_pending_messages_receiver_id ON pending_messages(receiver_id)", [])
            .context("Failed to create pending_messages receiver_id index")?;
        
        info!("IM manager initialized with database: {:?}", db_path);
        
        Ok(Self {
            db_path,
            conn: Arc::new(TokioMutex::new(conn)),
        })
    }
    
    pub async fn create_user(&self, username: &str, display_name: &str) -> Result<User> {
        debug!("Creating new user: {}", username);
        
        let id = generate_12digit_id();
        let created_at = Utc::now();
        
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO users (id, username, display_name, created_at, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                username,
                display_name,
                created_at.to_rfc3339(),
                None::<String>,
            ],
        ).context("Failed to insert user into database")?;
        
        Ok(User {
            id,
            username: username.to_string(),
            display_name: display_name.to_string(),
            created_at,
            last_seen: None,
        })
    }
    
    pub async fn create_conversation(
        &self,
        chat_type: ChatType,
        title: &str,
    ) -> Result<Conversation> {
        debug!("Creating new conversation: {} ({:?})", title, chat_type);
        
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let updated_at = created_at.clone();
        let chat_type_str = match chat_type {
            ChatType::OneOnOne => "one_on_one",
            ChatType::Group => "group",
        };
        
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO conversations (id, chat_type, title, created_at, updated_at, message_count, last_sequence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                chat_type_str,
                title,
                created_at.to_rfc3339(),
                updated_at.to_rfc3339(),
                0,
                0,
            ],
        ).context("Failed to insert conversation into database")?;
        
        Ok(Conversation {
            id,
            chat_type,
            title: title.to_string(),
            created_at,
            updated_at,
            message_count: 0,
            last_sequence: 0,
        })
    }
    
    pub async fn send_message(
        &self,
        conversation_id: &str,
        sender_id: &str,
        receiver_id: Option<&str>,
        content: &str,
        reply_to: Option<&str>,
    ) -> Result<Message> {
        debug!("Sending message to conversation: {} from user: {}", conversation_id, sender_id);
        
        let conn = self.conn.lock().await;
        
        // Calculate next sequence number for the sender (per-sender cyclic sequence)
        let last_seq: i64 = conn.query_row(
            "SELECT last_sequence FROM sender_sequences WHERE sender_id = ?1",
            params![sender_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        let next_seq = if last_seq >= MAX_SEQUENCE as i64 {
            1
        } else {
            last_seq + 1
        };
        
        // Update sender's sequence
        let updated_at = Utc::now();
        conn.execute(
            "INSERT INTO sender_sequences (id, sender_id, last_sequence, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(sender_id) DO UPDATE SET
                last_sequence = excluded.last_sequence,
                updated_at = excluded.updated_at",
            params![
                Uuid::new_v4().to_string(),
                sender_id,
                next_seq,
                updated_at.to_rfc3339(),
            ],
        ).context("Failed to update sender sequence")?;
        
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let timestamp_str = timestamp.to_rfc3339();
        
        // Generate integrity hash
        let integrity_hash = generate_integrity_hash(
            sender_id,
            receiver_id,
            content,
            next_seq as u32,
            &timestamp_str,
        );
        
        conn.execute(
            "INSERT INTO messages (id, conversation_id, sender_id, receiver_id, content, timestamp, sequence, reply_to, integrity_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                conversation_id,
                sender_id,
                receiver_id,
                content,
                timestamp_str,
                next_seq,
                reply_to,
                integrity_hash,
            ],
        ).context("Failed to insert message into database")?;
        
        conn.execute(
            "UPDATE conversations SET updated_at = ?1, message_count = message_count + 1 WHERE id = ?2",
            params![timestamp.to_rfc3339(), conversation_id],
        ).context("Failed to update conversation metadata")?;
        
        Ok(Message {
            id,
            conversation_id: conversation_id.to_string(),
            sender_id: sender_id.to_string(),
            receiver_id: receiver_id.map(|s| s.to_string()),
            content: content.to_string(),
            timestamp,
            sequence: Some(next_seq as u32),
            reply_to: reply_to.map(|s| s.to_string()),
            integrity_hash: Some(integrity_hash),
        })
    }
    
    pub async fn get_messages(&self, conversation_id: &str, limit: Option<u32>) -> Result<Vec<Message>> {
        let limit = limit.unwrap_or(100);
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, sender_id, receiver_id, content, timestamp, sequence, reply_to, integrity_hash
             FROM messages WHERE conversation_id = ?1 ORDER BY timestamp ASC LIMIT ?2"
        ).context("Failed to prepare get_messages query")?;
        
        let messages = stmt.query_map(
            params![conversation_id, limit],
            |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    receiver_id: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    sequence: row.get(6)?,
                    reply_to: row.get(7)?,
                    integrity_hash: row.get(8)?,
                })
            },
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(messages)
    }
    
    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Option<Conversation>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, chat_type, title, created_at, updated_at, message_count, last_sequence
             FROM conversations WHERE id = ?1"
        ).context("Failed to prepare get_conversation query")?;
        
        let conversation = stmt.query_row(
            params![conversation_id],
            |row| {
                let chat_type_str: String = row.get(1)?;
                let chat_type = if chat_type_str == "one_on_one" {
                    ChatType::OneOnOne
                } else {
                    ChatType::Group
                };
                
                Ok(Conversation {
                    id: row.get(0)?,
                    chat_type,
                    title: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    message_count: row.get(5)?,
                    last_sequence: row.get(6)?,
                })
            },
        );
        
        match conversation {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub async fn get_group_members(&self, conversation_id: &str) -> Result<Vec<GroupMember>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, user_id, joined_at, role
             FROM group_members WHERE conversation_id = ?1 ORDER BY joined_at ASC"
        ).context("Failed to prepare get_group_members query")?;
        
        let members = stmt.query_map(
            params![conversation_id],
            |row| {
                Ok(GroupMember {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    user_id: row.get(2)?,
                    joined_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    role: row.get(4)?,
                })
            },
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(members)
    }
    
    pub async fn add_pending_message(
        &self,
        message_id: &str,
        receiver_id: &str,
        conversation_id: &str,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO pending_messages (id, message_id, receiver_id, conversation_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                message_id,
                receiver_id,
                conversation_id,
                created_at.to_rfc3339(),
            ],
        ).context("Failed to insert pending message into database")?;
        
        Ok(())
    }
    
    pub async fn get_pending_messages(&self, receiver_id: &str) -> Result<Vec<PendingMessage>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, message_id, receiver_id, conversation_id, created_at
             FROM pending_messages WHERE receiver_id = ?1 ORDER BY created_at ASC"
        ).context("Failed to prepare get_pending_messages query")?;
        
        let pending = stmt.query_map(
            params![receiver_id],
            |row| {
                Ok(PendingMessage {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    receiver_id: row.get(2)?,
                    conversation_id: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(pending)
    }
    
    pub async fn clear_pending_messages(&self, receiver_id: &str) -> Result<usize> {
        let conn = self.conn.lock().await;
        let deleted = conn.execute(
            "DELETE FROM pending_messages WHERE receiver_id = ?1",
            params![receiver_id],
        ).context("Failed to delete pending messages")?;
        
        Ok(deleted)
    }
    
    pub async fn get_message_by_id(&self, message_id: &str) -> Result<Option<Message>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, sender_id, receiver_id, content, timestamp, sequence, reply_to, integrity_hash
             FROM messages WHERE id = ?1"
        ).context("Failed to prepare get_message_by_id query")?;
        
        let message = stmt.query_row(
            params![message_id],
            |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    receiver_id: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    sequence: row.get(6)?,
                    reply_to: row.get(7)?,
                    integrity_hash: row.get(8)?,
                })
            },
        );
        
        match message {
            Ok(m) => Ok(Some(m)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub async fn list_user_conversations(&self, user_id: &str) -> Result<Vec<Conversation>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.id, c.chat_type, c.title, c.created_at, c.updated_at, c.message_count, c.last_sequence
             FROM conversations c LEFT JOIN group_members gm ON c.id = gm.conversation_id
             WHERE c.chat_type = 'one_on_one' OR gm.user_id = ?1 ORDER BY c.updated_at DESC"
        ).context("Failed to prepare list_user_conversations query")?;
        
        let conversations = stmt.query_map(
            params![user_id],
            |row| {
                let chat_type_str: String = row.get(1)?;
                let chat_type = if chat_type_str == "one_on_one" {
                    ChatType::OneOnOne
                } else {
                    ChatType::Group
                };
                
                Ok(Conversation {
                    id: row.get(0)?,
                    chat_type,
                    title: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    message_count: row.get(5)?,
                    last_sequence: row.get(6)?,
                })
            },
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(conversations)
    }
    
    /// Update user's last seen sequence for a sender
    pub async fn update_user_sequence(
        &self,
        user_id: &str,
        sender_id: &str,
        last_sequence: u32,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let updated_at = Utc::now();
        
        conn.execute(
            "INSERT INTO user_sequences (id, user_id, sender_id, last_sequence, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(user_id, sender_id) DO UPDATE SET
                last_sequence = excluded.last_sequence,
                updated_at = excluded.updated_at",
            params![
                Uuid::new_v4().to_string(),
                user_id,
                sender_id,
                last_sequence,
                updated_at.to_rfc3339(),
            ],
        ).context("Failed to update user sequence")?;
        
        Ok(())
    }
    
    /// Get user's last seen sequence for a sender
    pub async fn get_user_sequence(
        &self,
        user_id: &str,
        sender_id: &str,
    ) -> Result<Option<UserSequence>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, user_id, sender_id, last_sequence, updated_at
             FROM user_sequences WHERE user_id = ?1 AND sender_id = ?2"
        ).context("Failed to prepare get_user_sequence query")?;
        
        let sequence = stmt.query_row(
            params![user_id, sender_id],
            |row| {
                Ok(UserSequence {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    last_sequence: row.get(3)?,
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        );
        
        match sequence {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Get sender's current sequence
    pub async fn get_sender_sequence(
        &self,
        sender_id: &str,
    ) -> Result<Option<SenderSequence>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, sender_id, last_sequence, updated_at
             FROM sender_sequences WHERE sender_id = ?1"
        ).context("Failed to prepare get_sender_sequence query")?;
        
        let sequence = stmt.query_row(
            params![sender_id],
            |row| {
                Ok(SenderSequence {
                    id: row.get(0)?,
                    sender_id: row.get(1)?,
                    last_sequence: row.get(2)?,
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        );
        
        match sequence {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Detect missing messages for a user from a specific sender
    /// Returns the range of missing sequence numbers
    pub async fn detect_missing_messages(
        &self,
        user_id: &str,
        sender_id: &str,
    ) -> Result<Option<(u32, u32)>> {
        let conn = self.conn.lock().await;
        
        // Get sender's current sequence
        let current_seq: i64 = conn.query_row(
            "SELECT last_sequence FROM sender_sequences WHERE sender_id = ?1",
            params![sender_id],
            |row| row.get(0),
        ).unwrap_or(0);
        
        // Get user's last seen sequence from this sender
        let user_seq_opt: Option<i64> = conn.query_row(
            "SELECT last_sequence FROM user_sequences WHERE user_id = ?1 AND sender_id = ?2",
            params![user_id, sender_id],
            |row| row.get(0),
        ).ok();
        
        let user_seq = user_seq_opt.unwrap_or(0);
        
        if current_seq == 0 || user_seq == current_seq {
            return Ok(None); // No missing messages
        }
        
        // Calculate missing range considering cyclic sequence
        let (missing_start, missing_end) = if current_seq > user_seq {
            (user_seq + 1, current_seq)
        } else {
            // Sequence wrapped around
            (user_seq + 1, MAX_SEQUENCE as i64)
        };
        
        Ok(Some((missing_start as u32, missing_end as u32)))
    }
    
    /// Get messages in a sequence range for a specific sender
    pub async fn get_messages_by_sequence_range(
        &self,
        sender_id: &str,
        start_sequence: u32,
        end_sequence: u32,
    ) -> Result<Vec<Message>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, sender_id, receiver_id, content, timestamp, sequence, reply_to, integrity_hash
             FROM messages WHERE sender_id = ?1 AND sequence >= ?2 AND sequence <= ?3
             ORDER BY sequence ASC"
        ).context("Failed to prepare get_messages_by_sequence_range query")?;
        
        let messages = stmt.query_map(
            params![sender_id, start_sequence, end_sequence],
            |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    receiver_id: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    sequence: row.get(6)?,
                    reply_to: row.get(7)?,
                    integrity_hash: row.get(8)?,
                })
            },
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(messages)
    }
    
    /// Create a message receipt
    pub async fn create_message_receipt(
        &self,
        message_id: &str,
        receiver_id: &str,
        sequence: u32,
    ) -> Result<MessageReceipt> {
        let id = Uuid::new_v4().to_string();
        let received_at = Utc::now();
        
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO message_receipts (id, message_id, receiver_id, sequence, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                message_id,
                receiver_id,
                sequence,
                received_at.to_rfc3339(),
            ],
        ).context("Failed to insert message receipt into database")?;
        
        Ok(MessageReceipt {
            id,
            message_id: message_id.to_string(),
            receiver_id: receiver_id.to_string(),
            sequence,
            received_at,
        })
    }
    
    /// Get receipts for a specific message
    pub async fn get_message_receipts(&self, message_id: &str) -> Result<Vec<MessageReceipt>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, message_id, receiver_id, sequence, received_at
             FROM message_receipts WHERE message_id = ?1 ORDER BY received_at ASC"
        ).context("Failed to prepare get_message_receipts query")?;
        
        let receipts = stmt.query_map(
            params![message_id],
            |row| {
                Ok(MessageReceipt {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    receiver_id: row.get(2)?,
                    sequence: row.get(3)?,
                    received_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(receipts)
    }
    
    /// Verify message integrity
    pub fn verify_message_integrity(message: &Message) -> bool {
        if let Some(ref hash) = message.integrity_hash {
            let calculated_hash = generate_integrity_hash(
                &message.sender_id,
                message.receiver_id.as_deref(),
                &message.content,
                message.sequence.unwrap_or(0),
                &message.timestamp.to_rfc3339(),
            );
            calculated_hash == *hash
        } else {
            false
        }
    }

    /// Get messages for synchronization (with pagination support)
    pub async fn get_messages_for_sync(
        &self,
        conversation_id: &str,
        after_sequence: Option<u32>,
        limit: usize,
    ) -> Result<SyncResponse> {
        debug!("Fetching messages for sync: conversation={}, after_seq={:?}, limit={}", conversation_id, after_sequence, limit);

        let conn = self.conn.lock().await;
        let query = if let Some(_after_seq) = after_sequence {
            "SELECT id, conversation_id, sender_id, receiver_id, content, timestamp, sequence, reply_to, integrity_hash
             FROM messages WHERE conversation_id = ?1 AND sequence > ?2
             ORDER BY sequence ASC LIMIT ?3"
        } else {
            "SELECT id, conversation_id, sender_id, receiver_id, content, timestamp, sequence, reply_to, integrity_hash
             FROM messages WHERE conversation_id = ?1
             ORDER BY sequence ASC LIMIT ?2"
        };

        let mut stmt = conn.prepare(query)
            .context("Failed to prepare get_messages_for_sync query")?;

        let messages = if let Some(after_seq) = after_sequence {
            stmt.query_map(params![conversation_id, after_seq, limit as i64], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    receiver_id: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    sequence: row.get(6)?,
                    reply_to: row.get(7)?,
                    integrity_hash: row.get(8)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![conversation_id, limit as i64], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    receiver_id: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    sequence: row.get(6)?,
                    reply_to: row.get(7)?,
                    integrity_hash: row.get(8)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?
        };

        let has_more = messages.len() == limit;
        let last_sequence = messages.last().and_then(|m| m.sequence).unwrap_or(0);

        debug!("Retrieved {} messages for sync, has_more={}", messages.len(), has_more);

        Ok(SyncResponse {
            messages,
            has_more,
            last_sequence,
        })
    }

    /// Compress messages for efficient transmission
    pub fn compress_messages(messages: &[Message]) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(messages)
            .context("Failed to serialize messages")?;
        let compressed = zstd::encode_all(&serialized[..], 3)
            .context("Failed to compress messages")?;
        Ok(compressed)
    }

    /// Decompress messages received from server
    pub fn decompress_messages(data: &[u8]) -> Result<Vec<Message>> {
        let decompressed = zstd::decode_all(data)
            .context("Failed to decompress messages")?;
        let messages: Vec<Message> = bincode::deserialize(&decompressed)
            .context("Failed to deserialize messages")?;
        Ok(messages)
    }

    /// Get compressed messages for sync
    pub async fn get_compressed_messages_for_sync(
        &self,
        conversation_id: &str,
        after_sequence: Option<u32>,
        limit: usize,
    ) -> Result<Vec<u8>> {
        let sync_response = self.get_messages_for_sync(conversation_id, after_sequence, limit).await?;
        Self::compress_messages(&sync_response.messages)
    }
}
