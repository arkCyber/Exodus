//! Media Casting for Exodus Browser
//! Provides casting support for Chromecast, AirPlay, and other devices

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Cast device type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastDeviceType {
    Chromecast,
    AirPlay,
    DLNA,
    WebRTC,
}

/// Cast device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastDevice {
    pub id: String,
    pub name: String,
    pub device_type: CastDeviceType,
    pub available: bool,
}

/// Cast state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CastState {
    Idle,
    Connecting,
    Connected,
    Casting,
    Disconnected,
    Error,
}

impl Default for CastState {
    fn default() -> Self {
        CastState::Idle
    }
}

/// Media casting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCastingSettings {
    pub enabled: bool,
    pub auto_discover: bool,
    pub show_cast_indicator: bool,
}

impl Default for MediaCastingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_discover: true,
            show_cast_indicator: true,
        }
    }
}

/// Media Casting Manager
pub struct MediaCastingManager {
    devices: Arc<Mutex<Vec<CastDevice>>>,
    current_device: Arc<Mutex<Option<String>>>, // device_id
    state: Arc<Mutex<CastState>>,
    settings: Arc<Mutex<MediaCastingSettings>>,
}

impl MediaCastingManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(vec![])),
            current_device: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(CastState::Idle)),
            settings: Arc::new(Mutex::new(MediaCastingSettings::default())),
        }
    }

    /// Discover cast devices
    pub fn discover_devices(&self, app: AppHandle) -> Vec<CastDevice> {
        // Placeholder for device discovery
        // In real implementation, this would scan the network for Chromecast, AirPlay, DLNA devices
        let discovered = vec![
            CastDevice {
                id: "chromecast-1".to_string(),
                name: "Living Room TV".to_string(),
                device_type: CastDeviceType::Chromecast,
                available: true,
            },
        ];
        
        if let Ok(mut devices) = self.devices.lock() {
            *devices = discovered.clone();
            let _ = app.emit("exodus-cast-devices-discovered", discovered.clone());
        }
        discovered
    }

    /// Get available devices
    pub fn get_devices(&self) -> Vec<CastDevice> {
        self.devices.lock()
            .map(|devices| devices.clone())
            .unwrap_or_default()
    }

    /// Start casting to a device
    pub fn start_cast(&self, device_id: String, url: String, app: AppHandle) -> bool {
        let devices = self.devices.lock();
        let device_exists = devices.as_ref().ok().map(|d| d.iter().any(|d| d.id == device_id)).unwrap_or(false);
        drop(devices);
        
        if !device_exists {
            return false;
        }
        
        if let Ok(mut state) = self.state.lock() {
            *state = CastState::Connecting;
            let _ = app.emit("exodus-cast-state-changed", CastState::Connecting);
        }

        if let Ok(mut current_device) = self.current_device.lock() {
            *current_device = Some(device_id.clone());
        }

        // Simulate connection
        if let Ok(mut state) = self.state.lock() {
            *state = CastState::Casting;
            let _ = app.emit("exodus-cast-state-changed", CastState::Casting);
            let _ = app.emit("exodus-cast-started", (device_id, url));
        }
        
        true
    }

    /// Stop casting
    pub fn stop_cast(&self, app: AppHandle) {
        if let Ok(mut current_device) = self.current_device.lock() {
            *current_device = None;
        }

        if let Ok(mut state) = self.state.lock() {
            *state = CastState::Idle;
            let _ = app.emit("exodus-cast-state-changed", CastState::Idle);
            let _ = app.emit("exodus-cast-stopped", ());
        }
    }

    /// Get current cast state
    pub fn get_state(&self) -> CastState {
        self.state.lock()
            .map(|state| state.clone())
            .unwrap_or_default()
    }

    /// Get current device
    pub fn get_current_device(&self) -> Option<CastDevice> {
        let current_device = self.current_device.lock();
        let devices = self.devices.lock();
        
        if let (Ok(current_device), Ok(devices)) = (current_device, devices) {
            if let Some(device_id) = current_device.as_ref() {
                devices.iter().find(|d| &d.id == device_id).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Enable media casting
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-media-casting-enabled", true);
        }
    }

    /// Disable media casting
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-media-casting-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set auto discover
    pub fn set_auto_discover(&self, enabled: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.auto_discover = enabled;
            let _ = app.emit("exodus-cast-auto-discover-changed", enabled);
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> MediaCastingSettings {
        self.settings.lock()
            .map(|settings| MediaCastingSettings {
                enabled: settings.enabled,
                auto_discover: settings.auto_discover,
                show_cast_indicator: settings.show_cast_indicator,
            })
            .unwrap_or_default()
    }
}

impl Default for MediaCastingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to discover cast devices
#[tauri::command]
pub fn discover_cast_devices(
    app: AppHandle,
    manager: State<'_, Arc<MediaCastingManager>>,
) -> Vec<CastDevice> {
    manager.discover_devices(app)
}

/// Tauri command to get cast devices
#[tauri::command]
pub fn get_cast_devices(
    manager: State<'_, Arc<MediaCastingManager>>,
) -> Vec<CastDevice> {
    manager.get_devices()
}

/// Tauri command to start cast
#[tauri::command]
pub fn start_cast(
    device_id: String,
    url: String,
    app: AppHandle,
    manager: State<'_, Arc<MediaCastingManager>>,
) -> bool {
    manager.start_cast(device_id, url, app)
}

/// Tauri command to stop cast
#[tauri::command]
pub fn stop_cast(
    app: AppHandle,
    manager: State<'_, Arc<MediaCastingManager>>,
) {
    manager.stop_cast(app);
}

/// Tauri command to get cast state
#[tauri::command]
pub fn get_cast_state(
    manager: State<'_, Arc<MediaCastingManager>>,
) -> CastState {
    manager.get_state()
}

/// Tauri command to get current cast device
#[tauri::command]
pub fn get_current_cast_device(
    manager: State<'_, Arc<MediaCastingManager>>,
) -> Option<CastDevice> {
    manager.get_current_device()
}

/// Tauri command to enable media casting
#[tauri::command]
pub fn enable_media_casting(
    app: AppHandle,
    manager: State<'_, Arc<MediaCastingManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable media casting
#[tauri::command]
pub fn disable_media_casting(
    app: AppHandle,
    manager: State<'_, Arc<MediaCastingManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_media_casting_enabled(
    manager: State<'_, Arc<MediaCastingManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set auto discover
#[tauri::command]
pub fn set_cast_auto_discover(
    enabled: bool,
    app: AppHandle,
    manager: State<'_, Arc<MediaCastingManager>>,
) {
    manager.set_auto_discover(enabled, app);
}

/// Tauri command to get media casting settings
#[tauri::command]
pub fn get_media_casting_settings(
    manager: State<'_, Arc<MediaCastingManager>>,
) -> MediaCastingSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_casting_manager_creation() {
        let manager = MediaCastingManager::new();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_discover_devices() {
        let manager = MediaCastingManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(manager.get_devices().is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = MediaCastingManager::new();
        
        let settings = manager.get_settings();
        assert!(settings.enabled);
        assert!(settings.auto_discover);
    }
}
