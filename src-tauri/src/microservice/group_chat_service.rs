//! Group Chat Service - Multi-user group messaging and 1-to-1 chat
//! 
//! This service provides group chat functionality with group management,
//! message broadcasting, and member management using the p2p-gossip service
//! for distributed message delivery. Also supports 1-to-1 direct messaging
//! with sequence numbers and integrity verification.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;
use sha2::{Sha256, Digest};
use super::chat_storage::{ChatStorage, ChatStorageConfig, DirectMessage as StoredDirectMessage, GroupMessage as StoredGroupMessage, MessageAttachment as StoredAttachment};
use super::message_resend::{MessageResendService, ResendRequest, ResendResponse};

/// Group chat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChat {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub avatar_url: Option<String>,
    pub owner_id: String,
    pub member_ids: Vec<String>, // Member agent IDs
    pub admin_ids: Vec<String>, // Admin agent IDs
    pub is_private: bool,
    pub created_at: u64,
    pub last_activity: u64,
    pub message_count: u32,
    pub public_account_id: Option<String>, // Associated public account ID
    pub last_sequence: u32, // Last sequence number for this group (1-9999)
    pub assistant_id: Option<String>, // Group assistant ID (for cloud relay)
}

/// Group message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub message_id: String,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String, // "text", "image", "file", "system"
    pub attachments: Vec<MessageAttachment>,
    pub reply_to: Option<String>, // Reply to message_id
    pub mentions: Vec<String>, // Mentioned user IDs
    pub timestamp: u64,
    pub is_edited: bool,
    pub edited_at: Option<u64>,
    pub sequence: u32, // Group sequence number (1-9999)
    pub integrity_hash: Option<String>, // SHA256 hash for integrity verification
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub attachment_id: String,
    pub file_type: String, // "image", "video", "audio", "file"
    pub blob_hash: String, // Reference to p2p-blobs
    pub file_name: String,
    pub file_size: u64,
    pub thumbnail_hash: Option<String>,
}

/// Group member info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub agent_id: String,
    pub agent_name: String,
    pub role: String, // "owner", "admin", "member"
    pub joined_at: u64,
    pub last_seen: u64,
    pub is_online: bool,
    pub nickname: Option<String>,
}

/// Group invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInvitation {
    pub invitation_id: String,
    pub group_id: String,
    pub group_name: String,
    pub inviter_id: String,
    pub inviter_name: String,
    pub invitee_id: String,
    pub status: String, // "pending", "accepted", "rejected", "expired"
    pub created_at: u64,
    pub expires_at: u64,
}

/// 1-to-1 chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectChat {
    pub chat_id: String,
    pub user_a: String, // First user ID
    pub user_b: String, // Second user ID
    pub created_at: u64,
    pub last_activity: u64,
    pub message_count: u32,
}

/// 1-to-1 direct message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub message_id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub message_type: String, // "text", "image", "file", "system"
    pub attachments: Vec<MessageAttachment>,
    pub reply_to: Option<String>,
    pub sequence: u32, // Cyclic sequence number 1-9999 per sender
    pub timestamp: u64,
    pub integrity_hash: Option<String>, // SHA256 hash for integrity verification
    pub is_edited: bool,
    pub edited_at: Option<u64>,
}

/// Message receipt for 1-to-1 chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    pub receipt_id: String,
    pub message_id: String,
    pub receiver_id: String,
    pub sequence: u32,
    pub received_at: u64,
}

/// User sequence tracking (for missing message detection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSequence {
    pub user_id: String,
    pub sender_id: String,
    pub last_sequence: u32,
    pub updated_at: u64,
}

/// Group sequence tracking (for group chat missing message detection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSequence {
    pub user_id: String,
    pub group_id: String,
    pub last_sequence: u32,
    pub updated_at: u64,
}

/// Maximum sequence number before cycling
const MAX_SEQUENCE: u32 = 9999;

