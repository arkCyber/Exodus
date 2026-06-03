//! Global Audio Control for Exodus Browser
//! Provides global audio control for all tabs

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Audio state for a tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabAudioState {
    pub label: String,
    pub muted: bool,
    pub volume: f32,
    pub is_playing: bool,
}

/// Global audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAudioSettings {
    pub global_muted: bool,
    pub global_volume: f32,
    pub show_volume_indicator: bool,
}

impl Default for GlobalAudioSettings {
    fn default() -> Self {
        Self {
            global_muted: false,
            global_volume: 1.0,
            show_volume_indicator: true,
        }
    }
}

/// Global Audio Manager
pub struct GlobalAudioManager {
    settings: Arc<Mutex<GlobalAudioSettings>>,
    tab_states: Arc<Mutex<Vec<TabAudioState>>>,
}

impl GlobalAudioManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(GlobalAudioSettings::default())),
            tab_states: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Mute all tabs
    pub fn mute_all(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.global_muted = true;
            let _ = app.emit("exodus-global-audio-muted", true);
        }
        
        // Also mute all individual tabs
        if let Ok(mut tab_states) = self.tab_states.lock() {
            for tab in tab_states.iter_mut() {
                tab.muted = true;
            }
            let _ = app.emit("exodus-all-tabs-muted", true);
        }
    }

    /// Unmute all tabs
    pub fn unmute_all(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.global_muted = false;
            let _ = app.emit("exodus-global-audio-muted", false);
        }
        
        // Also unmute all individual tabs
        if let Ok(mut tab_states) = self.tab_states.lock() {
            for tab in tab_states.iter_mut() {
                tab.muted = false;
            }
            let _ = app.emit("exodus-all-tabs-muted", false);
        }
    }

    /// Toggle global mute
    pub fn toggle_global_mute(&self, app: AppHandle) {
        let muted = self.is_globally_muted();
        if muted {
            self.unmute_all(app);
        } else {
            self.mute_all(app);
        }
    }

    /// Check if globally muted
    pub fn is_globally_muted(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.global_muted)
            .unwrap_or(false)
    }

    /// Set global volume
    pub fn set_global_volume(&self, volume: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.global_volume = volume.clamp(0.0, 1.0);
            let _ = app.emit("exodus-global-volume-changed", settings.global_volume);
        }
    }

    /// Get global volume
    pub fn get_global_volume(&self) -> f32 {
        self.settings.lock()
            .map(|settings| settings.global_volume)
            .unwrap_or(1.0)
    }

    /// Register a tab
    pub fn register_tab(&self, label: String, app: AppHandle) {
        if let Ok(mut tab_states) = self.tab_states.lock() {
            // Check if tab already exists
            if tab_states.iter().any(|t| t.label == label) {
                return;
            }
            
            tab_states.push(TabAudioState {
                label: label.clone(),
                muted: self.is_globally_muted(),
                volume: self.get_global_volume(),
                is_playing: false,
            });
            let _ = app.emit("exodus-tab-registered", label);
        }
    }

    /// Unregister a tab
    pub fn unregister_tab(&self, label: String, app: AppHandle) {
        if let Ok(mut tab_states) = self.tab_states.lock() {
            tab_states.retain(|t| t.label != label);
            let _ = app.emit("exodus-tab-unregistered", label);
        }
    }

    /// Set tab mute state
    pub fn set_tab_mute(&self, label: String, muted: bool, app: AppHandle) {
        if let Ok(mut tab_states) = self.tab_states.lock() {
            if let Some(tab) = tab_states.iter_mut().find(|t| t.label == label) {
                tab.muted = muted;
                let _ = app.emit("exodus-tab-mute-changed", (label.clone(), muted));
            }
        }
    }

    /// Set tab volume
    pub fn set_tab_volume(&self, label: String, volume: f32, app: AppHandle) {
        if let Ok(mut tab_states) = self.tab_states.lock() {
            if let Some(tab) = tab_states.iter_mut().find(|t| t.label == label) {
                tab.volume = volume.clamp(0.0, 1.0);
                let _ = app.emit("exodus-tab-volume-changed", (label.clone(), tab.volume));
            }
        }
    }

    /// Update tab playing state
    pub fn update_tab_playing(&self, label: String, is_playing: bool, app: AppHandle) {
        if let Ok(mut tab_states) = self.tab_states.lock() {
            if let Some(tab) = tab_states.iter_mut().find(|t| t.label == label) {
                tab.is_playing = is_playing;
                let _ = app.emit("exodus-tab-playing-changed", (label.clone(), is_playing));
            }
        }
    }

    /// Get all tab states
    pub fn get_all_tabs(&self) -> Vec<TabAudioState> {
        self.tab_states.lock()
            .map(|tab_states| tab_states.clone())
            .unwrap_or_default()
    }

    /// Get playing tabs count
    pub fn get_playing_count(&self) -> usize {
        self.tab_states.lock()
            .map(|tab_states| tab_states.iter().filter(|t| t.is_playing).count())
            .unwrap_or(0)
    }

    /// Show/hide volume indicator
    pub fn show_volume_indicator(&self, show: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.show_volume_indicator = show;
            let _ = app.emit("exodus-volume-indicator-changed", show);
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> GlobalAudioSettings {
        self.settings.lock()
            .map(|settings| GlobalAudioSettings {
                global_muted: settings.global_muted,
                global_volume: settings.global_volume,
                show_volume_indicator: settings.show_volume_indicator,
            })
            .unwrap_or_default()
    }
}

