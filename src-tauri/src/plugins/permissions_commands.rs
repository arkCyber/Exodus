//! Chrome Extension API - chrome.permissions
//!
//! Provides Tauri commands for the chrome.permissions API, allowing extensions
//! to request and manage permissions.

use crate::plugins::manager::ExtensionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Permissions container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub origins: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
}

/// Request permissions
#[tauri::command]
pub async fn chrome_permissions_request(
    permissions: Permissions,
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<bool, String> {
    // Request permissions from user
    // Placeholder implementation
    Ok(true)
}

/// Check if permissions are granted
#[tauri::command]
pub async fn chrome_permissions_contains(
    permissions: Permissions,
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<bool, String> {
    // Check if permissions are granted
    // Placeholder implementation
    Ok(true)
}

/// Get all granted permissions
#[tauri::command]
pub async fn chrome_permissions_get_all(
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<Permissions, String> {
    // Get all granted permissions
    // Placeholder implementation
    Ok(Permissions {
        origins: Some(vec!["<all_urls>".to_string()]),
        permissions: Some(vec!["storage".to_string(), "tabs".to_string()]),
    })
}

/// Remove permissions
#[tauri::command]
pub async fn chrome_permissions_remove(
    permissions: Permissions,
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<bool, String> {
    // Remove permissions
    // Placeholder implementation
    Ok(true)
}

/// On added event listener
#[tauri::command]
pub async fn chrome_permissions_on_added(
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<(), String> {
    // Listen for added permissions
    // Placeholder implementation
    Ok(())
}

/// On removed event listener
#[tauri::command]
pub async fn chrome_permissions_on_removed(
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<(), String> {
    // Listen for removed permissions
    // Placeholder implementation
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_serialization() {
        let permissions = Permissions {
            origins: Some(vec!["<all_urls>".to_string()]),
            permissions: Some(vec!["storage".to_string(), "tabs".to_string()]),
        };
        let json = serde_json::to_string(&permissions).unwrap();
        assert!(json.contains("<all_urls>"));
        assert!(json.contains("storage"));
    }

    #[test]
    fn test_permissions_deserialization() {
        let json = r#"{"origins":["<all_urls>"],"permissions":["storage","tabs"]}"#;
        let permissions: Permissions = serde_json::from_str(json).unwrap();
        assert_eq!(permissions.origins, Some(vec!["<all_urls>".to_string()]));
        assert_eq!(permissions.permissions, Some(vec!["storage".to_string(), "tabs".to_string()]));
    }
}
