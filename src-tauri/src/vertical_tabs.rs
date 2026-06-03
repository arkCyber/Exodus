//! Vertical Tabs for Exodus Browser
//! 
//! This module provides vertical tab layout and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Tab position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TabPosition {
    Left,
    Right,
}

impl TabPosition {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "left" => TabPosition::Left,
            "right" => TabPosition::Right,
            _ => TabPosition::Left,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            TabPosition::Left => "left",
            TabPosition::Right => "right",
        }
    }
}

/// Tab width mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TabWidthMode {
    Fixed,
    Auto,
    Compact,
}

impl TabWidthMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fixed" => TabWidthMode::Fixed,
            "auto" => TabWidthMode::Auto,
            "compact" => TabWidthMode::Compact,
            _ => TabWidthMode::Auto,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            TabWidthMode::Fixed => "fixed",
            TabWidthMode::Auto => "auto",
            TabWidthMode::Compact => "compact",
        }
    }
}

/// Vertical tab settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalTabSettings {
    /// Enable vertical tabs
    pub enabled: bool,
    /// Tab position
    pub position: TabPosition,
    /// Tab width mode
    pub width_mode: TabWidthMode,
    /// Fixed width in pixels
    pub fixed_width: u32,
    /// Show tab icons
    pub show_icons: bool,
    /// Show tab titles
    pub show_titles: bool,
    /// Show close buttons
    pub show_close_buttons: bool,
    /// Collapse inactive tabs
    pub collapse_inactive: bool,
    /// Tab spacing
    pub tab_spacing: u32,
}

impl Default for VerticalTabSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            position: TabPosition::Left,
            width_mode: TabWidthMode::Auto,
            fixed_width: 250,
            show_icons: true,
            show_titles: true,
            show_close_buttons: true,
            collapse_inactive: false,
            tab_spacing: 4,
        }
    }
}

/// Vertical tab state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalTabState {
    /// Tab ID
    pub tab_id: String,
    /// Is expanded
    pub expanded: bool,
    /// Is pinned
    pub pinned: bool,
    /// Custom width (if fixed mode)
    pub custom_width: Option<u32>,
    /// Last active timestamp
    pub last_active: u64,
}

impl VerticalTabState {
    pub fn new(tab_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            tab_id,
            expanded: true,
            pinned: false,
            custom_width: None,
            last_active: now,
        }
    }
    
    #[allow(dead_code)]
    pub fn set_active(&mut self) {
        self.last_active = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }
}

/// Vertical tabs manager
pub struct VerticalTabsManager {
    settings: Arc<Mutex<VerticalTabSettings>>,
    tab_states: Arc<Mutex<HashMap<String, VerticalTabState>>>,
    storage_path: PathBuf,
}

