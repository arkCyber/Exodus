//! Example 1-to-1 P2P communication
//!
//! Demonstrates direct peer-to-peer communication between two nodes.

use exodus_hub_server::p2p::{P2PManager, TopicId, NodeInfo};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 1-to-1 P2P Communication Example ===\n");

    // Create P2P manager for Node A
    let p2p_a = P2PManager::new().await?;
    println!("✓ Node A P2P Manager created");

    // Create P2P manager for Node B
    let p2p_b = P2PManager::new().await?;
    println!("✓ Node B P2P Manager created\n");

    // Create a private topic for 1-to-1 communication
    let topic_id = TopicId::from_string("private-chat-alice-bob");
    println!("✓ Private topic created: {}\n", topic_id);

    // Register Node A
    let node_a_addr: SocketAddr = "127.0.0.1:9001".parse()?;
    let node_a_info = NodeInfo {
        id: "alice".to_string(),
        address: node_a_addr,
        last_seen: chrono::Utc::now(),
        topics: vec![topic_id.to_hex()],
    };
    p2p_a.node().set_local_node(node_a_info.clone()).await;
    p2p_a.node().upsert_node(node_a_info.clone()).await;
    println!("✓ Node A registered: {} @ {}", node_a_info.id, node_a_info.address);

    // Register Node B
    let node_b_addr: SocketAddr = "127.0.0.1:9002".parse()?;
    let node_b_info = NodeInfo {
        id: "bob".to_string(),
        address: node_b_addr,
        last_seen: chrono::Utc::now(),
        topics: vec![topic_id.to_hex()],
    };
    p2p_b.node().set_local_node(node_b_info.clone()).await;
    p2p_b.node().upsert_node(node_b_info.clone()).await;
    println!("✓ Node B registered: {} @ {}\n", node_b_info.id, node_b_info.address);

    // Node A subscribes to the topic
    let mut rx_a = p2p_a.gossip().subscribe().await;
    println!("✓ Node A subscribed to topic");

    // Node B subscribes to the topic
    let mut rx_b = p2p_b.gossip().subscribe().await;
    println!("✓ Node B subscribed to topic\n");

    // Node A announces join
    p2p_a.gossip().announce_join(&topic_id, "alice").await?;
    println!("📢 Node A (alice) joined the topic");

    // Node B announces join
    p2p_b.gossip().announce_join(&topic_id, "bob").await?;
    println!("📢 Node B (bob) joined the topic\n");

    // Receive join events
    let event_a1 = rx_a.recv().await?;
    println!("📨 Node A received: {:?}", event_a1);
    let event_a2 = rx_a.recv().await?;
    println!("📨 Node A received: {:?}\n", event_a2);

    println!("=== 1-to-1 Chat ===\n");

    // Alice sends first message
    p2p_a.gossip().send_message(&topic_id, "alice", "Hi Bob, how are you?", 1).await?;
    println!("💬 Alice (seq 1): Hi Bob, how are you?");

    // Bob receives the message
    let msg_b1 = rx_b.recv().await?;
    if let exodus_hub_server::p2p::GossipEvent::Message { sender_id, content, sequence, .. } = msg_b1 {
        println!("📨 Bob received from {} (seq {}): {}", sender_id, sequence, content);
    }

    sleep(Duration::from_millis(500)).await;

    // Bob replies
    p2p_b.gossip().send_message(&topic_id, "bob", "I'm good! Thanks for asking. How about you?", 2).await?;
    println!("💬 Bob (seq 2): I'm good! Thanks for asking. How about you?");

    // Alice receives the reply
    let msg_a1 = rx_a.recv().await?;
    if let exodus_hub_server::p2p::GossipEvent::Message { sender_id, content, sequence, .. } = msg_a1 {
        println!("📨 Alice received from {} (seq {}): {}", sender_id, sequence, content);
    }

    sleep(Duration::from_millis(500)).await;

    // Alice replies
    p2p_a.gossip().send_message(&topic_id, "alice", "I'm doing great! Working on some code.", 3).await?;
    println!("💬 Alice (seq 3): I'm doing great! Working on some code.");

    // Bob receives the message
    let msg_b2 = rx_b.recv().await?;
    if let exodus_hub_server::p2p::GossipEvent::Message { sender_id, content, sequence, .. } = msg_b2 {
        println!("📨 Bob received from {} (seq {}): {}", sender_id, sequence, content);
    }

    sleep(Duration::from_millis(500)).await;

    // Bob sends final message
    p2p_b.gossip().send_message(&topic_id, "bob", "That's awesome! Let me know if you need help.", 4).await?;
    println!("💬 Bob (seq 4): That's awesome! Let me know if you need help.");

    // Alice receives the final message
    let msg_a2 = rx_a.recv().await?;
    if let exodus_hub_server::p2p::GossipEvent::Message { sender_id, content, sequence, .. } = msg_a2 {
        println!("📨 Alice received from {} (seq {}): {}", sender_id, sequence, content);
    }

    sleep(Duration::from_millis(500)).await;

    // Alice leaves
    p2p_a.gossip().announce_leave(&topic_id, "alice").await?;
    println!("📢 Alice left the topic");

    // Bob receives leave event
    let leave_event = rx_b.recv().await?;
    println!("📨 Bob received: {:?}", leave_event);

    println!("\n=== Communication Summary ===");
    println!("Topic ID: {}", topic_id);
    println!("Participants: alice, bob");
    println!("Messages exchanged: 4");
    println!("\n✓ 1-to-1 P2P communication test completed successfully!");

    Ok(())
}
