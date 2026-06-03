//! Reading Mode for Exodus Browser
//! 
//! This module provides a distraction-free reading experience.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Reading mode settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingModeSettings {
    /// Enable reading mode
    pub enabled: bool,
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: u32,
    /// Line height
    pub line_height: f32,
    /// Text color
    pub text_color: String,
    /// Background color
    pub background_color: String,
    /// Enable dark theme
    pub dark_theme: bool,
    /// Enable serif font
    pub serif_font: bool,
    /// Text alignment
    pub text_alignment: String,
    /// Max width
    pub max_width: u32,
}

impl Default for ReadingModeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            font_family: "Georgia".to_string(),
            font_size: 18,
            line_height: 1.6,
            text_color: "#333333".to_string(),
            background_color: "#ffffff".to_string(),
            dark_theme: false,
            serif_font: true,
            text_alignment: "justify".to_string(),
            max_width: 800,
        }
    }
}

/// Reading mode preset
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingModePreset {
    /// Preset name
    pub name: String,
    /// Settings
    pub settings: ReadingModeSettings,
}

impl ReadingModePreset {
    #[allow(dead_code)]
    pub fn new(name: String, settings: ReadingModeSettings) -> Self {
        Self { name, settings }
    }
}

/// Reading mode manager
pub struct ReadingModeManager {
    settings: Arc<Mutex<ReadingModeSettings>>,
    presets: Arc<Mutex<HashMap<String, ReadingModeSettings>>>,
    enabled_pages: Arc<Mutex<HashSet<String>>>,
    storage_path: PathBuf,
}

