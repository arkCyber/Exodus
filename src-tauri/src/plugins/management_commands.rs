//! Chrome Extension API - chrome.management

use crate::plugins::manager::{ExtensionManager, ExtensionState};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

#[tauri::command]
pub async fn chrome_management_get(
    extension_id: String,
    extension_state: State<'_, ExtensionState>,
) -> Result<ExtensionInfo, String> {
    // Get extension info by ID
    let manager = extension_state
        .lock()
        .map_err(|e| format!("Extension state lock error: {}", e))?;
    
    let extensions = manager.list();
    
    if let Some(ext) = extensions.iter().find(|e| e.id == extension_id) {
        Ok(ExtensionInfo {
            id: ext.id.clone(),
            name: ext.name.clone(),
            version: ext.version.clone(),
            enabled: ext.enabled,
        })
    } else {
        Err("Extension not found".to_string())
    }
}

#[tauri::command]
pub async fn chrome_management_get_all(
    extension_state: State<'_, ExtensionState>,
) -> Result<Vec<ExtensionInfo>, String> {
    // Get all extensions
    let manager = extension_state
        .lock()
        .map_err(|e| format!("Extension state lock error: {}", e))?;
    
    let extensions = manager.list();
    
    let result: Vec<ExtensionInfo> = extensions
        .into_iter()
        .map(|ext| ExtensionInfo {
            id: ext.id,
            name: ext.name,
            version: ext.version,
            enabled: ext.enabled,
        })
        .collect();
    
    Ok(result)
}

#[tauri::command]
pub async fn chrome_management_get_self(
    extension_state: State<'_, ExtensionState>,
) -> Result<ExtensionInfo, String> {
    // Get the calling extension's info
    // This requires knowing which extension is calling, which may need context
    // For now, return an error as we need caller context
    Err("get_self requires caller extension context".to_string())
}

#[tauri::command]
pub async fn chrome_management_set_enabled(
    extension_id: String,
    enabled: bool,
    extension_state: State<'_, ExtensionState>,
) -> Result<(), String> {
    // Enable or disable an extension
    let mut manager = extension_state
        .lock()
        .map_err(|e| format!("Extension state lock error: {}", e))?;
    
    manager.set_enabled(&extension_id, enabled)
        .map_err(|e| format!("Failed to set extension enabled state: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_info_serialization() {
        let info = ExtensionInfo {
            id: "test-ext".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("id"));
        assert!(json.contains("name"));
        assert!(json.contains("version"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn test_extension_info_all_fields() {
        let info = ExtensionInfo {
            id: "ext-123".to_string(),
            name: "My Extension".to_string(),
            version: "2.5.1".to_string(),
            enabled: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("ext-123"));
        assert!(json.contains("My Extension"));
        assert!(json.contains("2.5.1"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_extension_info_deserialization() {
        let json = r#"{"id":"test","name":"Test","version":"1.0","enabled":true}"#;
        let info: ExtensionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, "test");
        assert_eq!(info.name, "Test");
        assert_eq!(info.version, "1.0");
        assert_eq!(info.enabled, true);
    }
}
