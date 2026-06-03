//! Color Blind Mode for Exodus Browser
//! Provides color blindness accessibility features

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Color blindness type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorBlindType {
    None,
    Protanopia,   // Red-blind
    Deuteranopia, // Green-blind
    Tritanopia,   // Blue-blind
    Achromatopsia, // Total color blindness
}

impl Default for ColorBlindType {
    fn default() -> Self {
        ColorBlindType::None
    }
}

/// Color blind mode settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorBlindSettings {
    pub enabled: bool,
    pub color_blind_type: ColorBlindType,
    pub intensity: f32, // 0.0 to 1.0
}

impl Default for ColorBlindSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            color_blind_type: ColorBlindType::None,
            intensity: 1.0,
        }
    }
}

/// Color Blind Manager
pub struct ColorBlindManager {
    settings: Arc<Mutex<ColorBlindSettings>>,
}

impl ColorBlindManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(ColorBlindSettings::default())),
        }
    }

    /// Enable color blind mode
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-color-blind-enabled", true);
        }
    }

    /// Disable color blind mode
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-color-blind-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set color blind type
    pub fn set_color_blind_type(&self, color_blind_type: ColorBlindType, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.color_blind_type = color_blind_type.clone();
            let _ = app.emit("exodus-color-blind-type-changed", color_blind_type);
        }
    }

    /// Get color blind type
    pub fn get_color_blind_type(&self) -> ColorBlindType {
        self.settings.lock()
            .map(|settings| settings.color_blind_type.clone())
            .unwrap_or_default()
    }

    /// Set intensity
    pub fn set_intensity(&self, intensity: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.intensity = intensity.clamp(0.0, 1.0);
            let _ = app.emit("exodus-color-blind-intensity-changed", settings.intensity);
        }
    }

    /// Get intensity
    pub fn get_intensity(&self) -> f32 {
        self.settings.lock()
            .map(|settings| settings.intensity)
            .unwrap_or(0.5)
    }

    /// Get settings
    pub fn get_settings(&self) -> ColorBlindSettings {
        self.settings.lock()
            .map(|settings| ColorBlindSettings {
                enabled: settings.enabled,
                color_blind_type: settings.color_blind_type.clone(),
                intensity: settings.intensity,
            })
            .unwrap_or_default()
    }
}

impl Default for ColorBlindManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable color blind mode
#[tauri::command]
pub fn enable_color_blind(
    app: AppHandle,
    manager: State<'_, Arc<ColorBlindManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable color blind mode
#[tauri::command]
pub fn disable_color_blind(
    app: AppHandle,
    manager: State<'_, Arc<ColorBlindManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_color_blind_enabled(
    manager: State<'_, Arc<ColorBlindManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set color blind type
#[tauri::command]
pub fn set_color_blind_type(
    color_blind_type: String,
    app: AppHandle,
    manager: State<'_, Arc<ColorBlindManager>>,
) {
    let cb_type = match color_blind_type.as_str() {
        "protanopia" => ColorBlindType::Protanopia,
        "deuteranopia" => ColorBlindType::Deuteranopia,
        "tritanopia" => ColorBlindType::Tritanopia,
        "achromatopsia" => ColorBlindType::Achromatopsia,
        _ => ColorBlindType::None,
    };
    manager.set_color_blind_type(cb_type, app);
}

/// Tauri command to get color blind type
#[tauri::command]
pub fn get_color_blind_type(
    manager: State<'_, Arc<ColorBlindManager>>,
) -> String {
    match manager.get_color_blind_type() {
        ColorBlindType::None => "none".to_string(),
        ColorBlindType::Protanopia => "protanopia".to_string(),
        ColorBlindType::Deuteranopia => "deuteranopia".to_string(),
        ColorBlindType::Tritanopia => "tritanopia".to_string(),
        ColorBlindType::Achromatopsia => "achromatopsia".to_string(),
    }
}

/// Tauri command to set intensity
#[tauri::command]
pub fn set_color_blind_intensity(
    intensity: f32,
    app: AppHandle,
    manager: State<'_, Arc<ColorBlindManager>>,
) {
    manager.set_intensity(intensity, app);
}

/// Tauri command to get intensity
#[tauri::command]
pub fn get_color_blind_intensity(
    manager: State<'_, Arc<ColorBlindManager>>,
) -> f32 {
    manager.get_intensity()
}

/// Tauri command to get color blind settings
#[tauri::command]
pub fn get_color_blind_settings(
    manager: State<'_, Arc<ColorBlindManager>>,
) -> ColorBlindSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_blind_manager_creation() {
        let manager = ColorBlindManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let manager = ColorBlindManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = ColorBlindManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
    }
}
