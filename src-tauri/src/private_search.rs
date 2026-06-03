//! Private Search Tab for Exodus Browser
//! Provides dedicated private search tab functionality

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Private search settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateSearchSettings {
    pub enabled: bool,
    pub default_search_engine: String,
    pub block_trackers: bool,
    pub clear_on_close: bool,
    pub separate_history: bool,
}

impl Default for PrivateSearchSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            default_search_engine: "duckduckgo".to_string(),
            block_trackers: true,
            clear_on_close: true,
            separate_history: true,
        }
    }
}

/// Private Search Manager
pub struct PrivateSearchManager {
    settings: Arc<Mutex<PrivateSearchSettings>>,
}

impl PrivateSearchManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(PrivateSearchSettings::default())),
        }
    }

    /// Enable private search tab
    pub fn enable(&self, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.enabled = true;
        let _ = app.emit("exodus-private-search-enabled", true);
    }

    /// Disable private search tab
    pub fn disable(&self, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.enabled = false;
        let _ = app.emit("exodus-private-search-enabled", false);
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        settings.enabled
    }

    /// Set default search engine
    pub fn set_default_search_engine(&self, engine: String, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.default_search_engine = engine.clone();
        let _ = app.emit("exodus-private-search-engine-changed", engine);
    }

    /// Get default search engine
    pub fn get_default_search_engine(&self) -> String {
        let settings = self.settings.lock().unwrap();
        settings.default_search_engine.clone()
    }

    /// Set block trackers
    pub fn set_block_trackers(&self, block: bool, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.block_trackers = block;
        let _ = app.emit("exodus-private-search-block-trackers-changed", block);
    }

    /// Get block trackers
    pub fn get_block_trackers(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        settings.block_trackers
    }

    /// Set clear on close
    pub fn set_clear_on_close(&self, clear: bool, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.clear_on_close = clear;
        let _ = app.emit("exodus-private-search-clear-on-close-changed", clear);
    }

    /// Get clear on close
    pub fn get_clear_on_close(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        settings.clear_on_close
    }

    /// Set separate history
    pub fn set_separate_history(&self, separate: bool, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.separate_history = separate;
        let _ = app.emit("exodus-private-search-separate-history-changed", separate);
    }

    /// Get separate history
    pub fn get_separate_history(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        settings.separate_history
    }

    /// Get settings
    pub fn get_settings(&self) -> PrivateSearchSettings {
        let settings = self.settings.lock().unwrap();
        PrivateSearchSettings {
            enabled: settings.enabled,
            default_search_engine: settings.default_search_engine.clone(),
            block_trackers: settings.block_trackers,
            clear_on_close: settings.clear_on_close,
            separate_history: settings.separate_history,
        }
    }
}

impl Default for PrivateSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable private search tab
#[tauri::command]
pub fn enable_private_search_tab(
    app: AppHandle,
    manager: State<'_, Arc<PrivateSearchManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable private search tab
#[tauri::command]
pub fn disable_private_search_tab(
    app: AppHandle,
    manager: State<'_, Arc<PrivateSearchManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_private_search_tab_enabled(
    manager: State<'_, Arc<PrivateSearchManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set default search engine
#[tauri::command]
pub fn set_private_search_engine(
    engine: String,
    app: AppHandle,
    manager: State<'_, Arc<PrivateSearchManager>>,
) {
    manager.set_default_search_engine(engine, app);
}

/// Tauri command to get default search engine
#[tauri::command]
pub fn get_private_search_engine(
    manager: State<'_, Arc<PrivateSearchManager>>,
) -> String {
    manager.get_default_search_engine()
}

/// Tauri command to set block trackers
#[tauri::command]
pub fn set_private_search_block_trackers(
    block: bool,
    app: AppHandle,
    manager: State<'_, Arc<PrivateSearchManager>>,
) {
    manager.set_block_trackers(block, app);
}

/// Tauri command to get block trackers
#[tauri::command]
pub fn get_private_search_block_trackers(
    manager: State<'_, Arc<PrivateSearchManager>>,
) -> bool {
    manager.get_block_trackers()
}

/// Tauri command to set clear on close
#[tauri::command]
pub fn set_private_search_clear_on_close(
    clear: bool,
    app: AppHandle,
    manager: State<'_, Arc<PrivateSearchManager>>,
) {
    manager.set_clear_on_close(clear, app);
}

/// Tauri command to get clear on close
#[tauri::command]
pub fn get_private_search_clear_on_close(
    manager: State<'_, Arc<PrivateSearchManager>>,
) -> bool {
    manager.get_clear_on_close()
}

/// Tauri command to set separate history
#[tauri::command]
pub fn set_private_search_separate_history(
    separate: bool,
    app: AppHandle,
    manager: State<'_, Arc<PrivateSearchManager>>,
) {
    manager.set_separate_history(separate, app);
}

/// Tauri command to get separate history
#[tauri::command]
pub fn get_private_search_separate_history(
    manager: State<'_, Arc<PrivateSearchManager>>,
) -> bool {
    manager.get_separate_history()
}

/// Tauri command to get private search settings
#[tauri::command]
pub fn get_private_search_settings(
    manager: State<'_, Arc<PrivateSearchManager>>,
) -> PrivateSearchSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_search_manager_creation() {
        let manager = PrivateSearchManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = PrivateSearchManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert_eq!(settings.default_search_engine, "duckduckgo");
    }
}
