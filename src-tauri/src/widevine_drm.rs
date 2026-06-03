//! Widevine DRM Integration for Exodus Browser
//! 
//! This module provides Widevine DRM support for playing encrypted media content
//! from streaming services like Netflix, Disney+, etc.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;
use reqwest::Client;

/// Widevine DRM key system
pub const WIDEVINE_KEY_SYSTEM: &str = "com.widevine.alpha";

/// DRM session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrmSessionState {
    Idle,
    Created,
    KeyRequest,
    KeyReady,
    KeyError,
    Closed,
}

/// DRM key request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmKeyRequest {
    pub session_id: String,
    pub init_data_type: String,
    pub init_data: Vec<u8>,
    pub key_system: String,
}

/// DRM key response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmKeyResponse {
    pub session_id: String,
    pub key_data: Vec<u8>,
}

/// DRM session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmSession {
    pub session_id: String,
    pub key_system: String,
    pub state: DrmSessionState,
    pub created_at: u64,
    pub media_url: Option<String>,
}

/// Widevine DRM manager
pub struct WidevineDrmManager {
    sessions: Arc<Mutex<Vec<DrmSession>>>,
    cdm_path: PathBuf,
    enabled: Arc<Mutex<bool>>,
    max_sessions: usize,
    license_server_url: Arc<Mutex<Option<String>>>,
    client: Client,
}

impl WidevineDrmManager {
    /// Create a new Widevine DRM manager
    pub fn new(cdm_path: PathBuf) -> Result<Self, String> {
        // Create CDM directory if it doesn't exist
        fs::create_dir_all(&cdm_path)
            .map_err(|e| format!("Failed to create CDM directory: {}", e))?;

        let manager = Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
            cdm_path,
            enabled: Arc::new(Mutex::new(true)),
            max_sessions: 100, // Reasonable limit to prevent resource exhaustion
            license_server_url: Arc::new(Mutex::new(None)),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        };

        // Check if Widevine CDM is available
        manager.check_cdm_availability()?;

