//! P2P Group Coordinator - Coordination between P2P and cloud group assistant
//!
//! This module provides coordination logic for P2P group messaging with
//! cloud group assistant fallback. It implements priority logic for pulling
//! missing messages from P2P network before falling back to cloud assistant.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Message source priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageSourcePriority {
    LocalStorage = 0,     // Highest priority - already in local storage
    P2PNetwork = 1,       // Second priority - from nearby P2P peers
    CloudAssistant = 2,   // Lowest priority - from cloud group assistant
}

/// Message retrieval strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalStrategy {
    P2PFirst,      // Try P2P first, fallback to cloud
    CloudFirst,     // Try cloud first (for offline users)
    P2POnly,       // Only use P2P network
    CloudOnly,      // Only use cloud assistant
}

/// Missing message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingMessageRequest {
    pub user_id: String,
    pub group_id: String,
    pub start_sequence: u32,
    pub end_sequence: u32,
    pub strategy: RetrievalStrategy,
    pub timestamp: i64,
}

/// Message retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub messages: Vec<RetrievedMessage>,
    pub source: MessageSourcePriority,
    pub total_count: usize,
    pub timestamp: i64,
}

/// Retrieved message with source info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedMessage {
    pub message_id: String,
    pub content: String,
    pub sequence: u32,
    pub timestamp: i64,
    pub sender_id: String,
    pub integrity_hash: Option<String>,
    pub source: MessageSourcePriority,
}

/// P2P Group Coordinator
pub struct P2PGroupCoordinator {
    // Configuration
    prefer_p2p: bool,
    // P2P peer availability tracking
    peer_availability: Arc<Mutex<HashMap<String, bool>>>, // peer_id -> is_online
    // Cloud assistant availability
    cloud_available: Arc<Mutex<bool>>,
}

impl P2PGroupCoordinator {
    pub fn new(prefer_p2p: bool) -> Self {
        Self {
            prefer_p2p,
            peer_availability: Arc::new(Mutex::new(HashMap::new())),
            cloud_available: Arc::new(Mutex::new(true)),
        }
    }

    /// Determine retrieval strategy based on context
    pub fn determine_strategy(&self, user_id: &str, group_id: &str) -> RetrievalStrategy {
        let peers_online = self.count_online_peers(group_id);
        let cloud_ok = self.cloud_available.lock()
            .map(|guard| *guard)
            .unwrap_or(false);

        match (self.prefer_p2p, peers_online > 0, cloud_ok) {
            (true, true, _) => RetrievalStrategy::P2PFirst,
            (true, false, true) => RetrievalStrategy::CloudFirst,
            (true, false, false) => RetrievalStrategy::P2POnly, // Will fail but try anyway
            (false, _, true) => RetrievalStrategy::CloudFirst,
            (false, _, false) => RetrievalStrategy::P2PFirst, // Fallback to P2P
        }
    }

