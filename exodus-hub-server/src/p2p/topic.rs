//! Topic management for group chat
//!
//! Topics represent group chat rooms in the P2P network.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Topic ID (32-byte hash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicId([u8; 32]);

impl TopicId {
    /// Generate a new random topic ID
    pub fn new() -> Self {
        let mut bytes = [0u8; 32];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.fill(&mut bytes);
        Self(bytes)
    }

    /// Generate topic ID from string
    pub fn from_string(s: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        let hash = hasher.finalize();
        Self(hash.into())
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get hex string representation
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid topic ID length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl std::fmt::Display for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Topic metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMetadata {
    pub id: TopicId,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub member_count: usize,
}

/// Topic manager
pub struct TopicManager {
    topics: RwLock<HashMap<TopicId, TopicMetadata>>,
}

impl TopicManager {
    /// Create a new topic manager
    pub fn new() -> Self {
        Self {
            topics: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new topic
    pub async fn create_topic(&self, name: &str) -> Result<TopicId> {
        let id = TopicId::new();
        let metadata = TopicMetadata {
            id: id.clone(),
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            member_count: 0,
        };

        let mut topics = self.topics.write().await;
        topics.insert(id.clone(), metadata);
        
        Ok(id)
    }

    /// Get topic metadata
    pub async fn get_topic(&self, id: &TopicId) -> Option<TopicMetadata> {
        let topics = self.topics.read().await;
        topics.get(id).cloned()
    }

    /// List all topics
    pub async fn list_topics(&self) -> Vec<TopicMetadata> {
        let topics = self.topics.read().await;
        topics.values().cloned().collect()
    }

    /// Increment member count
    pub async fn increment_members(&self, id: &TopicId) -> Result<()> {
        let mut topics = self.topics.write().await;
        if let Some(metadata) = topics.get_mut(id) {
            metadata.member_count += 1;
        }
        Ok(())
    }

    /// Decrement member count
    pub async fn decrement_members(&self, id: &TopicId) -> Result<()> {
        let mut topics = self.topics.write().await;
        if let Some(metadata) = topics.get_mut(id) {
            if metadata.member_count > 0 {
                metadata.member_count -= 1;
            }
        }
        Ok(())
    }
}
