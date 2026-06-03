//! Tab Pinning for Exodus Browser
//! 
//! This module provides tab pinning and management capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use std::time::Duration;
/// Pinned tab info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedTab {
    /// Tab ID
    pub tab_id: String,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Favicon
    pub favicon: Option<String>,
    /// Pinned at timestamp
    pub pinned_at: u64,
    /// Position
    pub position: u32,
    /// Is muted
    pub muted: bool,
}

impl PinnedTab {
    pub fn new(tab_id: String, url: String, title: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            tab_id,
            url,
            title,
            favicon: None,
            pinned_at: now,
            position: 0,
            muted: false,
        }
    }
}

/// Pinning settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinningSettings {
    /// Enable tab pinning
    pub enabled: bool,
    /// Max pinned tabs
    pub max_pinned: usize,
    /// Auto-pin frequently visited
    pub auto_pin_frequent: bool,
    /// Show pinned tabs separately
    pub show_separately: bool,
    /// Pin position (left or right)
    pub pin_position: String,
}

impl Default for PinningSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_pinned: 10,
            auto_pin_frequent: false,
            show_separately: true,
            pin_position: "left".to_string(),
        }
    }
}

/// Tab pinning manager
pub struct TabPinningManager {
    pinned_tabs: Arc<Mutex<Vec<PinnedTab>>>,
    settings: Arc<Mutex<PinningSettings>>,
    storage_path: PathBuf,
}

impl TabPinningManager {
    /// Create a new tab pinning manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            pinned_tabs: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(PinningSettings::default())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Pin a tab
    pub fn pin_tab(&self, tab_id: String, url: String, title: String) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled {
            return Ok(());
        }
        
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Check if already pinned
        if pinned.iter().any(|t| t.tab_id == tab_id) {
            return Ok(());
        }
        
        // Check max pinned limit
        if pinned.len() >= settings.max_pinned {
            return Err("Maximum pinned tabs reached".into());
        }
        
        let mut pinned_tab = PinnedTab::new(tab_id, url, title);
        pinned_tab.position = pinned.len() as u32;
        
