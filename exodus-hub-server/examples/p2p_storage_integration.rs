//! Example demonstrating P2P message storage integration
//!
//! Shows how P2P messages are automatically stored to the database
//! for reliability and offline access.

use exodus_hub_server::p2p::{P2PManager, TopicId};
use exodus_hub_server::ImManager;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== P2P Storage Integration Example ===\n");

    // Create temporary database
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_p2p_storage.db");
    let im_manager = Arc::new(ImManager::new(db_path)?);
    println!("✓ Database created");

    // Create P2P manager with storage integration
    let p2p = P2PManager::with_storage(im_manager.clone()).await?;
    println!("✓ P2P Manager with storage integration created\n");

    // Create a topic
    let topic_id = TopicId::from_string("storage-test-topic");
    println!("✓ Topic created: {}\n", topic_id);

    // Subscribe to gossip events
    let mut rx = p2p.gossip().subscribe().await;
    println!("✓ Subscribed to gossip events\n");

    println!("=== Sending Messages ===\n");

    // Send messages with sequence numbers
    for i in 1..=5 {
        let sequence = i;
        p2p.gossip().send_message(&topic_id, "user1", &format!("Message {}", i), sequence).await?;
        println!("💬 Sent message {} (seq {})", i, sequence);

        // Handle event (stores to database)
        let event = rx.recv().await?;
        p2p.handle_event(&event).await?;
        println!("📨 Event handled and stored to database\n");

        sleep(Duration::from_millis(100)).await;
    }

    println!("=== Retrieving Messages from Database ===\n");

    // Retrieve messages from database
    if let Some(storage) = p2p.storage() {
        // Use the topic_id directly as conversation_id (they should match)
        let messages = storage.get_missing_messages(&topic_id.to_string(), 0, 10).await?;
        println!("📦 Retrieved {} messages from database", messages.len());
        
        for msg in messages {
            println!("  - Seq {:?}: {}", msg.sequence, msg.content);
        }
    }

    println!("\n=== Testing Receipt Mechanism ===\n");

    // Send a receipt for message delivery
    p2p.gossip().send_receipt(&topic_id, "msg-123", "user2", 1).await?;
    println!("📤 Sent receipt for message 1");

    // Handle receipt event
    let receipt_event = rx.recv().await?;
    p2p.handle_event(&receipt_event).await?;
    println!("📥 Receipt handled and stored\n");

    println!("✓ P2P storage integration test completed successfully!");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ P2P messages automatically stored to database");
    println!("  ✓ Sequence numbers for message ordering");
    println!("  ✓ Receipt mechanism for delivery confirmation");
    println!("  ✓ Messages can be retrieved from database");

    Ok(())
}
