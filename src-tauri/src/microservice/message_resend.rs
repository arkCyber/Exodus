//! Message Resend Service - Automatic resend of missing messages
//!
//! This service handles automatic resend of missing messages when a user
//! requests messages they missed. It checks the SQLite database for
//! missing messages and resends them to the requester.

use std::sync::{Arc, Mutex};
use super::chat_storage::{ChatStorage, GroupMessage as StoredGroupMessage, DirectMessage as StoredDirectMessage};
use serde::{Deserialize, Serialize};
use chrono;

/// Message resend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendRequest {
    pub requester_id: String,
    pub target_id: String, // For direct: sender_id, For group: group_id
    pub request_type: String, // "direct" or "group"
    pub start_sequence: u32,
    pub end_sequence: u32,
    pub timestamp: i64,
}

/// Message resend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendResponse {
    pub request_id: String,
    pub messages: Vec<ResentMessage>,
    pub total_count: usize,
    pub timestamp: i64,
}

/// Resent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResentMessage {
    pub message_id: String,
    pub content: String,
    pub sequence: u32,
    pub timestamp: i64,
    pub sender_id: String,
    pub integrity_hash: Option<String>,
}

/// Message resend service
pub struct MessageResendService {
    chat_storage: Arc<ChatStorage>,
}

impl MessageResendService {
    pub fn new(chat_storage: Arc<ChatStorage>) -> Self {
        Self { chat_storage }
    }

    /// Handle resend request for direct messages
    pub fn handle_direct_resend(&self, request: ResendRequest) -> Result<ResendResponse, String> {
        let messages = self.chat_storage.get_direct_messages_by_sequence(
            &request.requester_id,
            &request.target_id,
            request.start_sequence,
            request.end_sequence,
        ).map_err(|e| format!("Failed to get direct messages: {}", e))?;

        let resent_messages: Vec<ResentMessage> = messages.iter().map(|m| ResentMessage {
            message_id: m.message_id.clone(),
            content: m.content.clone(),
            sequence: m.sequence,
            timestamp: m.timestamp,
            sender_id: m.sender_id.clone(),
            integrity_hash: m.integrity_hash.clone(),
        }).collect();
        
        let total_count = resent_messages.len();
        Ok(ResendResponse {
            request_id: format!("resend-{}", uuid::Uuid::new_v4()),
            messages: resent_messages,
            total_count,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// Handle resend request for group messages
    pub fn handle_group_resend(&self, request: ResendRequest) -> Result<ResendResponse, String> {
        let messages = self.chat_storage.get_group_messages_by_sequence(
            &request.requester_id,
            &request.target_id,
            request.start_sequence,
            request.end_sequence,
        ).map_err(|e| format!("Failed to get group messages: {}", e))?;

        let resent_messages: Vec<ResentMessage> = messages.iter().map(|m| ResentMessage {
            message_id: m.message_id.clone(),
            content: m.content.clone(),
            sequence: m.sequence,
            timestamp: m.timestamp,
            sender_id: m.sender_id.clone(),
            integrity_hash: m.integrity_hash.clone(),
        }).collect();
        
        let total_count = resent_messages.len();
        Ok(ResendResponse {
            request_id: format!("resend-{}", uuid::Uuid::new_v4()),
            messages: resent_messages,
            total_count,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// Detect missing messages for a user
    pub fn detect_missing_direct(&self, user_id: &str, sender_id: &str) -> Result<Option<(u32, u32)>, String> {
        // Get current sequence from sender
        let current_seq = self.chat_storage.get_sequence(user_id, sender_id, "direct")
            .map_err(|e| format!("Failed to get sequence: {}", e))?;
        
        if let Some(current) = current_seq {
            let user_seq = self.chat_storage.get_sequence(user_id, sender_id, "direct")
                .map_err(|e| format!("Failed to get sequence: {}", e))?
                .unwrap_or(0);
            
            if current > user_seq {
                return Ok(Some((user_seq + 1, current)));
            }
        }
        
        Ok(None)
    }

    /// Detect missing messages for a group
    pub fn detect_missing_group(&self, user_id: &str, group_id: &str) -> Result<Option<(u32, u32)>, String> {
        let current_seq = self.chat_storage.get_sequence(user_id, group_id, "group")
            .map_err(|e| format!("Failed to get sequence: {}", e))?;
        
        if let Some(current) = current_seq {
            let user_seq = self.chat_storage.get_sequence(user_id, group_id, "group")
                .map_err(|e| format!("Failed to get sequence: {}", e))?
                .unwrap_or(0);
            
            if current > user_seq {
                return Ok(Some((user_seq + 1, current)));
            }
        }
        
        Ok(None)
    }

    /// Auto-request missing messages when user comes online
    pub fn auto_request_missing(&self, _user_id: &str) -> Result<Vec<ResendRequest>, String> {
        let requests = Vec::new();
        
        // This would be called when a user comes online
        // Check all direct chats and group chats for missing messages
        // For now, this is a placeholder that would be implemented
        // by iterating through user's contacts and groups
        
        Ok(requests)
    }
}
