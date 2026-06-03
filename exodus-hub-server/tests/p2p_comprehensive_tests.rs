//! Comprehensive P2P integration tests
//!
//! Tests the complete P2P system including:
//! - Topic management
//! - Node management
//! - Gossip protocol
//! - Storage integration
//! - Sync strategy
//! - API endpoints

use exodus_hub_server::p2p::{P2PManager, TopicId, NodeInfo, SyncStrategy, SyncConfig, SyncMode};
use exodus_hub_server::ImManager;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_p2p_topic_management() {
    let p2p = P2PManager::new().await.unwrap();
    
    // Create topic
    let topic_id = p2p.topic().create_topic("Test Group").await.unwrap();
    
    // Get topic metadata
    let retrieved = p2p.topic().get_topic(&topic_id).await;
    assert!(retrieved.is_some());
    let metadata = retrieved.unwrap();
    assert_eq!(metadata.name, "Test Group");
    assert_eq!(metadata.id, topic_id);
    
    // List topics
    let topics = p2p.topic().list_topics().await;
    assert_eq!(topics.len(), 1);
    
    println!("✓ Topic management test passed");
}

#[tokio::test]
async fn test_p2p_node_management() {
    let p2p = P2PManager::new().await.unwrap();
    
    // Create node
    let node = NodeInfo {
        id: "node1".to_string(),
        address: "127.0.0.1:8080".parse().unwrap(),
        last_seen: chrono::Utc::now(),
        topics: vec![],
    };
    
    p2p.node().upsert_node(node.clone()).await;
    
    // Get node
    let retrieved = p2p.node().get_node("node1").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "node1");
    
    // List nodes
    let nodes = p2p.node().list_nodes().await;
    assert_eq!(nodes.len(), 1);
    
    // Remove node
    p2p.node().remove_node("node1").await;
    assert!(p2p.node().get_node("node1").await.is_none());
    
    println!("✓ Node management test passed");
}

#[tokio::test]
async fn test_p2p_gossip_with_sequence() {
    let p2p = P2PManager::new().await.unwrap();
    let topic_id = TopicId::new();
    
    // Subscribe
    let mut rx = p2p.gossip().subscribe().await;
    
    // Send messages with sequence numbers
    for i in 1..=3 {
        p2p.gossip().send_message(&topic_id, "user1", &format!("Message {}", i), i).await.unwrap();
    }
    
    // Receive messages
    for i in 1..=3 {
        let event = rx.recv().await.unwrap();
        if let exodus_hub_server::p2p::GossipEvent::Message { sequence, .. } = event {
            assert_eq!(sequence, i);
        } else {
            panic!("Expected Message event");
        }
    }
    
    println!("✓ Gossip with sequence test passed");
}

#[tokio::test]
async fn test_p2p_receipt_mechanism() {
    let p2p = P2PManager::new().await.unwrap();
    let topic_id = TopicId::new();
    
    // Subscribe
    let mut rx = p2p.gossip().subscribe().await;
    
    // Send receipt
    p2p.gossip().send_receipt(&topic_id, "msg123", "user2", 1).await.unwrap();
    
    // Receive receipt
    let event = rx.recv().await.unwrap();
    if let exodus_hub_server::p2p::GossipEvent::Receipt { message_id, receiver_id, sequence, .. } = event {
        assert_eq!(message_id, "msg123");
        assert_eq!(receiver_id, "user2");
        assert_eq!(sequence, 1);
    } else {
        panic!("Expected Receipt event");
    }
    
    println!("✓ Receipt mechanism test passed");
}

#[tokio::test]
async fn test_p2p_storage_integration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_storage.db");
    let im_manager = Arc::new(ImManager::new(db_path).unwrap());
    let p2p = P2PManager::with_storage(im_manager.clone()).await.unwrap();
    let topic_id = TopicId::from_string("test-storage-topic");
    
    // Subscribe
    let mut rx = p2p.gossip().subscribe().await;
    
    // Send message
    p2p.gossip().send_message(&topic_id, "user1", "Test message", 1).await.unwrap();
    
    // Handle event (stores to database)
    let event = rx.recv().await.unwrap();
    p2p.handle_event(&event).await.unwrap();
    
    // Retrieve from database
    if let Some(storage) = p2p.storage() {
        let messages = storage.get_missing_messages(&topic_id.to_string(), 0, 10).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Test message");
    }
    
    println!("✓ Storage integration test passed");
}

#[tokio::test]
async fn test_p2p_sync_strategy_p2p_only() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_sync.db");
    let im_manager = Arc::new(ImManager::new(db_path).unwrap());
    let p2p = P2PManager::with_storage(im_manager.clone()).await.unwrap();
    let topic_id = TopicId::new();
    
    // Add some online peers
    for i in 0..5 {
        let node = NodeInfo {
            id: format!("peer{}", i),
            address: format!("127.0.0.1:{}", 9000 + i).parse().unwrap(),
            last_seen: chrono::Utc::now(),
            topics: vec![topic_id.to_hex()],
        };
        p2p.node().upsert_node(node).await;
    }
    
    let config = SyncConfig {
        min_online_peers: 2,
        max_message_gap: 100,
        p2p_timeout: 5000,
    };
    let sync_strategy = SyncStrategy::new(Arc::new(p2p.clone()), config);
    
    // Should use P2P only (enough peers, small gap)
    let mode = sync_strategy.determine_sync_mode(&topic_id, 0).await;
    assert_eq!(mode, SyncMode::P2POnly);
    
    println!("✓ Sync strategy P2P-only test passed");
}

