//! P2P networking module using Iroh
//!
//! This module provides decentralized peer-to-peer networking capabilities
//! using the Iroh library for group chat and message synchronization.

pub mod gossip;
pub mod topic;
pub mod node;
pub mod storage;
pub mod sync_strategy;

pub use gossip::{GossipManager, GossipEvent};
pub use topic::{TopicManager, TopicId};
pub use node::{NodeInfo, NodeManager};
pub use storage::StorageIntegration;
pub use sync_strategy::{SyncStrategy, SyncMode, SyncConfig};

use anyhow::Result;
use std::sync::Arc;
use crate::manager::ImManager;

/// P2P network manager
#[derive(Clone)]
pub struct P2PManager {
    gossip: Arc<GossipManager>,
    topic: Arc<TopicManager>,
    node: Arc<NodeManager>,
    storage: Option<Arc<StorageIntegration>>,
}

impl P2PManager {
    /// Create a new P2P manager
    pub async fn new() -> Result<Self> {
        let gossip = Arc::new(GossipManager::new().await?);
        let topic = Arc::new(TopicManager::new());
        let node = Arc::new(NodeManager::new());

        Ok(Self {
            gossip,
            topic,
            node,
            storage: None,
        })
    }

    /// Create a new P2P manager with storage integration
    pub async fn with_storage(im_manager: Arc<ImManager>) -> Result<Self> {
        let gossip = Arc::new(GossipManager::new().await?);
        let topic = Arc::new(TopicManager::new());
        let node = Arc::new(NodeManager::new());
        let storage = Arc::new(StorageIntegration::new(im_manager));

        Ok(Self {
            gossip,
            topic,
            node,
            storage: Some(storage),
        })
    }

    /// Create sync strategy
    pub fn create_sync_strategy(&self, config: SyncConfig) -> SyncStrategy {
        SyncStrategy::new(Arc::new(self.clone()), config)
    }

    /// Get gossip manager
    pub fn gossip(&self) -> &Arc<GossipManager> {
        &self.gossip
    }

    /// Get topic manager
    pub fn topic(&self) -> &Arc<TopicManager> {
        &self.topic
    }

    /// Get node manager
    pub fn node(&self) -> &Arc<NodeManager> {
        &self.node
    }

    /// Get storage integration
    pub fn storage(&self) -> Option<&Arc<StorageIntegration>> {
        self.storage.as_ref()
    }

    /// Handle gossip event with storage integration
    pub async fn handle_event(&self, event: &GossipEvent) -> Result<()> {
        if let Some(storage) = &self.storage {
            storage.handle_event(event).await?;
        }
        Ok(())
    }
}