impl VerticalTabsManager {
    /// Create a new vertical tabs manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(VerticalTabSettings::default())),
            tab_states: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Enable vertical tabs
    pub fn enable(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.enabled = true;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Disable vertical tabs
    pub fn disable(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.enabled = false;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set tab position
    pub fn set_position(&self, position: TabPosition) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.position = position;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set tab width mode
    pub fn set_width_mode(&self, width_mode: TabWidthMode) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.width_mode = width_mode;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set fixed width
    pub fn set_fixed_width(&self, width: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        settings.fixed_width = width;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add or update tab state
    pub fn update_tab_state(&self, tab_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut tab_states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if !tab_states.contains_key(&tab_id) {
            tab_states.insert(tab_id.clone(), VerticalTabState::new(tab_id.clone()));
        } else {
            tab_states.insert(tab_id.clone(), VerticalTabState::new(tab_id));
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove tab state
    pub fn remove_tab_state(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tab_states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        tab_states.remove(tab_id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Expand tab
    pub fn expand_tab(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tab_states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(state) = tab_states.get_mut(tab_id) {
            state.expanded = true;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Collapse tab
    pub fn collapse_tab(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tab_states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(state) = tab_states.get_mut(tab_id) {
            state.expanded = false;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get tab state
    pub fn get_tab_state(&self, tab_id: &str) -> Option<VerticalTabState> {
        let tab_states = self.tab_states.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        tab_states.get(tab_id).cloned()
    }
    
    /// Get all tab states
    pub fn get_all_tab_states(&self) -> Vec<VerticalTabState> {
        let tab_states = self.tab_states.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        tab_states.values().cloned().collect()
    }
    
    /// Get vertical tab settings
    pub fn get_settings(&self) -> VerticalTabSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update vertical tab settings
    pub fn update_settings(&self, settings: VerticalTabSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Collapse all inactive tabs
    pub fn collapse_inactive_tabs(&self, threshold_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let mut tab_states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for state in tab_states.values_mut() {
            if now - state.last_active > threshold_seconds {
                state.expanded = false;
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Expand all tabs
    pub fn expand_all_tabs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tab_states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for state in tab_states.values_mut() {
            state.expanded = true;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("vertical_tabs_settings.json");
        let states_path = self.storage_path.join("vertical_tab_states.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: VerticalTabSettings = serde_json::from_str(&content)?;
            if let Ok(mut settings_guard) = self.settings.lock() {
                *settings_guard = settings;
            }
        }
        
        if states_path.exists() {
            let content = std::fs::read_to_string(&states_path)?;
            let states: HashMap<String, VerticalTabState> = serde_json::from_str(&content)?;
            if let Ok(mut states_guard) = self.tab_states.lock() {
                *states_guard = states;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("vertical_tabs_settings.json");
        let states_path = self.storage_path.join("vertical_tab_states.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let states = self.tab_states.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let states_content = serde_json::to_string_pretty(&*states)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&states_path, states_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Enable vertical tabs
#[tauri::command]
pub fn enable_vertical_tabs(
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.enable()
        .map_err(|e| format!("Failed to enable vertical tabs: {}", e))
}

/// Disable vertical tabs
#[tauri::command]
pub fn disable_vertical_tabs(
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.disable()
        .map_err(|e| format!("Failed to disable vertical tabs: {}", e))
}

/// Set tab position
#[tauri::command]
pub fn set_vertical_tab_position(
    position: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    let pos = TabPosition::from_str(&position);
    manager.set_position(pos)
        .map_err(|e| format!("Failed to set position: {}", e))
}

/// Set tab width mode
#[tauri::command]
pub fn set_vertical_tab_width_mode(
    width_mode: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    let mode = TabWidthMode::from_str(&width_mode);
    manager.set_width_mode(mode)
        .map_err(|e| format!("Failed to set width mode: {}", e))
}

/// Set fixed width
#[tauri::command]
pub fn set_vertical_tab_fixed_width(
    width: u32,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.set_fixed_width(width)
        .map_err(|e| format!("Failed to set fixed width: {}", e))
}

/// Update tab state
#[tauri::command]
pub fn update_vertical_tab_state(
    tab_id: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.update_tab_state(tab_id)
        .map_err(|e| format!("Failed to update tab state: {}", e))
}

/// Remove tab state
#[tauri::command]
pub fn remove_vertical_tab_state(
    tab_id: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.remove_tab_state(&tab_id)
        .map_err(|e| format!("Failed to remove tab state: {}", e))
}

/// Expand tab
#[tauri::command]
pub fn expand_vertical_tab(
    tab_id: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.expand_tab(&tab_id)
        .map_err(|e| format!("Failed to expand tab: {}", e))
}

/// Collapse tab
#[tauri::command]
pub fn collapse_vertical_tab(
    tab_id: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.collapse_tab(&tab_id)
        .map_err(|e| format!("Failed to collapse tab: {}", e))
}

/// Get tab state
#[tauri::command]
pub fn get_vertical_tab_state(
    tab_id: String,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<Option<VerticalTabState>, String> {
    Ok(manager.get_tab_state(&tab_id))
}

/// Get all tab states
#[tauri::command]
pub fn get_all_vertical_tab_states(
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<Vec<VerticalTabState>, String> {
    Ok(manager.get_all_tab_states())
}

/// Get vertical tab settings
#[tauri::command]
pub fn get_vertical_tab_settings(
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<VerticalTabSettings, String> {
    Ok(manager.get_settings())
}

/// Update vertical tab settings
#[tauri::command]
pub fn update_vertical_tab_settings(
    settings: VerticalTabSettings,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Collapse all inactive tabs
#[tauri::command]
pub fn collapse_inactive_vertical_tabs(
    threshold_seconds: u64,
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.collapse_inactive_tabs(threshold_seconds)
        .map_err(|e| format!("Failed to collapse inactive tabs: {}", e))
}

/// Expand all tabs
#[tauri::command]
pub fn expand_all_vertical_tabs(
    manager: State<'_, Arc<VerticalTabsManager>>,
) -> Result<(), String> {
    manager.expand_all_tabs()
        .map_err(|e| format!("Failed to expand all tabs: {}", e))
}
