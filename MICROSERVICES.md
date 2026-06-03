# Exodus Microservices Documentation

## Overview

Exodus uses a microservice architecture with Unix Domain Socket (UDS) JSON-RPC communication. Each microservice runs independently and exposes functionality through Tauri commands for frontend interaction.

## Architecture

### Communication Protocol
- **Transport**: Unix Domain Sockets (UDS)
- **Protocol**: JSON-RPC 2.0
- **Concurrency**: Thread-safe shared state using `Arc<Mutex<...>>`
- **Async Runtime**: Tokio

### Service Lifecycle
1. Service configuration with socket path and storage directory
2. Service instantiation with shared state
3. Start service (binds to UDS socket)
4. JSON-RPC client-server communication
5. Stop service (cleanup socket and resources)

## Microservices

### 1. RAG Service (`rag_service.rs`)

**Purpose**: Retrieval-Augmented Generation for AI-powered search and content indexing.

**Features**:
- Vector embedding storage and retrieval
- Semantic search with similarity scoring
- Bookmark and visit tracking
- Page content capture and indexing

**Tauri Commands**:
- `rag_index_page` - Index page content with embeddings
- `rag_search` - Semantic search
- `rag_get_bookmarks` - Retrieve bookmarks
- `rag_add_bookmark` - Add bookmark
- `rag_delete_bookmark` - Delete bookmark

**Socket Path**: `/tmp/exodus_rag.sock`
**Storage**: `/tmp/exodus_rag/`

---

### 2. Crypto Service (`crypto_service.rs`)

**Purpose**: Cryptographic operations for secure data handling.

**Features**:
- Encryption/decryption
- Hash generation
- Digital signatures
- Key management

**Tauri Commands**:
- `crypto_encrypt` - Encrypt data
- `crypto_decrypt` - Decrypt data
- `crypto_hash` - Generate hash
- `crypto_sign` - Sign data
- `crypto_verify` - Verify signature

**Socket Path**: `/tmp/exodus_crypto.sock`
**Storage**: `/tmp/exodus_crypto/`

---

### 3. OS Service (`os_service.rs`)

**Purpose**: Operating system level operations and system monitoring.

**Features**:
- System information retrieval
- Process management
- File system operations
- Network monitoring

**Tauri Commands**:
- `os_get_system_info` - Get system information
- `os_get_process_info` - Get process information
- `os_list_processes` - List running processes
- `os_get_network_info` - Get network information

**Socket Path**: `/tmp/exodus_os.sock`
**Storage**: `/tmp/exodus_os/`

---

### 4. P2P Blobs Service (`p2p_blobs_service.rs`)

**Purpose**: Peer-to-peer blob sharing using iroh-blobs.

**Features**:
- Blob storage and retrieval
- P2P data sharing
- Ticket-based access control
- Hash verification

**Tauri Commands**:
- `p2p_blobs_service_start/stop` - Service lifecycle
- `p2p_blobs_store` - Store blob
- `p2p_blobs_retrieve` - Retrieve blob
- `p2p_blobs_list` - List blobs
- `p2p_blobs_delete` - Delete blob
- `p2p_blobs_get_ticket` - Get access ticket

**Socket Path**: `/tmp/exodus_p2p_blobs.sock`
**Storage**: `/tmp/exodus_p2p_blobs/`

---

### 5. P2P Gossip Service (`p2p_gossip_service.rs`)

**Purpose**: P2P state synchronization using gossip protocol.

**Features**:
- State replication across nodes
- Gossip message propagation
- Node discovery
- Conflict resolution

**Tauri Commands**:
- `p2p_gossip_service_start/stop` - Service lifecycle
- `p2p_gossip_publish` - Publish state
- `p2p_gossip_subscribe` - Subscribe to updates
- `p2p_gossip_get_state` - Get current state
- `p2p_gossip_list_peers` - List connected peers

**Socket Path**: `/tmp/exodus_p2p_gossip.sock`
**Storage**: `/tmp/exodus_p2p_gossip/`

---

### 6. AI Model Service (`ai_model_service.rs`)

**Purpose**: AI model sharing and registry.

**Features**:
- Model registration and metadata
- Model search by capability
- Model versioning
- Node discovery

**Tauri Commands**:
- `ai_model_service_start/stop` - Service lifecycle
- `ai_model_register` - Register model
- `ai_model_unregister` - Unregister model
- `ai_model_get` - Get model by ID
- `ai_model_search` - Search models
- `ai_model_list` - List all models
- `ai_model_node_info` - Get node information