/// Generate integrity hash for a message
fn generate_integrity_hash(
    sender_id: &str,
    receiver_id: &str,
    content: &str,
    sequence: u32,
    timestamp: u64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sender_id.as_bytes());
    hasher.update(receiver_id.as_bytes());
    hasher.update(content.as_bytes());
    hasher.update(sequence.to_string().as_bytes());
    hasher.update(timestamp.to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate integrity hash for a group message
fn generate_group_integrity_hash(
    group_id: &str,
    sender_id: &str,
    content: &str,
    sequence: u32,
    timestamp: u64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(group_id.as_bytes());
    hasher.update(sender_id.as_bytes());
    hasher.update(content.as_bytes());
    hasher.update(sequence.to_string().as_bytes());
    hasher.update(timestamp.to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Verify message integrity
fn verify_integrity_hash(message: &DirectMessage) -> bool {
    if let Some(ref hash) = message.integrity_hash {
        let calculated = generate_integrity_hash(
            &message.sender_id,
            &message.receiver_id,
            &message.content,
            message.sequence,
            message.timestamp,
        );
        calculated == *hash
    } else {
        false
    }
}

/// Verify group message integrity
fn verify_group_integrity_hash(message: &GroupMessage) -> bool {
    if let Some(ref hash) = message.integrity_hash {
        let calculated = generate_group_integrity_hash(
            &message.group_id,
            &message.sender_id,
            &message.content,
            message.sequence,
            message.timestamp,
        );
        calculated == *hash
    } else {
        false
    }
}

/// Configuration for Group Chat Service
#[derive(Debug, Clone)]
pub struct GroupChatServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for GroupChatServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_group_chat.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_group_chats");
        
        Self { socket_path, storage_dir }
    }
}

/// Group Chat Service
pub struct GroupChatService {
    config: GroupChatServiceConfig,
    groups: Arc<Mutex<HashMap<String, GroupChat>>>, // group_id -> group
    messages: Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>, // group_id -> messages
    members: Arc<Mutex<HashMap<String, Vec<GroupMember>>>>, // group_id -> members
    invitations: Arc<Mutex<HashMap<String, GroupInvitation>>>, // invitation_id -> invitation
    user_groups: Arc<Mutex<HashMap<String, Vec<String>>>>, // user_id -> group_ids
    // 1-to-1 chat storage
    direct_chats: Arc<Mutex<HashMap<String, DirectChat>>>, // chat_id -> chat
    direct_messages: Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>, // chat_id -> messages
    sender_sequences: Arc<Mutex<HashMap<String, u32>>>, // sender_id -> current sequence
    user_sequences: Arc<Mutex<HashMap<String, UserSequence>>>, // user_id + sender_id -> UserSequence
    receipts: Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>, // message_id -> receipts
    // Group chat reliability features
    group_sequences: Arc<Mutex<HashMap<String, u32>>>, // group_id -> current sequence
    group_user_sequences: Arc<Mutex<HashMap<String, GroupSequence>>>, // user_id + group_id -> GroupSequence
    group_receipts: Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>, // message_id -> receipts for group messages
    // SQLite persistent storage
    chat_storage: Arc<ChatStorage>,
    // Message resend service
    message_resend: Arc<MessageResendService>,
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl GroupChatService {
    pub fn new(config: GroupChatServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        // Initialize chat storage
        let mut storage_dir = config.storage_dir.clone();
        storage_dir.push("sqlite");
        let chat_storage_config = ChatStorageConfig {
            storage_dir,
        };
        let chat_storage = Arc::new(ChatStorage::new(chat_storage_config)?);
        
        // Initialize message resend service
        let message_resend = Arc::new(MessageResendService::new(Arc::clone(&chat_storage)));
        
        Ok(Self {
            config,
            groups: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            members: Arc::new(Mutex::new(HashMap::new())),
            invitations: Arc::new(Mutex::new(HashMap::new())),
            user_groups: Arc::new(Mutex::new(HashMap::new())),
            direct_chats: Arc::new(Mutex::new(HashMap::new())),
            direct_messages: Arc::new(Mutex::new(HashMap::new())),
            sender_sequences: Arc::new(Mutex::new(HashMap::new())),
            user_sequences: Arc::new(Mutex::new(HashMap::new())),
            receipts: Arc::new(Mutex::new(HashMap::new())),
            group_sequences: Arc::new(Mutex::new(HashMap::new())),
            group_user_sequences: Arc::new(Mutex::new(HashMap::new())),
            group_receipts: Arc::new(Mutex::new(HashMap::new())),
            chat_storage,
            message_resend,
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }

        let socket_path = self.config.socket_path.clone();
        let groups = Arc::clone(&self.groups);
        let messages = Arc::clone(&self.messages);
        let members = Arc::clone(&self.members);
        let invitations = Arc::clone(&self.invitations);
        let user_groups = Arc::clone(&self.user_groups);
        let direct_chats = Arc::clone(&self.direct_chats);
        let direct_messages = Arc::clone(&self.direct_messages);
        let sender_sequences = Arc::clone(&self.sender_sequences);
        let user_sequences = Arc::clone(&self.user_sequences);
        let receipts = Arc::clone(&self.receipts);
        let group_sequences = Arc::clone(&self.group_sequences);
        let group_user_sequences = Arc::clone(&self.group_user_sequences);
        let group_receipts = Arc::clone(&self.group_receipts);
        let chat_storage = Arc::clone(&self.chat_storage);
        let message_resend = Arc::clone(&self.message_resend);
        let node_id = self.node_id.clone();
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            *tx_guard = Some(shutdown_tx);
        }
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let groups = Arc::clone(&groups);
                                let messages = Arc::clone(&messages);
                                let members = Arc::clone(&members);
                                let invitations = Arc::clone(&invitations);
                                let user_groups = Arc::clone(&user_groups);
                                let direct_chats = Arc::clone(&direct_chats);
                                let direct_messages = Arc::clone(&direct_messages);
                                let sender_sequences = Arc::clone(&sender_sequences);
                                let user_sequences = Arc::clone(&user_sequences);
                                let receipts = Arc::clone(&receipts);
                                let group_sequences = Arc::clone(&group_sequences);
                                let group_user_sequences = Arc::clone(&group_user_sequences);
                                let group_receipts = Arc::clone(&group_receipts);
                                let chat_storage = Arc::clone(&chat_storage);
                                let message_resend = Arc::clone(&message_resend);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, groups, messages, members, invitations, user_groups, direct_chats, direct_messages, sender_sequences, user_sequences, receipts, group_sequences, group_user_sequences, group_receipts, chat_storage, message_resend, node_id).await;
                                });
                            }
                            Err(e) => {
                                eprintln!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        println!("Group Chat Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }

        if let Some(tx) = self.shutdown_tx.lock().ok().and_then(|mut tx| tx.take()) {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("Group Chat Service stopped");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Create a new group
    #[allow(dead_code)]
    pub fn create_group(&self, group: GroupChat) -> Result<(), String> {
        let group_id = group.group_id.clone();
        let owner_id = group.owner_id.clone();
        
        // Initialize group sequence to 0
        let mut group_seqs = self.group_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        group_seqs.insert(group_id.clone(), 0);
        drop(group_seqs);
        
        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        groups.insert(group_id.clone(), group.clone());
        drop(groups);

        let mut user_groups = self.user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        user_groups.entry(owner_id).or_insert_with(Vec::new).push(group_id.clone());

        Ok(())
    }

    /// Update group
    #[allow(dead_code)]
    pub fn update_group(&self, group: GroupChat, requester_id: String) -> Result<(), String> {
        let group_id = group.group_id.clone();
        
        // Check if requester has permission
        if !self.has_permission(group_id.clone(), requester_id) {
            return Err("Only owner or admin can update group".to_string());
        }

        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        groups.insert(group_id, group);
        Ok(())
    }

    /// Delete group
    #[allow(dead_code)]
    pub fn delete_group(&self, group_id: String, requester_id: String) -> Result<(), String> {
        // Check if requester is owner
        if !self.is_owner(group_id.clone(), requester_id) {
            return Err("Only owner can delete group".to_string());
        }

        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        let group = groups.remove(&group_id);
        drop(groups);

        if let Some(group) = group {
            let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
            messages.remove(&group_id);
            drop(messages);

            let mut members = self.members.lock().map_err(|e| format!("Lock error: {}", e))?;
            members.remove(&group_id);
            drop(members);

            let mut user_groups = self.user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
            for member_id in &group.member_ids {
                if let Some(group_ids) = user_groups.get_mut(member_id) {
                    group_ids.retain(|id| id != &group_id);
                    if group_ids.is_empty() {
                        user_groups.remove(member_id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get group
    #[allow(dead_code)]
    pub fn get_group(&self, group_id: String) -> Option<GroupChat> {
        let groups = self.groups.lock().ok()?;
        groups.get(&group_id).cloned()
    }

    /// List user's groups
    #[allow(dead_code)]
    pub fn list_user_groups(&self, user_id: String) -> Vec<GroupChat> {
        let user_groups = self.user_groups.lock();
        let groups = self.groups.lock();
        
        if let (Ok(user_groups), Ok(groups)) = (user_groups, groups) {
            if let Some(group_ids) = user_groups.get(&user_id) {
                group_ids.iter()
                    .filter_map(|id| groups.get(id).cloned())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Add member to group
    #[allow(dead_code)]
    pub fn add_member(&self, group_id: String, member: GroupMember) -> Result<(), String> {
        let agent_id = member.agent_id.clone();
        
        let mut members = self.members.lock().map_err(|e| format!("Lock error: {}", e))?;
        members.entry(group_id.clone()).or_insert_with(Vec::new).push(member.clone());
        drop(members);

        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group) = groups.get_mut(&group_id) {
            if !group.member_ids.contains(&agent_id) {
                group.member_ids.push(agent_id.clone());
            }
            group.last_activity = current_timestamp();
        }
        drop(groups);

        let mut user_groups = self.user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        user_groups.entry(agent_id).or_insert_with(Vec::new).push(group_id.clone());

        Ok(())
    }

    /// Remove member from group
    #[allow(dead_code)]
    pub fn remove_member(&self, group_id: String, agent_id: String, requester_id: String) -> Result<(), String> {
        // Check if requester has permission
        if !self.has_permission(group_id.clone(), requester_id) {
            return Err("Only owner or admin can remove members".to_string());
        }

        let mut members = self.members.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(member_list) = members.get_mut(&group_id) {
            member_list.retain(|m| m.agent_id != agent_id);
        }
        drop(members);

        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group) = groups.get_mut(&group_id) {
            group.member_ids.retain(|id| id != &agent_id);
            group.last_activity = current_timestamp();
        }
        drop(groups);

        let mut user_groups = self.user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group_ids) = user_groups.get_mut(&agent_id) {
            group_ids.retain(|id| id != &group_id);
            if group_ids.is_empty() {
                user_groups.remove(&agent_id);
            }
        }

        Ok(())
    }

    /// Get group members
    #[allow(dead_code)]
    pub fn get_members(&self, group_id: String) -> Vec<GroupMember> {
        self.members.lock()
            .map(|members| members.get(&group_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Link group to public account
    #[allow(dead_code)]
    pub fn link_to_public_account(&self, group_id: String, account_id: String) -> Result<(), String> {
        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group) = groups.get_mut(&group_id) {
            group.public_account_id = Some(account_id);
        }
        Ok(())
    }

    /// Unlink group from public account
    #[allow(dead_code)]
    pub fn unlink_from_public_account(&self, group_id: String) -> Result<(), String> {
        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group) = groups.get_mut(&group_id) {
            group.public_account_id = None;
        }
        Ok(())
    }

    /// Get groups linked to a public account
    #[allow(dead_code)]
    pub fn get_groups_by_public_account(&self, account_id: String) -> Vec<GroupChat> {
        self.groups.lock()
            .map(|groups| groups.values()
            .filter(|g| g.public_account_id.as_ref() == Some(&account_id))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Check if user is group owner
    #[allow(dead_code)]
    pub fn is_owner(&self, group_id: String, user_id: String) -> bool {
        self.groups.lock()
            .map(|groups| {
                if let Some(group) = groups.get(&group_id) {
                    group.owner_id == user_id
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    /// Check if user is group admin
    #[allow(dead_code)]
    pub fn is_admin(&self, group_id: String, user_id: String) -> bool {
        self.groups.lock()
            .map(|groups| {
                if let Some(group) = groups.get(&group_id) {
                    group.admin_ids.contains(&user_id) || group.owner_id == user_id
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    /// Check if user has permission (owner or admin)
    #[allow(dead_code)]
    pub fn has_permission(&self, group_id: String, user_id: String) -> bool {
        self.is_admin(group_id, user_id)
    }

    /// Add admin to group
    #[allow(dead_code)]
    pub fn add_admin(&self, group_id: String, user_id: String, requester_id: String) -> Result<(), String> {
        // Check if requester is owner
        if !self.is_owner(group_id.clone(), requester_id) {
            return Err("Only owner can add admins".to_string());
        }

        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group) = groups.get_mut(&group_id) {
            if !group.admin_ids.contains(&user_id) {
                group.admin_ids.push(user_id);
            }
        }
        Ok(())
    }

    /// Remove admin from group
    #[allow(dead_code)]
    pub fn remove_admin(&self, group_id: String, user_id: String, requester_id: String) -> Result<(), String> {
        // Check if requester is owner
        if !self.is_owner(group_id.clone(), requester_id) {
            return Err("Only owner can remove admins".to_string());
        }

        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(group) = groups.get_mut(&group_id) {
            group.admin_ids.retain(|id| id != &user_id);
        }
        Ok(())
    }

    /// Send message to group
    #[allow(dead_code)]
    pub fn send_message(&self, mut message: GroupMessage) -> Result<(), String> {
        let group_id = message.group_id.clone();
        
        // Generate sequence number for this group
        let mut seq_map = self.group_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        let current_seq = seq_map.get(&group_id).copied().unwrap_or(0);
        let next_seq = if current_seq >= MAX_SEQUENCE { 1 } else { current_seq + 1 };
        seq_map.insert(group_id.clone(), next_seq);
        message.sequence = next_seq;
        drop(seq_map);
        
        // Generate integrity hash for group message
        let integrity_hash = generate_group_integrity_hash(
            &message.group_id,
            &message.sender_id,
            &message.content,
            next_seq,
            message.timestamp,
        );
        message.integrity_hash = Some(integrity_hash.clone());
        
        let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        messages.entry(group_id.clone()).or_insert_with(Vec::new).push(message.clone());
        
        // Keep only last 1000 messages per group
        if let Some(msg_list) = messages.get_mut(&group_id) {
            let len = msg_list.len();
            if len > 1000 {
                msg_list.drain(0..len - 1000);
            }
        }
        drop(messages);

        let member_ids: Vec<String> = {
            let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(group) = groups.get_mut(&group_id) {
                group.message_count += 1;
                group.last_activity = current_timestamp();
                group.last_sequence = next_seq;
                group.member_ids.clone()
            } else {
                Vec::new()
            }
        };

        // Persist to SQLite for all members (no second lock on `groups` — avoids deadlock)
        for member_id in &member_ids {
            let stored_message = StoredGroupMessage {
                message_id: message.message_id.clone(),
                group_id: message.group_id.clone(),
                sender_id: message.sender_id.clone(),
                sender_name: message.sender_name.clone(),
                content: message.content.clone(),
                message_type: message.message_type.clone(),
                attachments: message.attachments.iter().map(|a| StoredAttachment {
                    attachment_id: a.attachment_id.clone(),
                    file_type: a.file_type.clone(),
                    blob_hash: a.blob_hash.clone(),
                    file_name: a.file_name.clone(),
                    file_size: a.file_size,
                    mime_type: None,
                }).collect(),
                reply_to: message.reply_to.clone(),
                mentions: message.mentions.clone(),
                sequence: message.sequence,
                timestamp: message.timestamp as i64,
                integrity_hash: message.integrity_hash.clone(),
                is_edited: message.is_edited,
                edited_at: message.edited_at.map(|t| t as i64),
            };
            let _ = self.chat_storage.save_group_message(member_id, stored_message);
        }

        Ok(())
    }

    /// Get group messages
    #[allow(dead_code)]
    pub fn get_messages(&self, group_id: String, limit: Option<usize>, before: Option<u64>) -> Vec<GroupMessage> {
        self.messages.lock()
            .map(|messages| {
                let limit = limit.unwrap_or(50);
                
                if let Some(msg_list) = messages.get(&group_id) {
                    let mut filtered: Vec<GroupMessage> = msg_list.clone();
                    
                    if let Some(before_ts) = before {
                        filtered.retain(|m| m.timestamp < before_ts);
                    }
                    
                    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    filtered.truncate(limit);
                    filtered
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default()
    }

    /// Edit message
    #[allow(dead_code)]
    pub fn edit_message(&self, group_id: String, message_id: String, new_content: String) -> Result<(), String> {
        let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get_mut(&group_id) {
            if let Some(msg) = msg_list.iter_mut().find(|m| m.message_id == message_id) {
                msg.content = new_content;
                msg.is_edited = true;
                msg.edited_at = Some(current_timestamp());
            }
        }
        Ok(())
    }

    /// Delete message
    #[allow(dead_code)]
    pub fn delete_message(&self, group_id: String, message_id: String) -> Result<(), String> {
        let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get_mut(&group_id) {
            msg_list.retain(|m| m.message_id != message_id);
        }
        Ok(())
    }

    /// Create invitation
    #[allow(dead_code)]
    pub fn create_invitation(&self, invitation: GroupInvitation) -> Result<(), String> {
        let invitation_id = invitation.invitation_id.clone();
        let mut invitations = self.invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
        invitations.insert(invitation_id, invitation);
        Ok(())
    }

    /// Accept invitation
    #[allow(dead_code)]
    pub fn accept_invitation(&self, invitation_id: String) -> Result<(), String> {
        let mut invitations = self.invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(invitation) = invitations.get_mut(&invitation_id) {
            invitation.status = "accepted".to_string();
        }
        Ok(())
    }

    /// Reject invitation
    #[allow(dead_code)]
    pub fn reject_invitation(&self, invitation_id: String) -> Result<(), String> {
        let mut invitations = self.invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(invitation) = invitations.get_mut(&invitation_id) {
            invitation.status = "rejected".to_string();
        }
        Ok(())
    }

    /// Get pending invitations for user
    #[allow(dead_code)]
    pub fn get_pending_invitations(&self, user_id: String) -> Vec<GroupInvitation> {
        self.invitations.lock()
            .map(|invitations| invitations.values()
            .filter(|i| i.invitee_id == user_id && i.status == "pending")
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Update member online status
    #[allow(dead_code)]
    pub fn update_member_online(&self, group_id: String, agent_id: String, is_online: bool) -> Result<(), String> {
        let mut members = self.members.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(member_list) = members.get_mut(&group_id) {
            if let Some(member) = member_list.iter_mut().find(|m| m.agent_id == agent_id) {
                member.is_online = is_online;
                member.last_seen = current_timestamp();
            }
        }
        Ok(())
    }

    /// Search groups
    #[allow(dead_code)]
    pub fn search_groups(&self, query: String, limit: Option<usize>) -> Vec<GroupChat> {
        self.groups.lock()
            .map(|groups| {
                let query_lower = query.to_lowercase();
                
                let mut results: Vec<GroupChat> = groups.values()
                    .filter(|g| {
                        g.name.to_lowercase().contains(&query_lower) ||
                        g.description.to_lowercase().contains(&query_lower)
                    })
                    .filter(|g| !g.is_private) // Only show public groups in search
                    .cloned()
                    .collect();

                results.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
                
                if let Some(limit) = limit {
                    results.truncate(limit);
                }

                results
            })
            .unwrap_or_default()
    }

    // ========== Group Chat Reliability Methods ==========

    /// Update user's sequence tracking for a group
    #[allow(dead_code)]
    pub fn update_group_sequence(&self, user_id: String, group_id: String, last_sequence: u32) -> Result<(), String> {
        let key = format!("{}:{}", user_id, group_id);
        let mut sequences = self.group_user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        sequences.insert(key, GroupSequence {
            user_id: user_id.clone(),
            group_id: group_id.clone(),
            last_sequence,
            updated_at: current_timestamp(),
        });
        Ok(())
    }

    /// Get user's sequence tracking for a group
    #[allow(dead_code)]
    pub fn get_group_sequence(&self, user_id: String, group_id: String) -> Option<GroupSequence> {
        let key = format!("{}:{}", user_id, group_id);
        self.group_user_sequences.lock().ok()?.get(&key).cloned()
    }

    /// Detect missing messages in a group for a user
    #[allow(dead_code)]
    pub fn detect_group_missing(&self, user_id: String, group_id: String) -> Result<(bool, u32, u32, u32), String> {
        let seqs = self.group_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        let current_seq = seqs.get(&group_id).copied().unwrap_or(0);
        drop(seqs);

        let user_seqs = self.group_user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        let key = format!("{}:{}", user_id, group_id);
        let user_seq = user_seqs.get(&key).map(|u| u.last_sequence).unwrap_or(0);

        if current_seq > user_seq {
            Ok((true, user_seq + 1, current_seq, current_seq - user_seq))
        } else if user_seq > 0 && current_seq < user_seq {
            // Sequence cycled
            Ok((true, user_seq + 1, MAX_SEQUENCE, MAX_SEQUENCE - user_seq))
        } else {
            Ok((false, 0, 0, 0))
        }
    }

    /// Get messages by sequence range for a group
    #[allow(dead_code)]
    pub fn get_messages_by_sequence(&self, group_id: String, start_seq: u32, end_seq: u32) -> Vec<GroupMessage> {
        self.messages.lock()
            .map(|messages| {
                if let Some(msg_list) = messages.get(&group_id) {
                    msg_list.iter()
                        .filter(|m| m.sequence >= start_seq && m.sequence <= end_seq)
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default()
    }

    /// Create receipt for a group message
    #[allow(dead_code)]
    pub fn create_group_receipt(&self, message_id: String, receiver_id: String, sequence: u32) -> Result<MessageReceipt, String> {
        let receipt_id = format!("receipt-{}", uuid::Uuid::new_v4());
        let receipt = MessageReceipt {
            receipt_id: receipt_id.clone(),
            message_id: message_id.clone(),
            receiver_id: receiver_id.clone(),
            sequence,
            received_at: current_timestamp(),
        };

        let mut receipts = self.group_receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
        receipts.entry(message_id).or_insert_with(Vec::new).push(receipt.clone());

        Ok(receipt)
    }

    /// Get receipts for a group message
    #[allow(dead_code)]
    pub fn get_group_receipts(&self, message_id: String) -> Vec<MessageReceipt> {
        self.group_receipts.lock()
            .map(|receipts| receipts.get(&message_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Verify group message integrity
    #[allow(dead_code)]
    pub fn verify_group_message(&self, group_id: String, message_id: String) -> Result<bool, String> {
        let messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get(&group_id) {
            if let Some(msg) = msg_list.iter().find(|m| m.message_id == message_id) {
                return Ok(verify_group_integrity_hash(msg));
            }
        }
        Ok(false)
    }

    // ========== 1-to-1 Chat Methods ==========

    /// Create or get a direct chat between two users
    #[allow(dead_code)]
    pub fn create_or_get_direct_chat(&self, user_a: String, user_b: String) -> Result<DirectChat, String> {
        // Generate stable chat ID based on sorted user IDs
        let mut ids = vec![user_a.clone(), user_b.clone()];
        ids.sort();
        let chat_id = format!("dm-{}-{}", ids[0], ids[1]);
        
        let mut chats = self.direct_chats.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(chat) = chats.get(&chat_id) {
            return Ok(chat.clone());
        }
        
        let chat = DirectChat {
            chat_id: chat_id.clone(),
            user_a: ids[0].clone(),
            user_b: ids[1].clone(),
            created_at: current_timestamp(),
            last_activity: current_timestamp(),
            message_count: 0,
        };
        
        chats.insert(chat_id.clone(), chat.clone());
        Ok(chat)
    }

    /// Send direct message
    #[allow(dead_code)]
    pub fn send_direct_message(&self, message: DirectMessage) -> Result<(), String> {
        let chat_id = message.chat_id.clone();
        let sender_id = message.sender_id.clone();
        
        // Generate sequence number for this sender
        let mut seq_map = self.sender_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        let current_seq = seq_map.get(&sender_id).copied().unwrap_or(0);
        let next_seq = if current_seq >= MAX_SEQUENCE { 1 } else { current_seq + 1 };
        seq_map.insert(sender_id.clone(), next_seq);
        drop(seq_map);
        
        // Generate integrity hash
        let _integrity_hash = generate_integrity_hash(
            &message.sender_id,
            &message.receiver_id,
            &message.content,
            next_seq,
            message.timestamp,
        );
        
        let mut messages = self.direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        messages.entry(chat_id.clone()).or_insert_with(Vec::new).push(message.clone());
        
        // Keep only last 1000 messages per chat
        if let Some(msg_list) = messages.get_mut(&chat_id) {
            let len = msg_list.len();
            if len > 1000 {
                msg_list.drain(0..len - 1000);
            }
        }
        drop(messages);

        let mut chats = self.direct_chats.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(chat) = chats.get_mut(&chat_id) {
            chat.message_count += 1;
            chat.last_activity = current_timestamp();
        }

        Ok(())
    }

    /// Get direct messages
    #[allow(dead_code)]
    pub fn get_direct_messages(&self, chat_id: String, limit: Option<usize>, before: Option<u64>) -> Vec<DirectMessage> {
        self.direct_messages.lock()
            .map(|messages| {
                let limit = limit.unwrap_or(50);
                
                if let Some(msg_list) = messages.get(&chat_id) {
                    let mut filtered: Vec<DirectMessage> = msg_list.clone();
                    
                    if let Some(before_ts) = before {
                        filtered.retain(|m| m.timestamp < before_ts);
                    }
                    
                    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    filtered.truncate(limit);
                    filtered
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default()
    }

    /// Edit direct message
    #[allow(dead_code)]
    pub fn edit_direct_message(&self, chat_id: String, message_id: String, new_content: String) -> Result<(), String> {
        let mut messages = self.direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get_mut(&chat_id) {
            if let Some(msg) = msg_list.iter_mut().find(|m| m.message_id == message_id) {
                msg.content = new_content;
                msg.is_edited = true;
                msg.edited_at = Some(current_timestamp());
            }
        }
        Ok(())
    }

    /// Delete direct message
    #[allow(dead_code)]
    pub fn delete_direct_message(&self, chat_id: String, message_id: String) -> Result<(), String> {
        let mut messages = self.direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get_mut(&chat_id) {
            msg_list.retain(|m| m.message_id != message_id);
        }
        Ok(())
    }

    /// Get direct chat
    #[allow(dead_code)]
    pub fn get_direct_chat(&self, chat_id: String) -> Option<DirectChat> {
        let chats = self.direct_chats.lock().ok()?;
        chats.get(&chat_id).cloned()
    }

    /// List direct chats for a user
    #[allow(dead_code)]
    pub fn list_direct_chats(&self, user_id: String) -> Vec<DirectChat> {
        self.direct_chats.lock()
            .map(|chats| chats.values()
            .filter(|c| c.user_a == user_id || c.user_b == user_id)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Update user sequence tracking
    #[allow(dead_code)]
    pub fn update_user_sequence(&self, user_id: String, sender_id: String, last_sequence: u32) -> Result<(), String> {
        let key = format!("{}:{}", user_id, sender_id);
        let mut sequences = self.user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        sequences.insert(key, UserSequence {
            user_id,
            sender_id,
            last_sequence,
            updated_at: current_timestamp(),
        });
        Ok(())
    }

    /// Get user sequence tracking
    #[allow(dead_code)]
    pub fn get_user_sequence(&self, user_id: String, sender_id: String) -> Option<UserSequence> {
        let key = format!("{}:{}", user_id, sender_id);
        let sequences = self.user_sequences.lock().ok()?;
        sequences.get(&key).cloned()
    }

    /// Detect missing messages for a user from a sender
    #[allow(dead_code)]
    pub fn detect_missing_messages(&self, user_id: String, sender_id: String) -> Option<(u32, u32)> {
        let sequences = self.sender_sequences.lock().ok()?;
        let current_seq = sequences.get(&sender_id).copied().unwrap_or(0);
        
        let user_seqs = self.user_sequences.lock().ok()?;
        let key = format!("{}:{}", user_id, sender_id);
        let user_seq = user_seqs.get(&key).map(|u| u.last_sequence).unwrap_or(0);
        
        if current_seq > user_seq {
            Some((user_seq + 1, current_seq))
        } else if user_seq > 0 && current_seq < user_seq {
            // Sequence cycled
            Some((user_seq + 1, MAX_SEQUENCE))
        } else {
            None
        }
    }

    /// Get messages by sequence range for direct chat
    #[allow(dead_code)]
    pub fn get_direct_messages_by_sequence(&self, sender_id: String, start_seq: u32, end_seq: u32) -> Vec<DirectMessage> {
        self.direct_messages.lock()
            .map(|messages| messages.values()
            .flat_map(|msg_list| msg_list.iter())
            .filter(|m| m.sender_id == sender_id && m.sequence >= start_seq && m.sequence <= end_seq)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Create receipt for a direct message
    #[allow(dead_code)]
    pub fn create_receipt(&self, message_id: String, receiver_id: String, sequence: u32) -> Result<MessageReceipt, String> {
        let receipt_id = format!("receipt-{}", uuid::Uuid::new_v4());
        let receipt = MessageReceipt {
            receipt_id: receipt_id.clone(),
            message_id,
            receiver_id,
            sequence,
            received_at: current_timestamp(),
        };
        
        let mut receipts = self.receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
        receipts.entry(receipt.message_id.clone()).or_insert_with(Vec::new).push(receipt.clone());
        
        Ok(receipt)
    }

    /// Get receipts for a message
    #[allow(dead_code)]
    pub fn get_receipts(&self, message_id: String) -> Vec<MessageReceipt> {
        self.receipts.lock()
            .map(|receipts| receipts.get(&message_id).cloned().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Verify message integrity
    #[allow(dead_code)]
    pub fn verify_direct_message(&self, chat_id: String, message_id: String) -> Result<bool, String> {
        let messages = self.direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get(&chat_id) {
            if let Some(msg) = msg_list.iter().find(|m| m.message_id == message_id) {
                return Ok(verify_integrity_hash(msg));
            }
        }
        Ok(false)
    }
}

async fn handle_client(
    mut stream: UnixStream,
    groups: Arc<Mutex<HashMap<String, GroupChat>>>,
    messages: Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
    members: Arc<Mutex<HashMap<String, Vec<GroupMember>>>>,
    invitations: Arc<Mutex<HashMap<String, GroupInvitation>>>,
    user_groups: Arc<Mutex<HashMap<String, Vec<String>>>>,
    direct_chats: Arc<Mutex<HashMap<String, DirectChat>>>,
    direct_messages: Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
    sender_sequences: Arc<Mutex<HashMap<String, u32>>>,
    user_sequences: Arc<Mutex<HashMap<String, UserSequence>>>,
    receipts: Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>,
    group_sequences: Arc<Mutex<HashMap<String, u32>>>,
    group_user_sequences: Arc<Mutex<HashMap<String, GroupSequence>>>,
    group_receipts: Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>,
    chat_storage: Arc<ChatStorage>,
    message_resend: Arc<MessageResendService>,
    node_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer): (tokio::net::unix::OwnedReadHalf, tokio::net::unix::OwnedWriteHalf) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: serde_json::Value = serde_json::from_str(&line)?;
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);

        let result = match method {
            "create_group" => handle_create_group(&params, &groups, &user_groups, &group_sequences).await,
            "update_group" => handle_update_group(&params, &groups).await,
            "delete_group" => handle_delete_group(&params, &groups, &messages, &members, &user_groups).await,
            "get_group" => handle_get_group(&params, &groups).await,
            "list_user_groups" => handle_list_user_groups(&params, &user_groups, &groups).await,
            "add_member" => handle_add_member(&params, &members, &groups, &user_groups).await,
            "remove_member" => handle_remove_member(&params, &members, &groups, &user_groups).await,
            "get_members" => handle_get_members(&params, &members).await,
            "link_to_public_account" => handle_link_to_public_account(&params, &groups).await,
            "unlink_from_public_account" => handle_unlink_from_public_account(&params, &groups).await,
            "get_groups_by_public_account" => handle_get_groups_by_public_account(&params, &groups).await,
            "send_message" => handle_send_message(&params, &messages, &groups, &group_sequences).await,
            "get_messages" => handle_get_messages(&params, &messages).await,
            "edit_message" => handle_edit_message(&params, &messages).await,
            "delete_message" => handle_delete_message(&params, &messages).await,
            "create_invitation" => handle_create_invitation(&params, &invitations).await,
            "accept_invitation" => handle_accept_invitation(&params, &invitations).await,
            "reject_invitation" => handle_reject_invitation(&params, &invitations).await,
            "get_pending_invitations" => handle_get_pending_invitations(&params, &invitations).await,
            "update_member_online" => handle_update_member_online(&params, &members).await,
            "search_groups" => handle_search_groups(&params, &groups).await,
            "add_admin" => handle_add_admin(&params, &groups).await,
            "remove_admin" => handle_remove_admin(&params, &groups).await,
            "is_admin" => handle_is_admin(&params, &groups).await,
            "is_owner" => handle_is_owner(&params, &groups).await,
            "has_permission" => handle_has_permission(&params, &groups).await,
            "node_info" => handle_node_info(&node_id).await,
            // Group chat reliability methods
            "group_update_sequence" => handle_group_update_sequence(&params, &group_user_sequences).await,
            "group_get_sequence" => handle_group_get_sequence(&params, &group_user_sequences).await,
            "group_detect_missing" => handle_group_detect_missing(&params, &group_sequences, &group_user_sequences).await,
            "group_get_messages_by_sequence" => handle_group_get_messages_by_sequence(&params, &messages).await,
            "group_create_receipt" => handle_group_create_receipt(&params, &group_receipts).await,
            "group_get_receipts" => handle_group_get_receipts(&params, &group_receipts).await,
            "group_verify_message" => handle_group_verify_message(&params, &messages).await,
            // Message resend methods
            "request_resend" => handle_request_resend(&params, &message_resend).await,
            "detect_missing" => handle_detect_missing(&params, &message_resend).await,
            // 1-to-1 chat methods
            "direct_chat_create_or_get" => handle_direct_chat_create_or_get(&params, &direct_chats).await,
            "direct_send_message" => handle_direct_send_message(&params, &direct_chats, &direct_messages, &sender_sequences).await,
            "direct_get_messages" => handle_direct_get_messages(&params, &direct_messages).await,
            "direct_edit_message" => handle_direct_edit_message(&params, &direct_messages).await,
            "direct_delete_message" => handle_direct_delete_message(&params, &direct_messages).await,
            "direct_get_chat" => handle_direct_get_chat(&params, &direct_chats).await,
            "direct_list_chats" => handle_direct_list_chats(&params, &direct_chats).await,
            "direct_update_sequence" => handle_direct_update_sequence(&params, &user_sequences).await,
            "direct_get_sequence" => handle_direct_get_sequence(&params, &user_sequences).await,
            "direct_detect_missing" => handle_direct_detect_missing(&params, &sender_sequences, &user_sequences).await,
            "direct_get_messages_by_sequence" => handle_direct_get_messages_by_sequence(&params, &direct_messages).await,
            "direct_create_receipt" => handle_direct_create_receipt(&params, &receipts).await,
            "direct_get_receipts" => handle_direct_get_receipts(&params, &receipts).await,
            "direct_verify_message" => handle_direct_verify_message(&params, &direct_messages).await,
            _ => Err(format!("Unknown method: {}", method)),
        };

        let response: serde_json::Value = if let Ok(res) = result {
            json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -1, "message": result.unwrap_err()},
                "id": id
            })
        };

        let response_bytes = response.to_string();
        writer.write_all(response_bytes.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        line.clear();
    }

    Ok(())
}

async fn handle_create_group(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
    user_groups: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    group_sequences: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<serde_json::Value, String> {
    let group: GroupChat = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid group: {}", e))?;
    
    let group_id = group.group_id.clone();
    let owner_id = group.owner_id.clone();
    
    // Initialize group sequence to 0
    let mut seq_map = group_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    seq_map.insert(group_id.clone(), 0);
    drop(seq_map);
    
    let mut guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(group_id.clone(), group.clone());
    drop(guard);

    let mut user_guard = user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    user_guard.entry(owner_id).or_insert_with(Vec::new).push(group_id.clone());

    Ok(json!({
        "group_id": group_id
    }))
}

async fn handle_update_group(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group: GroupChat = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid group: {}", e))?;
    let requester_id = params.get("requester_id").and_then(|r| r.as_str()).ok_or("Missing requester_id")?;
    let group_id = group.group_id.clone();
    
    // Check if requester has permission
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let existing_group = groups_guard.get(&group_id).ok_or("Group not found")?;
    let is_admin = existing_group.admin_ids.iter().any(|id| id == requester_id) || existing_group.owner_id == requester_id;
    drop(groups_guard);
    
    if !is_admin {
        return Err("Only owner or admin can update group".to_string());
    }
    
    let mut guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(group_id, group);

    Ok(json!({
        "updated": true
    }))
}

async fn handle_delete_group(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
    members: &Arc<Mutex<HashMap<String, Vec<GroupMember>>>>,
    user_groups: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let requester_id = params.get("requester_id").and_then(|r| r.as_str()).ok_or("Missing requester_id")?;
    
    // Check if requester is owner
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    if group.owner_id != requester_id {
        return Err("Only owner can delete group".to_string());
    }
    drop(groups_guard);
    
    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.remove(group_id);
    drop(groups_guard);

    if let Some(group) = group {
        let mut messages_guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        messages_guard.remove(group_id);
        drop(messages_guard);

        let mut members_guard = members.lock().map_err(|e| format!("Lock error: {}", e))?;
        members_guard.remove(group_id);
        drop(members_guard);

        let mut user_guard = user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        for member_id in &group.member_ids {
            if let Some(group_ids) = user_guard.get_mut(member_id) {
                group_ids.retain(|id| id != group_id);
                if group_ids.is_empty() {
                    user_guard.remove(member_id);
                }
            }
        }
    }

    Ok(json!({
        "deleted": true
    }))
}

async fn handle_get_group(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(group_id)
        .map(|g| json!(g))
        .ok_or_else(|| "Group not found".to_string())
}

async fn handle_list_user_groups(
    params: &serde_json::Value,
    user_groups: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let user_guard = user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(group_ids) = user_guard.get(user_id) {
        let group_list: Vec<GroupChat> = group_ids.iter()
            .filter_map(|id| groups_guard.get(id).cloned())
            .collect();
        return Ok(json!({ "groups": group_list }));
    }

    Ok(json!({
        "groups": Vec::<GroupChat>::new()
    }))
}

async fn handle_add_member(
    params: &serde_json::Value,
    members: &Arc<Mutex<HashMap<String, Vec<GroupMember>>>>,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
    user_groups: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let member: GroupMember = serde_json::from_value(params.get("member").cloned().unwrap_or(serde_json::Value::Null))
        .map_err(|e| format!("Invalid member: {}", e))?;
    
    let agent_id = member.agent_id.clone();
    
    let mut members_guard = members.lock().map_err(|e| format!("Lock error: {}", e))?;
    members_guard.entry(group_id.to_string()).or_insert_with(Vec::new).push(member.clone());
    drop(members_guard);

    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = groups_guard.get_mut(group_id) {
        if !group.member_ids.contains(&agent_id) {
            group.member_ids.push(agent_id.clone());
        }
        group.last_activity = current_timestamp();
    }
    drop(groups_guard);

    let mut user_guard = user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    user_guard.entry(agent_id).or_insert_with(Vec::new).push(group_id.to_string());

    Ok(json!({
        "added": true
    }))
}

async fn handle_remove_member(
    params: &serde_json::Value,
    members: &Arc<Mutex<HashMap<String, Vec<GroupMember>>>>,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
    user_groups: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    let requester_id = params.get("requester_id").and_then(|r| r.as_str()).ok_or("Missing requester_id")?;
    
    // Check if requester has permission
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    let is_admin = group.admin_ids.iter().any(|id| id == requester_id) || group.owner_id == requester_id;
    drop(groups_guard);
    
    if !is_admin {
        return Err("Only owner or admin can remove members".to_string());
    }

    let mut members_guard = members.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(member_list) = members_guard.get_mut(group_id) {
        member_list.retain(|m| m.agent_id != agent_id);
    }
    drop(members_guard);

    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = groups_guard.get_mut(group_id) {
        group.member_ids.retain(|id| id != agent_id);
        group.last_activity = current_timestamp();
    }
    drop(groups_guard);

    let mut user_guard = user_groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group_ids) = user_guard.get_mut(agent_id) {
        group_ids.retain(|id| id != group_id);
        if group_ids.is_empty() {
            user_guard.remove(agent_id);
        }
    }

    Ok(json!({
        "removed": true
    }))
}

async fn handle_add_admin(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let requester_id = params.get("requester_id").and_then(|r| r.as_str()).ok_or("Missing requester_id")?;
    
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    
    // Check if requester is owner
    if group.owner_id != requester_id {
        return Err("Only owner can add admins".to_string());
    }
    drop(groups_guard);
    
    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = groups_guard.get_mut(group_id) {
        if !group.admin_ids.iter().any(|id| id == user_id) {
            group.admin_ids.push(user_id.to_string());
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_remove_admin(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let requester_id = params.get("requester_id").and_then(|r| r.as_str()).ok_or("Missing requester_id")?;
    
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    
    // Check if requester is owner
    if group.owner_id != requester_id {
        return Err("Only owner can remove admins".to_string());
    }
    drop(groups_guard);
    
    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = groups_guard.get_mut(group_id) {
        group.admin_ids.retain(|id| id != user_id);
    }

    Ok(json!({
        "removed": true
    }))
}

async fn handle_is_admin(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    let is_admin = group.admin_ids.iter().any(|id| id == user_id) || group.owner_id == user_id;

    Ok(json!({
        "is_admin": is_admin
    }))
}

async fn handle_is_owner(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    let is_owner = group.owner_id == user_id;

    Ok(json!({
        "is_owner": is_owner
    }))
}

async fn handle_has_permission(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group = groups_guard.get(group_id).ok_or("Group not found")?;
    let has_permission = group.admin_ids.iter().any(|id| id == user_id) || group.owner_id == user_id;

    Ok(json!({
        "has_permission": has_permission
    }))
}

async fn handle_get_members(
    params: &serde_json::Value,
    members: &Arc<Mutex<HashMap<String, Vec<GroupMember>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let guard = members.lock().map_err(|e| format!("Lock error: {}", e))?;
    let member_list = guard.get(group_id).cloned().unwrap_or_default();

    Ok(json!({
        "members": member_list
    }))
}

async fn handle_link_to_public_account(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let mut guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = guard.get_mut(group_id) {
        group.public_account_id = Some(account_id.to_string());
    }

    Ok(json!({
        "linked": true
    }))
}

async fn handle_unlink_from_public_account(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let mut guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = guard.get_mut(group_id) {
        group.public_account_id = None;
    }

    Ok(json!({
        "unlinked": true
    }))
}

async fn handle_get_groups_by_public_account(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<GroupChat> = guard.values()
        .filter(|g| g.public_account_id.as_ref() == Some(&account_id.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "groups": found
    }))
}

async fn handle_send_message(
    params: &serde_json::Value,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
    group_sequences: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<serde_json::Value, String> {
    let mut message: GroupMessage = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid message: {}", e))?;
    
    let group_id = message.group_id.clone();
    let message_id = message.message_id.clone();
    
    // Generate sequence number for this group
    let mut seq_map = group_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    let current_seq = seq_map.get(&group_id).copied().unwrap_or(0);
    let next_seq = if current_seq >= MAX_SEQUENCE { 1 } else { current_seq + 1 };
    seq_map.insert(group_id.clone(), next_seq);
    message.sequence = next_seq;
    drop(seq_map);
    
    // Generate integrity hash for group message
    let integrity_hash = generate_group_integrity_hash(
        &message.group_id,
        &message.sender_id,
        &message.content,
        next_seq,
        message.timestamp,
    );
    message.integrity_hash = Some(integrity_hash.clone());
    
    let mut messages_guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    messages_guard.entry(group_id.clone()).or_insert_with(Vec::new).push(message.clone());
    
    // Keep only last 1000 messages per group
    if let Some(msg_list) = messages_guard.get_mut(&group_id) {
        let len = msg_list.len();
        if len > 1000 {
            msg_list.drain(0..len - 1000);
        }
    }
    drop(messages_guard);

    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(group) = groups_guard.get_mut(&group_id) {
        group.message_count += 1;
        group.last_activity = current_timestamp();
        group.last_sequence = next_seq;
    }

    Ok(json!({
        "message_id": message_id,
        "sequence": next_seq,
        "integrity_hash": integrity_hash
    }))
}

async fn handle_get_messages(
    params: &serde_json::Value,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    let before = params.get("before").and_then(|b| b.as_u64());
    
    let guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    let limit = limit.unwrap_or(50);
    
    if let Some(msg_list) = guard.get(group_id) {
        let mut filtered: Vec<GroupMessage> = msg_list.clone();
        
        if let Some(before_ts) = before {
            filtered.retain(|m| m.timestamp < before_ts);
        }
        
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        filtered.truncate(limit);

        Ok(json!({
            "messages": filtered
        }))
    } else {
        Ok(json!({
            "messages": Vec::<GroupMessage>::new()
        }))
    }
}

async fn handle_edit_message(
    params: &serde_json::Value,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    let new_content = params.get("new_content").and_then(|c| c.as_str()).ok_or("Missing new_content")?;
    
    let mut guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = guard.get_mut(group_id) {
        if let Some(msg) = msg_list.iter_mut().find(|m| m.message_id == message_id) {
            msg.content = new_content.to_string();
            msg.is_edited = true;
            msg.edited_at = Some(current_timestamp());
        }
    }

    Ok(json!({
        "edited": true
    }))
}

async fn handle_delete_message(
    params: &serde_json::Value,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    
    let mut guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = guard.get_mut(group_id) {
        msg_list.retain(|m| m.message_id != message_id);
    }

    Ok(json!({
        "deleted": true
    }))
}

async fn handle_create_invitation(
    params: &serde_json::Value,
    invitations: &Arc<Mutex<HashMap<String, GroupInvitation>>>,
) -> Result<serde_json::Value, String> {
    let invitation: GroupInvitation = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid invitation: {}", e))?;
    
    let invitation_id = invitation.invitation_id.clone();
    let mut guard = invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(invitation_id, invitation);

    Ok(json!({
        "created": true
    }))
}

async fn handle_accept_invitation(
    params: &serde_json::Value,
    invitations: &Arc<Mutex<HashMap<String, GroupInvitation>>>,
) -> Result<serde_json::Value, String> {
    let invitation_id = params.get("invitation_id").and_then(|i| i.as_str()).ok_or("Missing invitation_id")?;
    
    let mut guard = invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(invitation) = guard.get_mut(invitation_id) {
        invitation.status = "accepted".to_string();
    }

    Ok(json!({
        "accepted": true
    }))
}

async fn handle_reject_invitation(
    params: &serde_json::Value,
    invitations: &Arc<Mutex<HashMap<String, GroupInvitation>>>,
) -> Result<serde_json::Value, String> {
    let invitation_id = params.get("invitation_id").and_then(|i| i.as_str()).ok_or("Missing invitation_id")?;
    
    let mut guard = invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(invitation) = guard.get_mut(invitation_id) {
        invitation.status = "rejected".to_string();
    }

    Ok(json!({
        "rejected": true
    }))
}

async fn handle_get_pending_invitations(
    params: &serde_json::Value,
    invitations: &Arc<Mutex<HashMap<String, GroupInvitation>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let guard = invitations.lock().map_err(|e| format!("Lock error: {}", e))?;
    let pending: Vec<GroupInvitation> = guard.values()
        .filter(|i| i.invitee_id == user_id && i.status == "pending")
        .cloned()
        .collect();

    Ok(json!({
        "invitations": pending
    }))
}

async fn handle_update_member_online(
    params: &serde_json::Value,
    members: &Arc<Mutex<HashMap<String, Vec<GroupMember>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    let is_online = params.get("is_online").and_then(|i| i.as_bool()).ok_or("Missing is_online")?;
    
    let mut guard = members.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(member_list) = guard.get_mut(group_id) {
        if let Some(member) = member_list.iter_mut().find(|m| m.agent_id == agent_id) {
            member.is_online = is_online;
            member.last_seen = current_timestamp();
        }
    }

    Ok(json!({
        "updated": true
    }))
}

async fn handle_search_groups(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, GroupChat>>>,
) -> Result<serde_json::Value, String> {
    let query = params.get("query").and_then(|q| q.as_str()).ok_or("Missing query")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let query_lower = query.to_lowercase();
    
    let mut results: Vec<GroupChat> = guard.values()
        .filter(|g| {
            g.name.to_lowercase().contains(&query_lower) ||
            g.description.to_lowercase().contains(&query_lower)
        })
        .filter(|g| !g.is_private)
        .cloned()
        .collect();

    results.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    
    if let Some(limit) = limit {
        results.truncate(limit);
    }

    Ok(json!({
        "groups": results
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("group_chat_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

// ========== 1-to-1 Chat Handlers ==========

async fn handle_direct_chat_create_or_get(
    params: &serde_json::Value,
    direct_chats: &Arc<Mutex<HashMap<String, DirectChat>>>,
) -> Result<serde_json::Value, String> {
    let user_a = params.get("user_a").and_then(|u| u.as_str()).ok_or("Missing user_a")?;
    let user_b = params.get("user_b").and_then(|u| u.as_str()).ok_or("Missing user_b")?;
    
    let mut ids = vec![user_a.to_string(), user_b.to_string()];
    ids.sort();
    let chat_id = format!("dm-{}-{}", ids[0], ids[1]);
    
    let mut guard = direct_chats.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(chat) = guard.get(&chat_id) {
        return Ok(json!(chat));
    }
    
    let chat = DirectChat {
        chat_id: chat_id.clone(),
        user_a: ids[0].clone(),
        user_b: ids[1].clone(),
        created_at: current_timestamp(),
        last_activity: current_timestamp(),
        message_count: 0,
    };
    
    guard.insert(chat_id.clone(), chat.clone());
    
    Ok(json!(chat))
}

async fn handle_direct_send_message(
    params: &serde_json::Value,
    direct_chats: &Arc<Mutex<HashMap<String, DirectChat>>>,
    direct_messages: &Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
    sender_sequences: &Arc<Mutex<HashMap<String, u32>>>,
) -> Result<serde_json::Value, String> {
    let message: DirectMessage = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid message: {}", e))?;
    
    let chat_id = message.chat_id.clone();
    let sender_id = message.sender_id.clone();
    
    // Generate sequence number for this sender
    let mut seq_map = sender_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    let current_seq = seq_map.get(&sender_id).copied().unwrap_or(0);
    let next_seq = if current_seq >= MAX_SEQUENCE { 1 } else { current_seq + 1 };
    seq_map.insert(sender_id.clone(), next_seq);
    drop(seq_map);
    
    // Generate integrity hash
    let integrity_hash = generate_integrity_hash(
        &message.sender_id,
        &message.receiver_id,
        &message.content,
        next_seq,
        message.timestamp,
    );
    
    let mut messages = direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    messages.entry(chat_id.clone()).or_insert_with(Vec::new).push(message.clone());
    
    // Keep only last 1000 messages per chat
    if let Some(msg_list) = messages.get_mut(&chat_id) {
        let len = msg_list.len();
        if len > 1000 {
            msg_list.drain(0..len - 1000);
        }
    }
    drop(messages);

    let mut chats = direct_chats.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(chat) = chats.get_mut(&chat_id) {
        chat.message_count += 1;
        chat.last_activity = current_timestamp();
    }

    Ok(json!({
        "message_id": message.message_id,
        "sequence": next_seq,
        "integrity_hash": integrity_hash
    }))
}

async fn handle_direct_get_messages(
    params: &serde_json::Value,
    direct_messages: &Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
) -> Result<serde_json::Value, String> {
    let chat_id = params.get("chat_id").and_then(|c| c.as_str()).ok_or("Missing chat_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    let before = params.get("before").and_then(|b| b.as_u64());
    
    let messages = direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    let limit = limit.unwrap_or(50);
    
    if let Some(msg_list) = messages.get(chat_id) {
        let mut filtered: Vec<DirectMessage> = msg_list.clone();
        
        if let Some(before_ts) = before {
            filtered.retain(|m| m.timestamp < before_ts);
        }
        
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        filtered.truncate(limit);
        
        Ok(json!({ "messages": filtered }))
    } else {
        Ok(json!({ "messages": Vec::<DirectMessage>::new() }))
    }
}

async fn handle_direct_edit_message(
    params: &serde_json::Value,
    direct_messages: &Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
) -> Result<serde_json::Value, String> {
    let chat_id = params.get("chat_id").and_then(|c| c.as_str()).ok_or("Missing chat_id")?;
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    let new_content = params.get("new_content").and_then(|c| c.as_str()).ok_or("Missing new_content")?;
    
    let mut messages = direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = messages.get_mut(chat_id) {
        if let Some(msg) = msg_list.iter_mut().find(|m| m.message_id == message_id) {
            msg.content = new_content.to_string();
            msg.is_edited = true;
            msg.edited_at = Some(current_timestamp());
        }
    }
    
    Ok(json!({ "edited": true }))
}

async fn handle_direct_delete_message(
    params: &serde_json::Value,
    direct_messages: &Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
) -> Result<serde_json::Value, String> {
    let chat_id = params.get("chat_id").and_then(|c| c.as_str()).ok_or("Missing chat_id")?;
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    
    let mut messages = direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = messages.get_mut(chat_id) {
        msg_list.retain(|m| m.message_id != message_id);
    }
    
    Ok(json!({ "deleted": true }))
}

async fn handle_direct_get_chat(
    params: &serde_json::Value,
    direct_chats: &Arc<Mutex<HashMap<String, DirectChat>>>,
) -> Result<serde_json::Value, String> {
    let chat_id = params.get("chat_id").and_then(|c| c.as_str()).ok_or("Missing chat_id")?;
    
    let guard = direct_chats.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(chat_id)
        .map(|c| json!(c))
        .ok_or_else(|| "Chat not found".to_string())
}

async fn handle_direct_list_chats(
    params: &serde_json::Value,
    direct_chats: &Arc<Mutex<HashMap<String, DirectChat>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let guard = direct_chats.lock().map_err(|e| format!("Lock error: {}", e))?;
    let chats: Vec<DirectChat> = guard.values()
        .filter(|c| c.user_a == user_id || c.user_b == user_id)
        .cloned()
        .collect();
    
    Ok(json!({ "chats": chats }))
}

async fn handle_direct_update_sequence(
    params: &serde_json::Value,
    user_sequences: &Arc<Mutex<HashMap<String, UserSequence>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let sender_id = params.get("sender_id").and_then(|s| s.as_str()).ok_or("Missing sender_id")?;
    let last_sequence = params.get("last_sequence").and_then(|l| l.as_u64()).ok_or("Missing last_sequence")? as u32;
    
    let key = format!("{}:{}", user_id, sender_id);
    let mut sequences = user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    sequences.insert(key, UserSequence {
        user_id: user_id.to_string(),
        sender_id: sender_id.to_string(),
        last_sequence,
        updated_at: current_timestamp(),
    });
    
    Ok(json!({ "updated": true }))
}

async fn handle_direct_get_sequence(
    params: &serde_json::Value,
    user_sequences: &Arc<Mutex<HashMap<String, UserSequence>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let sender_id = params.get("sender_id").and_then(|s| s.as_str()).ok_or("Missing sender_id")?;
    
    let key = format!("{}:{}", user_id, sender_id);
    let sequences = user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    sequences.get(&key)
        .map(|s| json!(s))
        .ok_or_else(|| "Sequence not found".to_string())
}

async fn handle_direct_detect_missing(
    params: &serde_json::Value,
    sender_sequences: &Arc<Mutex<HashMap<String, u32>>>,
    user_sequences: &Arc<Mutex<HashMap<String, UserSequence>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let sender_id = params.get("sender_id").and_then(|s| s.as_str()).ok_or("Missing sender_id")?;
    
    let seqs = sender_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    let current_seq = seqs.get(sender_id).copied().unwrap_or(0);
    drop(seqs);
    
    let user_seqs = user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    let key = format!("{}:{}", user_id, sender_id);
    let user_seq = user_seqs.get(&key).map(|u| u.last_sequence).unwrap_or(0);
    
    if current_seq > user_seq {
        Ok(json!({
            "missing": true,
            "start_sequence": user_seq + 1,
            "end_sequence": current_seq,
            "missing_count": current_seq - user_seq
        }))
    } else if user_seq > 0 && current_seq < user_seq {
        // Sequence cycled
        Ok(json!({
            "missing": true,
            "start_sequence": user_seq + 1,
            "end_sequence": MAX_SEQUENCE,
            "missing_count": MAX_SEQUENCE - user_seq
        }))
    } else {
        Ok(json!({
            "missing": false,
            "start_sequence": null,
            "end_sequence": null,
            "missing_count": 0
        }))
    }
}

async fn handle_direct_get_messages_by_sequence(
    params: &serde_json::Value,
    direct_messages: &Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
) -> Result<serde_json::Value, String> {
    let sender_id = params.get("sender_id").and_then(|s| s.as_str()).ok_or("Missing sender_id")?;
    let start_seq = params.get("start_sequence").and_then(|s| s.as_u64()).ok_or("Missing start_sequence")? as u32;
    let end_seq = params.get("end_sequence").and_then(|e| e.as_u64()).ok_or("Missing end_sequence")? as u32;
    
    let messages = direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    let filtered: Vec<DirectMessage> = messages.values()
        .flat_map(|msg_list| msg_list.iter())
        .filter(|m| m.sender_id == sender_id)
        .filter(|m| m.sequence >= start_seq && m.sequence <= end_seq)
        .cloned()
        .collect();
    
    Ok(json!({ "messages": filtered }))
}

async fn handle_direct_create_receipt(
    params: &serde_json::Value,
    receipts: &Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>,
) -> Result<serde_json::Value, String> {
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    let receiver_id = params.get("receiver_id").and_then(|r| r.as_str()).ok_or("Missing receiver_id")?;
    let sequence = params.get("sequence").and_then(|s| s.as_u64()).ok_or("Missing sequence")? as u32;
    
    let receipt_id = format!("receipt-{}", uuid::Uuid::new_v4());
    let receipt = MessageReceipt {
        receipt_id: receipt_id.clone(),
        message_id: message_id.to_string(),
        receiver_id: receiver_id.to_string(),
        sequence,
        received_at: current_timestamp(),
    };
    
    let mut receipts_guard = receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
    receipts_guard.entry(message_id.to_string()).or_insert_with(Vec::new).push(receipt.clone());
    
    Ok(json!(receipt))
}

async fn handle_direct_get_receipts(
    params: &serde_json::Value,
    receipts: &Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>,
) -> Result<serde_json::Value, String> {
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    
    let receipts_guard = receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let receipt_list = receipts_guard.get(message_id).cloned().unwrap_or_default();
    
    Ok(json!({ "receipts": receipt_list }))
}

async fn handle_direct_verify_message(
    params: &serde_json::Value,
    direct_messages: &Arc<Mutex<HashMap<String, Vec<DirectMessage>>>>,
) -> Result<serde_json::Value, String> {
    let chat_id = params.get("chat_id").and_then(|c| c.as_str()).ok_or("Missing chat_id")?;
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    
    let messages = direct_messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = messages.get(chat_id) {
        if let Some(msg) = msg_list.iter().find(|m| m.message_id == message_id) {
            return Ok(json!({
                "valid": verify_integrity_hash(msg),
                "integrity_hash": msg.integrity_hash
            }));
        }
    }
    
    Ok(json!({
        "valid": false,
        "integrity_hash": null
    }))
}

// ========== Group Chat Reliability Handlers ==========

async fn handle_group_update_sequence(
    params: &serde_json::Value,
    group_user_sequences: &Arc<Mutex<HashMap<String, GroupSequence>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let last_sequence = params.get("last_sequence").and_then(|l| l.as_u64()).ok_or("Missing last_sequence")? as u32;
    
    let key = format!("{}:{}", user_id, group_id);
    let mut sequences = group_user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    sequences.insert(key, GroupSequence {
        user_id: user_id.to_string(),
        group_id: group_id.to_string(),
        last_sequence,
        updated_at: current_timestamp(),
    });
    
    Ok(json!({ "updated": true }))
}

async fn handle_group_get_sequence(
    params: &serde_json::Value,
    group_user_sequences: &Arc<Mutex<HashMap<String, GroupSequence>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let key = format!("{}:{}", user_id, group_id);
    let sequences = group_user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    sequences.get(&key)
        .map(|s| json!(s))
        .ok_or_else(|| "Sequence not found".to_string())
}

async fn handle_group_detect_missing(
    params: &serde_json::Value,
    group_sequences: &Arc<Mutex<HashMap<String, u32>>>,
    group_user_sequences: &Arc<Mutex<HashMap<String, GroupSequence>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let seqs = group_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    let current_seq = seqs.get(group_id).copied().unwrap_or(0);
    drop(seqs);
    
    let user_seqs = group_user_sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
    let key = format!("{}:{}", user_id, group_id);
    let user_seq = user_seqs.get(&key).map(|u| u.last_sequence).unwrap_or(0);
    
    if current_seq > user_seq {
        Ok(json!({
            "missing": true,
            "start_sequence": user_seq + 1,
            "end_sequence": current_seq,
            "missing_count": current_seq - user_seq
        }))
    } else if user_seq > 0 && current_seq < user_seq {
        // Sequence cycled
        Ok(json!({
            "missing": true,
            "start_sequence": user_seq + 1,
            "end_sequence": MAX_SEQUENCE,
            "missing_count": MAX_SEQUENCE - user_seq
        }))
    } else {
        Ok(json!({
            "missing": false,
            "start_sequence": null,
            "end_sequence": null,
            "missing_count": 0
        }))
    }
}

async fn handle_group_get_messages_by_sequence(
    params: &serde_json::Value,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let start_seq = params.get("start_sequence").and_then(|s| s.as_u64()).ok_or("Missing start_sequence")? as u32;
    let end_seq = params.get("end_sequence").and_then(|e| e.as_u64()).ok_or("Missing end_sequence")? as u32;
    
    let messages_guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = messages_guard.get(group_id) {
        let filtered: Vec<GroupMessage> = msg_list.iter()
            .filter(|m| m.sequence >= start_seq && m.sequence <= end_seq)
            .cloned()
            .collect();
        Ok(json!({ "messages": filtered }))
    } else {
        Ok(json!({ "messages": Vec::<GroupMessage>::new() }))
    }
}

async fn handle_group_create_receipt(
    params: &serde_json::Value,
    group_receipts: &Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>,
) -> Result<serde_json::Value, String> {
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    let receiver_id = params.get("receiver_id").and_then(|r| r.as_str()).ok_or("Missing receiver_id")?;
    let sequence = params.get("sequence").and_then(|s| s.as_u64()).ok_or("Missing sequence")? as u32;
    
    let receipt_id = format!("receipt-{}", uuid::Uuid::new_v4());
    let receipt = MessageReceipt {
        receipt_id: receipt_id.clone(),
        message_id: message_id.to_string(),
        receiver_id: receiver_id.to_string(),
        sequence,
        received_at: current_timestamp(),
    };
    
    let mut receipts_guard = group_receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
    receipts_guard.entry(message_id.to_string()).or_insert_with(Vec::new).push(receipt.clone());
    
    Ok(json!(receipt))
}

async fn handle_group_get_receipts(
    params: &serde_json::Value,
    group_receipts: &Arc<Mutex<HashMap<String, Vec<MessageReceipt>>>>,
) -> Result<serde_json::Value, String> {
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    
    let receipts_guard = group_receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let receipt_list = receipts_guard.get(message_id).cloned().unwrap_or_default();
    
    Ok(json!({ "receipts": receipt_list }))
}

async fn handle_group_verify_message(
    params: &serde_json::Value,
    messages: &Arc<Mutex<HashMap<String, Vec<GroupMessage>>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    let message_id = params.get("message_id").and_then(|m| m.as_str()).ok_or("Missing message_id")?;
    
    let messages_guard = messages.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(msg_list) = messages_guard.get(group_id) {
        if let Some(msg) = msg_list.iter().find(|m| m.message_id == message_id) {
            return Ok(json!({
                "valid": verify_group_integrity_hash(msg),
                "integrity_hash": msg.integrity_hash
            }));
        }
    }
    
    Ok(json!({
        "valid": false,
        "integrity_hash": null
    }))
}

// ========== Message Resend Handlers ==========

async fn handle_request_resend(
    params: &serde_json::Value,
    message_resend: &Arc<MessageResendService>,
) -> Result<serde_json::Value, String> {
    let requester_id = params.get("requester_id").and_then(|r| r.as_str()).ok_or("Missing requester_id")?;
    let target_id = params.get("target_id").and_then(|t| t.as_str()).ok_or("Missing target_id")?;
    let request_type = params.get("request_type").and_then(|t| t.as_str()).ok_or("Missing request_type")?;
    let start_sequence = params.get("start_sequence").and_then(|s| s.as_u64()).ok_or("Missing start_sequence")? as u32;
    let end_sequence = params.get("end_sequence").and_then(|e| e.as_u64()).ok_or("Missing end_sequence")? as u32;
    
    let request = ResendRequest {
        requester_id: requester_id.to_string(),
        target_id: target_id.to_string(),
        request_type: request_type.to_string(),
        start_sequence,
        end_sequence,
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    let response = if request_type == "direct" {
        message_resend.handle_direct_resend(request)?
    } else if request_type == "group" {
        message_resend.handle_group_resend(request)?
    } else {
        return Err("Invalid request_type, must be 'direct' or 'group'".to_string());
    };
    
    Ok(json!(response))
}

async fn handle_detect_missing(
    params: &serde_json::Value,
    message_resend: &Arc<MessageResendService>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let target_id = params.get("target_id").and_then(|t| t.as_str()).ok_or("Missing target_id")?;
    let detect_type = params.get("detect_type").and_then(|t| t.as_str()).ok_or("Missing detect_type")?;
    
    let missing = if detect_type == "direct" {
        message_resend.detect_missing_direct(user_id, target_id)?
    } else if detect_type == "group" {
        message_resend.detect_missing_group(user_id, target_id)?
    } else {
        return Err("Invalid detect_type, must be 'direct' or 'group'".to_string());
    };
    
    Ok(json!({
        "missing": missing.is_some(),
        "start_sequence": missing.map(|(s, _)| s),
        "end_sequence": missing.map(|(_, e)| e),
    }))
}

#[cfg(all(test, feature = "im-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_group() {
        let config = GroupChatServiceConfig::default();
        let service = GroupChatService::new(config).expect("Failed to create service");
        
        let group = GroupChat {
            group_id: "test-group-1".to_string(),
            name: "Test Group".to_string(),
            description: "A test group".to_string(),
            avatar_url: None,
            owner_id: "user-1".to_string(),
            member_ids: vec!["user-1".to_string()],
            admin_ids: vec!["user-1".to_string()],
            is_private: false,
            created_at: current_timestamp(),
            last_activity: current_timestamp(),
            message_count: 0,
            public_account_id: None,
            assistant_id: None,
            last_sequence: 0,
        };

        service.create_group(group).expect("Failed to create group");
        let retrieved = service.get_group("test-group-1".to_string()).expect("Expected group");
        assert_eq!(retrieved.name, "Test Group");
    }

    #[test]
    fn test_send_and_get_messages() {
        let config = GroupChatServiceConfig::default();
        let service = GroupChatService::new(config).expect("Failed to create service");
        
        let group = GroupChat {
            group_id: "test-group-2".to_string(),
            name: "Test Group 2".to_string(),
            description: "A test group".to_string(),
            avatar_url: None,
            owner_id: "user-1".to_string(),
            member_ids: vec!["user-1".to_string()],
            admin_ids: vec!["user-1".to_string()],
            is_private: false,
            created_at: current_timestamp(),
            last_activity: current_timestamp(),
            message_count: 0,
            public_account_id: None,
            assistant_id: None,
            last_sequence: 0,
        };

        service.create_group(group).expect("Failed to create group");
        
        let message = GroupMessage {
            message_id: "msg-1".to_string(),
            group_id: "test-group-2".to_string(),
            sender_id: "user-1".to_string(),
            sender_name: "User 1".to_string(),
            content: "Hello!".to_string(),
            message_type: "text".to_string(),
            attachments: vec![],
            reply_to: None,
            mentions: vec![],
            timestamp: current_timestamp(),
            is_edited: false,
            edited_at: None,
            integrity_hash: Some("test-hash".to_string()),
            sequence: 1,
        };

        service.send_message(message).expect("Failed to send message");
        let messages = service.get_messages("test-group-2".to_string(), Some(10), None);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello!");
    }
}
