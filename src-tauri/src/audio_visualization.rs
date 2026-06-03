//! Audio Visualization for Exodus Browser
//! Provides audio visualization features for media playback

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Visualization type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    None,
    Bars,
    Waveform,
    Circular,
    Spectrum,
}

impl Default for VisualizationType {
    fn default() -> Self {
        Self::None
    }
}

/// Visualization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVisualizationSettings {
    pub enabled: bool,
    pub visualization_type: VisualizationType,
    pub sensitivity: f32, // 0.0 to 1.0
    pub smoothing: f32, // 0.0 to 1.0
    pub color_scheme: String,
}

impl Default for AudioVisualizationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            visualization_type: VisualizationType::Bars,
            sensitivity: 0.7,
            smoothing: 0.3,
            color_scheme: "rainbow".to_string(),
        }
    }
}

/// Audio Visualization Manager
pub struct AudioVisualizationManager {
    settings: Arc<Mutex<AudioVisualizationSettings>>,
}

impl AudioVisualizationManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(AudioVisualizationSettings::default())),
        }
    }

    /// Enable audio visualization
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-audio-visualization-enabled", true);
        }
    }

    /// Disable audio visualization
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-audio-visualization-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set visualization type
    pub fn set_visualization_type(&self, viz_type: VisualizationType, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.visualization_type = viz_type.clone();
            let _ = app.emit("exodus-visualization-type-changed", viz_type);
        }
    }

    /// Get visualization type
    pub fn get_visualization_type(&self) -> VisualizationType {
        self.settings.lock()
            .map(|settings| settings.visualization_type.clone())
            .unwrap_or_default()
    }

    /// Set sensitivity
    pub fn set_sensitivity(&self, sensitivity: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.sensitivity = sensitivity.clamp(0.0, 1.0);
            let _ = app.emit("exodus-visualization-sensitivity-changed", settings.sensitivity);
        }
    }

    /// Get sensitivity
    pub fn get_sensitivity(&self) -> f32 {
        self.settings.lock()
            .map(|settings| settings.sensitivity)
            .unwrap_or(0.5)
    }

    /// Set smoothing
    pub fn set_smoothing(&self, smoothing: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.smoothing = smoothing.clamp(0.0, 1.0);
            let _ = app.emit("exodus-visualization-smoothing-changed", settings.smoothing);
        }
    }

    /// Get smoothing
    pub fn get_smoothing(&self) -> f32 {
        self.settings.lock()
            .map(|settings| settings.smoothing)
            .unwrap_or(0.5)
    }

    /// Set color scheme
    pub fn set_color_scheme(&self, scheme: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.color_scheme = scheme.clone();
            let _ = app.emit("exodus-visualization-color-changed", scheme);
        }
    }

    /// Get color scheme
    pub fn get_color_scheme(&self) -> String {
        self.settings.lock()
            .map(|settings| settings.color_scheme.clone())
            .unwrap_or_default()
    }

    /// Get settings
    pub fn get_settings(&self) -> AudioVisualizationSettings {
        self.settings.lock()
            .map(|settings| AudioVisualizationSettings {
                enabled: settings.enabled,
                visualization_type: settings.visualization_type.clone(),
                sensitivity: settings.sensitivity,
                smoothing: settings.smoothing,
                color_scheme: settings.color_scheme.clone(),
            })
            .unwrap_or_default()
    }
}

impl Default for AudioVisualizationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable audio visualization
#[tauri::command]
pub fn enable_audio_visualization(
    app: AppHandle,
    manager: State<'_, Arc<AudioVisualizationManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable audio visualization
#[tauri::command]
pub fn disable_audio_visualization(
    app: AppHandle,
    manager: State<'_, Arc<AudioVisualizationManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_audio_visualization_enabled(
    manager: State<'_, Arc<AudioVisualizationManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set visualization type
#[tauri::command]
pub fn set_visualization_type(
    viz_type: String,
    app: AppHandle,
    manager: State<'_, Arc<AudioVisualizationManager>>,
) {
    let vtype = match viz_type.as_str() {
        "bars" => VisualizationType::Bars,
        "waveform" => VisualizationType::Waveform,
        "circular" => VisualizationType::Circular,
        "spectrum" => VisualizationType::Spectrum,
        _ => VisualizationType::None,
    };
    manager.set_visualization_type(vtype, app);
}

/// Tauri command to get visualization type
#[tauri::command]
pub fn get_visualization_type(
    manager: State<'_, Arc<AudioVisualizationManager>>,
) -> String {
    match manager.get_visualization_type() {
        VisualizationType::None => "none".to_string(),
        VisualizationType::Bars => "bars".to_string(),
        VisualizationType::Waveform => "waveform".to_string(),
        VisualizationType::Circular => "circular".to_string(),
        VisualizationType::Spectrum => "spectrum".to_string(),
    }
}

/// Tauri command to set sensitivity
#[tauri::command]
pub fn set_visualization_sensitivity(
    sensitivity: f32,
    app: AppHandle,
    manager: State<'_, Arc<AudioVisualizationManager>>,
) {
    manager.set_sensitivity(sensitivity, app);
}

/// Tauri command to get sensitivity
#[tauri::command]
pub fn get_visualization_sensitivity(
    manager: State<'_, Arc<AudioVisualizationManager>>,
) -> f32 {
    manager.get_sensitivity()
}

/// Tauri command to set smoothing
#[tauri::command]
pub fn set_visualization_smoothing(
    smoothing: f32,
    app: AppHandle,
    manager: State<'_, Arc<AudioVisualizationManager>>,
) {
    manager.set_smoothing(smoothing, app);
}

/// Tauri command to get smoothing
#[tauri::command]
pub fn get_visualization_smoothing(
    manager: State<'_, Arc<AudioVisualizationManager>>,
) -> f32 {
    manager.get_smoothing()
}

/// Tauri command to set color scheme
#[tauri::command]
pub fn set_visualization_color_scheme(
    scheme: String,
    app: AppHandle,
    manager: State<'_, Arc<AudioVisualizationManager>>,
) {
    manager.set_color_scheme(scheme, app);
}

/// Tauri command to get color scheme
#[tauri::command]
pub fn get_visualization_color_scheme(
    manager: State<'_, Arc<AudioVisualizationManager>>,
) -> String {
    manager.get_color_scheme()
}

/// Tauri command to get audio visualization settings
#[tauri::command]
pub fn get_audio_visualization_settings(
    manager: State<'_, Arc<AudioVisualizationManager>>,
) -> AudioVisualizationSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_visualization_manager_creation() {
        let manager = AudioVisualizationManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = AudioVisualizationManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
    }
}
