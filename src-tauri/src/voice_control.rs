//! Voice Control for Exodus Browser
//! Provides voice command navigation and control

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Voice command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub id: String,
    pub phrase: String,
    pub action: String,
    pub enabled: bool,
}

/// Voice control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceControlSettings {
    pub enabled: bool,
    pub continuous_listening: bool,
    pub wake_word: String,
    pub confidence_threshold: f32,
}

impl Default for VoiceControlSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            continuous_listening: false,
            wake_word: "Exodus".to_string(),
            confidence_threshold: 0.7,
        }
    }
}

/// Voice Control Manager
pub struct VoiceControlManager {
    commands: Arc<Mutex<Vec<VoiceCommand>>>,
    settings: Arc<Mutex<VoiceControlSettings>>,
    is_listening: Arc<Mutex<bool>>,
}

impl VoiceControlManager {
    pub fn new() -> Self {
        let manager = Self {
            commands: Arc::new(Mutex::new(vec![])),
            settings: Arc::new(Mutex::new(VoiceControlSettings::default())),
            is_listening: Arc::new(Mutex::new(false)),
        };
        
        // Add default commands
        manager.add_default_commands();
        manager
    }

    fn add_default_commands(&self) {
        let default_commands = vec![
            VoiceCommand {
                id: "cmd-new-tab".to_string(),
                phrase: "new tab".to_string(),
                action: "browser_create_tab".to_string(),
                enabled: true,
            },
            VoiceCommand {
                id: "cmd-close-tab".to_string(),
                phrase: "close tab".to_string(),
                action: "browser_close_tab".to_string(),
                enabled: true,
            },
            VoiceCommand {
                id: "cmd-go-back".to_string(),
                phrase: "go back".to_string(),
                action: "browser_go_back".to_string(),
                enabled: true,
            },
            VoiceCommand {
                id: "cmd-go-forward".to_string(),
                phrase: "go forward".to_string(),
                action: "browser_go_forward".to_string(),
                enabled: true,
            },
            VoiceCommand {
                id: "cmd-refresh".to_string(),
                phrase: "refresh".to_string(),
                action: "browser_reload".to_string(),
                enabled: true,
            },
        ];
        
        if let Ok(mut commands) = self.commands.lock() {
            *commands = default_commands;
        }
    }

    /// Start listening for voice commands
    pub fn start_listening(&self, app: AppHandle) {
        if let Ok(mut is_listening) = self.is_listening.lock() {
            *is_listening = true;
            let _ = app.emit("exodus-voice-control-listening", true);
        }
    }

    /// Stop listening for voice commands
    pub fn stop_listening(&self, app: AppHandle) {
        if let Ok(mut is_listening) = self.is_listening.lock() {
            *is_listening = false;
            let _ = app.emit("exodus-voice-control-listening", false);
        }
    }

    /// Check if listening
    pub fn is_listening(&self) -> bool {
        self.is_listening.lock()
            .map(|l| *l)
            .unwrap_or(false)
    }

    /// Process voice command
    pub fn process_command(&self, text: String, app: AppHandle) -> Option<VoiceCommand> {
        if let Ok(commands) = self.commands.lock() {
            for cmd in commands.iter() {
                if cmd.enabled && text.to_lowercase().contains(&cmd.phrase.to_lowercase()) {
                    let _ = app.emit("exodus-voice-command-recognized", cmd.clone());
                    return Some(cmd.clone());
                }
            }
        }
        None
    }

    /// Add custom command
    pub fn add_command(&self, phrase: String, action: String, app: AppHandle) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let command = VoiceCommand {
            id: id.clone(),
            phrase,
            action,
            enabled: true,
        };
        