impl Default for GlobalAudioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to mute all tabs
#[tauri::command]
pub fn mute_all_tabs(
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.mute_all(app);
}

/// Tauri command to unmute all tabs
#[tauri::command]
pub fn unmute_all_tabs(
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.unmute_all(app);
}

/// Tauri command to toggle global mute
#[tauri::command]
pub fn toggle_global_mute(
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.toggle_global_mute(app);
}

/// Tauri command to check if globally muted
#[tauri::command]
pub fn is_globally_muted(
    manager: State<'_, Arc<GlobalAudioManager>>,
) -> bool {
    manager.is_globally_muted()
}

/// Tauri command to set global volume
#[tauri::command]
pub fn set_global_volume(
    volume: f32,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.set_global_volume(volume, app);
}

/// Tauri command to get global volume
#[tauri::command]
pub fn get_global_volume(
    manager: State<'_, Arc<GlobalAudioManager>>,
) -> f32 {
    manager.get_global_volume()
}

/// Tauri command to register a tab
#[tauri::command]
pub fn register_audio_tab(
    label: String,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.register_tab(label, app);
}

/// Tauri command to unregister a tab
#[tauri::command]
pub fn unregister_audio_tab(
    label: String,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.unregister_tab(label, app);
}

/// Tauri command to set tab mute
#[tauri::command]
pub fn set_global_tab_mute(
    label: String,
    muted: bool,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.set_tab_mute(label, muted, app);
}

/// Tauri command to set tab volume
#[tauri::command]
pub fn set_tab_volume(
    label: String,
    volume: f32,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.set_tab_volume(label, volume, app);
}

/// Tauri command to update tab playing state
#[tauri::command]
pub fn update_tab_playing(
    label: String,
    is_playing: bool,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.update_tab_playing(label, is_playing, app);
}

/// Tauri command to get all tab states
#[tauri::command]
pub fn get_all_audio_tabs(
    manager: State<'_, Arc<GlobalAudioManager>>,
) -> Vec<TabAudioState> {
    manager.get_all_tabs()
}

/// Tauri command to get playing tabs count
#[tauri::command]
pub fn get_playing_tabs_count(
    manager: State<'_, Arc<GlobalAudioManager>>,
) -> usize {
    manager.get_playing_count()
}

/// Tauri command to show/hide volume indicator
#[tauri::command]
pub fn show_volume_indicator(
    show: bool,
    app: AppHandle,
    manager: State<'_, Arc<GlobalAudioManager>>,
) {
    manager.show_volume_indicator(show, app);
}

/// Tauri command to get global audio settings
#[tauri::command]
pub fn get_global_audio_settings(
    manager: State<'_, Arc<GlobalAudioManager>>,
) -> GlobalAudioSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_audio_manager_creation() {
        let manager = GlobalAudioManager::new();
        assert!(!manager.is_globally_muted());
        assert_eq!(manager.get_global_volume(), 1.0);
    }

    #[test]
    fn test_mute_unmute() {
        let manager = GlobalAudioManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_globally_muted());
    }

    #[test]
    fn test_register_unregister() {
        let manager = GlobalAudioManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        let tabs = manager.get_all_tabs();
        assert!(tabs.is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = GlobalAudioManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.global_muted);
        assert_eq!(settings.global_volume, 1.0);
        assert!(settings.show_volume_indicator);
    }
}