    /// Execute retrieval with priority logic
    pub async fn retrieve_missing_messages(
        &self,
        request: MissingMessageRequest,
        // These would be actual service references in a real implementation
        _p2p_service: Option<&dyn P2PMessageService>,
        _cloud_service: Option<&dyn CloudAssistantService>,
    ) -> Result<RetrievalResult, String> {
        match request.strategy {
            RetrievalStrategy::P2PFirst => {
                // Try P2P first
                if let Some(p2p) = _p2p_service {
                    match p2p.get_messages(&request.group_id, request.start_sequence, request.end_sequence).await {
                        Ok(messages) if !messages.is_empty() => {
                            let total_count = messages.len();
                            return Ok(RetrievalResult {
                                messages: messages.into_iter().map(|m| RetrievedMessage {
                                    message_id: m.message_id,
                                    content: m.content,
                                    sequence: m.sequence,
                                    timestamp: m.timestamp,
                                    sender_id: m.sender_id,
                                    integrity_hash: m.integrity_hash,
                                    source: MessageSourcePriority::P2PNetwork,
                                }).collect(),
                                source: MessageSourcePriority::P2PNetwork,
                                total_count,
                                timestamp: chrono::Utc::now().timestamp(),
                            });
                        }
                        _ => {}
                    }
                }
                
                // Fallback to cloud
                if let Some(cloud) = _cloud_service {
                    match cloud.get_messages(&request.group_id, request.start_sequence, request.end_sequence).await {
                        Ok(messages) => {
                            let total_count = messages.len();
                            return Ok(RetrievalResult {
                                total_count,
                                messages: messages.into_iter().map(|m| RetrievedMessage {
                                    message_id: m.message_id,
                                    content: m.content,
                                    sequence: m.sequence,
                                    timestamp: m.timestamp,
                                    sender_id: m.sender_id,
                                    integrity_hash: m.integrity_hash,
                                    source: MessageSourcePriority::CloudAssistant,
                                }).collect(),
                                source: MessageSourcePriority::CloudAssistant,
                                timestamp: chrono::Utc::now().timestamp(),
                            });
                        }
                        Err(e) => return Err(format!("Cloud retrieval failed: {}", e)),
                    }
                }
                
                Err("No available sources".to_string())
            }
            
            RetrievalStrategy::CloudFirst => {
                // Try cloud first
                if let Some(cloud) = _cloud_service {
                    match cloud.get_messages(&request.group_id, request.start_sequence, request.end_sequence).await {
                        Ok(messages) => {
                            let total_count = messages.len();
                            return Ok(RetrievalResult {
                                total_count,
                                messages: messages.into_iter().map(|m| RetrievedMessage {
                                    message_id: m.message_id,
                                    content: m.content,
                                    sequence: m.sequence,
                                    timestamp: m.timestamp,
                                    sender_id: m.sender_id,
                                    integrity_hash: m.integrity_hash,
                                    source: MessageSourcePriority::CloudAssistant,
                                }).collect(),
                                source: MessageSourcePriority::CloudAssistant,
                                timestamp: chrono::Utc::now().timestamp(),
                            });
                        }
                        Err(e) => {
                            eprintln!("Cloud retrieval failed: {}, falling back to P2P", e);
                        }
                    }
                }
                
                // Fallback to P2P
                if let Some(p2p) = _p2p_service {
                    match p2p.get_messages(&request.group_id, request.start_sequence, request.end_sequence).await {
                        Ok(messages) => {
                            let total_count = messages.len();
                            return Ok(RetrievalResult {
                                total_count,
                                messages: messages.into_iter().map(|m| RetrievedMessage {
                                    message_id: m.message_id,
                                    content: m.content,
                                    sequence: m.sequence,
                                    timestamp: m.timestamp,
                                    sender_id: m.sender_id,
                                    integrity_hash: m.integrity_hash,
                                    source: MessageSourcePriority::P2PNetwork,
                                }).collect(),
                                source: MessageSourcePriority::P2PNetwork,
                                timestamp: chrono::Utc::now().timestamp(),
                            });
                        }
                        Err(e) => return Err(format!("P2P retrieval failed: {}", e)),
                    }
                }
                
                Err("No available sources".to_string())
            }
            
            RetrievalStrategy::P2POnly => {
                if let Some(p2p) = _p2p_service {
                    match p2p.get_messages(&request.group_id, request.start_sequence, request.end_sequence).await {
                        Ok(messages) => {
                            let total_count = messages.len();
                            return Ok(RetrievalResult {
                                total_count,
                                messages: messages.into_iter().map(|m| RetrievedMessage {
                                    message_id: m.message_id,
                                    content: m.content,
                                    sequence: m.sequence,
                                    timestamp: m.timestamp,
                                    sender_id: m.sender_id,
                                    integrity_hash: m.integrity_hash,
                                    source: MessageSourcePriority::P2PNetwork,
                                }).collect(),
                                source: MessageSourcePriority::P2PNetwork,
                                timestamp: chrono::Utc::now().timestamp(),
                            });
                        }
                        Err(e) => return Err(format!("P2P retrieval failed: {}", e)),
                    }
                }
                
                Err("P2P service not available".to_string())
            }
            
            RetrievalStrategy::CloudOnly => {
                if let Some(cloud) = _cloud_service {
                    match cloud.get_messages(&request.group_id, request.start_sequence, request.end_sequence).await {
                        Ok(messages) => {
                            let total_count = messages.len();
                            return Ok(RetrievalResult {
                                total_count,
                                messages: messages.into_iter().map(|m| RetrievedMessage {
                                    message_id: m.message_id,
                                    content: m.content,
                                    sequence: m.sequence,
                                    timestamp: m.timestamp,
                                    sender_id: m.sender_id,
                                    integrity_hash: m.integrity_hash,
                                    source: MessageSourcePriority::CloudAssistant,
                                }).collect(),
                                source: MessageSourcePriority::CloudAssistant,
                                timestamp: chrono::Utc::now().timestamp(),
                            });
                        }
                        Err(e) => return Err(format!("Cloud retrieval failed: {}", e)),
                    }
                }
                
                Err("Cloud service not available".to_string())
            }
        }
    }

    /// Update peer availability
    pub fn update_peer_availability(&self, peer_id: &str, is_online: bool) {
        if let Ok(mut peers) = self.peer_availability.lock() {
            peers.insert(peer_id.to_string(), is_online);
        }
    }

    /// Update cloud availability
    pub fn update_cloud_availability(&self, is_available: bool) {
        if let Ok(mut cloud) = self.cloud_available.lock() {
            *cloud = is_available;
        }
    }

    /// Count online peers for a group
    pub fn count_online_peers(&self, _group_id: &str) -> usize {
        self.peer_availability.lock()
            .map(|peers| peers.values().filter(|&&online| online).count())
            .unwrap_or(0)
    }

    /// Set P2P preference
    pub fn set_prefer_p2p(&self, _prefer: bool) {
        // In a real implementation, this would update a field
        // For now, this is a placeholder
    }
}

// Trait definitions for external services

#[async_trait::async_trait]
pub trait P2PMessageService: Send + Sync {
    async fn get_messages(&self, group_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<P2PMessage>, String>;
}

#[async_trait::async_trait]
pub trait CloudAssistantService: Send + Sync {
    async fn get_messages(&self, group_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<CloudMessage>, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub message_id: String,
    pub content: String,
    pub sequence: u32,
    pub timestamp: i64,
    pub sender_id: String,
    pub integrity_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudMessage {
    pub message_id: String,
    pub content: String,
    pub sequence: u32,
    pub timestamp: i64,
    pub sender_id: String,
    pub integrity_hash: Option<String>,
}
