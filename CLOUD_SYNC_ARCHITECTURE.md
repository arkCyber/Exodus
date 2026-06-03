# Cloud Sync Architecture Design

## Overview

Phase 3 cloud sync functionality for Exodus Browser to enable cross-device synchronization of bookmarks and history.

## Requirements

### Functional Requirements
1. **Bookmark Sync**: Synchronize bookmarks across multiple devices
2. **History Sync**: Synchronize browsing history across devices
3. **Conflict Resolution**: Handle conflicts when data is modified on multiple devices
4. **Authentication**: Secure user authentication for cloud sync
5. **Privacy**: Encrypt sensitive data before uploading to cloud
6. **Offline Support**: Continue working offline and sync when connection is available

### Non-Functional Requirements
1. **Security**: End-to-end encryption for user data
2. **Performance**: Efficient sync with minimal bandwidth usage
3. **Reliability**: Handle network failures gracefully
4. **Scalability**: Support growing user base and data volume

## Architecture Options

### Option 1: Self-Hosted Backend (Recommended for Privacy)

**Components:**
- Rust backend server (Actix-web or Axum)
- PostgreSQL database for user data
- Redis for caching and session management
- JWT for authentication
- WebSockets for real-time sync notifications

**Pros:**
- Full control over data
- Privacy-focused
- No vendor lock-in
- Customizable

**Cons:**
- Requires infrastructure maintenance
- Higher development effort
- Need to handle scaling

**Implementation:**
```rust
// Backend structure
src-tauri/src/
  - sync.rs          // Cloud sync client
  - crypto.rs        // Encryption utilities
  - conflict.rs      // Conflict resolution
```

### Option 2: Third-Party Cloud Service (E.g., Firebase, Supabase)

**Components:**
- Use existing cloud provider SDK
- Minimal backend development
- Built-in authentication
- Real-time sync capabilities

**Pros:**
- Faster development
- Built-in scalability
- Less maintenance
- Real-time sync out of the box

**Cons:**
- Vendor lock-in
- Privacy concerns
- Potential costs
- Limited customization

### Option 3: Decentralized (IPFS / Blockchain)

**Components:**
- IPFS for distributed storage
- Blockchain for identity and access control
- Peer-to-peer sync

**Pros:**
- Fully decentralized
- No single point of failure
- Censorship resistant

**Cons:**
- Complex implementation
- Performance concerns
- User experience challenges
- Not suitable for average users

## Recommended Architecture: Option 1 (Self-Hosted)

### System Components

#### 1. Authentication Service
```rust
// src-tauri/src/auth.rs
pub struct AuthService {
    jwt_secret: String,
    refresh_token_expiry: Duration,
    access_token_expiry: Duration,
}

impl AuthService {
    pub async fn register(email: &str, password: &str) -> Result<User, AuthError>;
    pub async fn login(email: &str, password: &str) -> Result<Tokens, AuthError>;
    pub async fn refresh_token(refresh_token: &str) -> Result<Token, AuthError>;
}
```

#### 2. Sync Service
```rust
// src-tauri/src/sync.rs
pub struct SyncService {
    api_base: String,
    auth_tokens: Arc<RwLock<Option<Tokens>>>,
    encryption_key: Arc<RwLock<Option<EncryptionKey>>>,
}

impl SyncService {
    pub async fn sync_bookmarks(&self) -> Result<SyncResult, SyncError>;
    pub async fn sync_history(&self) -> Result<SyncResult, SyncError>;
    pub async fn push_changes(&self, changes: Vec<SyncChange>) -> Result<(), SyncError>;
    pub async fn pull_changes(&self, since: DateTime<Utc>) -> Result<Vec<SyncChange>, SyncError>;
}
```

#### 3. Encryption Service
```rust
// src-tauri/src/crypto.rs
pub struct CryptoService;

impl CryptoService {
    pub fn generate_key() -> EncryptionKey;
    pub fn encrypt_data(data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, CryptoError>;
    pub fn decrypt_data(encrypted: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, CryptoError>;
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> EncryptionKey;
}
```

#### 4. Conflict Resolution
```rust
// src-tauri/src/conflict.rs
pub enum ConflictResolutionStrategy {
    LastWriteWins,
    ManualResolution,
    Merge,
}

pub struct ConflictResolver;

impl ConflictResolver {
    pub fn resolve_bookmark_conflict(
        local: &Bookmark,
        remote: &Bookmark,
        strategy: ConflictResolutionStrategy,
    ) -> Result<Bookmark, ConflictError>;
}
```

### Data Models

#### Sync Change
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncChange {
    pub id: String,
    pub change_type: ChangeType, // Create, Update, Delete
    pub entity_type: EntityType, // Bookmark, History
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Bookmark,
    History,
}
```

#### Sync State
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub last_sync_time: Option<DateTime<Utc>>,
    pub pending_changes: Vec<SyncChange>,
    pub conflicts: Vec<Conflict>,
    pub sync_enabled: bool,
    pub device_id: String,
}
```

