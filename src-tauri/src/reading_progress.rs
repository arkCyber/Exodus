//! Reading Progress Tracking for Exodus Browser
//! Tracks scroll position and reading progress for web pages

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Reading progress entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub url: String,
    pub title: String,
    pub scroll_position: f64, // 0.0 to 1.0 (percentage)
    pub scroll_y: i32, // pixel position
    pub total_height: i32,
    pub last_read_time: i64, // timestamp
    pub is_completed: bool,
}

/// Reading progress settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgressSettings {
    pub enabled: bool,
    pub auto_save: bool,
    pub save_interval_seconds: u32,
    pub show_progress_indicator: bool,
    pub mark_completed_threshold: f64, // percentage to mark as completed
}

impl Default for ReadingProgressSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_save: true,
            save_interval_seconds: 5,
            show_progress_indicator: true,
            mark_completed_threshold: 0.95,
        }
    }
}

/// Reading Progress Manager
pub struct ReadingProgressManager {
    progress: Arc<Mutex<HashMap<String, ReadingProgress>>>,
    settings: Arc<Mutex<ReadingProgressSettings>>,
}

impl ReadingProgressManager {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(ReadingProgressSettings::default())),
        }
    }

    /// Update reading progress for a URL
    pub fn update_progress(&self, url: String, title: String, scroll_position: f64, scroll_y: i32, total_height: i32, app: AppHandle) {
        let now = chrono::Utc::now().timestamp();
        let is_completed = scroll_position >= self.get_settings().mark_completed_threshold;
        
        let entry = ReadingProgress {
            url: url.clone(),
            title,
            scroll_position,
            scroll_y,
            total_height,
            last_read_time: now,
            is_completed,
        };
        
        if let Ok(mut progress) = self.progress.lock() {
            progress.insert(url.clone(), entry.clone());
            let _ = app.emit("exodus-reading-progress-updated", entry);
        }
    }

    /// Get reading progress for a URL
    pub fn get_progress(&self, url: &str) -> Option<ReadingProgress> {
        self.progress.lock()
            .ok()?
            .get(url)
            .cloned()
    }

    /// Get all reading progress entries
    pub fn get_all_progress(&self) -> Vec<ReadingProgress> {
        self.progress.lock()
            .map(|progress| {
                let mut entries: Vec<ReadingProgress> = progress.values().cloned().collect();
                entries.sort_by(|a, b| b.last_read_time.cmp(&a.last_read_time));
                entries
            })
            .unwrap_or_default()
    }

    /// Get completed articles
    pub fn get_completed(&self) -> Vec<ReadingProgress> {
        self.progress.lock()
            .map(|progress| progress.values()
                .filter(|p| p.is_completed)
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    /// Get in-progress articles
    pub fn get_in_progress(&self) -> Vec<ReadingProgress> {
        self.progress.lock()
            .map(|progress| progress.values()
                .filter(|p| !p.is_completed && p.scroll_position > 0.0)
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    /// Mark article as completed
    pub fn mark_completed(&self, url: String, app: AppHandle) {
        if let Ok(mut progress) = self.progress.lock() {
            if let Some(entry) = progress.get_mut(&url) {
                entry.is_completed = true;
                entry.scroll_position = 1.0;
                let _ = app.emit("exodus-reading-progress-completed", url);
            }
        }
    }

    /// Reset progress for a URL
    pub fn reset_progress(&self, url: String, app: AppHandle) {
        if let Ok(mut progress) = self.progress.lock() {
            if let Some(entry) = progress.get_mut(&url) {
                entry.scroll_position = 0.0;
                entry.scroll_y = 0;
                entry.is_completed = false;
                let _ = app.emit("exodus-reading-progress-reset", url);
            }
        }
    }

    /// Delete progress for a URL
    pub fn delete_progress(&self, url: String, app: AppHandle) {
        if let Ok(mut progress) = self.progress.lock() {
            progress.remove(&url);
            let _ = app.emit("exodus-reading-progress-deleted", url);
        }
    }

    /// Clear all progress
    pub fn clear_all(&self, app: AppHandle) {
        if let Ok(mut progress) = self.progress.lock() {
            progress.clear();
            let _ = app.emit("exodus-reading-progress-cleared", ());
        }
    }

    /// Enable reading progress tracking
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-reading-progress-enabled", true);
        }
    }

    /// Disable reading progress tracking
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-reading-progress-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set auto-save
    pub fn set_auto_save(&self, enabled: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.auto_save = enabled;
            let _ = app.emit("exodus-reading-progress-auto-save-changed", enabled);
        }
    }

    /// Set save interval
    pub fn set_save_interval(&self, seconds: u32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.save_interval_seconds = seconds;
            let _ = app.emit("exodus-reading-progress-interval-changed", seconds);
        }
    }

    /// Set show progress indicator
    pub fn set_show_indicator(&self, show: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.show_progress_indicator = show;
            let _ = app.emit("exodus-reading-progress-indicator-changed", show);
        }
    }

    /// Set mark completed threshold
    pub fn set_completed_threshold(&self, threshold: f64, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.mark_completed_threshold = threshold.clamp(0.0, 1.0);
            let _ = app.emit("exodus-reading-progress-threshold-changed", settings.mark_completed_threshold);
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> ReadingProgressSettings {
        self.settings.lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }
}