        Ok(manager)
    }

    /// Get platform-specific CDM paths
    fn get_cdm_paths(&self) -> Vec<PathBuf> {
        #[cfg(target_os = "macos")]
        let cdm_paths = vec![
            self.cdm_path.join("widevinecdm.dylib"),
            "/Library/Application Support/Google/Chrome/WidevineCdm/_platform_specific/mac_x64/widevinecdm.dylib".into(),
        ];

        #[cfg(target_os = "linux")]
        let cdm_paths = vec![
            self.cdm_path.join("libwidevinecdm.so"),
            "/opt/google/chrome/WidevineCdm/_platform_specific/linux_x64/libwidevinecdm.so".into(),
            "/usr/lib/chromium-browser/libwidevinecdm.so".into(),
        ];

        #[cfg(target_os = "windows")]
        let cdm_paths = vec![
            self.cdm_path.join("widevinecdm.dll"),
            r"C:\Program Files\Google\Chrome\Application\WidevineCdm\_platform_specific\win_x64\widevinecdm.dll".into(),
        ];

        cdm_paths
    }

    /// Check if Widevine CDM is available
    fn check_cdm_availability(&self) -> Result<(), String> {
        for path in self.get_cdm_paths() {
            if path.exists() {
                return Ok(());
            }
        }

        // CDM not found, but we'll still allow the manager to be created
        // The actual DRM operations will fail gracefully
        Ok(())
    }

    /// Load CDM library (placeholder for future implementation)
    /// 
    /// NOTE: This is a placeholder for future CDM library integration.
    /// Actual implementation will require:
    /// 1. libloading crate for dynamic library loading
    /// 2. FFI bindings to CDM functions
    /// 3. Proper error handling for CDM-specific errors
    /// 4. Platform-specific CDM path handling
    pub fn load_cdm_library(&self) -> Result<(), String> {
        // Placeholder - CDM library loading will be implemented in future
        // For now, we just check if the library file exists
        self.find_cdm_path()?;
        Ok(())
    }

    /// Find available CDM path
    fn find_cdm_path(&self) -> Result<PathBuf, String> {
        for path in self.get_cdm_paths() {
            if path.exists() {
                return Ok(path);
            }
        }
        Err("No CDM library found".to_string())
    }

    /// Create a DRM session
    pub fn create_session(
        &self,
        key_system: String,
        _init_data_type: String,
        _init_data: Vec<u8>,
        media_url: Option<String>,
    ) -> Result<DrmSession, String> {
        if !self.is_enabled() {
            return Err("Widevine DRM is disabled".to_string());
        }

        if key_system != WIDEVINE_KEY_SYSTEM {
            return Err(format!("Unsupported key system: {}", key_system));
        }

        // Check session limit to prevent resource exhaustion
        {
            let sessions = self.sessions.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            if sessions.len() >= self.max_sessions {
                return Err(format!("Maximum session limit ({}) reached", self.max_sessions));
            }
        }

        let session_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let session = DrmSession {
            session_id: session_id.clone(),
            key_system,
            state: DrmSessionState::Created,
            created_at: now,
            media_url,
        };

        let mut sessions = self.sessions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        sessions.push(session.clone());

        Ok(session)
    }

    /// Generate key request
    pub fn generate_key_request(&self, session_id: String) -> Result<DrmKeyRequest, String> {
        let sessions = self.sessions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        let session = sessions.iter()
            .find(|s| s.session_id == session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        // Placeholder for actual CDM integration
        // In production, this would call the actual CDM functions
        // For now, we return a placeholder to maintain compatibility
        let key_request = DrmKeyRequest {
            session_id: session_id.clone(),
            init_data_type: "cenc".to_string(),
            init_data: vec
![],
            key_system: session.key_system.clone(),
        };

        Ok(key_request)
    }

    /// Process key response
    pub fn process_key_response(&self, response: DrmKeyResponse, app: AppHandle) -> Result<(), String> {
        let mut sessions = self.sessions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == response.session_id) {
            // Placeholder for actual CDM integration
            // In production, this would call the actual CDM functions
            session.state = DrmSessionState::KeyReady;

            // Emit event to frontend
            let _ = app.emit("exodus-drm-key-ready", &session);

            Ok(())
        } else {
            Err(format!("Session not found: {}", response.session_id))
        }
    }

    /// Close a DRM session
    pub fn close_session(&self, session_id: String, app: AppHandle) -> Result<(), String> {
        let mut sessions = self.sessions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(session) = sessions.iter_mut().find(|s| s.session_id == session_id) {
            session.state = DrmSessionState::Closed;

            // Emit event to frontend
            let _ = app.emit("exodus-drm-session-closed", &session_id);

            Ok(())
        } else {
            Err(format!("Session not found: {}", session_id))
        }
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: String) -> Option<DrmSession> {
        let sessions = self.sessions.lock().ok()?;
        sessions.iter().find(|s| s.session_id == session_id).cloned()
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<DrmSession> {
        let sessions = match self.sessions.lock() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to acquire sessions lock: {}", e);
                return Vec::new();
            }
        };

        sessions.iter()
            .filter(|s| matches!(s.state, DrmSessionState::Created | DrmSessionState::KeyReady))
            .cloned()
            .collect()
    }

    /// Enable Widevine DRM
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = true;
            let _ = app.emit("exodus-drm-enabled", true);
        }
    }

    /// Disable Widevine DRM
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = false;
            let _ = app.emit("exodus-drm-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.lock()
            .map(|enabled| *enabled)
            .unwrap_or(false)
    }

    /// Get CDM path
    pub fn get_cdm_path(&self) -> PathBuf {
        self.cdm_path.clone()
    }

    /// Check if CDM is available
    pub fn is_cdm_available(&self) -> bool {
        for path in self.get_cdm_paths() {
            if path.exists() {
                return true;
            }
        }
        false
    }

    /// Set license server URL
    pub fn set_license_server_url(&self, url: String) {
        let mut license_server_url = self.license_server_url.lock().unwrap();
        *license_server_url = Some(url);
    }

    /// Get license server URL
    pub fn get_license_server_url(&self) -> Option<String> {
        let license_server_url = self.license_server_url.lock().unwrap();
        license_server_url.clone()
    }

    /// Acquire license from license server
    pub async fn acquire_license(
        &self,
        session_id: String,
        challenge: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let license_server_url = self.get_license_server_url()
            .ok_or_else(|| "License server URL not set".to_string())?;

        let response = self.client
            .post(&license_server_url)
            .header("Content-Type", "application/octet-stream")
            .body(challenge)
            .send()
            .await
            .map_err(|e| format!("License request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("License server returned error: {}", response.status()));
        }

        let license_data = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read license response: {}", e))?
            .to_vec();

        Ok(license_data)
    }

    /// Clean up old closed sessions
    pub fn cleanup_old_sessions(&self, max_age_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut sessions = match self.sessions.lock() {
            Ok(s) => s,
            Err(_) => return,
        };

        sessions.retain(|session| {
            // Keep active sessions
            if matches!(session.state, DrmSessionState::Created | DrmSessionState::KeyReady) {
                return true;
            }

            // Remove closed sessions older than max_age_secs
            if now.saturating_sub(session.created_at) > max_age_secs {
                return false;
            }

            true
        });
    }
}

// Tauri Commands

