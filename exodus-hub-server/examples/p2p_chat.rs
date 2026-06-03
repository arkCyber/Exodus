//! Example P2P chat application
//!
//! Demonstrates point-to-point communication using the gossip protocol.

use exodus_hub_server::p2p::{P2PManager, TopicManager, TopicId, NodeManager, NodeInfo, GossipManager, GossipEvent};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== P2P Chat Example ===\n");

    // Create P2P manager
    let p2p = P2PManager::new().await?;
    println!("✓ P2P Manager created");

    // Create a topic (group chat)
    let topic_id = p2p.topic().create_topic("Test Group").await?;
    println!("✓ Topic created: {}", topic_id);

    // Create two nodes (simulating two users)
    let node1_addr: SocketAddr = "127.0.0.1:8081".parse()?;
    let node1_info = NodeInfo {
        id: "user1".to_string(),
        address: node1_addr,
        last_seen: chrono::Utc::now(),
        topics: vec![topic_id.to_hex()],
    };
    p2p.node().upsert_node(node1_info.clone()).await;
    println!("✓ Node 1 created: {} @ {}", node1_info.id, node1_info.address);

    let node2_addr: SocketAddr = "127.0.0.1:8082".parse()?;
    let node2_info = NodeInfo {
        id: "user2".to_string(),
        address: node2_addr,
        last_seen: chrono::Utc::now(),
        topics: vec![topic_id.to_hex()],
    };
    p2p.node().upsert_node(node2_info.clone()).await;
    println!("✓ Node 2 created: {} @ {}", node2_info.id, node2_info.address);

    // Subscribe to gossip events
    let mut rx = p2p.gossip().subscribe().await;
    println!("✓ Subscribed to gossip events\n");

    // Simulate node 1 joining the topic
    p2p.gossip().announce_join(&topic_id, "user1").await?;
    println!("📢 User 1 joined the topic");

    // Simulate node 2 joining the topic
    p2p.gossip().announce_join(&topic_id, "user2").await?;
    println!("📢 User 2 joined the topic\n");

    // Receive join events
    let event1 = rx.recv().await?;
    println!("📨 Received: {:?}", event1);
    let event2 = rx.recv().await?;
    println!("📨 Received: {:?}\n", event2);

    // Simulate chat messages
    println!("=== Chat Simulation ===\n");

    // User 1 sends a message
    p2p.gossip().send_message(&topic_id, "user1", "Hello from user 1!", 1).await?;
    println!("💬 User 1: Hello from user 1!");

    // Receive message
    let msg_event = rx.recv().await?;
    if let GossipEvent::Message { sender_id, content, sequence, .. } = msg_event {
        println!("📨 Received message from {} (seq {}): {}", sender_id, sequence, content);
    }

    sleep(Duration::from_millis(500)).await;

    // User 2 sends a message
    p2p.gossip().send_message(&topic_id, "user2", "Hello from user 2!", 2).await?;
    println!("💬 User 2: Hello from user 2!");

    // Receive message
    let msg_event = rx.recv().await?;
    if let GossipEvent::Message { sender_id, content, sequence, .. } = msg_event {
        println!("📨 Received message from {} (seq {}): {}", sender_id, sequence, content);
    }

    sleep(Duration::from_millis(500)).await;

    // Simulate node 2 leaving
    p2p.gossip().announce_leave(&topic_id, "user2").await?;
    println!("📢 User 2 left the topic");

    // Receive leave event
    let leave_event = rx.recv().await?;
    println!("📨 Received: {:?}", leave_event);

    // Display topic info
    let metadata = p2p.topic().get_topic(&topic_id).await.unwrap();
    println!("\n=== Topic Info ===");
    println!("ID: {}", metadata.id);
    println!("Name: {}", metadata.name);
    println!("Member count: {}", metadata.member_count);

    // Display nodes
    let nodes = p2p.node().list_nodes().await;
    println!("\n=== Nodes ===");
    for node in nodes {
        println!("ID: {}, Address: {}, Topics: {:?}", node.id, node.address, node.topics);
    }

    println!("\n✓ P2P chat test completed successfully!");
    Ok(())
}