**Socket Path**: `/tmp/exodus_ai_model.sock`
**Storage**: `/tmp/exodus_ai_models/`

---

### 7. File Transfer Service (`file_transfer_service.rs`)

**Purpose**: File transfer using P2P blobs.

**Features**:
- File chunking and reassembly
- Transfer progress tracking
- P2P file sharing
- Transfer metadata

**Tauri Commands**:
- `file_transfer_service_start/stop` - Service lifecycle
- `file_transfer_initiate` - Initiate transfer
- `file_transfer_get` - Get transfer info
- `file_transfer_list` - List transfers
- `file_transfer_get_chunks` - Get file chunks
- `file_transfer_update_status` - Update transfer status
- `file_transfer_cancel` - Cancel transfer

**Socket Path**: `/tmp/exodus_file_transfer.sock`
**Storage**: `/tmp/exodus_file_transfers/`

---

### 8. AI Agent Service (`ai_agent_service.rs`)

**Purpose**: AI agent communication and presence.

**Features**:
- Agent identity management
- Message passing
- Presence broadcasting
- Agent discovery

**Tauri Commands**:
- `ai_agent_service_start/stop` - Service lifecycle
- `ai_agent_register` - Register agent
- `ai_agent_unregister` - Unregister agent
- `ai_agent_get` - Get agent info
- `ai_agent_list` - List agents
- `ai_agent_find` - Find agents by capability
- `ai_agent_send_message` - Send message
- `ai_agent_get_messages` - Get messages
- `ai_agent_broadcast_presence` - Broadcast presence

**Socket Path**: `/tmp/exodus_ai_agent.sock`
**Storage**: `/tmp/exodus_ai_agents/`

---

### 9. Contact Directory Service (`contact_directory_service.rs`)

**Purpose**: Contact management and discovery.

**Features**:
- Contact CRUD operations
- Contact groups
- Presence tracking
- Contact recommendations
- Search and discovery

**Tauri Commands**:
- `contact_service_start/stop` - Service lifecycle
- `contact_add` - Add contact
- `contact_remove` - Remove contact
- `contact_update` - Update contact
- `contact_get` - Get contact
- `contact_list` - List contacts
- `contact_search` - Search contacts
- `contact_group_create` - Create group
- `contact_group_update` - Update group
- `contact_group_delete` - Delete group
- `contact_group_get` - Get group
- `contact_group_list` - List groups
- `contact_group_add_member` - Add member to group
- `contact_group_remove_member` - Remove member from group
- `contact_get_by_agent` - Get contact by agent ID
- `contact_update_presence` - Update presence
- `contact_get_online` - Get online contacts
- `contact_recommend` - Get recommendations

**Socket Path**: `/tmp/exodus_contact_directory.sock`
**Storage**: `/tmp/exodus_contacts/`

---

### 10. Group Chat Service (`group_chat_service.rs`)

**Purpose**: Group-based messaging and collaboration.

**Features**:
- Group creation and management
- Member management
- Message sending and retrieval
- Group invitations
- Member online status
- Group search

**Tauri Commands**:
- `group_chat_service_start/stop` - Service lifecycle
- `group_create` - Create group
- `group_update` - Update group
- `group_delete` - Delete group
- `group_get` - Get group info
- `group_list_user` - List user's groups
- `group_add_member` - Add member
- `group_remove_member` - Remove member
- `group_get_members` - Get group members
- `group_send_message` - Send message
- `group_get_messages` - Get messages
- `group_edit_message` - Edit message
- `group_delete_message` - Delete message
- `group_create_invitation` - Create invitation
- `group_accept_invitation` - Accept invitation
- `group_reject_invitation` - Reject invitation
- `group_get_pending_invitations` - Get pending invitations
- `group_update_member_online` - Update member online status
- `group_search` - Search groups

**Socket Path**: `/tmp/exodus_group_chat.sock`
**Storage**: `/tmp/exodus_group_chats/`

---

### 11. Social Feed Service (`social_feed_service.rs`)

**Purpose**: Social media-like feed functionality.

**Features**:
- Post creation and management
- Timeline generation
- Comments and reactions
- User following
- Feed search

**Tauri Commands**:
- `social_feed_service_start/stop` - Service lifecycle
- `social_post_create` - Create post
- `social_post_update` - Update post
- `social_post_delete` - Delete post
- `social_post_get` - Get post
- `social_post_get_user` - Get user's posts
- `social_feed_get_timeline` - Get timeline
- `social_post_search` - Search posts
- `social_comment_add` - Add comment
- `social_comment_get` - Get comments
- `social_comment_delete` - Delete comment
- `social_reaction_add` - Add reaction
- `social_reaction_remove` - Remove reaction
- `social_reaction_get` - Get reactions
- `social_user_follow` - Follow user
- `social_user_unfollow` - Unfollow user
- `social_user_get_followings` - Get followings
- `social_user_get_followers` - Get followers
- `social_user_is_following` - Check if following

