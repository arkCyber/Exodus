# Message Synchronization API

## Overview

The Exodus Hub Server provides two API endpoints for synchronizing historical messages, supporting both JSON and compressed binary formats for efficient data transfer.

## API Endpoints

### 1. GET /api/sync/messages

Get historical messages in JSON format with pagination support.

**Query Parameters:**
- `conversation_id` (required): The ID of the conversation to sync
- `after_sequence` (optional): Only return messages with sequence number greater than this value
- `limit` (optional): Maximum number of messages to return (default: 100)

**Response:**
```json
{
  "messages": [
    {
      "id": "uuid",
      "conversation_id": "uuid",
      "sender_id": "user_id",
      "receiver_id": "user_id",
      "content": "message content",
      "timestamp": "2024-01-01T00:00:00Z",
      "sequence": 1,
      "reply_to": "uuid",
      "integrity_hash": "hash"
    }
  ],
  "has_more": true,
  "last_sequence": 100
}
```

**Example:**
```bash
curl "http://localhost:3000/api/sync/messages?conversation_id=xxx&after_sequence=50&limit=100"
```

### 2. GET /api/sync/compressed

Get historical messages in compressed binary format for efficient transfer.

**Query Parameters:**
- `conversation_id` (required): The ID of the conversation to sync
- `after_sequence` (optional): Only return messages with sequence number greater than this value
- `limit` (optional): Maximum number of messages to return (default: 1000)

**Response:**
- Content-Type: `application/octet-stream`
- Content-Encoding: `zstd`
- Body: Compressed binary data

**Example:**
```bash
curl "http://localhost:3000/api/sync/compressed?conversation_id=xxx&after_sequence=50&limit=1000" \
  --output messages.zst
```

## Client Implementation

### Rust Client Example

```rust
use reqwest::Client;
use exodus_hub_server::manager::{ImManager, Message};

async fn sync_messages(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
    after_sequence: Option<u32>,
) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let mut url = format!("{}/api/sync/messages", base_url);
    url.push_str(&format!("?conversation_id={}", conversation_id));
    
    if let Some(seq) = after_sequence {
        url.push_str(&format!("&after_sequence={}", seq));
    }
    
    let response = client.get(&url).send().await?;
    let sync_response: serde_json::Value = response.json().await?;
    
    let messages: Vec<Message> = serde_json::from_value(
        sync_response["messages"].clone()
    )?;
    
    Ok(messages)
}

async fn sync_messages_compressed(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
    after_sequence: Option<u32>,
) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let mut url = format!("{}/api/sync/compressed", base_url);
    url.push_str(&format!("?conversation_id={}", conversation_id));
    
    if let Some(seq) = after_sequence {
        url.push_str(&format!("&after_sequence={}", seq));
    }
    
    let response = client.get(&url).send().await?;
    let compressed = response.bytes().await?;
    let messages = ImManager::decompress_messages(&compressed)?;
    
    Ok(messages)
}

// Batch sync with pagination
async fn sync_all_messages(
    client: &Client,
    base_url: &str,
    conversation_id: &str,
) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    let mut all_messages = Vec::new();
    let mut after_sequence = None;
    
    loop {
        let messages = sync_messages(client, base_url, conversation_id, after_sequence).await?;
        
        if messages.is_empty() {
            break;
        }
        
        let last_seq = messages.last().and_then(|m| m.sequence);
        all_messages.extend(messages);
        
        // Check if there are more messages
        let has_more = last_seq.is_some();
        if !has_more {
            break;
        }
        
        after_sequence = last_seq;
    }
    
    Ok(all_messages)
}
```

### JavaScript/TypeScript Client Example

```typescript
interface SyncResponse {
  messages: Message[];
  has_more: boolean;
  last_sequence: number;
}

interface Message {
  id: string;
  conversation_id: string;
  sender_id: string;
  receiver_id?: string;
  content: string;
  timestamp: string;
  sequence?: number;
  reply_to?: string;
  integrity_hash?: string;
}

async function syncMessages(
  baseUrl: string,
  conversationId: string,
  afterSequence?: number
): Promise<SyncResponse> {
  const url = new URL(`${baseUrl}/api/sync/messages`);
  url.searchParams.append('conversation_id', conversationId);
  if (afterSequence !== undefined) {
    url.searchParams.append('after_sequence', afterSequence.toString());
  }
  
  const response = await fetch(url.toString());
  const data: SyncResponse = await response.json();
  return data;
}

async function syncAllMessages(
  baseUrl: string,
  conversationId: string
): Promise<Message[]> {
  const allMessages: Message[] = [];
  let afterSequence: number | undefined = undefined;
  
  while (true) {
    const response = await syncMessages(baseUrl, conversationId, afterSequence);
    allMessages.push(...response.messages);
    
    if (!response.has_more) {
      break;
    }
    
    afterSequence = response.last_sequence;
  }
  
  return allMessages;
}
```

## Best Practices

1. **Use Compressed Format**: For large message histories (1000+ messages), use the compressed endpoint to reduce bandwidth by 70-90%.

2. **Batch Processing**: Implement pagination to handle large datasets efficiently.

3. **Error Handling**: Always handle network errors and retry failed requests with exponential backoff.

4. **Sequence Tracking**: Store the last sequence number locally to enable incremental sync.

5. **Integrity Verification**: Verify message integrity using the `integrity_hash` field when security is critical.

## Performance Considerations

- **Compression Ratio**: Typical compression ratio is 70-90% for text messages
- **Batch Size**: Recommended batch size is 100-1000 messages
- **Network**: Use compressed format for mobile networks or slow connections
- **Storage**: Decompressed messages should be stored locally for offline access

## Sequence Number System

The system uses cyclic sequence numbers (1-9999) per sender:
- Messages from the same sender have sequential numbers
- Sequence wraps around after reaching 9999
- Use `after_sequence` parameter to get messages newer than a specific sequence
- The `last_sequence` in response indicates the highest sequence in the batch
