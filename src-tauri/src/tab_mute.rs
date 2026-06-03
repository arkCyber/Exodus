//! Tab Mute functionality for Exodus Browser
//! Allows users to mute/unmute audio in individual tabs

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Tab mute state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabMuteState {
    pub label: String,
    pub is_muted: bool,
    pub audio_playing: bool,
}

/// Tab Mute Manager
pub struct TabMuteManager {
    states: Arc<Mutex<Vec<TabMuteState>>>,
}

impl TabMuteManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a tab for mute tracking
    pub fn register_tab(&self, label: String) {
        let mut states = self.states.lock().unwrap();
        if !states.iter().any(|s| s.label == label) {
            states.push(TabMuteState {
                label,
                is_muted: false,
                audio_playing: false,
            });
        }
    }

    /// Unregister a tab
    pub fn unregister_tab(&self, label: &str) {
        let mut states = self.states.lock().unwrap();
        states.retain(|s| s.label != label);
    }

    /// Set mute state for a tab
    pub fn set_mute(&self, label: &str, is_muted: bool, app: AppHandle) {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.iter_mut().find(|s| s.label == label) {
            state.is_muted = is_muted;
            let _ = app.emit("exodus-tab-mute-changed", state.clone());
        }
    }

    /// Get mute state for a tab
    pub fn get_mute_state(&self, label: &str) -> Option<bool> {
        let states = self.states.lock().unwrap();
        states.iter().find(|s| s.label == label).map(|s| s.is_muted)
    }

    /// Update audio playing state for a tab
    pub fn set_audio_playing(&self, label: &str, audio_playing: bool, app: AppHandle) {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.iter_mut().find(|s| s.label == label) {
            state.audio_playing = audio_playing;
            let _ = app.emit("exodus-tab-audio-changed", state.clone());
        }
    }

    /// Get all tabs with audio playing
    pub fn get_audio_playing_tabs(&self) -> Vec<TabMuteState> {
        let states = self.states.lock().unwrap();
        states.iter().filter(|s| s.audio_playing).cloned().collect()
    }

    /// Get all tabs
    pub fn get_all_tabs(&self) -> Vec<TabMuteState> {
        let states = self.states.lock().unwrap();
        states.clone()
    }
}

impl Default for TabMuteManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to register a tab for mute tracking
#[tauri::command]
pub fn register_mute_tab(
    label: String,
    manager: State<'_, Arc<TabMuteManager>>,
) {
    manager.register_tab(label);
}

/// Tauri command to unregister a tab
#[tauri::command]
pub fn unregister_mute_tab(
    label: String,
    manager: State<'_, Arc<TabMuteManager>>,
) {
    manager.unregister_tab(&label);
}

/// Tauri command to set mute state
#[tauri::command]
pub fn set_tab_mute(
    label: String,
    is_muted: bool,
    app: AppHandle,
    manager: State<'_, Arc<TabMuteManager>>,
) {
    manager.set_mute(&label, is_muted, app);
}

/// Tauri command to get mute state
#[tauri::command]
pub fn get_tab_mute_state(
    label: String,
    manager: State<'_, Arc<TabMuteManager>>,
) -> Option<bool> {
    manager.get_mute_state(&label)
}

/// Tauri command to set audio playing state
#[tauri::command]
pub fn set_tab_audio_playing(
    label: String,
    audio_playing: bool,
    app: AppHandle,
    manager: State<'_, Arc<TabMuteManager>>,
) {
    manager.set_audio_playing(&label, audio_playing, app);
}

/// Tauri command to get all audio playing tabs
#[tauri::command]
pub fn get_audio_playing_tabs(
    manager: State<'_, Arc<TabMuteManager>>,
) -> Vec<TabMuteState> {
    manager.get_audio_playing_tabs()
}

/// Tauri command to get all tab mute states
#[tauri::command]
pub fn get_all_tab_mute_states(
    manager: State<'_, Arc<TabMuteManager>>,
) -> Vec<TabMuteState> {
    manager.get_all_tabs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_mute_manager_creation() {
        let manager = TabMuteManager::new();
        let tabs = manager.get_all_tabs();
        assert!(tabs.is_empty());
    }

    #[test]
    fn test_register_and_unregister_tab() {
        let manager = TabMuteManager::new();
        manager.register_tab("tab1".to_string());
        assert_eq!(manager.get_all_tabs().len(), 1);
        
        manager.unregister_tab("tab1");
        assert_eq!(manager.get_all_tabs().len(), 0);
    }

    #[test]
    fn test_set_and_get_mute_state() {
        let manager = TabMuteManager::new();
        manager.register_tab("tab1".to_string());
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        let is_muted = manager.get_mute_state("tab1");
        assert_eq!(is_muted, Some(false));
    }
}
