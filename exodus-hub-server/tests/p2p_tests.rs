//! Unit tests for P2P module

#[cfg(test)]
mod tests {
    use exodus_hub_server::p2p::{TopicManager, TopicId, NodeManager, NodeInfo, GossipManager, GossipEvent};
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_topic_creation() {
        let manager = TopicManager::new();
        
        let topic_id = manager.create_topic("Test Group").await.unwrap();
        
        let metadata = manager.get_topic(&topic_id).await.unwrap();
        assert_eq!(metadata.name, "Test Group");
        assert_eq!(metadata.member_count, 0);
    }

    #[tokio::test]
    async fn test_topic_from_string() {
        let topic_id = TopicId::from_string("test-group");
        
        let hex = topic_id.to_hex();
        let parsed = TopicId::from_hex(&hex).unwrap();
        
        assert_eq!(topic_id, parsed);
    }

    #[tokio::test]
    async fn test_topic_member_count() {
        let manager = TopicManager::new();
        let topic_id = manager.create_topic("Test Group").await.unwrap();
        
        manager.increment_members(&topic_id).await.unwrap();
        manager.increment_members(&topic_id).await.unwrap();
        
        let metadata = manager.get_topic(&topic_id).await.unwrap();
        assert_eq!(metadata.member_count, 2);
        
        manager.decrement_members(&topic_id).await.unwrap();
        let metadata = manager.get_topic(&topic_id).await.unwrap();
        assert_eq!(metadata.member_count, 1);
    }

    #[tokio::test]
    async fn test_node_management() {
        let manager = NodeManager::new();
        
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let info = NodeInfo {
            id: "node1".to_string(),
            address: addr,
            last_seen: chrono::Utc::now(),
            topics: vec!["topic1".to_string()],
        };
        
        manager.upsert_node(info.clone()).await;
        
        let retrieved = manager.get_node("node1").await.unwrap();
        assert_eq!(retrieved.id, "node1");
        
        manager.remove_node("node1").await;
        assert!(manager.get_node("node1").await.is_none());
    }

    #[tokio::test]
    async fn test_gossip_message() {
        let manager = GossipManager::new().await.unwrap();
        
        let topic_id = TopicId::new();
        manager.send_message(&topic_id, "user1", "Hello World", 1).await.unwrap();
        
        let mut rx = manager.subscribe().await;
        let event = rx.recv().await.unwrap();
        
        match event {
            GossipEvent::Message { content, sequence, .. } => {
                assert_eq!(content, "Hello World");
                assert_eq!(sequence, 1);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_gossip_join_leave() {
        let manager = GossipManager::new().await.unwrap();
        
        let topic_id = TopicId::new();
        
        manager.announce_join(&topic_id, "node1").await.unwrap();
        manager.announce_leave(&topic_id, "node1").await.unwrap();
        
        let mut rx = manager.subscribe().await;
        
        let event1 = rx.recv().await.unwrap();
        match event1 {
            GossipEvent::Join { node_id, .. } => {
                assert_eq!(node_id, "node1");
            }
            _ => panic!("Expected Join event"),
        }
        
        let event2 = rx.recv().await.unwrap();
        match event2 {
            GossipEvent::Leave { node_id, .. } => {
                assert_eq!(node_id, "node1");
            }
            _ => panic!("Expected Leave event"),
        }
    }

    #[tokio::test]
    async fn test_p2p_manager() {
        let manager = exodus_hub_server::p2p::P2PManager::new().await.unwrap();
        
        // Test topic creation
        let topic_id = manager.topic().create_topic("Test").await.unwrap();
        assert!(manager.topic().get_topic(&topic_id).await.is_some());
        
        // Test node management
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let info = NodeInfo {
            id: "node1".to_string(),
            address: addr,
            last_seen: chrono::Utc::now(),
            topics: vec![],
        };
        manager.node().upsert_node(info).await;
        assert!(manager.node().get_node("node1").await.is_some());
        
        // Test gossip
        manager.gossip().send_message(&topic_id, "user1", "Test", 1).await.unwrap();
    }
}
