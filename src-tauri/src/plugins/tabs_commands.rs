//! Exodus Browser — Chrome Tabs API Tauri commands
//!
//! Provides data structures and commands for chrome.tabs API

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use super::tabs::{ExtensionTabInfo, TabRegistry};

/// Tab query info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInfo {
    pub active: Option<bool>,
    pub current_window: Option<bool>,
    pub highlighted: Option<bool>,
    pub pinned: Option<bool>,
    pub url: Option<String>,
    pub title: Option<String>,
}

/// Get tab by ID (chrome.tabs.get).
#[tauri::command]
pub fn extension_tabs_get(tab_id: i64, registry: State<'_, TabRegistry>) -> Result<ExtensionTabInfo, String> {
    registry.get(tab_id).ok_or_else(|| format!("Tab {} not found", tab_id))
}

/// Get current active tab (chrome.tabs.getCurrent).
#[tauri::command]
pub fn extension_tabs_get_current(registry: State<'_, TabRegistry>) -> Result<ExtensionTabInfo, String> {
    registry.get_current().ok_or_else(|| "No active tab found".to_string())
}

/// Move tab to new index (chrome.tabs.move).
#[tauri::command]
pub fn extension_tabs_move(tab_id: i64, move_properties: serde_json::Value, registry: State<'_, TabRegistry>) -> Result<ExtensionTabInfo, String> {
    let new_index = move_properties
        .get("index")
        .and_then(|v| v.as_i64())
        .ok_or("index is required")?;
    
    registry.move_tab(tab_id, new_index)?;
    
    // Return the moved tab
    registry.get(tab_id).ok_or_else(|| format!("Tab {} not found", tab_id))
}

/// Duplicate tab (chrome.tabs.duplicate).
#[tauri::command]
pub fn extension_tabs_duplicate(tab_id: i64, app: AppHandle) -> Result<ExtensionTabInfo, String> {
    let ops = vec![super::runtime::TabOpRequest {
        op: "duplicate".to_string(),
        extension_id: None,
        chrome_tab_id: tab_id,
        tab_ids: vec![],
        update_properties: None,
    }];
    let _ = app.emit(
        "exodus-extension-tabs-ops",
        super::runtime::ExtensionTabOpsPayload { ops },
    );
    // Return placeholder - actual tab creation happens in frontend
    Err("Tab duplication requires frontend coordination".to_string())
}

/// Go back in tab history (chrome.tabs.goBack).
#[tauri::command]
pub fn extension_tabs_go_back(tab_id: i64, app: AppHandle) -> Result<(), String> {
    let ops = vec![super::runtime::TabOpRequest {
        op: "goBack".to_string(),
        extension_id: None,
        chrome_tab_id: tab_id,
        tab_ids: vec![],
        update_properties: None,
    }];
    let _ = app.emit(
        "exodus-extension-tabs-ops",
        super::runtime::ExtensionTabOpsPayload { ops },
    );
    Ok(())
}

/// Go forward in tab history (chrome.tabs.goForward).
#[tauri::command]
pub fn extension_tabs_go_forward(tab_id: i64, app: AppHandle) -> Result<(), String> {
    let ops = vec![super::runtime::TabOpRequest {
        op: "goForward".to_string(),
        extension_id: None,
        chrome_tab_id: tab_id,
        tab_ids: vec![],
        update_properties: None,
    }];
    let _ = app.emit(
        "exodus-extension-tabs-ops",
        super::runtime::ExtensionTabOpsPayload { ops },
    );
    Ok(())
}

/// Detect tab language (chrome.tabs.detectLanguage).
#[tauri::command]
pub fn extension_tabs_detect_language(tab_id: i64, registry: State<'_, TabRegistry>) -> Result<String, String> {
    let tab = registry.get(tab_id).ok_or_else(|| format!("Tab {} not found", tab_id))?;
    // Simple language detection based on URL or content
    // In a full implementation, this would use a language detection library
    Ok("en".to_string()) // Placeholder
}

/// Capture visible tab (chrome.tabs.captureVisibleTab).
#[tauri::command]
pub fn extension_tabs_capture_visible_tab(
    window_id: Option<i32>,
    options: serde_json::Value,
    app: AppHandle,
) -> Result<String, String> {
    // Capture visible tab screenshot
    // In a full implementation, this would use the screenshot API
    // For now, return a placeholder error
    Err("Tab capture requires screenshot API integration".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_info_serialization() {
        let query = QueryInfo {
            active: Some(true),
            url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("active"));
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_query_info_default() {
        let query = QueryInfo::default();
        assert!(query.active.is_none());
        assert!(query.current_window.is_none());
        assert!(query.url.is_none());
    }

    #[test]
    fn test_query_info_all_fields() {
        let query = QueryInfo {
            active: Some(true),
            current_window: Some(false),
            highlighted: Some(true),
            pinned: Some(false),
            url: Some("https://example.com".to_string()),
            title: Some("Example".to_string()),
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("active"));
        assert!(json.contains("currentWindow"));
        assert!(json.contains("highlighted"));
        assert!(json.contains("pinned"));
        assert!(json.contains("url"));
        assert!(json.contains("title"));
    }
}

impl Default for QueryInfo {
    fn default() -> Self {
        Self {
            active: None,
            current_window: None,
            highlighted: None,
            pinned: None,
            url: None,
            title: None,
        }
    }
}