impl ReadingModeManager {
    /// Create a new reading mode manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(ReadingModeSettings::default())),
            presets: Arc::new(Mutex::new(HashMap::new())),
            enabled_pages: Arc::new(Mutex::new(HashSet::new())),
            storage_path,
        };
        
        manager.initialize_presets()?;
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Initialize default presets
    fn initialize_presets(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut presets = self.presets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Comfortable preset
        presets.insert("comfortable".to_string(), ReadingModeSettings {
            font_family: "Georgia".to_string(),
            font_size: 18,
            line_height: 1.6,
            text_color: "#333333".to_string(),
            background_color: "#ffffff".to_string(),
            dark_theme: false,
            serif_font: true,
            text_alignment: "justify".to_string(),
            max_width: 800,
            ..Default::default()
        });
        
        // Dark preset
        presets.insert("dark".to_string(), ReadingModeSettings {
            font_family: "Georgia".to_string(),
            font_size: 18,
            line_height: 1.6,
            text_color: "#e0e0e0".to_string(),
            background_color: "#1a1a1a".to_string(),
            dark_theme: true,
            serif_font: true,
            text_alignment: "justify".to_string(),
            max_width: 800,
            ..Default::default()
        });
        
        // Sepia preset
        presets.insert("sepia".to_string(), ReadingModeSettings {
            font_family: "Georgia".to_string(),
            font_size: 18,
            line_height: 1.6,
            text_color: "#5b4636".to_string(),
            background_color: "#f4ecd8".to_string(),
            dark_theme: false,
            serif_font: true,
            text_alignment: "justify".to_string(),
            max_width: 800,
            ..Default::default()
        });
        
        // Large preset
        presets.insert("large".to_string(), ReadingModeSettings {
            font_family: "Georgia".to_string(),
            font_size: 24,
            line_height: 1.8,
            text_color: "#333333".to_string(),
            background_color: "#ffffff".to_string(),
            dark_theme: false,
            serif_font: true,
            text_alignment: "justify".to_string(),
            max_width: 900,
            ..Default::default()
        });
        
        Ok(())
    }
    
    /// Enable reading mode for a page
    pub fn enable_for_page(&self, url: String) {
        let mut pages = self.enabled_pages.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pages.insert(url);
    }
    
    /// Disable reading mode for a page
    pub fn disable_for_page(&self, url: String) {
        let mut pages = self.enabled_pages.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pages.remove(&url);
    }
    
    /// Check if reading mode is enabled for a page
    pub fn is_enabled_for_page(&self, url: &str) -> bool {
        let pages = self.enabled_pages.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pages.contains(url)
    }
    
    /// Get all enabled pages
    pub fn get_enabled_pages(&self) -> Vec<String> {
        let pages = self.enabled_pages.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pages.iter().cloned().collect()
    }
    
    /// Get reading mode settings
    pub fn get_settings(&self) -> ReadingModeSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update reading mode settings
    pub fn update_settings(&self, settings: ReadingModeSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get preset
    pub fn get_preset(&self, name: &str) -> Option<ReadingModeSettings> {
        let presets = self.presets.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        presets.get(name).cloned()
    }
    
    /// Apply preset
    pub fn apply_preset(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(preset) = self.get_preset(name) {
            self.update_settings(preset)?;
            Ok(())
        } else {
            Err(format!("Preset '{}' not found", name).into())
        }
    }
    
    /// Get all presets
    pub fn get_all_presets(&self) -> Vec<String> {
        let presets = self.presets.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        presets.keys().cloned().collect()
    }
    
    /// Create custom preset
    pub fn create_preset(&self, name: String, settings: ReadingModeSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut presets = self.presets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        presets.insert(name, settings);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Delete preset
    pub fn delete_preset(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut presets = self.presets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        presets.remove(name);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Generate CSS for reading mode
    pub fn generate_css(&self) -> String {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        format!(
            "body {{ \
                font-family: '{}', serif; \
                font-size: {}px; \
                line-height: {}; \
                color: {}; \
                background-color: {}; \
                text-align: {}; \
                max-width: {}px; \
                margin: 0 auto; \
                padding: 40px 20px; \
            }} \
            p {{ margin-bottom: 1.5em; }} \
            h1, h2, h3 {{ color: {}; margin-top: 1.5em; }} \
            a {{ color: {}; text-decoration: underline; }} \
            img {{ max-width: 100%; height: auto; }}",
            settings.font_family,
            settings.font_size,
            settings.line_height,
            settings.text_color,
            settings.background_color,
            settings.text_alignment,
            settings.max_width,
            settings.text_color,
            if settings.dark_theme { "#4a9eff" } else { "#0066cc" }
        )
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("reading_mode_settings.json");
        let presets_path = self.storage_path.join("reading_mode_presets.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: ReadingModeSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if presets_path.exists() {
            let content = std::fs::read_to_string(&presets_path)?;
            let presets: HashMap<String, ReadingModeSettings> = serde_json::from_str(&content)?;
            if let Ok(mut p) = self.presets.lock() {
                *p = presets;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("reading_mode_settings.json");
        let presets_path = self.storage_path.join("reading_mode_presets.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let presets = self.presets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let presets_content = serde_json::to_string_pretty(&*presets)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&presets_path, presets_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Enable reading mode for a page
#[tauri::command]
pub fn enable_reading_mode(
    url: String,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<(), String> {
    manager.enable_for_page(url);
    Ok(())
}

/// Disable reading mode for a page
#[tauri::command]
pub fn disable_reading_mode(
    url: String,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<(), String> {
    manager.disable_for_page(url);
    Ok(())
}

/// Check if reading mode is enabled for a page
#[tauri::command]
pub fn is_reading_mode_enabled(
    url: String,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<bool, String> {
    Ok(manager.is_enabled_for_page(&url))
}

/// Get all pages with reading mode enabled
#[tauri::command]
pub fn get_reading_mode_pages(
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_enabled_pages())
}

/// Get reading mode settings
#[tauri::command]
pub fn get_reading_mode_settings(
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<ReadingModeSettings, String> {
    Ok(manager.get_settings())
}

/// Update reading mode settings
#[tauri::command]
pub fn update_reading_mode_settings(
    settings: ReadingModeSettings,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get reading mode preset
#[tauri::command]
pub fn get_reading_mode_preset(
    name: String,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<Option<ReadingModeSettings>, String> {
    Ok(manager.get_preset(&name))
}

/// Apply reading mode preset
#[tauri::command]
pub fn apply_reading_mode_preset(
    name: String,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<(), String> {
    manager.apply_preset(&name)
        .map_err(|e| format!("Failed to apply preset: {}", e))
}

/// Get all reading mode presets
#[tauri::command]
pub fn get_reading_mode_presets(
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_all_presets())
}

/// Create custom reading mode preset
#[tauri::command]
pub fn create_reading_mode_preset(
    name: String,
    settings: ReadingModeSettings,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<(), String> {
    manager.create_preset(name, settings)
        .map_err(|e| format!("Failed to create preset: {}", e))
}

/// Delete reading mode preset
#[tauri::command]
pub fn delete_reading_mode_preset(
    name: String,
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<(), String> {
    manager.delete_preset(&name)
        .map_err(|e| format!("Failed to delete preset: {}", e))
}

/// Generate CSS for reading mode
#[tauri::command]
pub fn generate_reading_mode_css(
    manager: State<'_, Arc<ReadingModeManager>>,
) -> Result<String, String> {
    Ok(manager.generate_css())
}