/// Create DRM session
#[tauri::command]
pub fn drm_create_session(
    key_system: String,
    init_data_type: String,
    init_data: Vec<u8>,
    media_url: Option<String>,
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<DrmSession, String> {
    manager.create_session(key_system, init_data_type, init_data, media_url)
}

/// Generate key request
#[tauri::command]
pub fn drm_generate_key_request(
    session_id: String,
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<DrmKeyRequest, String> {
    manager.generate_key_request(session_id)
}

/// Process key response
#[tauri::command]
pub fn drm_process_key_response(
    response: DrmKeyResponse,
    app: AppHandle,
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<(), String> {
    manager.process_key_response(response, app)
}

/// Close DRM session
#[tauri::command]
pub fn drm_close_session(
    session_id: String,
    app: AppHandle,
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<(), String> {
    manager.close_session(session_id, app)
}

/// Get DRM session
#[tauri::command]
pub fn drm_get_session(
    session_id: String,
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<Option<DrmSession>, String> {
    Ok(manager.get_session(session_id))
}

/// Get active DRM sessions
#[tauri::command]
pub fn drm_get_active_sessions(
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<Vec<DrmSession>, String> {
    Ok(manager.get_active_sessions())
}

/// Enable Widevine DRM
#[tauri::command]
pub fn drm_enable(
    app: AppHandle,
    manager: State<'_, Arc<WidevineDrmManager>>,
) {
    manager.enable(app);
}

/// Disable Widevine DRM
#[tauri::command]
pub fn drm_disable(
    app: AppHandle,
    manager: State<'_, Arc<WidevineDrmManager>>,
) {
    manager.disable(app);
}

/// Check if Widevine DRM is enabled
#[tauri::command]
pub fn drm_is_enabled(
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<bool, String> {
    Ok(manager.is_enabled())
}

/// Check if CDM is available
#[tauri::command]
pub fn drm_is_cdm_available(
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<bool, String> {
    Ok(manager.is_cdm_available())
}

/// Get CDM path
#[tauri::command]
pub fn drm_get_cdm_path(
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<String, String> {
    Ok(manager.get_cdm_path().to_string_lossy().to_string())
}

/// Set license server URL
#[tauri::command]
pub fn drm_set_license_server_url(
    url: String,
    manager: State<'_, Arc<WidevineDrmManager>>,
) {
    manager.set_license_server_url(url);
}

/// Get license server URL
#[tauri::command]
pub fn drm_get_license_server_url(
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<Option<String>, String> {
    Ok(manager.get_license_server_url())
}

/// Acquire license from license server
#[tauri::command]
pub async fn drm_acquire_license(
    session_id: String,
    challenge: Vec<u8>,
    manager: State<'_, Arc<WidevineDrmManager>>,
) -> Result<Vec<u8>, String> {
    manager.acquire_license(session_id, challenge).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_widevine_drm_manager_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = WidevineDrmManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_create_session() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = WidevineDrmManager::new(temp_dir.path().to_path_buf()).expect("manager creation");

        let session = manager.create_session(
            WIDEVINE_KEY_SYSTEM.to_string(),
            "cenc".to_string(),
            vec
![1, 2, 3],
            Some("https://example.com/video.mp4".to_string()),
        );

        assert!(session.is_ok());
        let session = session.unwrap();
        assert_eq!(session.key_system, WIDEVINE_KEY_SYSTEM);
    }

    #[test]
    fn test_create_session_unsupported_key_system() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = WidevineDrmManager::new(temp_dir.path().to_path_buf()).expect("manager creation");

        let session = manager.create_session(
            "unsupported.key.system".to_string(),
            "cenc".to_string(),
            vec
![1, 2, 3],
            None,
        );

        assert!(session.is_err());
    }

    #[test]
    fn test_get_session() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = WidevineDrmManager::new(temp_dir.path().to_path_buf()).expect("manager creation");

        let session = manager.create_session(
            WIDEVINE_KEY_SYSTEM.to_string(),
            "cenc".to_string(),
            vec
![1, 2, 3],
            None,
        ).expect("create session");

        let retrieved = manager.get_session(session.session_id.clone());
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, session.session_id);
    }

    #[test]
    fn test_enable_disable() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = WidevineDrmManager::new(temp_dir.path().to_path_buf()).expect("manager creation");

        assert!(manager.is_enabled());

        // Use public methods to change state
        {
            let mut enabled = manager.enabled.lock().unwrap();
            *enabled = false;
        }
        assert!(!manager.is_enabled());

        {
            let mut enabled = manager.enabled.lock().unwrap();
            *enabled = true;
        }
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_constants() {
        assert_eq!(WIDEVINE_KEY_SYSTEM, "com.widevine.alpha");
    }
}
