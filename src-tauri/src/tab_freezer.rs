//! Tab Freezer for Memory Optimization
//! 
//! This module provides tab freezing functionality to reduce memory usage
//! by suspending inactive tabs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

use std::time::Duration;
/// Tab state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TabState {
    Active,
    Frozen,
    Suspended,
}

/// Tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// Tab label
    pub label: String,
    /// Tab URL
    pub url: String,
    /// Tab title
    pub title: String,
    /// Current state
    pub state: TabState,
    /// Last active timestamp
    pub last_active: u64,
    /// Memory usage estimate (bytes)
    pub memory_usage: usize,
    /// Frozen timestamp
    pub frozen_at: Option<u64>,
}

impl TabInfo {
    pub fn new(label: String, url: String, title: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            label,
            url,
            title,
            state: TabState::Active,
            last_active: now,
            memory_usage: 0,
            frozen_at: None,
        }
    }
    
    pub fn update_last_active(&mut self) {
        self.last_active = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }
    
    pub fn freeze(&mut self) {
        self.state = TabState::Frozen;
        self.frozen_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs()
        );
    }
    
    pub fn unfreeze(&mut self) {
        self.state = TabState::Active;
        self.frozen_at = None;
        self.update_last_active();
    }
    
    pub fn is_frozen(&self) -> bool {
        self.state == TabState::Frozen
    }
    
    pub fn inactive_duration(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        now - self.last_active
    }
}

/// Tab freezer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabFreezerSettings {
    /// Enable tab freezing
    pub enabled: bool,
    /// Inactivity threshold in seconds before freezing
    pub inactivity_threshold: u64,
    /// Maximum number of active tabs
    pub max_active_tabs: usize,
    /// Freeze tabs when memory usage exceeds threshold (MB)
    pub memory_threshold_mb: usize,
    /// Auto-freeze background tabs
    pub auto_freeze_background: bool,
}

impl Default for TabFreezerSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            inactivity_threshold: 300, // 5 minutes
            max_active_tabs: 10,
            memory_threshold_mb: 2000,
            auto_freeze_background: true,
        }
    }
}

/// Tab freezer manager
pub struct TabFreezer {
    tabs: Arc<Mutex<HashMap<String, TabInfo>>>,
    settings: Arc<Mutex<TabFreezerSettings>>,
    total_memory_saved: Arc<Mutex<usize>>,
}

impl TabFreezer {
    /// Create a new tab freezer
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(TabFreezerSettings::default())),
            total_memory_saved: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Register a tab
    pub fn register_tab(&self, label: String, url: String, title: String) {
        let mut tabs = self.tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let tab_info = TabInfo::new(label.clone(), url, title);
        tabs.insert(label, tab_info);
    }
    
    /// Unregister a tab
    pub fn unregister_tab(&self, label: &str) {
        let mut tabs = self.tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        tabs.remove(label);
    }
    
    /// Update tab activity
    pub fn update_tab_activity(&self, label: &str) {
        let mut tabs = self.tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(tab) = tabs.get_mut(label) {
            tab.update_last_active();
            if tab.is_frozen() {
                tab.unfreeze();
            }
        }
    }
    
    /// Freeze a specific tab
    pub fn freeze_tab(&self, label: &str) -> Result<(), String> {
        let mut tabs = self.tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(tab) = tabs.get_mut(label) {
            if !tab.is_frozen() {
                tab.freeze();
                
                // Estimate memory saved
                let memory_saved = tab.memory_usage;
                let mut total_saved = self.total_memory_saved.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                *total_saved += memory_saved;
            }
            Ok(())
        } else {
            Err(format!("Tab not found: {}", label))
        }
    }
    
    /// Unfreeze a specific tab
    pub fn unfreeze_tab(&self, label: &str) -> Result<(), String> {
        let mut tabs = self.tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(tab) = tabs.get_mut(label) {
            if tab.is_frozen() {
                tab.unfreeze();
            }
            Ok(())
        } else {
            Err(format!("Tab not found: {}", label))
        }
    }
    
    /// Auto-freeze inactive tabs
    pub fn auto_freeze_inactive_tabs(&self) -> Vec<String> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled {
            return Vec::new();
        }
        
        let mut tabs = self.tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut frozen_tabs = Vec::new();
        let mut active_count = 0;
        
        // First, count active tabs
        for tab in tabs.values() {
            if !tab.is_frozen() {
                active_count += 1;
            }
        }
        
        // Freeze inactive tabs if needed
        for (label, tab) in tabs.iter_mut() {
            if !tab.is_frozen() && tab.inactive_duration() > settings.inactivity_threshold {
                // Check if we need to freeze due to max active tabs
                if active_count > settings.max_active_tabs {
                    tab.freeze();
                    frozen_tabs.push(label.clone());
                    active_count -= 1;
                }
            }
        }
        
        frozen_tabs
    }
    
    /// Get all tabs
    pub fn get_tabs(&self) -> Vec<TabInfo> {
        let tabs = self.tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        tabs.values().cloned().collect()
    }
    
    /// Get frozen tabs
    pub fn get_frozen_tabs(&self) -> Vec<TabInfo> {
        let tabs = self.tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        tabs.values()
            .filter(|t| t.is_frozen())
            .cloned()
            .collect()
    }
    
    /// Get tab freezer settings
    pub fn get_settings(&self) -> TabFreezerSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update tab freezer settings
    pub fn update_settings(&self, settings: TabFreezerSettings) {
        let mut current = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        *current = settings;
    }
    
    /// Get total memory saved
    pub fn get_total_memory_saved(&self) -> usize {
        let saved = self.total_memory_saved.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        *saved
    }
    
    /// Reset memory saved counter
    pub fn reset_memory_saved(&self) {
        let mut saved = self.total_memory_saved.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        *saved = 0;
    }
}

