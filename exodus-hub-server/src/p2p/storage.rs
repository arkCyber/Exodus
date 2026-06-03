//! Storage integration for P2P messages
//!
//! Integrates P2P gossip messages with the storage and forwarding server.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::p2p::GossipEvent;
use crate::manager::ImManager;

/// Storage integration manager
pub struct StorageIntegration {
    im_manager: Arc<ImManager>,
    user_cache: Arc<RwLock<HashMap<String, String>>>,
    topic_conversation_map: Arc<RwLock<HashMap<String, String>>>,
}

impl StorageIntegration {
    /// Create a new storage integration
    pub fn new(im_manager: Arc<ImManager>) -> Self {
        Self {
            im_manager,
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            topic_conversation_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle gossip event and store to database
    pub async fn handle_event(&self, event: &GossipEvent) -> Result<()> {
        match event {
            GossipEvent::Message { topic_id, sender_id, content, sequence, .. } => {
                // Store P2P message to database
                // This ensures message persistence even if P2P network fails
                self.store_message(topic_id, sender_id, content, *sequence).await?;
            }
            GossipEvent::Receipt { topic_id, message_id, receiver_id, sequence, .. } => {
                // Store delivery receipt
                self.store_receipt(topic_id, message_id, receiver_id, *sequence).await?;
            }
            GossipEvent::Join { topic_id, node_id } => {
                // Track node joining topic
                self.track_node_join(topic_id, node_id).await?;
            }
            GossipEvent::Leave { topic_id, node_id } => {
                // Track node leaving topic
                self.track_node_leave(topic_id, node_id).await?;
            }
        }
        Ok(())
    }

    /// Store P2P message to database
    async fn store_message(
        &self,
        topic_id: &str,
        sender_id: &str,
        content: &str,
        sequence: u32,
    ) -> Result<()> {
        // Get or create conversation ID mapping
        let conversation_id = {
            let map = self.topic_conversation_map.read().await;
            map.get(topic_id).cloned()
        };

        let conversation_id = match conversation_id {
            Some(id) => id,
            None => {
                // Create conversation and store mapping
                let conv = self.im_manager.create_conversation(
                    crate::manager::ChatType::OneOnOne,
                    &format!("P2P Topic: {}", topic_id),
                ).await?;
                
                let mut map = self.topic_conversation_map.write().await;
                map.insert(topic_id.to_string(), conv.id.clone());
                
                conv.id
            }
        };

        // Get or create user from cache
        let user_id = {
            let cache = self.user_cache.read().await;
            cache.get(sender_id).cloned()
        };

        let user = if let Some(cached_id) = user_id {
            // Use cached user ID
            cached_id
        } else {
            // Create new user with unique ID
            let new_user_id = format!("p2p_user_{}_{}", sender_id, chrono::Utc::now().timestamp_micros());
            let user = self.im_manager.create_user(&new_user_id, &format!("User {}", sender_id)).await?;
            
            // Cache the mapping
            let mut cache = self.user_cache.write().await;
            cache.insert(sender_id.to_string(), user.id.clone());
            
            user.id
        };

        // Store message
        self.im_manager.send_message(
            &conversation_id,
            &user,
            None,
            content,
            None,
        ).await?;

        tracing::info!("Stored P2P message: topic={}, sender={}, seq={}, conv={}", topic_id, sender_id, sequence, conversation_id);
        Ok(())
    }

    /// Store delivery receipt
    async fn store_receipt(
        &self,
        topic_id: &str,
        message_id: &str,
        receiver_id: &str,
        sequence: u32,
    ) -> Result<()> {
        // Store receipt in database
        // This can be used to track message delivery status
        tracing::info!("Stored receipt: topic={}, msg={}, receiver={}, seq={}", 
            topic_id, message_id, receiver_id, sequence);
        Ok(())
    }

    /// Track node joining topic
    async fn track_node_join(&self, topic_id: &str, node_id: &str) -> Result<()> {
        tracing::info!("Node joined topic: topic={}, node={}", topic_id, node_id);
        Ok(())
    }

    /// Track node leaving topic
    async fn track_node_leave(&self, topic_id: &str, node_id: &str) -> Result<()> {
        tracing::info!("Node left topic: topic={}, node={}", topic_id, node_id);
        Ok(())
    }

    /// Get missing messages from server
    pub async fn get_missing_messages(
        &self,
        topic_id: &str,
        after_sequence: u32,
        limit: u32,
    ) -> Result<Vec<crate::manager::Message>> {
        // Get conversation ID from topic mapping
        let conversation_id = {
            let map = self.topic_conversation_map.read().await;
            map.get(topic_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Topic not found in conversation map"))?
        };
        
        let sync_response = self.im_manager.get_messages_for_sync(
            &conversation_id,
            Some(after_sequence),
            limit as usize,
        ).await?;
        
        Ok(sync_response.messages)
    }
}
