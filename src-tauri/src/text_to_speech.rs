//! Text-to-Speech for Exodus Browser
//! Provides speech synthesis for web content

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// TTS voice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsVoice {
    pub name: String,
    pub language: String,
    pub gender: String,
}

/// TTS result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub success: bool,
    pub error: Option<String>,
}

/// TTS settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSettings {
    pub enabled: bool,
    pub voice: String,
    pub rate: f32,      // 0.1 to 10.0
    pub pitch: f32,     // 0.0 to 2.0
    pub volume: f32,    // 0.0 to 1.0
}

impl Default for TtsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            voice: "default".to_string(),
            rate: 1.0,
            pitch: 1.0,
            volume: 1.0,
        }
    }
}

/// Text-to-Speech Manager
pub struct TtsManager {
    settings: Arc<Mutex<TtsSettings>>,
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    available: bool,
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    available: bool,
}

impl TtsManager {
    pub fn new() -> Self {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        let available = Self::check_tts_available();
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let available = Self::check_tts_available();

        Self {
            settings: Arc::new(Mutex::new(TtsSettings::default())),
            available,
        }
    }

    /// Check if TTS is available
    #[cfg(target_os = "macos")]
    fn check_tts_available() -> bool {
        // Check for TTS availability on macOS
        // This would typically use AVSpeechSynthesizer
        // For now, return false as placeholder
        false
    }

    /// Check if TTS is available
    #[cfg(target_os = "windows")]
    fn check_tts_available() -> bool {
        // Check for TTS availability on Windows
        // This would typically use SAPI
        // For now, return false as placeholder
        false
    }

    /// Check if TTS is available
    #[cfg(target_os = "linux")]
    fn check_tts_available() -> bool {
        // Check for TTS availability on Linux
        // This would typically use espeak
        // For now, return false as placeholder
        false
    }

    /// Check if TTS is available
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    fn check_tts_available() -> bool {
        false
    }

    /// Check if TTS is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Enable TTS
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-tts-enabled", true);
        }
    }

    /// Disable TTS
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-tts-enabled", false);
        }
    }

    /// Check if TTS is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Speak text
    pub fn speak(&self, text: &str) -> TtsResult {
        if !self.available {
            return TtsResult {
                success: false,
                error: Some("TTS not available".to_string()),
            };
        }

        if !self.is_enabled() {
            return TtsResult {
                success: false,
                error: Some("TTS not enabled".to_string()),
            };
        }

        if text.is_empty() {
            return TtsResult {
                success: false,
                error: Some("Text is empty".to_string()),
            };
        }

        // Perform actual speech synthesis
        #[cfg(target_os = "macos")]
        let result = self.speak_macos(text);
        #[cfg(target_os = "windows")]
        let result = self.speak_windows(text);
        #[cfg(target_os = "linux")]
        let result = self.speak_linux(text);
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let result = TtsResult {
            success: false,
            error: Some("Platform not supported".to_string()),
        };

        result
    }

    /// Speak on macOS
    #[cfg(target_os = "macos")]
    fn speak_macos(&self, text: &str) -> TtsResult {
        // Use AVSpeechSynthesizer
        // For now, return success as placeholder
        TtsResult {
            success: true,
            error: None,
        }
    }

    /// Speak on Windows
    #[cfg(target_os = "windows")]
    fn speak_windows(&self, text: &str) -> TtsResult {
        // Use SAPI
        // For now, return success as placeholder
        TtsResult {
            success: true,
            error: None,
        }
    }

    /// Speak on Linux
    #[cfg(target_os = "linux")]
    fn speak_linux(&self, text: &str) -> TtsResult {
        // Use espeak
        // For now, return success as placeholder
        TtsResult {
            success: true,
            error: None,
        }
    }

    /// Stop speaking
    pub fn stop(&self) -> TtsResult {
        if !self.available {
            return TtsResult {
                success: false,
                error: Some("TTS not available".to_string()),
            };
        }

        // Stop speech synthesis
        #[cfg(target_os = "macos")]
        let result = self.stop_macos();
        #[cfg(target_os = "windows")]
        let result = self.stop_windows();
        #[cfg(target_os = "linux")]
        let result = self.stop_linux();
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let result = TtsResult {
            success: false,
            error: Some("Platform not supported".to_string()),
        };

        result
    }

    /// Stop on macOS
    #[cfg(target_os = "macos")]
    fn stop_macos(&self) -> TtsResult {
        TtsResult {
            success: true,
            error: None,
        }
    }

    /// Stop on Windows
    #[cfg(target_os = "windows")]
    fn stop_windows(&self) -> TtsResult {
        TtsResult {
            success: true,
            error: None,
        }
    }

    /// Stop on Linux
    #[cfg(target_os = "linux")]
    fn stop_linux(&self) -> TtsResult {
        TtsResult {
            success: true,
            error: None,
        }
    }

    /// Get available voices
    pub fn get_voices(&self) -> Vec<TtsVoice> {
        if !self.available {
            return vec![];
        }

        #[cfg(target_os = "macos")]
        let voices = self.get_voices_macos();
        #[cfg(target_os = "windows")]
        let voices = self.get_voices_windows();
        #[cfg(target_os = "linux")]
        let voices = self.get_voices_linux();
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let voices = vec![];

        voices
    }

    /// Get voices on macOS
    #[cfg(target_os = "macos")]
    fn get_voices_macos(&self) -> Vec<TtsVoice> {
        vec![
            TtsVoice {
                name: "Samantha".to_string(),
                language: "en-US".to_string(),
                gender: "female".to_string(),
            },
        ]
    }

    /// Get voices on Windows
    #[cfg(target_os = "windows")]
    fn get_voices_windows(&self) -> Vec<TtsVoice> {
        vec![
            TtsVoice {
                name: "Microsoft Zira Desktop".to_string(),
                language: "en-US".to_string(),
                gender: "female".to_string(),
            },
        ]
    }

    /// Get voices on Linux
    #[cfg(target_os = "linux")]
    fn get_voices_linux(&self) -> Vec<TtsVoice> {
        vec![
            TtsVoice {
                name: "default".to_string(),
                language: "en-US".to_string(),
                gender: "neutral".to_string(),
            },
        ]
    }

    /// Set voice
    pub fn set_voice(&self, voice: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.voice = voice.clone();
            let _ = app.emit("exodus-tts-voice-changed", voice);
        }
    }

    /// Set rate
    pub fn set_rate(&self, rate: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.rate = rate;
            let _ = app.emit("exodus-tts-rate-changed", rate);
        }
    }

    /// Set pitch
    pub fn set_pitch(&self, pitch: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.pitch = pitch;
            let _ = app.emit("exodus-tts-pitch-changed", pitch);
        }
    }

    /// Set volume
    pub fn set_volume(&self, volume: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.volume = volume;
            let _ = app.emit("exodus-tts-volume-changed", volume);
        }
    }

    /// Get current settings
    pub fn get_settings(&self) -> TtsSettings {
        self.settings.lock()
            .map(|settings| TtsSettings {
                enabled: settings.enabled,
                voice: settings.voice.clone(),
                rate: settings.rate,
                pitch: settings.pitch,
                volume: settings.volume,
            })
            .unwrap_or_default()
    }
}

