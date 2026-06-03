//! Example client for message synchronization
//!
//! This example demonstrates how to use the sync API to fetch historical messages
//! from the Exodus Hub Server.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize)]
struct SyncResponse {
    messages: Vec<Message>,
    has_more: bool,
    last_sequence: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    id: String,
    conversation_id: String,
    sender_id: String,
    receiver_id: Option<String>,
    content: String,
    timestamp: String,
    sequence: Option<u32>,
    reply_to: Option<String>,
    integrity_hash: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let base_url = "http://localhost:3000";
    let conversation_id = "your-conversation-id";

    println!("Starting message synchronization...");

    // Example 1: Sync all messages (JSON format)
    println!("\n=== Syncing all messages (JSON) ===");
    let messages = sync_all_messages(&client, base_url, conversation_id).await?;
    println!("Fetched {} messages", messages.len());

    // Example 2: Sync messages after a specific sequence
    println!("\n=== Syncing messages after sequence 50 ===");
    let messages = sync_messages(&client, base_url, conversation_id, Some(50)).await?;
    println!("Fetched {} messages", messages.len());

    // Example 3: Sync with custom batch size
    println!("\n=== Syncing with batch size 50 ===");
    let messages = sync_messages_with_limit(&client, base_url, conversation_id, None, 50).await?;
    println!("Fetched {} messages", messages.len());

    Ok(())
}

/// Sync all messages for a conversation using pagination
async fn sync_all_messages(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
) -> Result<Vec<Message>, Box<dyn Error>> {
    let mut all_messages = Vec::new();
    let mut after_sequence = None;
    let mut batch_count = 0;

    loop {
        batch_count += 1;
        let response = fetch_sync_response(client, base_url, conversation_id, after_sequence).await?;
        
        println!("  Batch {}: fetched {} messages", batch_count, response.messages.len());
        
        if response.messages.is_empty() {
            break;
        }
        
        all_messages.extend(response.messages);
        
        if !response.has_more {
            break;
        }
        
        after_sequence = Some(response.last_sequence);
    }

    Ok(all_messages)
}

/// Fetch sync response from server
async fn fetch_sync_response(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
    after_sequence: Option<u32>,
) -> Result<SyncResponse, Box<dyn Error>> {
    let mut url = format!("{}/api/sync/messages", base_url);
    url.push_str(&format!("?conversation_id={}", conversation_id));
    
    if let Some(seq) = after_sequence {
        url.push_str(&format!("&after_sequence={}", seq));
    }
    
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<SyncResponse>()
        .await?;
    
    Ok(response)
}

/// Sync messages with optional after_sequence parameter
async fn sync_messages(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
    after_sequence: Option<u32>,
) -> Result<Vec<Message>, Box<dyn Error>> {
    let response = fetch_sync_response(client, base_url, conversation_id, after_sequence).await?;
    Ok(response.messages)
}

/// Sync messages with custom limit
async fn sync_messages_with_limit(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
    after_sequence: Option<u32>,
    limit: u32,
) -> Result<Vec<Message>, Box<dyn Error>> {
    let mut url = format!("{}/api/sync/messages", base_url);
    url.push_str(&format!("?conversation_id={}", conversation_id));
    url.push_str(&format!("&limit={}", limit));
    
    if let Some(seq) = after_sequence {
        url.push_str(&format!("&after_sequence={}", seq));
    }
    
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<SyncResponse>()
        .await?;
    
    Ok(response.messages)
}

/// Sync messages using compressed format (more efficient)
async fn sync_messages_compressed(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
    after_sequence: Option<u32>,
) -> Result<Vec<Message>, Box<dyn Error>> {
    let mut url = format!("{}/api/sync/compressed", base_url);
    url.push_str(&format!("?conversation_id={}", conversation_id));
    url.push_str("&limit=1000");
    
    if let Some(seq) = after_sequence {
        url.push_str(&format!("&after_sequence={}", seq));
    }
    
    let response = client.get(&url).send().await?;
    let compressed = response.bytes().await?;
    
    // Decompress using zstd
    let decompressed = zstd::decode_all(&compressed[..])?;
    let messages: Vec<Message> = bincode::deserialize(&decompressed)?;
    
    Ok(messages)
}