### Backend API Design

#### Authentication Endpoints
```
POST /api/auth/register
POST /api/auth/login
POST /api/auth/refresh
POST /api/auth/logout
```

#### Sync Endpoints
```
POST /api/sync/bookmarks/push
POST /api/sync/bookmarks/pull
POST /api/sync/history/push
POST /api/sync/history/pull
GET /api/sync/status
POST /api/sync/resolve-conflict
```

### Frontend Integration

#### Settings UI
```svelte
<!-- src/lib/components/CloudSyncSettings.svelte -->
<script lang="ts">
  let syncEnabled = $state(false);
  let syncEmail = $state('');
  let syncStatus = $state<'disconnected' | 'syncing' | 'connected'>('disconnected');
  let lastSyncTime = $state<Date | null>(null);
  let conflicts = $state<Conflict[]>([]);

  async function toggleSync() {
    if (syncEnabled) {
      await invoke('disable_sync');
    } else {
      await invoke('enable_sync', { email: syncEmail, password: /*...*/ });
    }
  }

  async function manualSync() {
    syncStatus = 'syncing';
    await invoke('manual_sync');
    lastSyncTime = new Date();
    syncStatus = 'connected';
  }

  async function resolveConflict(conflict: Conflict, resolution: ConflictResolution) {
    await invoke('resolve_conflict', { conflictId: conflict.id, resolution });
  }
</script>
```

#### Sync Status Indicator
```svelte
<!-- Status indicator in main UI -->
<div class="sync-status" class:syncing={syncStatus === 'syncing'}>
  {#if syncStatus === 'syncing'}
    <span>Syncing...</span>
  {:else if syncStatus === 'connected'}
    <span>Synced {lastSyncTime}</span>
  {:else}
    <span>Sync disabled</span>
  {/if}
</div>
```

### Security Considerations

1. **End-to-End Encryption**
   - Client-side encryption before upload
   - Server never sees plaintext data
   - User controls encryption key

2. **Authentication**
   - JWT with short-lived access tokens
   - Refresh token rotation
   - Secure password storage (Argon2)

3. **Transport Security**
   - HTTPS/TLS for all API calls
   - Certificate pinning

4. **Data Privacy**
   - Minimal data collection
   - Clear data retention policy
   - User data export/delete functionality

### Implementation Phases

#### Phase 1: Authentication (Week 1-2)
- User registration/login UI
- JWT token handling
- Secure password storage
- Token refresh mechanism

#### Phase 2: Bookmark Sync (Week 3-4)
- Bookmark sync service
- Encryption implementation
- Conflict resolution for bookmarks
- Sync status UI

#### Phase 3: History Sync (Week 5-6)
- History sync service
- Incremental sync (only new history)
- History conflict resolution
- History sync settings

#### Phase 4: Advanced Features (Week 7-8)
- Real-time sync (WebSockets)
- Offline queue
- Sync scheduling
- Performance optimization

### Database Schema

#### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### Devices Table
```sql
CREATE TABLE devices (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    device_id VARCHAR(255) UNIQUE NOT NULL,
    device_name VARCHAR(255),
    last_sync TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
```

#### Bookmarks Table
```sql
CREATE TABLE bookmarks (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    bookmark_id VARCHAR(255) NOT NULL,
    encrypted_data TEXT NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, bookmark_id)
);
```

#### History Table
```sql
CREATE TABLE history (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    visit_id VARCHAR(255) NOT NULL,
    encrypted_data TEXT NOT NULL,
    visited_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### Testing Strategy

#### Unit Tests
- Encryption/decryption
- Conflict resolution logic
- Sync change generation
- Authentication logic

#### Integration Tests
- End-to-end sync flow
- Authentication flow
- Conflict resolution scenarios
- Offline/online transitions

#### Manual Testing
- Multi-device sync testing
- Network failure scenarios
- Conflict resolution UI
- Performance testing with large datasets

## Migration Path

1. **Phase 1**: Implement backend infrastructure
2. **Phase 2**: Implement sync client in Tauri
3. **Phase 3**: Add sync settings UI
4. **Phase 4**: Beta testing with small user group
5. **Phase 5**: Full rollout

## Alternatives Considered

### Sync via File System (Dropbox, Google Drive)
- Simpler implementation
- No backend needed
- Limited real-time sync
- Privacy concerns with third-party storage

### WebDAV
- Standard protocol
- Self-hostable
- Limited real-time capabilities
- Complex conflict resolution

## Conclusion

Recommended approach: Self-hosted backend with end-to-end encryption provides the best balance of privacy, control, and functionality for Exodus Browser users.

## Next Steps

1. Set up backend project structure
2. Implement authentication service
3. Implement sync service core functionality
4. Add encryption layer
5. Implement conflict resolution
6. Create frontend sync settings UI
7. Add sync status indicators
8. Test end-to-end sync flow
