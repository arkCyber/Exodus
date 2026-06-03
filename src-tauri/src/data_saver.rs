//! Data Saver Mode for Exodus Browser
//! Reduces data usage by compressing images and blocking media

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Data saver settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSaverSettings {
    pub enabled: bool,
    pub block_images: bool,
    pub block_videos: bool,
    pub block_autoplay_media: bool,
    pub compress_images: bool,
    pub quality_level: f32, // 0.0 to 1.0
}

impl Default for DataSaverSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            block_images: false,
            block_videos: false,
            block_autoplay_media: true,
            compress_images: true,
            quality_level: 0.7,
        }
    }
}

/// Data Saver Manager
pub struct DataSaverManager {
    settings: Arc<Mutex<DataSaverSettings>>,
    bytes_saved: Arc<Mutex<u64>>,
}

impl DataSaverManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(DataSaverSettings::default())),
            bytes_saved: Arc::new(Mutex::new(0)),
        }
    }

    /// Enable data saver
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-data-saver-enabled", true);
        }
    }

    /// Disable data saver
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-data-saver-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Set block images
    pub fn set_block_images(&self, block: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.block_images = block;
            let _ = app.emit("exodus-data-saver-block-images-changed", block);
        }
    }

    /// Set block videos
    pub fn set_block_videos(&self, block: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.block_videos = block;
            let _ = app.emit("exodus-data-saver-block-videos-changed", block);
        }
    }

    /// Set block autoplay media
    pub fn set_block_autoplay_media(&self, block: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.block_autoplay_media = block;
            let _ = app.emit("exodus-data-saver-block-autoplay-changed", block);
        }
    }

    /// Set compress images
    pub fn set_compress_images(&self, compress: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.compress_images = compress;
            let _ = app.emit("exodus-data-saver-compress-images-changed", compress);
        }
    }

    /// Set quality level
    pub fn set_quality_level(&self, level: f32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.quality_level = level.clamp(0.0, 1.0);
            let _ = app.emit("exodus-data-saver-quality-changed", settings.quality_level);
        }
    }

    /// Record bytes saved
    pub fn record_bytes_saved(&self, bytes: u64, app: AppHandle) {
        if let Ok(mut bytes_saved) = self.bytes_saved.lock() {
            *bytes_saved += bytes;
            let _ = app.emit("exodus-data-saver-bytes-saved", bytes);
        }
    }

    /// Get bytes saved
    pub fn get_bytes_saved(&self) -> u64 {
        self.bytes_saved.lock()
            .map(|b| *b)
            .unwrap_or(0)
    }

    /// Reset bytes saved
    pub fn reset_bytes_saved(&self, app: AppHandle) {
        if let Ok(mut bytes_saved) = self.bytes_saved.lock() {
            *bytes_saved = 0;
            let _ = app.emit("exodus-data-saver-bytes-reset", ());
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> DataSaverSettings {
        self.settings.lock()
            .map(|s| DataSaverSettings {
                enabled: s.enabled,
                block_images: s.block_images,
                block_videos: s.block_videos,
                block_autoplay_media: s.block_autoplay_media,
                compress_images: s.compress_images,
                quality_level: s.quality_level,
            })
            .unwrap_or_default()
    }
}

impl Default for DataSaverManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable data saver
#[tauri::command]
pub fn enable_data_saver(
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable data saver
#[tauri::command]
pub fn disable_data_saver(
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_data_saver_enabled(
    manager: State<'_, Arc<DataSaverManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set block images
#[tauri::command]
pub fn set_data_saver_block_images(
    block: bool,
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.set_block_images(block, app);
}

/// Tauri command to set block videos
#[tauri::command]
pub fn set_data_saver_block_videos(
    block: bool,
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.set_block_videos(block, app);
}

/// Tauri command to set block autoplay media
#[tauri::command]
pub fn set_data_saver_block_autoplay(
    block: bool,
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.set_block_autoplay_media(block, app);
}

/// Tauri command to set compress images
#[tauri::command]
pub fn set_data_saver_compress_images(
    compress: bool,
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.set_compress_images(compress, app);
}

/// Tauri command to set quality level
#[tauri::command]
pub fn set_data_saver_quality_level(
    level: f32,
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.set_quality_level(level, app);
}

/// Tauri command to record bytes saved
#[tauri::command]
pub fn record_data_saver_bytes(
    bytes: u64,
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.record_bytes_saved(bytes, app);
}

/// Tauri command to get bytes saved
#[tauri::command]
pub fn get_data_saver_bytes_saved(
    manager: State<'_, Arc<DataSaverManager>>,
) -> u64 {
    manager.get_bytes_saved()
}

/// Tauri command to reset bytes saved
#[tauri::command]
pub fn reset_data_saver_bytes(
    app: AppHandle,
    manager: State<'_, Arc<DataSaverManager>>,
) {
    manager.reset_bytes_saved(app);
}

/// Tauri command to get data saver settings
#[tauri::command]
pub fn get_data_saver_settings(
    manager: State<'_, Arc<DataSaverManager>>,
) -> DataSaverSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_saver_manager_creation() {
        let manager = DataSaverManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_bytes_saved() {
        let manager = DataSaverManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert_eq!(manager.get_bytes_saved(), 0);
    }

    #[test]
    fn test_settings() {
        let manager = DataSaverManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert!(settings.compress_images);
    }
}
