//! Group Assistant Service - Cloud-based relay for group messages
//!
//! This service provides a cloud-based "group assistant" that acts as a fallback
//! for group message delivery. It stores messages, sends receipts, and can
//! resend missing messages when users are offline.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use uuid::Uuid;

/// Group assistant configuration
#[derive(Debug, Clone)]
pub struct GroupAssistantConfig {
    pub storage_dir: std::path::PathBuf,
}

impl Default for GroupAssistantConfig {
    fn default() -> Self {
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_group_assistants");
        Self { storage_dir }
    }
}

/// Group assistant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAssistant {
    pub assistant_id: String,
    pub group_id: String,
    pub created_at: i64,
    pub last_activity: i64,
    pub message_count: u32,
    pub is_active: bool,
}

/// Assistant message storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub message_id: String,
    pub group_id: String,
    pub sender_id: String,
    pub content: String,
    pub sequence: u32,
    pub timestamp: i64,
    pub integrity_hash: Option<String>,
    pub stored_at: i64,
}

/// Group assistant service
pub struct GroupAssistantService {
    config: GroupAssistantConfig,
    // Per-assistant storage
    assistants: Arc<Mutex<HashMap<String, GroupAssistant>>>,
    // Messages stored by assistant
    messages: Arc<Mutex<HashMap<String, Vec<AssistantMessage>>>>, // assistant_id -> messages
    // Sequence tracking per group
    sequences: Arc<Mutex<HashMap<String, u32>>>, // group_id -> current sequence
    // Receipt tracking
    receipts: Arc<Mutex<HashMap<String, Vec<String>>>>, // message_id -> receiver_ids
}

impl GroupAssistantService {
    pub fn new(config: GroupAssistantConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        
        Ok(Self {
            config,
            assistants: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            sequences: Arc::new(Mutex::new(HashMap::new())),
            receipts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create or get assistant for a group
    pub fn get_or_create_assistant(&self, group_id: &str) -> Result<GroupAssistant, String> {
        let mut assistants = self.assistants.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // Check if assistant already exists
        if let Some(assistant) = assistants.get(group_id) {
            return Ok(assistant.clone());
        }
        
        // Create new assistant
        let assistant_id = format!("assistant-{}", Uuid::new_v4());
        let assistant = GroupAssistant {
            assistant_id: assistant_id.clone(),
            group_id: group_id.to_string(),
            created_at: Utc::now().timestamp(),
            last_activity: Utc::now().timestamp(),
            message_count: 0,
            is_active: true,
        };
        
        assistants.insert(group_id.to_string(), assistant.clone());
        
        // Initialize message storage for this assistant
        let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        messages.insert(assistant_id, Vec::new());
        
        Ok(assistant)
    }

    /// Store message in assistant
    pub fn store_message(&self, group_id: &str, message: AssistantMessage) -> Result<(), String> {
        let assistant = self.get_or_create_assistant(group_id)?;
        
        // Store message
        let mut messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get_mut(&assistant.assistant_id) {
            msg_list.push(message.clone());
            
            // Keep only last 10000 messages
            if msg_list.len() > 10000 {
                msg_list.drain(0..msg_list.len() - 10000);
            }
        }
        
        // Update sequence
        let mut sequences = self.sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        sequences.insert(group_id.to_string(), message.sequence);
        
        // Update assistant activity
        let mut assistants = self.assistants.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(ast) = assistants.get_mut(group_id) {
            ast.last_activity = Utc::now().timestamp();
            ast.message_count += 1;
        }
        
        Ok(())
    }

    /// Get messages from assistant by sequence range
    pub fn get_messages(&self, group_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<AssistantMessage>, String> {
        let assistant = self.get_or_create_assistant(group_id)?;
        
        let messages = self.messages.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(msg_list) = messages.get(&assistant.assistant_id) {
            let filtered: Vec<AssistantMessage> = msg_list.iter()
                .filter(|m| m.sequence >= start_seq && m.sequence <= end_seq)
                .cloned()
                .collect();
            return Ok(filtered);
        }
        
        Ok(Vec::new())
    }

    /// Get current sequence for a group
    pub fn get_sequence(&self, group_id: &str) -> Result<Option<u32>, String> {
        let sequences = self.sequences.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(sequences.get(group_id).copied())
    }

    /// Record receipt for a message
    pub fn record_receipt(&self, message_id: &str, receiver_id: &str) -> Result<(), String> {
        let mut receipts = self.receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
        receipts.entry(message_id.to_string())
            .or_insert_with(Vec::new)
            .push(receiver_id.to_string());
        Ok(())
    }

    /// Get receipts for a message
    pub fn get_receipts(&self, message_id: &str) -> Result<Vec<String>, String> {
        let receipts = self.receipts.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(receipts.get(message_id).cloned().unwrap_or_default())
    }

    /// Auto-reply to missing message request
    pub fn handle_missing_request(&self, group_id: &str, _requester_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<AssistantMessage>, String> {
        let current_seq = self.get_sequence(group_id)?;
        
        if let Some(current) = current_seq {
            // Validate the requested range
            if start_seq > current {
                return Ok(Vec::new()); // No messages in this range
            }
            
            let actual_end = end_seq.min(current);
            return self.get_messages(group_id, start_seq, actual_end);
        }
        
        Ok(Vec::new())
    }

    /// Get assistant info
    pub fn get_assistant_info(&self, group_id: &str) -> Result<Option<GroupAssistant>, String> {
        let assistants = self.assistants.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(assistants.get(group_id).cloned())
    }

    /// Deactivate assistant for a group
    pub fn deactivate_assistant(&self, group_id: &str) -> Result<(), String> {
        let mut assistants = self.assistants.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(ast) = assistants.get_mut(group_id) {
            ast.is_active = false;
        }
        Ok(())
    }

    /// Activate assistant for a group
    pub fn activate_assistant(&self, group_id: &str) -> Result<(), String> {
        let mut assistants = self.assistants.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(ast) = assistants.get_mut(group_id) {
            ast.is_active = true;
            ast.last_activity = Utc::now().timestamp();
        }
        Ok(())
    }
}
