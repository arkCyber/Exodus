//! Exodus Browser — Chrome Runtime API Tauri commands
//!
//! Provides Tauri commands for chrome.runtime API

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use super::manager::ExtensionState;
use super::runtime::{get_platform_info, SuspendTracker, UpdateTracker};

/// Get platform info
#[tauri::command]
pub fn extension_runtime_get_platform_info() -> Result<serde_json::Value, String> {
    let info = get_platform_info();
    serde_json::to_value(info).map_err(|e| e.to_string())
}

/// Get background page
#[tauri::command]
pub fn extension_runtime_get_background_page(
    _extension_id: String,
    _state: State<'_, ExtensionState>,
) -> Result<Option<String>, String> {
    // Returns null for service workers (Manifest V3)
    Ok(None)
}

/// Check suspend status
#[tauri::command]
pub fn extension_runtime_check_suspend(
    extension_id: String,
    suspend_tracker: State<'_, Arc<SuspendTracker>>,
) -> Result<bool, String> {
    let tracker = suspend_tracker.inner();
    Ok(tracker.is_suspended(&extension_id))
}

/// Request update check
#[tauri::command]
pub fn extension_runtime_request_update_check(
    extension_id: String,
    current_version: String,
    update_tracker: State<'_, Arc<UpdateTracker>>,
) -> Result<Option<String>, String> {
    update_tracker
        .inner()
        .request_update_check(&extension_id, &current_version)
}

/// Record update available (for testing/manual use)
#[tauri::command]
pub fn extension_runtime_record_update_available(
    extension_id: String,
    version: String,
    update_tracker: State<'_, Arc<UpdateTracker>>,
) -> Result<(), String> {
    update_tracker
        .inner()
        .record_update_available(&extension_id, version);
    Ok(())
}

/// Get extension manifest
#[tauri::command]
pub fn extension_runtime_get_manifest(
    _extension_id: String,
    _state: State<'_, ExtensionState>,
) -> Result<serde_json::Value, String> {
    // Placeholder implementation
    Ok(serde_json::json!({
        "name": "Example Extension",
        "version": "1.0.0",
        "manifest_version": 3
    }))
}

/// Get extension URL
#[tauri::command]
pub fn extension_runtime_get_url(
    extension_id: String,
    path: String,
    _state: State<'_, ExtensionState>,
) -> Result<String, String> {
    Ok(format!("chrome-extension://{}/{}", extension_id, path.trim_start_matches('/')))
}

/// Reload extension
#[tauri::command]
pub fn extension_runtime_reload(
    _extension_id: String,
    _state: State<'_, ExtensionState>,
) -> Result<(), String> {
    // Placeholder implementation
    Ok(())
}

/// Get browser info
#[tauri::command]
pub fn extension_runtime_get_browser_info() -> Result<BrowserInfo, String> {
    Ok(BrowserInfo {
        name: "Exodus".to_string(),
        vendor: "Exodus Project".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        product: "Exodus Browser".to_string(),
    })
}

/// Open options page
#[tauri::command]
pub fn extension_runtime_open_options_page(
    _extension_id: String,
    _state: State<'_, ExtensionState>,
) -> Result<(), String> {
    // Placeholder implementation
    Ok(())
}

/// Set uninstall URL
#[tauri::command]
pub fn extension_runtime_set_uninstall_url(
    _extension_id: String,
    _url: String,
    _state: State<'_, ExtensionState>,
) -> Result<(), String> {
    // Placeholder implementation
    Ok(())
}

/// Send message to extension
#[tauri::command]
pub fn extension_runtime_send_message(
    _extension_id: String,
    _message: serde_json::Value,
    _state: State<'_, ExtensionState>,
) -> Result<serde_json::Value, String> {
    // Placeholder implementation
    Ok(serde_json::json!(null))
}

/// Browser info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserInfo {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub product: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_info_serialization() {
        let info = BrowserInfo {
            name: "Exodus".to_string(),
            vendor: "Exodus Project".to_string(),
            version: "1.0.0".to_string(),
            product: "Exodus Browser".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Exodus"));
        assert!(json.contains("1.0.0"));
    }
}