impl Default for ReadingProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to update reading progress
#[tauri::command]
pub fn update_reading_progress(
    url: String,
    title: String,
    scroll_position: f64,
    scroll_y: i32,
    total_height: i32,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.update_progress(url, title, scroll_position, scroll_y, total_height, app);
}

/// Tauri command to get reading progress
#[tauri::command]
pub fn get_reading_progress(
    url: String,
    manager: State<'_, Arc<ReadingProgressManager>>,
) -> Option<ReadingProgress> {
    manager.get_progress(&url)
}

/// Tauri command to get all reading progress
#[tauri::command]
pub fn get_all_reading_progress(
    manager: State<'_, Arc<ReadingProgressManager>>,
) -> Vec<ReadingProgress> {
    manager.get_all_progress()
}

/// Tauri command to get completed articles
#[tauri::command]
pub fn get_completed_reading(
    manager: State<'_, Arc<ReadingProgressManager>>,
) -> Vec<ReadingProgress> {
    manager.get_completed()
}

/// Tauri command to get in-progress articles
#[tauri::command]
pub fn get_in_progress_reading(
    manager: State<'_, Arc<ReadingProgressManager>>,
) -> Vec<ReadingProgress> {
    manager.get_in_progress()
}

/// Tauri command to mark as completed
#[tauri::command]
pub fn mark_reading_completed(
    url: String,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.mark_completed(url, app);
}

/// Tauri command to reset progress
#[tauri::command]
pub fn reset_reading_progress(
    url: String,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.reset_progress(url, app);
}

/// Tauri command to delete progress
#[tauri::command]
pub fn delete_reading_progress(
    url: String,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.delete_progress(url, app);
}

/// Tauri command to clear all progress
#[tauri::command]
pub fn clear_reading_progress(
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.clear_all(app);
}

/// Tauri command to enable reading progress
#[tauri::command]
pub fn enable_reading_progress(
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable reading progress
#[tauri::command]
pub fn disable_reading_progress(
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_reading_progress_enabled(
    manager: State<'_, Arc<ReadingProgressManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set auto-save
#[tauri::command]
pub fn set_reading_progress_auto_save(
    enabled: bool,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.set_auto_save(enabled, app);
}

/// Tauri command to set save interval
#[tauri::command]
pub fn set_reading_progress_interval(
    seconds: u32,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.set_save_interval(seconds, app);
}

/// Tauri command to set show indicator
#[tauri::command]
pub fn set_reading_progress_indicator(
    show: bool,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.set_show_indicator(show, app);
}

/// Tauri command to set completed threshold
#[tauri::command]
pub fn set_reading_progress_threshold(
    threshold: f64,
    app: AppHandle,
    manager: State<'_, Arc<ReadingProgressManager>>,
) {
    manager.set_completed_threshold(threshold, app);
}

/// Tauri command to get reading progress settings
#[tauri::command]
pub fn get_reading_progress_settings(
    manager: State<'_, Arc<ReadingProgressManager>>,
) -> ReadingProgressSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_progress_manager_creation() {
        let manager = ReadingProgressManager::new();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_update_get_progress() {
        let manager = ReadingProgressManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(manager.get_all_progress().is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = ReadingProgressManager::new();
        
        let settings = manager.get_settings();
        assert!(settings.enabled);
        assert!(settings.auto_save);
    }
}