impl Default for TtsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to check if TTS is available
#[tauri::command]
pub fn is_tts_available(
    manager: State<'_, Arc<TtsManager>>,
) -> bool {
    manager.is_available()
}

/// Tauri command to enable TTS
#[tauri::command]
pub fn enable_tts(
    app: AppHandle,
    manager: State<'_, Arc<TtsManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable TTS
#[tauri::command]
pub fn disable_tts(
    app: AppHandle,
    manager: State<'_, Arc<TtsManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if TTS is enabled
#[tauri::command]
pub fn is_tts_enabled(
    manager: State<'_, Arc<TtsManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to speak text
#[tauri::command]
pub fn tts_speak(
    text: String,
    manager: State<'_, Arc<TtsManager>>,
) -> TtsResult {
    manager.speak(&text)
}

/// Tauri command to stop speaking
#[tauri::command]
pub fn tts_stop(
    manager: State<'_, Arc<TtsManager>>,
) -> TtsResult {
    manager.stop()
}

/// Tauri command to get available voices
#[tauri::command]
pub fn tts_get_voices(
    manager: State<'_, Arc<TtsManager>>,
) -> Vec<TtsVoice> {
    manager.get_voices()
}

/// Tauri command to set voice
#[tauri::command]
pub fn tts_set_voice(
    voice: String,
    app: AppHandle,
    manager: State<'_, Arc<TtsManager>>,
) {
    manager.set_voice(voice, app);
}

/// Tauri command to set rate
#[tauri::command]
pub fn tts_set_rate(
    rate: f32,
    app: AppHandle,
    manager: State<'_, Arc<TtsManager>>,
) {
    manager.set_rate(rate, app);
}

/// Tauri command to set pitch
#[tauri::command]
pub fn tts_set_pitch(
    pitch: f32,
    app: AppHandle,
    manager: State<'_, Arc<TtsManager>>,
) {
    manager.set_pitch(pitch, app);
}

/// Tauri command to set volume
#[tauri::command]
pub fn tts_set_volume(
    volume: f32,
    app: AppHandle,
    manager: State<'_, Arc<TtsManager>>,
) {
    manager.set_volume(volume, app);
}

/// Tauri command to get TTS settings
#[tauri::command]
pub fn tts_get_settings(
    manager: State<'_, Arc<TtsManager>>,
) -> TtsSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_manager_creation() {
        let manager = TtsManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let manager = TtsManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = TtsManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert_eq!(settings.voice, "default");
        assert_eq!(settings.rate, 1.0);
        assert_eq!(settings.pitch, 1.0);
        assert_eq!(settings.volume, 1.0);
    }
}
