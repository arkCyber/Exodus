//! Tab Grouping for Exodus Browser
//! 
//! This module provides tab grouping and organization capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;

/// Tab group color
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GroupColor {
    Grey,
    Blue,
    Red,
    Yellow,
    Green,
    Pink,
    Purple,
    Cyan,
    Orange,
}

impl GroupColor {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "grey" | "gray" => GroupColor::Grey,
            "blue" => GroupColor::Blue,
            "red" => GroupColor::Red,
            "yellow" => GroupColor::Yellow,
            "green" => GroupColor::Green,
            "pink" => GroupColor::Pink,
            "purple" => GroupColor::Purple,
            "cyan" => GroupColor::Cyan,
            "orange" => GroupColor::Orange,
            _ => GroupColor::Grey,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            GroupColor::Grey => "grey",
            GroupColor::Blue => "blue",
            GroupColor::Red => "red",
            GroupColor::Yellow => "yellow",
            GroupColor::Green => "green",
            GroupColor::Pink => "pink",
            GroupColor::Purple => "purple",
            GroupColor::Cyan => "cyan",
            GroupColor::Orange => "orange",
        }
    }
}

/// Tab group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroup {
    /// Group ID
    pub id: String,
    /// Group title
    pub title: String,
    /// Group color
    pub color: GroupColor,
    /// Tab IDs in the group
    pub tab_ids: Vec<String>,
    /// Created timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub last_modified: u64,
    /// Is collapsed
    pub collapsed: bool,
}

impl TabGroup {
    pub fn new(title: String, color: GroupColor) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            color,
            tab_ids: Vec::new(),
            created_at: now,
            last_modified: now,
            collapsed: false,
        }
    }
    
    pub fn add_tab(&mut self, tab_id: String) {
        if !self.tab_ids.contains(&tab_id) {
            self.tab_ids.push(tab_id);
            self.last_modified = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
        }
    }
    
    pub fn remove_tab(&mut self, tab_id: &str) {
        self.tab_ids.retain(|id| id != tab_id);
        self.last_modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }
    
    pub fn tab_count(&self) -> usize {
        self.tab_ids.len()
    }
}

/// Tab grouping settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroupingSettings {
    /// Enable tab grouping
    pub enabled: bool,
    /// Auto-group by domain
    pub auto_group_by_domain: bool,
    /// Auto-group by topic
    pub auto_group_by_topic: bool,
    /// Show group labels
    pub show_labels: bool,
    /// Collapse inactive groups
    pub collapse_inactive: bool,
    /// Inactivity threshold in seconds
    pub inactivity_threshold: u64,
}

impl Default for TabGroupingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_group_by_domain: false,
            auto_group_by_topic: false,
            show_labels: true,
            collapse_inactive: false,
            inactivity_threshold: 3600, // 1 hour
        }
    }
}

/// Tab grouping manager
pub struct TabGroupingManager {
    groups: Arc<Mutex<HashMap<String, TabGroup>>>,
    tab_to_group: Arc<Mutex<HashMap<String, String>>>,
    settings: Arc<Mutex<TabGroupingSettings>>,
    storage_path: PathBuf,
}

impl TabGroupingManager {
    /// Create a new tab grouping manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            groups: Arc::new(Mutex::new(HashMap::new())),
            tab_to_group: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(TabGroupingSettings::default())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Create a tab group
    pub fn create_group(&self, title: String, color: GroupColor) -> Result<String, Box<dyn std::error::Error>> {
        let group = TabGroup::new(title, color);
        let group_id = group.id.clone();
        
        let mut groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        groups.insert(group_id.clone(), group);
        self.save_to_disk()?;
        Ok(group_id)
    }
    
