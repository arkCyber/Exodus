//! Node management for P2P network
//!
//! Manages peer information and connections.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::RwLock;

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub address: SocketAddr,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub topics: Vec<String>,
}

/// Node manager
pub struct NodeManager {
    nodes: RwLock<HashMap<String, NodeInfo>>,
    local_node: RwLock<Option<NodeInfo>>,
}

impl NodeManager {
    /// Create a new node manager
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            local_node: RwLock::new(None),
        }
    }

    /// Set local node info
    pub async fn set_local_node(&self, info: NodeInfo) {
        let mut local = self.local_node.write().await;
        *local = Some(info);
    }

    /// Get local node info
    pub async fn get_local_node(&self) -> Option<NodeInfo> {
        let local = self.local_node.read().await;
        local.clone()
    }

    /// Add or update a node
    pub async fn upsert_node(&self, info: NodeInfo) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(info.id.clone(), info);
    }

    /// Get node info
    pub async fn get_node(&self, id: &str) -> Option<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.get(id).cloned()
    }

    /// List all nodes
    pub async fn list_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Remove a node
    pub async fn remove_node(&self, id: &str) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(id);
    }

    /// Get nodes for a specific topic
    pub async fn get_nodes_for_topic(&self, topic_id: &str) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values()
            .filter(|node| node.topics.contains(&topic_id.to_string()))
            .cloned()
            .collect()
    }

    /// Update node last seen time
    pub async fn update_last_seen(&self, id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(id) {
            node.last_seen = chrono::Utc::now();
        }
    }
}
