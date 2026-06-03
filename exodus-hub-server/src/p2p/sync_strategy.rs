//! Smart synchronization strategy
//!
//! Implements intelligent sync strategy: P2P for online peers, server for offline.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::p2p::{P2PManager, TopicId, NodeInfo};

/// Sync mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncMode {
    /// Use P2P network only
    P2POnly,
    /// Use server only
    ServerOnly,
    /// Hybrid: P2P with server fallback
    Hybrid,
}

/// Sync strategy configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Minimum online peers to use P2P
    pub min_online_peers: usize,
    /// Maximum message gap before using server
    pub max_message_gap: u32,
    /// Timeout for P2P requests (milliseconds)
    pub p2p_timeout: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            min_online_peers: 2,
            max_message_gap: 100,
            p2p_timeout: 5000,
        }
    }
}

/// Smart sync manager
pub struct SyncStrategy {
    p2p: Arc<P2PManager>,
    config: SyncConfig,
    peer_status: Arc<RwLock<std::collections::HashMap<String, PeerStatus>>>,
}

/// Peer status tracking
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PeerStatus {
    last_seen: chrono::DateTime<chrono::Utc>,
    is_online: bool,
}

impl SyncStrategy {
    /// Create a new sync strategy manager
    pub fn new(
        p2p: Arc<P2PManager>,
        config: SyncConfig,
    ) -> Self {
        Self {
            p2p,
            config,
            peer_status: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Determine sync mode based on current conditions
    pub async fn determine_sync_mode(
        &self,
        topic_id: &TopicId,
        last_sequence: u32,
    ) -> SyncMode {
        let online_peers = self.count_online_peers(topic_id).await;
        let server_sequence = self.get_server_sequence(topic_id).await.unwrap_or(0);
        let message_gap = server_sequence.saturating_sub(last_sequence);

        tracing::info!("Sync decision: online_peers={}, server_seq={}, last_seq={}, gap={}", 
            online_peers, server_sequence, last_sequence, message_gap);

        // Decision logic
        if online_peers < self.config.min_online_peers {
            // Not enough online peers, use server
            SyncMode::ServerOnly
        } else if message_gap > self.config.max_message_gap {
            // Too many missing messages, use server for efficiency
            SyncMode::ServerOnly
        } else if online_peers >= 3 && message_gap < 10 {
            // Good P2P conditions, use P2P only
            SyncMode::P2POnly
        } else {
            // Mixed conditions, use hybrid
            SyncMode::Hybrid
        }
    }

    /// Sync messages using determined strategy
    pub async fn sync_messages(
        &self,
        topic_id: &TopicId,
        last_sequence: u32,
        limit: u32,
    ) -> Result<Vec<crate::manager::Message>> {
        let mode = self.determine_sync_mode(topic_id, last_sequence).await;

        match mode {
            SyncMode::P2POnly => self.sync_p2p(topic_id, last_sequence, limit).await,
            SyncMode::ServerOnly => self.sync_server(topic_id, last_sequence, limit).await,
            SyncMode::Hybrid => self.sync_hybrid(topic_id, last_sequence, limit).await,
        }
    }

    /// Sync using P2P network
    async fn sync_p2p(
        &self,
        topic_id: &TopicId,
        last_sequence: u32,
        limit: u32,
    ) -> Result<Vec<crate::manager::Message>> {
        // Request messages from online peers
        let peers = self.get_online_peers(topic_id).await;
        
        if peers.is_empty() {
            return Err(anyhow::anyhow!("No online peers available"));
        }

        // For now, return empty - actual P2P sync would request from peers
        tracing::info!("P2P sync: topic={}, after={}, limit={}, peers={}", 
            topic_id, last_sequence, limit, peers.len());
        
        Ok(vec![])
    }

    /// Sync using server
    async fn sync_server(
        &self,
        topic_id: &TopicId,
        last_sequence: u32,
        limit: u32,
    ) -> Result<Vec<crate::manager::Message>> {
        if let Some(storage) = self.p2p.storage() {
            let messages = storage.get_missing_messages(
                &topic_id.to_string(),
                last_sequence,
                limit,
            ).await?;
            
            tracing::info!("Server sync: topic={}, after={}, limit={}, messages={}", 
                topic_id, last_sequence, limit, messages.len());
            
            Ok(messages)
        } else {
            Err(anyhow::anyhow!("Storage integration not available"))
        }
    }

    /// Sync using hybrid approach
    async fn sync_hybrid(
        &self,
        topic_id: &TopicId,
        last_sequence: u32,
        limit: u32,
    ) -> Result<Vec<crate::manager::Message>> {
        // Try P2P first with timeout
        let p2p_result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.p2p_timeout),
            self.sync_p2p(topic_id, last_sequence, limit),
        ).await;

        match p2p_result {
            Ok(Ok(messages)) if !messages.is_empty() => {
                // P2P successful
                tracing::info!("Hybrid sync: P2P successful, got {} messages", messages.len());
                Ok(messages)
            }
            _ => {
                // P2P failed or timeout, fallback to server
                tracing::info!("Hybrid sync: P2P failed, falling back to server");
                self.sync_server(topic_id, last_sequence, limit).await
            }
        }
    }

    /// Count online peers for a topic
    async fn count_online_peers(&self, topic_id: &TopicId) -> usize {
        self.get_online_peers(topic_id).await.len()
    }

    /// Get online peers for a topic
    async fn get_online_peers(&self, topic_id: &TopicId) -> Vec<NodeInfo> {
        let all_peers = self.p2p.node().list_nodes().await;
        let topic_hex = topic_id.to_hex();
        
        all_peers
            .into_iter()
            .filter(|peer| {
                peer.topics.contains(&topic_hex) && self.is_peer_online(&peer.id)
            })
            .collect()
    }

    /// Check if peer is online
    fn is_peer_online(&self, _peer_id: &str) -> bool {
        // For now, assume all peers are online
        // In production, this would check last_seen timestamp
        true
    }

    /// Get latest sequence number from server
    async fn get_server_sequence(&self, topic_id: &TopicId) -> Result<u32> {
        if let Some(storage) = self.p2p.storage() {
            let messages = storage.get_missing_messages(&topic_id.to_string(), 0, 1).await?;
            if let Some(last_msg) = messages.last() {
                return Ok(last_msg.sequence.unwrap_or(0));
            }
        }
        Ok(0)
    }

    /// Update peer status
    pub async fn update_peer_status(&self, peer_id: String, is_online: bool) {
        let mut status = self.peer_status.write().await;
        status.insert(peer_id, PeerStatus {
            last_seen: chrono::Utc::now(),
            is_online,
        });
    }
}