    /// Update a tab group
    pub fn update_group(&self, id: String, title: Option<String>, color: Option<GroupColor>) -> Result<(), Box<dyn std::error::Error>> {
        let mut groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(group) = groups.get_mut(&id) {
            if let Some(title) = title {
                group.title = title;
            }
            if let Some(color) = color {
                group.color = color;
            }
            group.last_modified = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Delete a tab group
    pub fn delete_group(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(group) = groups.remove(id) {
            // Remove tab to group mappings
            let mut tab_to_group = self.tab_to_group.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            for tab_id in &group.tab_ids {
                tab_to_group.remove(tab_id);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add a tab to a group
    pub fn add_tab_to_group(&self, group_id: String, tab_id: String) -> Result<(), Box<dyn std::error::Error>> {
        // Remove tab from existing group if any
        let mut tab_to_group = self.tab_to_group.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(old_group_id) = tab_to_group.remove(&tab_id) {
            let mut groups = self.groups.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if let Some(group) = groups.get_mut(&old_group_id) {
                group.remove_tab(&tab_id);
            }
        }
        
        // Add to new group
        tab_to_group.insert(tab_id.clone(), group_id.clone());
        
        let mut groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(group) = groups.get_mut(&group_id) {
            group.add_tab(tab_id);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a tab from its group
    pub fn remove_tab_from_group(&self, tab_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tab_to_group = self.tab_to_group.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(group_id) = tab_to_group.remove(tab_id) {
            let mut groups = self.groups.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if let Some(group) = groups.get_mut(&group_id) {
                group.remove_tab(tab_id);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get group for a tab
    pub fn get_group_for_tab(&self, tab_id: &str) -> Option<TabGroup> {
        let tab_to_group = self.tab_to_group.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(group_id) = tab_to_group.get(tab_id) {
            let groups = self.groups.lock()
                .unwrap_or_else(|_| panic!("Lock error"));
            groups.get(group_id).cloned()
        } else {
            None
        }
    }
    
    /// Get all groups
    pub fn get_all_groups(&self) -> Vec<TabGroup> {
        let groups = self.groups.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        groups.values().cloned().collect()
    }
    
    /// Get a group by ID
    pub fn get_group(&self, id: &str) -> Option<TabGroup> {
        let groups = self.groups.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        groups.get(id).cloned()
    }
    
    /// Collapse a group
    pub fn collapse_group(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(group) = groups.get_mut(id) {
            group.collapsed = true;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Expand a group
    pub fn expand_group(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(group) = groups.get_mut(id) {
            group.collapsed = false;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get tab grouping settings
    pub fn get_settings(&self) -> TabGroupingSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update tab grouping settings
    pub fn update_settings(&self, settings: TabGroupingSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Auto-group tabs by domain (placeholder implementation)
    pub fn auto_group_by_domain(&self, tab_domains: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.auto_group_by_domain {
            return Ok(());
        }
        
        // Group tabs by domain
        let mut domain_groups: HashMap<String, Vec<String>> = HashMap::new();
        
        for (tab_id, domain) in tab_domains {
            domain_groups.entry(domain).or_insert_with(Vec::new).push(tab_id);
        }
        
        // Create groups for domains with multiple tabs
        for (domain, tab_ids) in domain_groups {
            if tab_ids.len() > 1 {
                let group_id = self.create_group(domain.clone(), GroupColor::Blue)?;
                
                for tab_id in tab_ids {
                    self.add_tab_to_group(group_id.clone(), tab_id)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get grouping statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let groups = self.groups.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_groups".to_string(), groups.len() as u64);
        
        let total_tabs: u64 = groups.values().map(|g| g.tab_count() as u64).sum();
        stats.insert("total_tabs_in_groups".to_string(), total_tabs);
        
        let collapsed_count = groups.values().filter(|g| g.collapsed).count() as u64;
        stats.insert("collapsed_groups".to_string(), collapsed_count);
        
        stats
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("tab_grouping_settings.json");
        let groups_path = self.storage_path.join("tab_groups.json");
        let tab_to_group_path = self.storage_path.join("tab_to_group.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: TabGroupingSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if groups_path.exists() {
            let content = std::fs::read_to_string(&groups_path)?;
            let groups: HashMap<String, TabGroup> = serde_json::from_str(&content)?;
            if let Ok(mut g) = self.groups.lock() {
                *g = groups;
            }
        }
        
        if tab_to_group_path.exists() {
            let content = std::fs::read_to_string(&tab_to_group_path)?;
            let tab_to_group: HashMap<String, String> = serde_json::from_str(&content)?;
            if let Ok(mut t) = self.tab_to_group.lock() {
                *t = tab_to_group;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("tab_grouping_settings.json");
        let groups_path = self.storage_path.join("tab_groups.json");
        let tab_to_group_path = self.storage_path.join("tab_to_group.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let groups = self.groups.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let tab_to_group = self.tab_to_group.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let groups_content = serde_json::to_string_pretty(&*groups)?;
        let tab_to_group_content = serde_json::to_string_pretty(&*tab_to_group)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&groups_path, groups_content)?;
        std::fs::write(&tab_to_group_path, tab_to_group_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Create a tab group
#[tauri::command]
pub fn create_tab_group(
    title: String,
    color: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<String, String> {
    let c = GroupColor::from_str(&color);
    manager.create_group(title, c)
        .map_err(|e| format!("Failed to create group: {}", e))
}

/// Update a tab group
#[tauri::command]
pub fn update_tab_group(
    id: String,
    title: Option<String>,
    color: Option<String>,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    let c = color.map(|c| GroupColor::from_str(&c));
    manager.update_group(id, title, c)
        .map_err(|e| format!("Failed to update group: {}", e))
}

/// Delete a tab group
#[tauri::command]
pub fn delete_tab_group(
    id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.delete_group(&id)
        .map_err(|e| format!("Failed to delete group: {}", e))
}

/// Add a tab to a group
#[tauri::command]
pub fn add_tab_to_group(
    group_id: String,
    tab_id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.add_tab_to_group(group_id, tab_id)
        .map_err(|e| format!("Failed to add tab to group: {}", e))
}

/// Remove a tab from its group
#[tauri::command]
pub fn remove_tab_from_group(
    tab_id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.remove_tab_from_group(&tab_id)
        .map_err(|e| format!("Failed to remove tab from group: {}", e))
}

/// Get group for a tab
#[tauri::command]
pub fn get_group_for_tab(
    tab_id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<Option<TabGroup>, String> {
    Ok(manager.get_group_for_tab(&tab_id))
}

/// Get all groups
#[tauri::command]
pub fn get_all_tab_groups(
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<Vec<TabGroup>, String> {
    Ok(manager.get_all_groups())
}

/// Get a group by ID
#[tauri::command]
pub fn get_tab_group(
    id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<Option<TabGroup>, String> {
    Ok(manager.get_group(&id))
}

/// Collapse a group
#[tauri::command]
pub fn collapse_tab_group(
    id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.collapse_group(&id)
        .map_err(|e| format!("Failed to collapse group: {}", e))
}

/// Expand a group
#[tauri::command]
pub fn expand_tab_group(
    id: String,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.expand_group(&id)
        .map_err(|e| format!("Failed to expand group: {}", e))
}

/// Get tab grouping settings
#[tauri::command]
pub fn get_tab_grouping_settings(
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<TabGroupingSettings, String> {
    Ok(manager.get_settings())
}

/// Update tab grouping settings
#[tauri::command]
pub fn update_tab_grouping_settings(
    settings: TabGroupingSettings,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Auto-group tabs by domain
#[tauri::command]
pub fn auto_group_tabs_by_domain(
    tab_domains: HashMap<String, String>,
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<(), String> {
    manager.auto_group_by_domain(tab_domains)
        .map_err(|e| format!("Failed to auto-group tabs: {}", e))
}

/// Get grouping statistics
#[tauri::command]
pub fn get_tab_grouping_stats(
    manager: State<'_, Arc<TabGroupingManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}
