//! Custom New Tab Page for Exodus Browser
//! 
//! This module provides custom new tab page configuration and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// New tab page layout
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NewTabPageLayout {
    Classic,
    Modern,
    Minimal,
    Grid,
    List,
}

impl NewTabPageLayout {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "classic" => NewTabPageLayout::Classic,
            "modern" => NewTabPageLayout::Modern,
            "minimal" => NewTabPageLayout::Minimal,
            "grid" => NewTabPageLayout::Grid,
            "list" => NewTabPageLayout::List,
            _ => NewTabPageLayout::Modern,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            NewTabPageLayout::Classic => "classic",
            NewTabPageLayout::Modern => "modern",
            NewTabPageLayout::Minimal => "minimal",
            NewTabPageLayout::Grid => "grid",
            NewTabPageLayout::List => "list",
        }
    }
}

/// New tab page widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTabPageWidget {
    /// Widget ID
    pub id: String,
    /// Widget type
    pub widget_type: String,
    /// Widget title
    pub title: String,
    /// Widget position
    pub position: u32,
    /// Widget configuration
    pub config: HashMap<String, String>,
    /// Is enabled
    pub enabled: bool,
}

impl NewTabPageWidget {
    pub fn new(widget_type: String, title: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            widget_type,
            title,
            position: 0,
            config: HashMap::new(),
            enabled: true,
        }
    }
}

/// New tab page settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTabPageSettings {
    /// Layout style
    pub layout: NewTabPageLayout,
    /// Show search bar
    pub show_search_bar: bool,
    /// Show bookmarks
    pub show_bookmarks: bool,
    /// Show recent history
    pub show_recent_history: bool,
    /// Show top sites
    pub show_top_sites: bool,
    /// Background image URL
    pub background_image: Option<String>,
    /// Wallpaper catalog id (Brave-style library; see `ntp_wallpapers`).
    #[serde(default = "default_ntp_wallpaper_id")]
    pub wallpaper_id: String,
    /// Background color
    pub background_color: String,
    /// Custom CSS
    pub custom_css: Option<String>,
    /// Custom HTML
    pub custom_html: Option<String>,
    /// Enable widgets
    pub enable_widgets: bool,
}

fn default_ntp_wallpaper_id() -> String {
    crate::ntp_wallpapers::default_wallpaper_id()
}

impl Default for NewTabPageSettings {
    fn default() -> Self {
        Self {
            layout: NewTabPageLayout::Modern,
            show_search_bar: true,
            show_bookmarks: true,
            show_recent_history: true,
            show_top_sites: true,
            background_image: None,
            wallpaper_id: default_ntp_wallpaper_id(),
            background_color: "#ffffff".to_string(),
            custom_css: None,
            custom_html: None,
            enable_widgets: false,
        }
    }
}

/// New tab page manager
pub struct NewTabPageManager {
    settings: Arc<Mutex<NewTabPageSettings>>,
    widgets: Arc<Mutex<Vec<NewTabPageWidget>>>,
    storage_path: PathBuf,
}