#[tokio::test]
async fn test_p2p_sync_strategy_server_only() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_sync_server.db");
    let im_manager = Arc::new(ImManager::new(db_path).unwrap());
    let p2p = P2PManager::with_storage(im_manager.clone()).await.unwrap();
    let topic_id = TopicId::new();
    
    // No online peers
    let config = SyncConfig {
        min_online_peers: 2,
        max_message_gap: 100,
        p2p_timeout: 5000,
    };
    let sync_strategy = SyncStrategy::new(Arc::new(p2p.clone()), config);
    
    // Should use server only (no peers)
    let mode = sync_strategy.determine_sync_mode(&topic_id, 0).await;
    assert_eq!(mode, SyncMode::ServerOnly);
    
    println!("✓ Sync strategy server-only test passed");
}

#[tokio::test]
async fn test_p2p_full_integration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_full.db");
    let im_manager = Arc::new(ImManager::new(db_path).unwrap());
    let p2p = P2PManager::with_storage(im_manager.clone()).await.unwrap();
    let topic_id = TopicId::from_string("full-integration-test");
    
    // Create topic
    let _topic_id = p2p.topic().create_topic("Full Integration Test").await.unwrap();
    
    // Register nodes
    for i in 0..3 {
        let node = NodeInfo {
            id: format!("node{}", i),
            address: format!("127.0.0.1:{}", 8000 + i).parse().unwrap(),
            last_seen: chrono::Utc::now(),
            topics: vec![topic_id.to_hex()],
        };
        p2p.node().upsert_node(node).await;
    }
    
    // Subscribe
    let mut rx = p2p.gossip().subscribe().await;
    
    // Send multiple messages
    for i in 1..=5 {
        p2p.gossip().send_message(&topic_id, "user1", &format!("Message {}", i), i).await.unwrap();
        let event = rx.recv().await.unwrap();
        p2p.handle_event(&event).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }
    
    // Verify messages in storage
    if let Some(storage) = p2p.storage() {
        let messages = storage.get_missing_messages(&topic_id.to_string(), 0, 10).await.unwrap();
        assert_eq!(messages.len(), 5);
    }
    
    // Test sync strategy
    let config = SyncConfig::default();
    let sync_strategy = SyncStrategy::new(Arc::new(p2p.clone()), config);
    let mode = sync_strategy.determine_sync_mode(&topic_id, 0).await;
    
    // Should use P2P (3 online peers)
    assert_eq!(mode, SyncMode::P2POnly);
    
    println!("✓ Full integration test passed");
}

#[tokio::test]
async fn test_p2p_topic_id_conversion() {
    let topic_str = "test-topic-id";
    let topic_id = TopicId::from_string(topic_str);
    
    // Convert to hex and back
    let hex = topic_id.to_hex();
    assert!(!hex.is_empty());
    
    let topic_id2 = TopicId::from_hex(&hex).unwrap();
    assert_eq!(topic_id, topic_id2);
    
    println!("✓ Topic ID conversion test passed");
}

#[tokio::test]
async fn test_p2p_join_leave_events() {
    let p2p = P2PManager::new().await.unwrap();
    let topic_id = TopicId::new();
    
    // Subscribe
    let mut rx = p2p.gossip().subscribe().await;
    
    // Announce join
    p2p.gossip().announce_join(&topic_id, "user1").await.unwrap();
    let event = rx.recv().await.unwrap();
    if let exodus_hub_server::p2p::GossipEvent::Join { node_id, .. } = event {
        assert_eq!(node_id, "user1");
    } else {
        panic!("Expected Join event");
    }
    
    // Announce leave
    p2p.gossip().announce_leave(&topic_id, "user1").await.unwrap();
    let event = rx.recv().await.unwrap();
    if let exodus_hub_server::p2p::GossipEvent::Leave { node_id, .. } = event {
        assert_eq!(node_id, "user1");
    } else {
        panic!("Expected Leave event");
    }
    
    println!("✓ Join/Leave events test passed");
}

#[tokio::test]
async fn test_p2p_multiple_subscribers() {
    let p2p = P2PManager::new().await.unwrap();
    let topic_id = TopicId::new();
    
    // Multiple subscribers
    let mut rx1 = p2p.gossip().subscribe().await;
    let mut rx2 = p2p.gossip().subscribe().await;
    let mut rx3 = p2p.gossip().subscribe().await;
    
    // Send message
    p2p.gossip().send_message(&topic_id, "user1", "Broadcast test", 1).await.unwrap();
    
    // All subscribers should receive
    let event1 = rx1.recv().await.unwrap();
    let event2 = rx2.recv().await.unwrap();
    let event3 = rx3.recv().await.unwrap();
    
    // Verify all received the message
    assert!(matches!(event1, exodus_hub_server::p2p::GossipEvent::Message { .. }));
    assert!(matches!(event2, exodus_hub_server::p2p::GossipEvent::Message { .. }));
    assert!(matches!(event3, exodus_hub_server::p2p::GossipEvent::Message { .. }));
    
    println!("✓ Multiple subscribers test passed");
}
