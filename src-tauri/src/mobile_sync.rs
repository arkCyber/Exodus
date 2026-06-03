//! Enhanced Mobile Sync for Exodus Browser
//! Provides enhanced synchronization features for mobile devices

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSyncSettings {
    pub enabled: bool,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub sync_over_wifi_only: bool,
    pub sync_bookmarks: bool,
    pub sync_history: bool,
    pub sync_passwords: bool,
    pub sync_reading_list: bool,
}

impl Default for MobileSyncSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_sync: true,
            sync_interval_minutes: 30,
            sync_over_wifi_only: true,
            sync_bookmarks: true,
            sync_history: true,
            sync_passwords: false,
            sync_reading_list: true,
        }
    }
}

/// Mobile Sync Manager
pub struct MobileSyncManager {
    settings: Arc<Mutex<MobileSyncSettings>>,
    last_sync: Arc<Mutex<Option<String>>>,
}

impl MobileSyncManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(MobileSyncSettings::default())),
            last_sync: Arc::new(Mutex::new(None)),
        }
    }

    /// Enable mobile sync
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-mobile-sync-enabled", true);
        }
    }

    /// Disable mobile sync
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-mobile-sync-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set auto sync
    pub fn set_auto_sync(&self, auto: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.auto_sync = auto;
            let _ = app.emit("exodus-mobile-sync-auto-changed", auto);
        }
    }

    /// Get auto sync
    pub fn get_auto_sync(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.auto_sync)
            .unwrap_or(false)
    }

    /// Set sync interval
    pub fn set_sync_interval(&self, minutes: u32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sync_interval_minutes = minutes;
            let _ = app.emit("exodus-mobile-sync-interval-changed", minutes);
        }
    }

    /// Get sync interval
    pub fn get_sync_interval(&self) -> u32 {
        self.settings.lock()
            .map(|settings| settings.sync_interval_minutes)
            .unwrap_or(30)
    }

    /// Set sync over wifi only
    pub fn set_sync_over_wifi_only(&self, wifi_only: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sync_over_wifi_only = wifi_only;
            let _ = app.emit("exodus-mobile-sync-wifi-changed", wifi_only);
        }
    }

    /// Get sync over wifi only
    pub fn get_sync_over_wifi_only(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.sync_over_wifi_only)
            .unwrap_or(true)
    }

    /// Set sync bookmarks
    pub fn set_sync_bookmarks(&self, sync: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sync_bookmarks = sync;
            let _ = app.emit("exodus-mobile-sync-bookmarks-changed", sync);
        }
    }

    /// Get sync bookmarks
    pub fn get_sync_bookmarks(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.sync_bookmarks)
            .unwrap_or(true)
    }

    /// Set sync history
    pub fn set_sync_history(&self, sync: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sync_history = sync;
            let _ = app.emit("exodus-mobile-sync-history-changed", sync);
        }
    }

    /// Get sync history
    pub fn get_sync_history(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.sync_history)
            .unwrap_or(true)
    }

    /// Set sync passwords
    pub fn set_sync_passwords(&self, sync: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sync_passwords = sync;
            let _ = app.emit("exodus-mobile-sync-passwords-changed", sync);
        }
    }

    /// Get sync passwords
    pub fn get_sync_passwords(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.sync_passwords)
            .unwrap_or(false)
    }

    /// Set sync reading list
    pub fn set_sync_reading_list(&self, sync: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sync_reading_list = sync;
            let _ = app.emit("exodus-mobile-sync-reading-list-changed", sync);
        }
    }

    /// Get sync reading list
    pub fn get_sync_reading_list(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.sync_reading_list)
            .unwrap_or(true)
    }

    /// Trigger manual sync
    pub fn trigger_sync(&self, app: AppHandle) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        if let Ok(mut last_sync) = self.last_sync.lock() {
            *last_sync = Some(timestamp.clone());
        }
        let _ = app.emit("exodus-mobile-sync-triggered", timestamp);
    }

    /// Get last sync time
    pub fn get_last_sync(&self) -> Option<String> {
        self.last_sync.lock()
            .ok()
            .and_then(|last_sync| last_sync.clone())
    }

    /// Get settings
    pub fn get_settings(&self) -> MobileSyncSettings {
        self.settings.lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }
}

impl Default for MobileSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable mobile sync
#[tauri::command]
pub fn enable_mobile_sync(
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable mobile sync
#[tauri::command]
pub fn disable_mobile_sync(
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_mobile_sync_enabled(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set auto sync
#[tauri::command]
pub fn set_mobile_sync_auto(
    auto: bool,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_auto_sync(auto, app);
}

/// Tauri command to get auto sync
#[tauri::command]
pub fn get_mobile_sync_auto(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.get_auto_sync()
}

/// Tauri command to set sync interval
#[tauri::command]
pub fn set_mobile_sync_interval(
    minutes: u32,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_sync_interval(minutes, app);
}

/// Tauri command to get sync interval
#[tauri::command]
pub fn get_mobile_sync_interval(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> u32 {
    manager.get_sync_interval()
}

/// Tauri command to set sync over wifi only
#[tauri::command]
pub fn set_mobile_sync_wifi_only(
    wifi_only: bool,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_sync_over_wifi_only(wifi_only, app);
}

/// Tauri command to get sync over wifi only
#[tauri::command]
pub fn get_mobile_sync_wifi_only(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.get_sync_over_wifi_only()
}

/// Tauri command to set sync bookmarks
#[tauri::command]
pub fn set_mobile_sync_bookmarks(
    sync: bool,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_sync_bookmarks(sync, app);
}

/// Tauri command to get sync bookmarks
#[tauri::command]
pub fn get_mobile_sync_bookmarks(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.get_sync_bookmarks()
}

/// Tauri command to set sync history
#[tauri::command]
pub fn set_mobile_sync_history(
    sync: bool,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_sync_history(sync, app);
}

/// Tauri command to get sync history
#[tauri::command]
pub fn get_mobile_sync_history(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.get_sync_history()
}

/// Tauri command to set sync passwords
#[tauri::command]
pub fn set_mobile_sync_passwords(
    sync: bool,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_sync_passwords(sync, app);
}

/// Tauri command to get sync passwords
#[tauri::command]
pub fn get_mobile_sync_passwords(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.get_sync_passwords()
}

/// Tauri command to set sync reading list
#[tauri::command]
pub fn set_mobile_sync_reading_list(
    sync: bool,
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.set_sync_reading_list(sync, app);
}

/// Tauri command to get sync reading list
#[tauri::command]
pub fn get_mobile_sync_reading_list(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> bool {
    manager.get_sync_reading_list()
}

/// Tauri command to trigger manual sync
#[tauri::command]
pub fn trigger_mobile_sync(
    app: AppHandle,
    manager: State<'_, Arc<MobileSyncManager>>,
) {
    manager.trigger_sync(app);
}

/// Tauri command to get last sync time
#[tauri::command]
pub fn get_mobile_sync_last_sync(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> Option<String> {
    manager.get_last_sync()
}

/// Tauri command to get mobile sync settings
#[tauri::command]
pub fn get_mobile_sync_settings(
    manager: State<'_, Arc<MobileSyncManager>>,
) -> MobileSyncSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_sync_manager_creation() {
        let manager = MobileSyncManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = MobileSyncManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert!(settings.auto_sync);
    }
}
