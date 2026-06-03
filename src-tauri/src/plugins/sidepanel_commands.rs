//! Chrome Extension API - chrome.sidePanel
//!
//! Provides Tauri commands for the chrome.sidePanel API, allowing extensions
//! to interact with the browser's side panel.

use crate::plugins::manager::ExtensionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Side panel options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidePanelOptions {
    pub tab_id: Option<i32>,
    pub path: Option<String>,
    pub enabled: Option<bool>,
    pub title: Option<String>,
}

/// Side panel behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelBehavior {
    pub open_panel_on_action_click: Option<bool>,
}

/// Get side panel options
#[tauri::command]
pub async fn chrome_sidepanel_get_options(
    tab_id: Option<i32>,
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<SidePanelOptions, String> {
    // Get side panel options for a specific tab or globally
    // Placeholder implementation
    Ok(SidePanelOptions {
        tab_id,
        path: None,
        enabled: Some(true),
        title: None,
    })
}

/// Set side panel options
#[tauri::command]
pub async fn chrome_sidepanel_set_options(
    options: SidePanelOptions,
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<(), String> {
    // Set side panel options
    // Placeholder implementation - requires UI integration for side panel management
    // In a full implementation, this would:
    // 1. Store the options per extension/tab
    // 2. Update the UI to reflect the new path/title/enabled state
    // 3. Emit events to notify the frontend
    Ok(())
}

/// Set panel behavior
#[tauri::command]
pub async fn chrome_sidepanel_set_panel_behavior(
    behavior: PanelBehavior,
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<(), String> {
    // Set panel behavior
    // Placeholder implementation - requires UI integration
    // In a full implementation, this would:
    // 1. Store the behavior configuration per extension
    // 2. Update the UI to reflect the new behavior
    Ok(())
}

/// Get panel behavior
#[tauri::command]
pub async fn chrome_sidepanel_get_panel_behavior(
    extension_manager: State<'_, Arc<ExtensionManager>>,
) -> Result<PanelBehavior, String> {
    // Get panel behavior
    Ok(PanelBehavior {
        open_panel_on_action_click: Some(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_panel_options_deserialization() {
        let options_json = r#"{"tabId":1,"path":"panel.html","enabled":true}"#;
        let options: SidePanelOptions = serde_json::from_str(options_json).unwrap();
        assert_eq!(options.tab_id, Some(1));
        assert_eq!(options.path, Some("panel.html".to_string()));
        assert_eq!(options.enabled, Some(true));
    }

    #[test]
    fn test_panel_behavior_deserialization() {
        let behavior_json = r#"{"openPanelOnActionClick":true}"#;
        let behavior: PanelBehavior = serde_json::from_str(behavior_json).unwrap();
        assert_eq!(behavior.open_panel_on_action_click, Some(true));
    }
}
