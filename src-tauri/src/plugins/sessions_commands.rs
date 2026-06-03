//! Chrome Extension API - chrome.sessions
//!
//! Provides Tauri commands for the chrome.sessions API, allowing extensions
//! to interact with the browser's session management and tab restoration.

use crate::plugins::manager::ExtensionManager;
use crate::plugins::tabs::TabRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// Session registry for tracking recently closed tabs/windows
pub struct SessionRegistry {
    recently_closed: Arc<Mutex<Vec<Session>>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            recently_closed: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a recently closed tab to the registry
    pub fn add_closed_tab(&self, url: String, title: String) {
        let mut closed = self.recently_closed.lock().unwrap();
        let session = Session {
            last_modified: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            tab: Some(Tab {
                entries: vec![SessionEntry {
                    url: url.clone(),
                    title: title.clone(),
                    transition_type: "link".to_string(),
                    scroll_position: 0,
                }],
                index: 0,
                pinned: false,
            }),
            window: None,
        };
        closed.push(session);
        
        // Keep only last 25 closed sessions
        let len = closed.len();
        if len > 25 {
            closed.drain(0..len - 25);
        }
    }

    /// Get recently closed sessions
    pub fn get_recently_closed(&self, max_results: Option<u32>) -> Vec<Session> {
        let closed = self.recently_closed.lock().unwrap();
        let limit = max_results.unwrap_or(25) as usize;
        closed.iter().rev().take(limit).cloned().collect()
    }

    /// Clear all recently closed sessions
    pub fn clear(&self) {
        let mut closed = self.recently_closed.lock().unwrap();
        closed.clear();
    }

    /// Remove a session by timestamp
    pub fn remove_session(&self, timestamp: u64) {
        let mut closed = self.recently_closed.lock().unwrap();
        closed.retain(|s| s.last_modified != timestamp);
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub last_modified: u64,
    pub tab: Option<Tab>,
    pub window: Option<Window>,
}

/// Tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub entries: Vec<SessionEntry>,
    pub index: u32,
    pub pinned: bool,
}

/// Window information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub tabs: Vec<Tab>,
    pub selected_tab_index: u32,
}

/// Session entry (navigation history)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntry {
    pub url: String,
    pub title: String,
    pub transition_type: String,
    pub scroll_position: u32,
}

/// Filter for session retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub max_results: Option<u32>,
}

/// Get recently closed sessions
#[tauri::command]
pub async fn chrome_sessions_get_recently_closed(
    filter: Option<Filter>,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    session_registry: State<'_, SessionRegistry>,
) -> Result<Vec<Session>, String> {
    // Get recently closed sessions
    let max_results = filter.and_then(|f| f.max_results);
    Ok(session_registry.get_recently_closed(max_results))
}

/// Get all devices sessions
#[tauri::command]
pub async fn chrome_sessions_get_devices(
    filter: Option<Filter>,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    session_registry: State<'_, SessionRegistry>,
) -> Result<Device, String> {
    // Get sessions from all devices
    // For now, we only support the local device
    let max_results = filter.and_then(|f| f.max_results);
    let sessions = session_registry.get_recently_closed(max_results);
    
    Ok(Device {
        device_name: "Local Device".to_string(),
        sessions,
    })
}

/// Restore a session
#[tauri::command]
pub async fn chrome_sessions_restore(
    session_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    session_registry: State<'_, SessionRegistry>,
    tab_registry: State<'_, TabRegistry>,
) -> Result<String, String> {
    // Restore a session by ID
    // Since the backend cannot directly create tabs (that's a frontend operation),
    // we return the URL that should be restored and remove it from recently closed
    
    // Find the session by ID (session_id is the last_modified timestamp as string)
    let session_timestamp: u64 = session_id.parse()
        .map_err(|_| "Invalid session ID".to_string())?;
    
    // Get recently closed sessions
    let sessions = session_registry.get_recently_closed(None);
    
    // Find the matching session
    if let Some(session) = sessions.iter().find(|s| s.last_modified == session_timestamp) {
        // Extract the URL from the session
        if let Some(ref tab) = session.tab {
            if let Some(ref entry) = tab.entries.first() {
                let url = entry.url.clone();
                
                // Remove the session from recently closed
                session_registry.remove_session(session_timestamp);
                
                return Ok(url);
            }
        }
    }
    
    Err("Session not found".to_string())
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub device_name: String,
    pub sessions: Vec<Session>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_serialization() {
        let session = Session {
            last_modified: 1234567890,
            tab: None,
            window: None,
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("lastModified"));
    }

    #[test]
    fn test_session_entry_serialization() {
        let entry = SessionEntry {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            transition_type: "link".to_string(),
            scroll_position: 0,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_filter_deserialization() {
        let filter_json = r#"{"maxResults":10}"#;
        let filter: Filter = serde_json::from_str(filter_json).unwrap();
        assert_eq!(filter.max_results, Some(10));
    }

    #[test]
    fn test_session_registry_add_closed_tab() {
        let registry = SessionRegistry::new();
        registry.add_closed_tab("https://example.com".to_string(), "Example".to_string());
        let sessions = registry.get_recently_closed(None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].tab.as_ref().unwrap().entries[0].url, "https://example.com");
    }

    #[test]
    fn test_session_registry_clear() {
        let registry = SessionRegistry::new();
        registry.add_closed_tab("https://example.com".to_string(), "Example".to_string());
        registry.clear();
        let sessions = registry.get_recently_closed(None);
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_session_registry_remove_session() {
        let registry = SessionRegistry::new();
        registry.add_closed_tab("https://example.com".to_string(), "Example".to_string());
        let sessions = registry.get_recently_closed(None);
        let timestamp = sessions[0].last_modified;
        registry.remove_session(timestamp);
        let sessions = registry.get_recently_closed(None);
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_session_registry_max_results() {
        let registry = SessionRegistry::new();
        for i in 0..30 {
            registry.add_closed_tab(format!("https://example{}.com", i), format!("Example {}", i));
        }
        let sessions = registry.get_recently_closed(None);
        assert_eq!(sessions.len(), 25);
    }

    #[test]
    fn test_device_serialization() {
        let device = Device {
            device_name: "My Device".to_string(),
            sessions: vec![],
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("deviceName"));
        assert!(json.contains("My Device"));
    }

    #[test]
    fn test_tab_serialization() {
        let tab = Tab {
            entries: vec![SessionEntry {
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                transition_type: "link".to_string(),
                scroll_position: 100,
            }],
            index: 0,
            pinned: false,
        };
        let json = serde_json::to_string(&tab).unwrap();
        assert!(json.contains("entries"));
        assert!(json.contains("index"));
        assert!(json.contains("pinned"));
    }
}
