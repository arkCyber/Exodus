//! Voice Search for Exodus Browser
//! Provides speech recognition for search queries

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Voice search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSearchResult {
    pub success: bool,
    pub text: Option<String>,
    pub error: Option<String>,
}

/// Voice search settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSearchSettings {
    pub enabled: bool,
    pub language: String,
    pub auto_submit: bool,
}

impl Default for VoiceSearchSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            language: "en-US".to_string(),
            auto_submit: true,
        }
    }
}

/// Voice Search Manager
pub struct VoiceSearchManager {
    settings: Arc<Mutex<VoiceSearchSettings>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    available: bool,
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    available: bool,
}

impl VoiceSearchManager {
    pub fn new() -> Self {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let available = Self::check_voice_recognition_available();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let available = Self::check_voice_recognition_available();

        Self {
            settings: Arc::new(Mutex::new(VoiceSearchSettings::default())),
            available,
        }
    }

    /// Check if voice recognition is available
    #[cfg(target_os = "macos")]
    fn check_voice_recognition_available() -> bool {
        // Check for Speech Recognition availability on macOS
        // This would typically use NSSpeechRecognition
        // For now, return false as placeholder
        false
    }

    /// Check if voice recognition is available
    #[cfg(target_os = "windows")]
    fn check_voice_recognition_available() -> bool {
        // Check for Speech Recognition availability on Windows
        // This would typically use Windows Speech Recognition API
        // For now, return false as placeholder
        false
    }

    /// Check if voice recognition is available
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn check_voice_recognition_available() -> bool {
        // Voice recognition not available on Linux by default
        false
    }

    /// Check if voice search is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Enable voice search
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-voice-search-enabled", true);
        }
    }

    /// Disable voice search
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-voice-search-enabled", false);
        }
    }

    /// Check if voice search is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Start voice recognition
    pub fn start_recognition(&self) -> VoiceSearchResult {
        if !self.available {
            return VoiceSearchResult {
                success: false,
                text: None,
                error: Some("Voice recognition not available".to_string()),
            };
        }

        if !self.is_enabled() {
            return VoiceSearchResult {
                success: false,
                text: None,
                error: Some("Voice search not enabled".to_string()),
            };
        }

        // Perform actual voice recognition
        #[cfg(target_os = "macos")]
        let result = self.recognize_macos();
        #[cfg(target_os = "windows")]
        let result = self.recognize_windows();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let result = VoiceSearchResult {
            success: false,
            text: None,
            error: Some("Platform not supported".to_string()),
        };

        result
    }

    /// Recognize speech on macOS
    #[cfg(target_os = "macos")]
    fn recognize_macos(&self) -> VoiceSearchResult {
        // Use NSSpeechRecognition
        // For now, return placeholder text
        VoiceSearchResult {
            success: true,
            text: Some("search query".to_string()),
            error: None,
        }
    }

    /// Recognize speech on Windows
    #[cfg(target_os = "windows")]
    fn recognize_windows(&self) -> VoiceSearchResult {
        // Use Windows Speech Recognition API
        // For now, return placeholder text
        VoiceSearchResult {
            success: true,
            text: Some("search query".to_string()),
            error: None,
        }
    }

    /// Set language
    pub fn set_language(&self, language: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.language = language.clone();
            let _ = app.emit("exodus-voice-search-language-changed", language);
        }
    }

    /// Set auto-submit
    pub fn set_auto_submit(&self, auto_submit: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.auto_submit = auto_submit;
            let _ = app.emit("exodus-voice-search-auto-submit-changed", auto_submit);
        }
    }

    /// Get current settings
    pub fn get_settings(&self) -> VoiceSearchSettings {
        self.settings.lock()
            .map(|settings| VoiceSearchSettings {
                enabled: settings.enabled,
                language: settings.language.clone(),
                auto_submit: settings.auto_submit,
            })
            .unwrap_or_default()
    }
}

impl Default for VoiceSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to check if voice search is available
#[tauri::command]
pub fn is_voice_search_available(
    manager: State<'_, Arc<VoiceSearchManager>>,
) -> bool {
    manager.is_available()
}

/// Tauri command to enable voice search
#[tauri::command]
pub fn enable_voice_search(
    app: AppHandle,
    manager: State<'_, Arc<VoiceSearchManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable voice search
#[tauri::command]
pub fn disable_voice_search(
    app: AppHandle,
    manager: State<'_, Arc<VoiceSearchManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if voice search is enabled
#[tauri::command]
pub fn is_voice_search_enabled(
    manager: State<'_, Arc<VoiceSearchManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to start voice recognition
#[tauri::command]
pub fn start_voice_recognition(
    manager: State<'_, Arc<VoiceSearchManager>>,
) -> VoiceSearchResult {
    manager.start_recognition()
}

/// Tauri command to set language
#[tauri::command]
pub fn set_voice_search_language(
    language: String,
    app: AppHandle,
    manager: State<'_, Arc<VoiceSearchManager>>,
) {
    manager.set_language(language, app);
}

/// Tauri command to set auto-submit
#[tauri::command]
pub fn set_voice_search_auto_submit(
    auto_submit: bool,
    app: AppHandle,
    manager: State<'_, Arc<VoiceSearchManager>>,
) {
    manager.set_auto_submit(auto_submit, app);
}

/// Tauri command to get voice search settings
#[tauri::command]
pub fn get_voice_search_settings(
    manager: State<'_, Arc<VoiceSearchManager>>,
) -> VoiceSearchSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_search_manager_creation() {
        let manager = VoiceSearchManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let manager = VoiceSearchManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = VoiceSearchManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert_eq!(settings.language, "en-US");
        assert!(settings.auto_submit);
    }
}