impl NewTabPageManager {
    /// Create a new new tab page manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(NewTabPageSettings::default())),
            widgets: Arc::new(Mutex::new(Vec::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Get new tab page settings
    pub fn get_settings(&self) -> NewTabPageSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update new tab page settings
    pub fn update_settings(&self, settings: NewTabPageSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set layout
    pub fn set_layout(&self, layout: NewTabPageLayout) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.layout = layout;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set background image
    pub fn set_background_image(&self, url: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.background_image = url;
        self.save_to_disk()?;
        Ok(())
    }

    /// Set wallpaper catalog id (synced with frontend `exodus-ntp-wallpaper-id`).
    pub fn set_wallpaper_id(&self, wallpaper_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        settings.wallpaper_id = wallpaper_id;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set background color
    pub fn set_background_color(&self, color: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.background_color = color;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set custom CSS
    pub fn set_custom_css(&self, css: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.custom_css = css;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set custom HTML
    pub fn set_custom_html(&self, html: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.custom_html = html;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add a widget
    pub fn add_widget(&self, widget_type: String, title: String) -> Result<String, Box<dyn std::error::Error>> {
        let mut widget = NewTabPageWidget::new(widget_type, title);
        widget.position = self.widgets.lock()
            .map(|w| w.len() as u32)
            .unwrap_or(0);
        
        let widget_id = widget.id.clone();
        
        let mut widgets = self.widgets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        widgets.push(widget);
        self.save_to_disk()?;
        Ok(widget_id)
    }
    
    /// Remove a widget
    pub fn remove_widget(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut widgets = self.widgets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        widgets.retain(|w| w.id != id);
        
        // Update positions
        for (i, widget) in widgets.iter_mut().enumerate() {
            widget.position = i as u32;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Update a widget
    pub fn update_widget(&self, id: String, title: Option<String>, config: Option<HashMap<String, String>>, enabled: Option<bool>) -> Result<(), Box<dyn std::error::Error>> {
        let mut widgets = self.widgets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(widget) = widgets.iter_mut().find(|w| w.id == id) {
            if let Some(title) = title {
                widget.title = title;
            }
            if let Some(config) = config {
                widget.config = config;
            }
            if let Some(enabled) = enabled {
                widget.enabled = enabled;
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Reorder widgets
    pub fn reorder_widgets(&self, widget_ids: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut widgets = self.widgets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let mut new_order: Vec<NewTabPageWidget> = Vec::new();
        
        for (i, widget_id) in widget_ids.iter().enumerate() {
            if let Some(widget) = widgets.iter().find(|w| &w.id == widget_id) {
                let mut widget = widget.clone();
                widget.position = i as u32;
                new_order.push(widget);
            }
        }
        
        *widgets = new_order;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get all widgets
    pub fn get_all_widgets(&self) -> Vec<NewTabPageWidget> {
        let widgets = self.widgets.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        widgets.clone()
    }
    
    /// Get enabled widgets
    pub fn get_enabled_widgets(&self) -> Vec<NewTabPageWidget> {
        let widgets = self.widgets.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        widgets.iter()
            .filter(|w| w.enabled)
            .cloned()
            .collect()
    }
    
    /// Reset to default settings
    pub fn reset_to_default(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *settings = NewTabPageSettings::default();
        
        let mut widgets = self.widgets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        widgets.clear();
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("new_tab_settings.json");
        let widgets_path = self.storage_path.join("new_tab_widgets.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: NewTabPageSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if widgets_path.exists() {
            let content = std::fs::read_to_string(&widgets_path)?;
            let widgets: Vec<NewTabPageWidget> = serde_json::from_str(&content)?;
            if let Ok(mut w) = self.widgets.lock() {
                *w = widgets;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("new_tab_settings.json");
        let widgets_path = self.storage_path.join("new_tab_widgets.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let widgets = self.widgets.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let widgets_content = serde_json::to_string_pretty(&*widgets)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&widgets_path, widgets_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get new tab page settings
#[tauri::command]
pub fn get_new_tab_settings(
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<NewTabPageSettings, String> {
    Ok(manager.get_settings())
}

/// Update new tab page settings
#[tauri::command]
pub fn update_new_tab_settings(
    settings: NewTabPageSettings,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Set layout
#[tauri::command]
pub fn set_new_tab_layout(
    layout: String,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    let l = NewTabPageLayout::from_str(&layout);
    manager.set_layout(l)
        .map_err(|e| format!("Failed to set layout: {}", e))
}

/// Set background image
#[tauri::command]
pub fn set_new_tab_background_image(
    url: Option<String>,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.set_background_image(url)
        .map_err(|e| format!("Failed to set background image: {}", e))
}

/// Set wallpaper catalog id for the new tab page.
#[tauri::command]
pub fn set_new_tab_wallpaper_id(
    id: String,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager
        .set_wallpaper_id(id)
        .map_err(|e| format!("Failed to set wallpaper id: {}", e))
}

/// Set background color
#[tauri::command]
pub fn set_new_tab_background_color(
    color: String,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.set_background_color(color)
        .map_err(|e| format!("Failed to set background color: {}", e))
}

/// Set custom CSS
#[tauri::command]
pub fn set_new_tab_custom_css(
    css: Option<String>,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.set_custom_css(css)
        .map_err(|e| format!("Failed to set custom CSS: {}", e))
}

/// Set custom HTML
#[tauri::command]
pub fn set_new_tab_custom_html(
    html: Option<String>,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.set_custom_html(html)
        .map_err(|e| format!("Failed to set custom HTML: {}", e))
}

/// Add a widget
#[tauri::command]
pub fn add_new_tab_widget(
    widget_type: String,
    title: String,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<String, String> {
    manager.add_widget(widget_type, title)
        .map_err(|e| format!("Failed to add widget: {}", e))
}

/// Remove a widget
#[tauri::command]
pub fn remove_new_tab_widget(
    id: String,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.remove_widget(&id)
        .map_err(|e| format!("Failed to remove widget: {}", e))
}

/// Update a widget
#[tauri::command]
pub fn update_new_tab_widget(
    id: String,
    title: Option<String>,
    config: Option<HashMap<String, String>>,
    enabled: Option<bool>,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.update_widget(id, title, config, enabled)
        .map_err(|e| format!("Failed to update widget: {}", e))
}

/// Reorder widgets
#[tauri::command]
pub fn reorder_new_tab_widgets(
    widget_ids: Vec<String>,
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.reorder_widgets(widget_ids)
        .map_err(|e| format!("Failed to reorder widgets: {}", e))
}

/// Get all widgets
#[tauri::command]
pub fn get_all_new_tab_widgets(
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<Vec<NewTabPageWidget>, String> {
    Ok(manager.get_all_widgets())
}

/// Get enabled widgets
#[tauri::command]
pub fn get_enabled_new_tab_widgets(
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<Vec<NewTabPageWidget>, String> {
    Ok(manager.get_enabled_widgets())
}

/// Reset to default settings
#[tauri::command]
pub fn reset_new_tab_to_default(
    manager: State<'_, Arc<NewTabPageManager>>,
) -> Result<(), String> {
    manager.reset_to_default()
        .map_err(|e| format!("Failed to reset to default: {}", e))
}