**Socket Path**: `/tmp/exodus_social_feed.sock`
**Storage**: `/tmp/exodus_social_feed/`

---

### 12. Agent Discovery Service (`agent_discovery_service.rs`)

**Purpose**: AI agent discovery and recommendation.

**Features**:
- Agent registration
- Activity tracking
- Discovery by criteria
- Trending agents
- Capability search

**Tauri Commands**:
- `agent_discovery_service_start/stop` - Service lifecycle
- `agent_discovery_register` - Register agent
- `agent_discovery_update_activity` - Update activity
- `agent_discovery_discover` - Discover agents
- `agent_discovery_trending` - Get trending agents
- `agent_discovery_search_capability` - Search by capability

**Socket Path**: `/tmp/exodus_agent_discovery.sock`
**Storage**: `/tmp/exodus_agent_discovery/`

---

### 13. Media Streaming Service (`media_streaming_service.rs`)

**Purpose**: Real-time video and audio streaming.

**Features**:
- Stream session management
- Viewer management
- Quality settings (360p, 720p, 1080p, 4K)
- Active streams listing
- Stream search
- Trending streams

**Tauri Commands**:
- `media_streaming_service_start/stop` - Service lifecycle
- `media_stream_create` - Create stream
- `media_stream_update` - Update stream
- `media_stream_end` - End stream
- `media_stream_get` - Get stream
- `media_stream_list_active` - List active streams
- `media_stream_list_user` - List user's streams
- `media_stream_join` - Join as viewer
- `media_stream_leave` - Leave stream
- `media_stream_get_viewers` - Get viewers
- `media_stream_get_qualities` - Get quality options
- `media_stream_search` - Search streams
- `media_stream_get_trending` - Get trending streams

**Socket Path**: `/tmp/exodus_media_streaming.sock`
**Storage**: `/tmp/exodus_media_streams/`

---

### 14. News Aggregation Service (`news_aggregation_service.rs`)

**Purpose**: News aggregation from various sources.

**Features**:
- Article management
- Source management (RSS, API, custom)
- Category indexing
- Article search
- Custom feeds with filters
- Statistics

**Tauri Commands**:
- `news_aggregation_service_start/stop` - Service lifecycle
- `news_add_article` - Add article
- `news_add_source` - Add source
- `news_update_source` - Update source
- `news_remove_source` - Remove source
- `news_get_article` - Get article
- `news_get_articles_by_source` - Get articles by source
- `news_get_articles_by_category` - Get articles by category
- `news_search_articles` - Search articles
- `news_get_latest_articles` - Get latest articles
- `news_get_sources` - Get all sources
- `news_create_feed` - Create custom feed
- `news_get_feed_articles` - Get feed articles
- `news_get_statistics` - Get statistics

**Socket Path**: `/tmp/exodus_news_aggregation.sock`
**Storage**: `/tmp/exodus_news/`

---

### 15. P2P CDN Module (`p2p_cdn/`)

**Purpose**: Content delivery network for P2P asset distribution.

**Features**:
- Asset announcement
- Room-based sharing
- Peer discovery
- Asset downloading
- Local seed registration

**Tauri Commands**:
- `p2p_cdn_node_info` - Get node information
- `p2p_cdn_join_room` - Join CDN room
- `p2p_cdn_leave_room` - Leave CDN room
- `p2p_cdn_room_feed` - Get room feed
- `p2p_cdn_list_peers` - List peers in room
- `p2p_cdn_announce_asset` - Announce asset
- `p2p_cdn_register_local_seed` - Register local seed
- `p2p_cdn_download` - Download asset
- `p2p_cdn_get_asset` - Get asset info

---

## Integration Guide

### Adding a New Microservice

1. **Create Service File** (`src-tauri/src/microservice/<service_name>_service.rs`):
   ```rust
   use serde::{Deserialize, Serialize};
   use std::collections::HashMap;
   use std::path::PathBuf;
   use std::sync::{Arc, Mutex};
   use tokio::net::UnixListener;
   // ... implement service with JSON-RPC handlers
   ```

2. **Create Commands File** (`src-tauri/src/microservice/<service_name>_commands.rs`):
   ```rust
   use tauri::Command;
   // ... implement Tauri commands that call JSON-RPC
   ```

