//! Gossip protocol implementation
//!
//! Implements message broadcasting using gossip protocol.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::debug;
use crate::p2p::topic::TopicId;

/// Gossip event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GossipEvent {
    #[serde(rename = "message")]
    Message {
        topic_id: String,
        sender_id: String,
        content: String,
        sequence: u32,
        timestamp: i64,
    },
    #[serde(rename = "receipt")]
    Receipt {
        topic_id: String,
        message_id: String,
        receiver_id: String,
        sequence: u32,
        timestamp: i64,
    },
    #[serde(rename = "join")]
    Join {
        topic_id: String,
        node_id: String,
    },
    #[serde(rename = "leave")]
    Leave {
        topic_id: String,
        node_id: String,
    },
}

/// Gossip manager
pub struct GossipManager {
    subscribers: broadcast::Sender<GossipEvent>,
}

impl GossipManager {
    /// Create a new gossip manager
    pub async fn new() -> Result<Self> {
        let (subscribers, _) = broadcast::channel(1000);
        
        Ok(Self {
            subscribers,
        })
    }

    /// Subscribe to gossip events
    pub async fn subscribe(&self) -> broadcast::Receiver<GossipEvent> {
        self.subscribers.subscribe()
    }

    /// Broadcast a message to all subscribers
    pub async fn broadcast(&self, event: GossipEvent) -> Result<()> {
        debug!("Broadcasting gossip event: {:?}", std::mem::discriminant(&event));
        let _ = self.subscribers.send(event);
        Ok(())
    }

    /// Send a message to a topic
    pub async fn send_message(
        &self,
        topic_id: &TopicId,
        sender_id: &str,
        content: &str,
        sequence: u32,
    ) -> Result<()> {
        let event = GossipEvent::Message {
            topic_id: topic_id.to_string(),
            sender_id: sender_id.to_string(),
            content: content.to_string(),
            sequence,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        self.broadcast(event).await
    }

    /// Announce node joining a topic
    pub async fn announce_join(&self, topic_id: &TopicId, node_id: &str) -> Result<()> {
        let event = GossipEvent::Join {
            topic_id: topic_id.to_string(),
            node_id: node_id.to_string(),
        };
        
        self.broadcast(event).await
    }

    /// Announce node leaving a topic
    pub async fn announce_leave(&self, topic_id: &TopicId, node_id: &str) -> Result<()> {
        let event = GossipEvent::Leave {
            topic_id: topic_id.to_string(),
            node_id: node_id.to_string(),
        };
        
        self.broadcast(event).await
    }

    /// Send a receipt for message delivery
    pub async fn send_receipt(
        &self,
        topic_id: &TopicId,
        message_id: &str,
        receiver_id: &str,
        sequence: u32,
    ) -> Result<()> {
        let event = GossipEvent::Receipt {
            topic_id: topic_id.to_string(),
            message_id: message_id.to_string(),
            receiver_id: receiver_id.to_string(),
            sequence,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        self.broadcast(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gossip_broadcast() {
        let manager = GossipManager::new().await.unwrap();
        
        // Subscribe to events
        let mut rx1 = manager.subscribe().await;
        let mut rx2 = manager.subscribe().await;
        
        // Broadcast a message
        let topic_id = TopicId::new();
        manager.send_message(&topic_id, "user1", "Hello", 1).await.unwrap();
        
        // Both subscribers should receive the message
        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();
        
        match (&event1, &event2) {
            (GossipEvent::Message { content: c1, sequence: s1, .. }, GossipEvent::Message { content: c2, sequence: s2, .. }) => {
                assert_eq!(c1, "Hello");
                assert_eq!(c2, "Hello");
                assert_eq!(s1, &1);
                assert_eq!(s2, &1);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_gossip_join_leave() {
        let manager = GossipManager::new().await.unwrap();
        
        let topic_id = TopicId::new();
        
        // Announce join
        manager.announce_join(&topic_id, "node1").await.unwrap();
        
        // Announce leave
        manager.announce_leave(&topic_id, "node1").await.unwrap();
    }
}
