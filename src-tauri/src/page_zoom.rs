//! Page Zoom Optimization for Exodus Browser
//! 
//! This module provides intelligent page zoom and accessibility features.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Zoom level
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomLevel {
    /// Page URL
    pub url: String,
    /// Zoom level (0.5 to 3.0)
    pub level: f32,
    /// Timestamp
    pub timestamp: u64,
}

impl ZoomLevel {
    #[allow(dead_code)]
    pub fn new(url: String, level: f32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            url,
            level: level.clamp(0.5, 3.0),
            timestamp: now,
        }
    }
}

/// Zoom settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomSettings {
    /// Default zoom level
    pub default_level: f32,
    /// Enable auto-zoom based on content
    pub auto_zoom: bool,
    /// Minimum zoom level
    pub min_level: f32,
    /// Maximum zoom level
    pub max_level: f32,
    /// Zoom step size
    pub zoom_step: f32,
    /// Remember zoom per site
    pub remember_per_site: bool,
    /// Enable text-only zoom
    pub text_only: bool,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            default_level: 1.0,
            auto_zoom: false,
            min_level: 0.5,
            max_level: 3.0,
            zoom_step: 0.1,
            remember_per_site: true,
            text_only: false,
        }
    }
}

/// Page zoom manager
pub struct PageZoomManager {
    settings: Arc<Mutex<ZoomSettings>>,
    zoom_levels: Arc<Mutex<HashMap<String, f32>>>,
    storage_path: PathBuf,
}

impl PageZoomManager {
    /// Create a new page zoom manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(ZoomSettings::default())),
            zoom_levels: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Set zoom level for a page
    pub fn set_zoom_level(&self, url: String, level: f32) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let clamped_level = level.clamp(settings.min_level, settings.max_level);
        
        let mut zoom_levels = self.zoom_levels.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        zoom_levels.insert(url, clamped_level);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get zoom level for a page
    pub fn get_zoom_level(&self, url: &str) -> f32 {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if settings.remember_per_site {
            let zoom_levels = self.zoom_levels.lock()
                .unwrap_or_else(|_| panic!("Lock error"));
            
            if let Some(&level) = zoom_levels.get(url) {
                return level;
            }
        }
        
        settings.default_level
    }
    
    /// Reset zoom level for a page
    pub fn reset_zoom_level(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut zoom_levels = self.zoom_levels.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        zoom_levels.remove(url);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Zoom in
    pub fn zoom_in(&self, url: String) -> Result<f32, Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let current = self.get_zoom_level(&url);
        let new_level = (current + settings.zoom_step).min(settings.max_level);
        
        self.set_zoom_level(url, new_level)?;
        Ok(new_level)
    }
    
    /// Zoom out
    pub fn zoom_out(&self, url: String) -> Result<f32, Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let current = self.get_zoom_level(&url);
        let new_level = (current - settings.zoom_step).max(settings.min_level);
        
        self.set_zoom_level(url, new_level)?;
        Ok(new_level)
    }
    
    /// Reset zoom
    pub fn reset_zoom(&self, url: String) -> Result<f32, Box<dyn std::error::Error>> {
        self.reset_zoom_level(&url)?;
        Ok(self.get_zoom_level(&url))
    }
    
    /// Get zoom settings
    pub fn get_settings(&self) -> ZoomSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update zoom settings
    pub fn update_settings(&self, settings: ZoomSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get all zoom levels
    pub fn get_all_zoom_levels(&self) -> HashMap<String, f32> {
        let zoom_levels = self.zoom_levels.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        zoom_levels.clone()
    }
    
    /// Clear all zoom levels
    pub fn clear_all_zoom_levels(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut zoom_levels = self.zoom_levels.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        zoom_levels.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Auto-detect optimal zoom level based on content
    pub fn auto_detect_zoom(&self, _url: String, content_length: usize) -> f32 {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.auto_zoom {
            return settings.default_level;
        }
        
        // Simple heuristic: adjust zoom based on content length
        // In a real implementation, this would analyze the actual content
        let level: f32 = if content_length > 10000 {
            0.9 // Zoom out for long content
        } else if content_length < 1000 {
            1.1 // Zoom in for short content
        } else {
            1.0 // Default for medium content
        };
        
        level.clamp(settings.min_level, settings.max_level)
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("zoom_settings.json");
        let levels_path = self.storage_path.join("zoom_levels.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: ZoomSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if levels_path.exists() {
            let content = std::fs::read_to_string(&levels_path)?;
            let levels: HashMap<String, f32> = serde_json::from_str(&content)?;
            if let Ok(mut zl) = self.zoom_levels.lock() {
                *zl = levels;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("zoom_settings.json");
        let levels_path = self.storage_path.join("zoom_levels.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let levels = self.zoom_levels.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let levels_content = serde_json::to_string_pretty(&*levels)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&levels_path, levels_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Set zoom level for a page
#[tauri::command]
pub fn set_page_zoom(
    url: String,
    level: f32,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<(), String> {
    manager.set_zoom_level(url, level)
        .map_err(|e| format!("Failed to set zoom level: {}", e))
}

/// Get zoom level for a page
#[tauri::command]
pub fn get_page_zoom(
    url: String,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<f32, String> {
    Ok(manager.get_zoom_level(&url))
}

/// Reset zoom level for a page
#[tauri::command]
pub fn reset_page_zoom(
    url: String,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<(), String> {
    manager.reset_zoom_level(&url)
        .map_err(|e| format!("Failed to reset zoom level: {}", e))
}

/// Zoom in
#[tauri::command]
pub fn zoom_in(
    url: String,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<f32, String> {
    manager.zoom_in(url)
        .map_err(|e| format!("Failed to zoom in: {}", e))
}

/// Zoom out
#[tauri::command]
pub fn zoom_out(
    url: String,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<f32, String> {
    manager.zoom_out(url)
        .map_err(|e| format!("Failed to zoom out: {}", e))
}

/// Reset zoom
#[tauri::command]
pub fn reset_zoom(
    url: String,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<f32, String> {
    manager.reset_zoom(url)
        .map_err(|e| format!("Failed to reset zoom: {}", e))
}

/// Get zoom settings
#[tauri::command]
pub fn get_zoom_settings(
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<ZoomSettings, String> {
    Ok(manager.get_settings())
}

/// Update zoom settings
#[tauri::command]
pub fn update_zoom_settings(
    settings: ZoomSettings,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get all zoom levels
#[tauri::command]
pub fn get_all_zoom_levels(
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<HashMap<String, f32>, String> {
    Ok(manager.get_all_zoom_levels())
}

/// Clear all zoom levels
#[tauri::command]
pub fn clear_all_zoom_levels(
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<(), String> {
    manager.clear_all_zoom_levels()
        .map_err(|e| format!("Failed to clear zoom levels: {}", e))
}

/// Auto-detect zoom level
#[tauri::command]
pub fn auto_detect_zoom(
    url: String,
    content_length: usize,
    manager: State<'_, Arc<PageZoomManager>>,
) -> Result<f32, String> {
    Ok(manager.auto_detect_zoom(url, content_length))
}
