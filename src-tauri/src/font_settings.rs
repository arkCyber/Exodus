//! Font Settings for Exodus Browser
//! 
//! This module provides font configuration and management capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Font family setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    /// Standard font
    pub standard: String,
    /// Serif font
    pub serif: String,
    /// Sans-serif font
    pub sans_serif: String,
    /// Monospace font
    pub monospace: String,
    /// Cursive font
    pub cursive: String,
    /// Fantasy font
    pub fantasy: String,
}

impl Default for FontFamily {
    fn default() -> Self {
        Self {
            standard: "Arial".to_string(),
            serif: "Times New Roman".to_string(),
            sans_serif: "Arial".to_string(),
            monospace: "Courier New".to_string(),
            cursive: "Comic Sans MS".to_string(),
            fantasy: "Impact".to_string(),
        }
    }
}

/// Font size setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSize {
    /// Minimum font size
    pub minimum: u32,
    /// Default font size
    pub default: u32,
    /// Default fixed font size
    pub default_fixed: u32,
}

impl Default for FontSize {
    fn default() -> Self {
        Self {
            minimum: 12,
            default: 16,
            default_fixed: 13,
        }
    }
}

/// Font smoothing setting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontSmoothing {
    None,
    Standard,
    Subpixel,
}

impl FontSmoothing {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "subpixel" => FontSmoothing::Subpixel,
            "standard" => FontSmoothing::Standard,
            _ => FontSmoothing::None,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            FontSmoothing::None => "none",
            FontSmoothing::Standard => "standard",
            FontSmoothing::Subpixel => "subpixel",
        }
    }
}

/// Font settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    /// Font families
    pub font_family: FontFamily,
    /// Font sizes
    pub font_size: FontSize,
    /// Font smoothing
    pub font_smoothing: FontSmoothing,
    /// Enable custom fonts
    pub enable_custom_fonts: bool,
    /// Allow font downloads
    pub allow_font_downloads: bool,
    /// Enable text direction
    pub enable_text_direction: bool,
    /// Default text direction
    pub default_text_direction: String,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            font_family: FontFamily::default(),
            font_size: FontSize::default(),
            font_smoothing: FontSmoothing::Standard,
            enable_custom_fonts: true,
            allow_font_downloads: true,
            enable_text_direction: false,
            default_text_direction: "ltr".to_string(),
        }
    }
}

/// Per-site font settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteFontSettings {
    /// Site domain
    pub domain: String,
    /// Font family override
    pub font_family: Option<String>,
    /// Font size override
    pub font_size: Option<u32>,
    /// Minimum font size override
    pub minimum_font_size: Option<u32>,
    /// Custom CSS
    pub custom_css: Option<String>,
    /// Created timestamp
    pub created_at: u64,
}

impl SiteFontSettings {
    #[allow(dead_code)]
    pub fn new(domain: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            domain,
            font_family: None,
            font_size: None,
            minimum_font_size: None,
            custom_css: None,
            created_at: now,
        }
    }
}

/// Font settings manager
pub struct FontSettingsManager {
    settings: Arc<Mutex<FontSettings>>,
    site_settings: Arc<Mutex<HashMap<String, SiteFontSettings>>>,
    storage_path: PathBuf,
}