3. **Update Module Declaration** (`src-tauri/src/microservice/mod.rs`):
   ```rust
   pub mod <service_name>_service;
   pub mod <service_name>_commands;
   pub use <service_name>_service::{<Service>, <Config>, <DataStructs>};
   ```

4. **Import Commands** (`src-tauri/src/lib.rs`):
   ```rust
   use microservice::<service_name>_commands::{<command1>, <command2>, ...};
   ```

5. **Register Commands** (`src-tauri/src/lib.rs`):
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       <command1>,
       <command2>,
   ])
   ```

### Service Configuration Pattern

```rust
#[derive(Debug, Clone)]
pub struct <Service>Config {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
}

impl Default for <Service>Config {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_<service>.sock");
        
        let mut storage_dir = std::env::temp_dir();
        storage_dir.push("exodus_<service>");
        
        Self { socket_path, storage_dir }
    }
}
```

### JSON-RPC Handler Pattern

```rust
async fn handle_<operation>(
    params: &serde_json::Value,
    <state>: &Arc<Mutex<HashMap<String, <Data>>>>,
) -> Result<serde_json::Value, String> {
    // Parse parameters
    let param: <Type> = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid param: {}", e))?;
    
    // Acquire lock
    let mut guard = <state>.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    // Perform operation
    guard.insert(key, value);
    
    // Return result
    Ok(json!({ "result": "success" }))
}
```

### Tauri Command Pattern

```rust
#[tauri::command]
pub async fn <service>_<operation>(
    <param>: <Type>,
) -> Result<<ReturnType>, String> {
    let config = <Service>Config::default();
    let params = json!({ "param": <param> });
    let result = send_<service>_request(&config.socket_path, "<operation>", params).await?;
    
    serde_json::from_value(result).map_err(|e| e.to_string())
}
```

## Testing

### Unit Tests

Each service includes unit tests in the service file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_<operation>() {
        let config = <Service>Config::default();
        let service = <Service>::new(config).unwrap();
        // ... test implementation
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific service tests
cargo test --package exodus-tauri --lib <service_name>_service::tests
```

## Deployment

### Service Startup Order

Services can be started independently, but some dependencies exist:

1. **Core Services** (no dependencies):
   - RAG Service
   - Crypto Service
   - OS Service

2. **P2P Services** (depend on core):
   - P2P Blobs Service
   - P2P Gossip Service
   - P2P CDN

3. **Application Services** (depend on P2P):
   - AI Model Service
   - File Transfer Service
   - AI Agent Service

4. **Communication Services** (depend on Application):
   - Contact Directory Service
   - Group Chat Service
   - Social Feed Service

5. **Discovery and Media** (independent):
   - Agent Discovery Service
   - Media Streaming Service
   - News Aggregation Service

### Service Health Monitoring

Each service provides:
- `is_running()` method to check status
- `node_info()` command to get node information
- Error handling with descriptive messages

## Troubleshooting

### Common Issues

1. **Socket Already in Use**
   - Check if service is already running
   - Remove socket file: `rm /tmp/exodus_<service>.sock`

2. **Storage Directory Missing**
   - Service auto-creates directory on startup
   - Check permissions on `/tmp/`

3. **Lock Errors**
   - Indicates concurrent access issues
   - Ensure proper lock ordering
   - Use `drop()` guards when needed

4. **JSON-RPC Errors**
   - Check parameter types match
   - Verify method name is correct
   - Check socket path is accessible

### Debug Mode

Enable debug logging by setting environment variable:

```bash
RUST_LOG=debug cargo run
```

## Performance Considerations

1. **Lock Contention**: Minimize lock duration, use `drop()` guards
2. **Memory Usage**: Monitor HashMap sizes, implement cleanup if needed
3. **Socket Buffer**: Default 8192 bytes, adjust if needed for large payloads
4. **Async Operations**: Use `tokio::spawn` for concurrent operations

## Security Considerations

1. **Socket Permissions**: Unix Domain Sockets are local-only
2. **Input Validation**: Validate all JSON-RPC parameters
3. **Error Messages**: Don't expose sensitive information in errors
4. **Storage Encryption**: Use Crypto Service for sensitive data

## Future Enhancements

- [ ] Add service health monitoring dashboard
- [ ] Implement service auto-restart
- [ ] Add metrics collection
- [ ] Implement service discovery
- [ ] Add load balancing for high-demand services
- [ ] Implement persistent storage for state
- [ ] Add service versioning and migration
- [ ] Implement rate limiting
- [ ] Add audit logging

## License

Part of the Exodus project. See LICENSE file for details.