// Tauri Commands

/// Register a tab
#[tauri::command]
pub fn register_tab(
    label: String,
    url: String,
    title: String,
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<(), String> {
    freezer.register_tab(label, url, title);
    Ok(())
}

/// Unregister a tab
#[tauri::command]
pub fn unregister_tab(
    label: String,
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<(), String> {
    freezer.unregister_tab(&label);
    Ok(())
}

/// Update tab activity
#[tauri::command]
pub fn update_tab_activity(
    label: String,
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<(), String> {
    freezer.update_tab_activity(&label);
    Ok(())
}

/// Freeze a tab
#[tauri::command]
pub fn freeze_tab(
    label: String,
    freezer: State<'_, Arc<TabFreezer>>,
    app: AppHandle,
) -> Result<(), String> {
    freezer.freeze_tab(&label)?;
    
    // Emit event to notify frontend
    let _ = app.emit("tab-frozen", label);
    
    Ok(())
}

/// Unfreeze a tab
#[tauri::command]
pub fn unfreeze_tab(
    label: String,
    freezer: State<'_, Arc<TabFreezer>>,
    app: AppHandle,
) -> Result<(), String> {
    freezer.unfreeze_tab(&label)?;
    
    // Emit event to notify frontend
    let _ = app.emit("tab-unfrozen", label);
    
    Ok(())
}

/// Auto-freeze inactive tabs
#[tauri::command]
pub fn auto_freeze_inactive_tabs(
    freezer: State<'_, Arc<TabFreezer>>,
    app: AppHandle,
) -> Result<Vec<String>, String> {
    let frozen = freezer.auto_freeze_inactive_tabs();
    
    // Emit event for each frozen tab
    for label in &frozen {
        let _ = app.emit("tab-frozen", label);
    }
    
    Ok(frozen)
}

/// Get all tabs
#[tauri::command]
pub fn get_tabs(
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<Vec<TabInfo>, String> {
    Ok(freezer.get_tabs())
}

/// Get frozen tabs
#[tauri::command]
pub fn get_frozen_tabs(
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<Vec<TabInfo>, String> {
    Ok(freezer.get_frozen_tabs())
}

/// Get tab freezer settings
#[tauri::command]
pub fn get_tab_freezer_settings(
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<TabFreezerSettings, String> {
    Ok(freezer.get_settings())
}

/// Update tab freezer settings
#[tauri::command]
pub fn update_tab_freezer_settings(
    settings: TabFreezerSettings,
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<(), String> {
    freezer.update_settings(settings);
    Ok(())
}

/// Get total memory saved
#[tauri::command]
pub fn get_total_memory_saved(
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<usize, String> {
    Ok(freezer.get_total_memory_saved())
}

/// Reset memory saved counter
#[tauri::command]
pub fn reset_memory_saved(
    freezer: State<'_, Arc<TabFreezer>>,
) -> Result<(), String> {
    freezer.reset_memory_saved();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tab_info_creation() {
        let tab = TabInfo::new(
            "tab1".to_string(),
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        
        assert_eq!(tab.label, "tab1");
        assert_eq!(tab.state, TabState::Active);
        assert!(!tab.is_frozen());
    }
    
    #[test]
    fn test_tab_freeze_unfreeze() {
        let mut tab = TabInfo::new(
            "tab1".to_string(),
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        
        tab.freeze();
        assert!(tab.is_frozen());
        
        tab.unfreeze();
        assert!(!tab.is_frozen());
    }
    
    #[test]
    fn test_tab_freezer() {
        let freezer = TabFreezer::new();
        
        freezer.register_tab(
            "tab1".to_string(),
            "https://example.com".to_string(),
            "Example".to_string(),
        );
        
        freezer.freeze_tab("tab1").expect("Failed to freeze tab");
        
        let frozen = freezer.get_frozen_tabs();
        assert_eq!(frozen.len(), 1);
        assert_eq!(frozen[0].label, "tab1");
    }
    
    #[test]
    fn test_tab_freezer_settings() {
        let settings = TabFreezerSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.inactivity_threshold, 300);
    }
}