        if let Ok(mut commands) = self.commands.lock() {
            commands.push(command.clone());
            let _ = app.emit("exodus-voice-command-added", command);
        }
        id
    }

    /// Remove command
    pub fn remove_command(&self, id: String, app: AppHandle) {
        if let Ok(mut commands) = self.commands.lock() {
            commands.retain(|c| c.id != id);
            let _ = app.emit("exodus-voice-command-removed", id);
        }
    }

    /// Enable command
    pub fn enable_command(&self, id: String, app: AppHandle) {
        if let Ok(mut commands) = self.commands.lock() {
            if let Some(cmd) = commands.iter_mut().find(|c| c.id == id) {
                cmd.enabled = true;
                let _ = app.emit("exodus-voice-command-enabled", id);
            }
        }
    }

    /// Disable command
    pub fn disable_command(&self, id: String, app: AppHandle) {
        if let Ok(mut commands) = self.commands.lock() {
            if let Some(cmd) = commands.iter_mut().find(|c| c.id == id) {
                cmd.enabled = false;
                let _ = app.emit("exodus-voice-command-disabled", id);
            }
        }
    }

    /// Get all commands
    pub fn get_commands(&self) -> Vec<VoiceCommand> {
        self.commands.lock()
            .map(|c| c.clone())
            .unwrap_or_default()
    }

    /// Enable voice control
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-voice-control-enabled", true);
        }
    }

    /// Disable voice control
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            self.stop_listening(app.clone());
            let _ = app.emit("exodus-voice-control-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Set wake word
    pub fn set_wake_word(&self, wake_word: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.wake_word = wake_word.clone();
            let _ = app.emit("exodus-voice-control-wake-word-changed", wake_word);
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> VoiceControlSettings {
        self.settings.lock()
            .map(|s| VoiceControlSettings {
                enabled: s.enabled,
                continuous_listening: s.continuous_listening,
                wake_word: s.wake_word.clone(),
                confidence_threshold: s.confidence_threshold,
            })
            .unwrap_or_default()
    }
}

impl Default for VoiceControlManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to start listening
#[tauri::command]
pub fn start_voice_listening(
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.start_listening(app);
}

/// Tauri command to stop listening
#[tauri::command]
pub fn stop_voice_listening(
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.stop_listening(app);
}

/// Tauri command to check if listening
#[tauri::command]
pub fn is_voice_listening(
    manager: State<'_, Arc<VoiceControlManager>>,
) -> bool {
    manager.is_listening()
}

/// Tauri command to process voice command
#[tauri::command]
pub fn process_voice_command(
    text: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) -> Option<VoiceCommand> {
    manager.process_command(text, app)
}

/// Tauri command to add command
#[tauri::command]
pub fn add_voice_command(
    phrase: String,
    action: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) -> String {
    manager.add_command(phrase, action, app)
}

/// Tauri command to remove command
#[tauri::command]
pub fn remove_voice_command(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.remove_command(id, app);
}

/// Tauri command to enable command
#[tauri::command]
pub fn enable_voice_command(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.enable_command(id, app);
}

/// Tauri command to disable command
#[tauri::command]
pub fn disable_voice_command(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.disable_command(id, app);
}

/// Tauri command to get commands
#[tauri::command]
pub fn get_voice_commands(
    manager: State<'_, Arc<VoiceControlManager>>,
) -> Vec<VoiceCommand> {
    manager.get_commands()
}

/// Tauri command to enable voice control
#[tauri::command]
pub fn enable_voice_control(
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable voice control
#[tauri::command]
pub fn disable_voice_control(
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_voice_control_enabled(
    manager: State<'_, Arc<VoiceControlManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set wake word
#[tauri::command]
pub fn set_voice_control_wake_word(
    wake_word: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceControlManager>>,
) {
    manager.set_wake_word(wake_word, app);
}

/// Tauri command to get voice control settings
#[tauri::command]
pub fn get_voice_control_settings(
    manager: State<'_, Arc<VoiceControlManager>>,
) -> VoiceControlSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_control_manager_creation() {
        let manager = VoiceControlManager::new();
        assert!(!manager.is_enabled());
        assert!(!manager.is_listening());
    }

    #[test]
    fn test_commands() {
        let manager = VoiceControlManager::new();
        
        let commands = manager.get_commands();
        assert!(!commands.is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = VoiceControlManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert_eq!(settings.wake_word, "Exodus");
    }
}