        pinned.push(pinned_tab);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Unpin a tab
    pub fn unpin_tab(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let original_len = pinned.len();
        pinned.retain(|t| t.tab_id != tab_id);
        
        // Update positions
        for (i, tab) in pinned.iter_mut().enumerate() {
            tab.position = i as u32;
        }
        
        if pinned.len() != original_len {
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Check if a tab is pinned
    pub fn is_pinned(&self, tab_id: &str) -> bool {
        let pinned = self.pinned_tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pinned.iter().any(|t| t.tab_id == tab_id)
    }
    
    /// Get all pinned tabs
    pub fn get_all_pinned(&self) -> Vec<PinnedTab> {
        let pinned = self.pinned_tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pinned.clone()
    }
    
    /// Get pinned tab by ID
    pub fn get_pinned_tab(&self, tab_id: &str) -> Option<PinnedTab> {
        let pinned = self.pinned_tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        pinned.iter().find(|t| t.tab_id == tab_id).cloned()
    }
    
    /// Update pinned tab info
    pub fn update_pinned_tab(&self, tab_id: &str, title: Option<String>, favicon: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(tab) = pinned.iter_mut().find(|t| t.tab_id == tab_id) {
            if let Some(title) = title {
                tab.title = title;
            }
            if let Some(favicon) = favicon {
                tab.favicon = Some(favicon);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Reorder pinned tabs
    pub fn reorder_pinned_tabs(&self, tab_ids: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let mut new_order: Vec<PinnedTab> = Vec::new();
        
        for (i, tab_id) in tab_ids.iter().enumerate() {
            if let Some(tab) = pinned.iter().find(|t| &t.tab_id == tab_id) {
                let mut tab = tab.clone();
                tab.position = i as u32;
                new_order.push(tab);
            }
        }
        
        *pinned = new_order;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Mute pinned tab
    pub fn mute_pinned_tab(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(tab) = pinned.iter_mut().find(|t| t.tab_id == tab_id) {
            tab.muted = true;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Unmute pinned tab
    pub fn unmute_pinned_tab(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(tab) = pinned.iter_mut().find(|t| t.tab_id == tab_id) {
            tab.muted = false;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Clear all pinned tabs
    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        pinned.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get pinning settings
    pub fn get_settings(&self) -> PinningSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update pinning settings
    pub fn update_settings(&self, settings: PinningSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get pinning statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let pinned = self.pinned_tabs.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_pinned".to_string(), pinned.len() as u64);
        stats.insert("muted_count".to_string(), pinned.iter().filter(|t| t.muted).count() as u64);
        
        stats
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("pinning_settings.json");
        let pinned_path = self.storage_path.join("pinned_tabs.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let loaded: PinningSettings = serde_json::from_str(&content)?;
            if let Ok(mut guard) = self.settings.lock() {
                *guard = loaded;
            }
        }
        
        if pinned_path.exists() {
            let content = std::fs::read_to_string(&pinned_path)?;
            let loaded: Vec<PinnedTab> = serde_json::from_str(&content)?;
            if let Ok(mut guard) = self.pinned_tabs.lock() {
                *guard = loaded;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("pinning_settings.json");
        let pinned_path = self.storage_path.join("pinned_tabs.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let pinned = self.pinned_tabs.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let pinned_content = serde_json::to_string_pretty(&*pinned)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&pinned_path, pinned_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Pin a tab
#[tauri::command]
pub fn pin_tab(
    tab_id: String,
    url: String,
    title: String,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.pin_tab(tab_id, url, title)
        .map_err(|e| format!("Failed to pin tab: {}", e))
}

/// Unpin a tab
#[tauri::command]
pub fn unpin_tab(
    tab_id: String,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.unpin_tab(&tab_id)
        .map_err(|e| format!("Failed to unpin tab: {}", e))
}

/// Check if a tab is pinned
#[tauri::command]
pub fn is_tab_pinned(
    tab_id: String,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<bool, String> {
    Ok(manager.is_pinned(&tab_id))
}

/// Get all pinned tabs
#[tauri::command]
pub fn get_all_pinned_tabs(
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<Vec<PinnedTab>, String> {
    Ok(manager.get_all_pinned())
}

/// Get pinned tab by ID
#[tauri::command]
pub fn get_pinned_tab(
    tab_id: String,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<Option<PinnedTab>, String> {
    Ok(manager.get_pinned_tab(&tab_id))
}

/// Update pinned tab info
#[tauri::command]
pub fn update_pinned_tab(
    tab_id: String,
    title: Option<String>,
    favicon: Option<String>,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.update_pinned_tab(&tab_id, title, favicon)
        .map_err(|e| format!("Failed to update pinned tab: {}", e))
}

/// Reorder pinned tabs
#[tauri::command]
pub fn reorder_pinned_tabs(
    tab_ids: Vec<String>,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.reorder_pinned_tabs(tab_ids)
        .map_err(|e| format!("Failed to reorder pinned tabs: {}", e))
}

/// Mute pinned tab
#[tauri::command]
pub fn mute_pinned_tab(
    tab_id: String,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.mute_pinned_tab(&tab_id)
        .map_err(|e| format!("Failed to mute pinned tab: {}", e))
}

/// Unmute pinned tab
#[tauri::command]
pub fn unmute_pinned_tab(
    tab_id: String,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.unmute_pinned_tab(&tab_id)
        .map_err(|e| format!("Failed to unmute pinned tab: {}", e))
}

/// Clear all pinned tabs
#[tauri::command]
pub fn clear_all_pinned_tabs(
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.clear_all()
        .map_err(|e| format!("Failed to clear pinned tabs: {}", e))
}

/// Get pinning settings
#[tauri::command]
pub fn get_pinning_settings(
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<PinningSettings, String> {
    Ok(manager.get_settings())
}

/// Update pinning settings
#[tauri::command]
pub fn update_pinning_settings(
    settings: PinningSettings,
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get pinning statistics
#[tauri::command]
pub fn get_pinning_stats(
    manager: State<'_, Arc<TabPinningManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}