impl FontSettingsManager {
    /// Create a new font settings manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(FontSettings::default())),
            site_settings: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Get font settings
    pub fn get_settings(&self) -> FontSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update font settings
    pub fn update_settings(&self, settings: FontSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add site font settings
    pub fn add_site_settings(&self, site_settings: SiteFontSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.site_settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.insert(site_settings.domain.clone(), site_settings);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove site font settings
    pub fn remove_site_settings(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.site_settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.remove(domain);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get site font settings
    pub fn get_site_settings(&self, domain: &str) -> Option<SiteFontSettings> {
        let settings = self.site_settings.lock().ok()?;
        settings.get(domain).cloned()
    }
    
    /// Get all site font settings
    pub fn get_all_site_settings(&self) -> Vec<SiteFontSettings> {
        let settings = self.site_settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.values().cloned().collect()
    }
    
    /// Update site font settings
    pub fn update_site_settings(&self, domain: String, site_settings: SiteFontSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.site_settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if settings.contains_key(&domain) {
            settings.insert(domain, site_settings);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Get effective font settings for a domain
    pub fn get_effective_settings(&self, domain: &str) -> FontSettings {
        let base_settings = self.get_settings();
        
        if let Some(site_settings) = self.get_site_settings(domain) {
            let mut effective = base_settings;
            
            if let Some(font_family) = site_settings.font_family {
                effective.font_family.standard = font_family;
            }
            
            if let Some(font_size) = site_settings.font_size {
                effective.font_size.default = font_size;
            }
            
            if let Some(minimum) = site_settings.minimum_font_size {
                effective.font_size.minimum = minimum;
            }
            
            effective
        } else {
            base_settings
        }
    }
    
    /// Get available system fonts
    pub fn get_system_fonts(&self) -> Vec<String> {
        // Return a list of common system fonts
        let mut fonts = vec![
            "Arial".to_string(),
            "Helvetica".to_string(),
            "Times New Roman".to_string(),
            "Courier New".to_string(),
            "Verdana".to_string(),
            "Georgia".to_string(),
            "Palatino".to_string(),
            "Garamond".to_string(),
            "Bookman".to_string(),
            "Comic Sans MS".to_string(),
            "Trebuchet MS".to_string(),
            "Arial Black".to_string(),
            "Impact".to_string(),
        ];
        
        fonts.sort();
        fonts
    }
    
    /// Reset to default settings
    pub fn reset_to_default(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.update_settings(FontSettings::default())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("font_settings.json");
        let site_settings_path = self.storage_path.join("site_font_settings.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: FontSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if site_settings_path.exists() {
            let content = std::fs::read_to_string(&site_settings_path)?;
            let site_settings: HashMap<String, SiteFontSettings> = serde_json::from_str(&content)?;
            if let Ok(mut ss) = self.site_settings.lock() {
                *ss = site_settings;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("font_settings.json");
        let site_settings_path = self.storage_path.join("site_font_settings.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let site_settings = self.site_settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let site_settings_content = serde_json::to_string_pretty(&*site_settings)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&site_settings_path, site_settings_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get font settings
#[tauri::command]
pub fn get_font_settings(
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<FontSettings, String> {
    Ok(manager.get_settings())
}

/// Update font settings
#[tauri::command]
pub fn update_font_settings(
    settings: FontSettings,
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Add site font settings
#[tauri::command]
pub fn add_site_font_settings(
    site_settings: SiteFontSettings,
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<(), String> {
    manager.add_site_settings(site_settings)
        .map_err(|e| format!("Failed to add site settings: {}", e))
}

/// Remove site font settings
#[tauri::command]
pub fn remove_site_font_settings(
    domain: String,
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<(), String> {
    manager.remove_site_settings(&domain)
        .map_err(|e| format!("Failed to remove site settings: {}", e))
}

/// Get site font settings
#[tauri::command]
pub fn get_site_font_settings(
    domain: String,
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<Option<SiteFontSettings>, String> {
    Ok(manager.get_site_settings(&domain))
}

/// Get all site font settings
#[tauri::command]
pub fn get_all_site_font_settings(
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<Vec<SiteFontSettings>, String> {
    Ok(manager.get_all_site_settings())
}

/// Update site font settings
#[tauri::command]
pub fn update_site_font_settings(
    domain: String,
    site_settings: SiteFontSettings,
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<(), String> {
    manager.update_site_settings(domain, site_settings)
        .map_err(|e| format!("Failed to update site settings: {}", e))
}

/// Get effective font settings for domain
#[tauri::command]
pub fn get_effective_font_settings(
    domain: String,
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<FontSettings, String> {
    Ok(manager.get_effective_settings(&domain))
}

/// Get system fonts
#[tauri::command]
pub fn get_system_fonts(
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_system_fonts())
}

/// Reset font settings to default
#[tauri::command]
pub fn reset_font_settings(
    manager: State<'_, Arc<FontSettingsManager>>,
) -> Result<(), String> {
    manager.reset_to_default()
        .map_err(|e| format!("Failed to reset settings: {}", e))
}
