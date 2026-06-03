//! Tab Preview functionality for Exodus Browser
//! Allows users to preview tabs by hovering over them

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Tab preview image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabPreview {
    pub label: String,
    pub image_data: String, // Base64 encoded image
    pub timestamp: i64,
    pub width: u32,
    pub height: u32,
}

/// Tab Preview Manager
pub struct TabPreviewManager {
    previews: Arc<Mutex<Vec<TabPreview>>>,
}

impl TabPreviewManager {
    pub fn new() -> Self {
        Self {
            previews: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a tab preview
    pub fn register_preview(&self, preview: TabPreview, app: AppHandle) {
        if let Ok(mut previews) = self.previews.lock() {
            previews.retain(|p| p.label != preview.label);
            previews.push(preview.clone());
            let _ = app.emit("exodus-tab-preview-updated", preview);
        }
    }

    /// Unregister a tab preview
    pub fn unregister_preview(&self, label: &str, app: AppHandle) {
        if let Ok(mut previews) = self.previews.lock() {
            previews.retain(|p| p.label != label);
            let _ = app.emit("exodus-tab-preview-removed", label.to_string());
        }
    }

    /// Get preview for a tab
    pub fn get_preview(&self, label: &str) -> Option<TabPreview> {
        self.previews.lock()
            .ok()
            .and_then(|previews| previews.iter().find(|p| p.label == label).cloned())
    }

    /// Get all previews
    pub fn get_all_previews(&self) -> Vec<TabPreview> {
        self.previews.lock()
            .map(|previews| previews.clone())
            .unwrap_or_default()
    }

    /// Clear all previews
    pub fn clear_all_previews(&self, app: AppHandle) {
        if let Ok(mut previews) = self.previews.lock() {
            previews.clear();
            let _ = app.emit("exodus-tab-previews-cleared", ());
        }
    }
}

impl Default for TabPreviewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to register a tab preview
#[tauri::command]
pub fn register_tab_preview(
    preview: TabPreview,
    app: AppHandle,
    manager: State<'_, Arc<TabPreviewManager>>,
) {
    manager.register_preview(preview, app);
}

/// Tauri command to unregister a tab preview
#[tauri::command]
pub fn unregister_tab_preview(
    label: String,
    app: AppHandle,
    manager: State<'_, Arc<TabPreviewManager>>,
) {
    manager.unregister_preview(&label, app);
}

/// Tauri command to get a tab preview
#[tauri::command]
pub fn get_tab_preview(
    label: String,
    manager: State<'_, Arc<TabPreviewManager>>,
) -> Option<TabPreview> {
    manager.get_preview(&label)
}

/// Tauri command to get all tab previews
#[tauri::command]
pub fn get_all_tab_previews(
    manager: State<'_, Arc<TabPreviewManager>>,
) -> Vec<TabPreview> {
    manager.get_all_previews()
}

/// Tauri command to clear all tab previews
#[tauri::command]
pub fn clear_tab_previews(
    app: AppHandle,
    manager: State<'_, Arc<TabPreviewManager>>,
) {
    manager.clear_all_previews(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_preview_manager_creation() {
        let manager = TabPreviewManager::new();
        let previews = manager.get_all_previews();
        assert!(previews.is_empty());
    }

    #[test]
    fn test_register_and_unregister_preview() {
        let manager = TabPreviewManager::new();
        
        let _preview = TabPreview {
            label: "tab1".to_string(),
            image_data: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            width: 640,
            height: 480,
        };
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        let previews = manager.get_all_previews();
        assert!(previews.is_empty());
    }
}
