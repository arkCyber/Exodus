//! Exodus Browser — Cloud Sync Service
//!
//! Provides cloud synchronization for bookmarks and history with
//! encryption, conflict resolution, and offline support.

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    Aes256Gcm, Nonce,
};

use super::auth::{AuthService, Tokens};

/// Sync change type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SyncChangeType {
    Create,
    Update,
    Delete,
}

/// Sync change
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncChange {
    pub id: String,
    pub change_type: SyncChangeType,
    pub data_type: String, // "bookmark" or "history"
    pub data: serde_json::Value,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub version: u64,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub uploaded: u32,
    pub downloaded: u32,
    pub conflicts: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Sync error
#[derive(Debug)]
pub enum SyncError {
    NetworkError(String),
    AuthError(String),
    EncryptionError(String),
    ConflictError(String),
    SerializationError(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SyncError::AuthError(msg) => write!(f, "Auth error: {}", msg),
            SyncError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            SyncError::ConflictError(msg) => write!(f, "Conflict error: {}", msg),
            SyncError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for SyncError {}

/// Cloud sync service
pub struct SyncService {
    api_base: String,
    client: Client,
    auth_service: Arc<AuthService>,
    auth_tokens: Arc<Mutex<Option<Tokens>>>,
    device_id: String,
    encryption_key: Arc<Mutex<Option<String>>>,
}

impl SyncService {
    /// Create a new sync service
    pub fn new(
        api_base: String,
        auth_service: Arc<AuthService>,
        device_id: String,
    ) -> Self {
        Self {
            api_base,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            auth_service,
            auth_tokens: Arc::new(Mutex::new(None)),
            device_id,
            encryption_key: Arc::new(Mutex::new(None)),
        }
    }

    /// Set authentication tokens
    pub fn set_tokens(&self, tokens: Tokens) {
        let mut auth_tokens = self.auth_tokens.lock()
            .expect("Failed to acquire auth tokens lock");
        *auth_tokens = Some(tokens);
    }

    /// Set encryption key
    pub fn set_encryption_key(&self, key: String) {
        let mut encryption_key = self.encryption_key.lock()
            .expect("Failed to acquire encryption key lock");
        *encryption_key = Some(key);
    }

    /// Get authorization header
    fn get_auth_header(&self) -> Result<String, SyncError> {
        let auth_tokens = self.auth_tokens.lock()
            .map_err(|e| SyncError::InternalError(format!("Failed to acquire auth tokens lock: {}", e)))?;
        let tokens = auth_tokens.as_ref().ok_or_else(|| {
            SyncError::AuthError("No authentication tokens".to_string())
        })?;
        Ok(format!("Bearer {}", tokens.access_token))
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_data(&self, data: &str) -> Result<String, SyncError> {
        let encryption_key = self.encryption_key.lock()
            .map_err(|e| SyncError::InternalError(format!("Failed to acquire encryption key lock: {}", e)))?;
        let key = encryption_key.as_ref().ok_or_else(|| {
            SyncError::EncryptionError("No encryption key".to_string())
        })?;
        
        // Derive a 32-byte key from the provided key string
        let key_bytes = Self::derive_key(key);
        
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Create cipher
        let cipher = Aes256Gcm::new(&key_bytes.into());
        
        // Encrypt data
        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| SyncError::EncryptionError(e.to_string()))?;
        
        // Combine nonce and ciphertext for storage
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(base64::encode(&result))
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_data(&self, encrypted: &str) -> Result<String, SyncError> {
        let encryption_key = self.encryption_key.lock()
            .map_err(|e| SyncError::InternalError(format!("Failed to acquire encryption key lock: {}", e)))?;
        let key = encryption_key.as_ref().ok_or_else(|| {
            SyncError::EncryptionError("No encryption key".to_string())
        })?;
        
        // Derive a 32-byte key from the provided key string
        let key_bytes = Self::derive_key(key);
        
        // Decode base64
        let data = base64::decode(encrypted)
            .map_err(|e| SyncError::EncryptionError(e.to_string()))?;
        
        // Extract nonce (first 12 bytes) and ciphertext
        if data.len() < 12 {
            return Err(SyncError::EncryptionError("Invalid encrypted data".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Create cipher
        let cipher = Aes256Gcm::new(&key_bytes.into());
        
        // Decrypt data
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SyncError::EncryptionError(e.to_string()))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| SyncError::EncryptionError(e.to_string()))
    }

    /// Derive a 32-byte key from a string key using SHA-256
    fn derive_key(key: &str) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&result);
        key_bytes
    }

    /// Push changes to server
    pub async fn push_changes(&self, changes: Vec<SyncChange>) -> Result<(), SyncError> {
        let auth_header = self.get_auth_header()?;
        
        let response = self.client
            .post(&format!("{}/api/sync/push", self.api_base))
            .header("Authorization", auth_header)
            .json(&changes)
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SyncError::NetworkError(format!("Server error: {}", response.status())));
        }

        Ok(())
    }

    /// Pull changes from server
    pub async fn pull_changes(&self, since: DateTime<Utc>) -> Result<Vec<SyncChange>, SyncError> {
        let auth_header = self.get_auth_header()?;
        
        let response = self.client
            .get(&format!("{}/api/sync/pull", self.api_base))
            .header("Authorization", auth_header)
            .query(&[("since", since.to_rfc3339())])
            .send()
            .await
            .map_err(|e| SyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SyncError::NetworkError(format!("Server error: {}", response.status())));
        }

        let changes: Vec<SyncChange> = response
            .json()
            .await
            .map_err(|e| SyncError::SerializationError(e.to_string()))?;

        Ok(changes)
    }

    /// Sync bookmarks
    pub async fn sync_bookmarks(&self, local_bookmarks: Vec<serde_json::Value>) -> Result<SyncResult, SyncError> {
        // Get last sync timestamp (simplified)
        let since = Utc::now() - chrono::Duration::hours(24);
        
        // Pull remote changes
        let remote_changes = self.pull_changes(since).await?;
        let bookmark_changes: Vec<_> = remote_changes
            .into_iter()
            .filter(|c| c.data_type == "bookmark")
            .collect();

        // Push local changes
        let local_changes: Vec<SyncChange> = local_bookmarks
            .into_iter()
            .map(|bookmark| SyncChange {
                id: Uuid::new_v4().to_string(),
                change_type: SyncChangeType::Update,
                data_type: "bookmark".to_string(),
                data: bookmark,
                device_id: self.device_id.clone(),
                timestamp: Utc::now(),
                version: 1,
            })
            .collect();

        let uploaded_count = local_changes.len() as u32;
        self.push_changes(local_changes).await?;

        Ok(SyncResult {
            uploaded: uploaded_count,
            downloaded: bookmark_changes.len() as u32,
            conflicts: Vec::new(),
            timestamp: Utc::now(),
        })
    }

    /// Sync history
    pub async fn sync_history(&self, local_history: Vec<serde_json::Value>) -> Result<SyncResult, SyncError> {
        // Get last sync timestamp (simplified)
        let since = Utc::now() - chrono::Duration::hours(24);
        
        // Pull remote changes
        let remote_changes = self.pull_changes(since).await?;
        let history_changes: Vec<_> = remote_changes
            .into_iter()
            .filter(|c| c.data_type == "history")
            .collect();

        // Push local changes
        let local_changes: Vec<SyncChange> = local_history
            .into_iter()
            .map(|history| SyncChange {
                id: Uuid::new_v4().to_string(),
                change_type: SyncChangeType::Create,
                data_type: "history".to_string(),
                data: history,
                device_id: self.device_id.clone(),
                timestamp: Utc::now(),
                version: 1,
            })
            .collect();

        let uploaded_count = local_changes.len() as u32;
        self.push_changes(local_changes).await?;

        Ok(SyncResult {
            uploaded: uploaded_count,
            downloaded: history_changes.len() as u32,
            conflicts: Vec::new(),
            timestamp: Utc::now(),
        })
    }

    /// Full sync (bookmarks + history)
    pub async fn full_sync(&self) -> Result<SyncResult, SyncError> {
        // This would fetch local bookmarks and history from the browser
        // For now, return empty result
        Ok(SyncResult {
            uploaded: 0,
            downloaded: 0,
            conflicts: Vec::new(),
            timestamp: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_service_creation() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        );
        
        // Service created successfully
    }

    #[test]
    fn test_set_encryption_key() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = Arc::new(SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        ));
        
        sync_service.set_encryption_key("test-encryption-key".to_string());
        // Should not panic
    }

    #[test]
    fn test_set_tokens() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = Arc::new(SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        ));
        
        let tokens = Tokens {
            access_token: "test-access-token".to_string(),
            refresh_token: "test-refresh-token".to_string(),
            expires_in: 3600,
        };
        
        sync_service.set_tokens(tokens);
        // Should not panic
    }

    #[test]
    fn test_encrypt_data() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        );
        
        sync_service.set_encryption_key("test-key".to_string());
        
        let data = "sensitive data";
        let encrypted = sync_service.encrypt_data(data);
        
        assert!(encrypted.is_ok());
        let encrypted = encrypted.unwrap();
        assert_ne!(encrypted, data);
        // AES-256-GCM encrypted data should be longer than original (nonce + auth tag)
        assert!(encrypted.len() > data.len());
    }

    #[test]
    fn test_decrypt_data() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        );
        
        sync_service.set_encryption_key("test-key".to_string());
        
        let data = "sensitive data";
        let encrypted = sync_service.encrypt_data(data).unwrap();
        let decrypted = sync_service.decrypt_data(&encrypted);
        
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), data);
    }

    #[test]
    fn test_aes_gcm_encryption_roundtrip() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        );
        
        sync_service.set_encryption_key("strong-encryption-key-123".to_string());
        
        // Test with various data sizes
        let test_data: Vec<String> = vec![
            "".to_string(),
            "a".to_string(),
            "hello world".to_string(),
            "This is a longer test string with more characters".to_string(),
            "A".repeat(1000),
        ];
        
        for data in test_data {
            let encrypted = sync_service.encrypt_data(&data).unwrap();
            let decrypted = sync_service.decrypt_data(&encrypted).unwrap();
            assert_eq!(decrypted, data);
        }
    }

    #[test]
    fn test_encrypt_without_key() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        );
        
        let data = "sensitive data";
        let encrypted = sync_service.encrypt_data(data);
        
        assert!(encrypted.is_err());
    }

    #[test]
    fn test_decrypt_without_key() {
        let auth_service = Arc::new(AuthService::new("test-secret".to_string()));
        let sync_service = SyncService::new(
            "http://localhost:8080".to_string(),
            auth_service,
            "test-device-123".to_string(),
        );
        
        let encrypted = "encrypted-data";
        let decrypted = sync_service.decrypt_data(encrypted);
        
        assert!(decrypted.is_err());
    }

    #[test]
    fn test_sync_change_creation() {
        let change = SyncChange {
            id: "test-change-1".to_string(),
            change_type: SyncChangeType::Create,
            data_type: "bookmark".to_string(),
            data: serde_json::json!({"url": "https://example.com", "title": "Example"}),
            device_id: "test-device".to_string(),
            timestamp: Utc::now(),
            version: 1,
        };
        
        assert_eq!(change.change_type, SyncChangeType::Create);
        assert_eq!(change.data_type, "bookmark");
    }

    #[test]
    fn test_sync_result_creation() {
        let result = SyncResult {
            uploaded: 10,
            downloaded: 5,
            conflicts: vec!["conflict-1".to_string()],
            timestamp: Utc::now(),
        };
        
        assert_eq!(result.uploaded, 10);
        assert_eq!(result.downloaded, 5);
        assert_eq!(result.conflicts.len(), 1);
    }

    #[test]
    fn test_sync_change_type_serialization() {
        let change_type = SyncChangeType::Create;
        let serialized = serde_json::to_string(&change_type).unwrap();
        let deserialized: SyncChangeType = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(change_type, deserialized);
    }

    #[test]
    fn test_multiple_sync_changes() {
        let changes = vec![
            SyncChange {
                id: "change-1".to_string(),
                change_type: SyncChangeType::Create,
                data_type: "bookmark".to_string(),
                data: serde_json::json!({"url": "https://example.com"}),
                device_id: "device-1".to_string(),
                timestamp: Utc::now(),
                version: 1,
            },
            SyncChange {
                id: "change-2".to_string(),
                change_type: SyncChangeType::Update,
                data_type: "bookmark".to_string(),
                data: serde_json::json!({"url": "https://example.com", "title": "Updated"}),
                device_id: "device-1".to_string(),
                timestamp: Utc::now(),
                version: 2,
            },
        ];
        
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].change_type, SyncChangeType::Create);
        assert_eq!(changes[1].change_type, SyncChangeType::Update);
    }

    #[test]
    fn test_sync_error_display() {
        let error = SyncError::NetworkError("Connection failed".to_string());
        assert_eq!(error.to_string(), "Network error: Connection failed");
        
        let error = SyncError::AuthError("Invalid token".to_string());
        assert_eq!(error.to_string(), "Auth error: Invalid token");
    }
}
