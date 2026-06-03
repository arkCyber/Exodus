//! Omnibox Image Search for Exodus Browser
//! Provides image search functionality from the omnibox

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Image search settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmniboxImageSearchSettings {
    pub enabled: bool,
    pub default_engine: String,
    pub show_preview: bool,
    pub safe_search: bool,
}

impl Default for OmniboxImageSearchSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            default_engine: "google".to_string(),
            show_preview: true,
            safe_search: true,
        }
    }
}

/// Omnibox Image Search Manager
pub struct OmniboxImageSearchManager {
    settings: Arc<Mutex<OmniboxImageSearchSettings>>,
}

impl OmniboxImageSearchManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(OmniboxImageSearchSettings::default())),
        }
    }

    /// Enable omnibox image search
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-omnibox-image-search-enabled", true);
        }
    }

    /// Disable omnibox image search
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-omnibox-image-search-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set default engine
    pub fn set_default_engine(&self, engine: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.default_engine = engine.clone();
            let _ = app.emit("exodus-omnibox-image-search-engine-changed", engine);
        }
    }

    /// Get default engine
    pub fn get_default_engine(&self) -> String {
        self.settings.lock()
            .map(|settings| settings.default_engine.clone())
            .unwrap_or_default()
    }

    /// Set show preview
    pub fn set_show_preview(&self, show: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.show_preview = show;
            let _ = app.emit("exodus-omnibox-image-search-preview-changed", show);
        }
    }

    /// Get show preview
    pub fn get_show_preview(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.show_preview)
            .unwrap_or(false)
    }

    /// Set safe search
    pub fn set_safe_search(&self, safe: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.safe_search = safe;
            let _ = app.emit("exodus-omnibox-image-search-safe-changed", safe);
        }
    }

    /// Get safe search
    pub fn get_safe_search(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.safe_search)
            .unwrap_or(false)
    }

    /// Get settings
    pub fn get_settings(&self) -> OmniboxImageSearchSettings {
        self.settings.lock()
            .map(|settings| OmniboxImageSearchSettings {
                enabled: settings.enabled,
                default_engine: settings.default_engine.clone(),
                show_preview: settings.show_preview,
                safe_search: settings.safe_search,
            })
            .unwrap_or_default()
    }
}

impl Default for OmniboxImageSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable omnibox image search
#[tauri::command]
pub fn enable_omnibox_image_search(
    app: AppHandle,
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable omnibox image search
#[tauri::command]
pub fn disable_omnibox_image_search(
    app: AppHandle,
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_omnibox_image_search_enabled(
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set default engine
#[tauri::command]
pub fn set_omnibox_image_search_engine(
    engine: String,
    app: AppHandle,
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) {
    manager.set_default_engine(engine, app);
}

/// Tauri command to get default engine
#[tauri::command]
pub fn get_omnibox_image_search_engine(
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) -> String {
    manager.get_default_engine()
}

/// Tauri command to set show preview
#[tauri::command]
pub fn set_omnibox_image_search_show_preview(
    show: bool,
    app: AppHandle,
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) {
    manager.set_show_preview(show, app);
}

/// Tauri command to get show preview
#[tauri::command]
pub fn get_omnibox_image_search_show_preview(
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) -> bool {
    manager.get_show_preview()
}

/// Tauri command to set safe search
#[tauri::command]
pub fn set_omnibox_image_search_safe_search(
    safe: bool,
    app: AppHandle,
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) {
    manager.set_safe_search(safe, app);
}

/// Tauri command to get safe search
#[tauri::command]
pub fn get_omnibox_image_search_safe_search(
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) -> bool {
    manager.get_safe_search()
}

/// Tauri command to get omnibox image search settings
#[tauri::command]
pub fn get_omnibox_image_search_settings(
    manager: State<'_, Arc<OmniboxImageSearchManager>>,
) -> OmniboxImageSearchSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_omnibox_image_search_manager_creation() {
        let manager = OmniboxImageSearchManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = OmniboxImageSearchManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert_eq!(settings.default_engine, "google");
    }
}
