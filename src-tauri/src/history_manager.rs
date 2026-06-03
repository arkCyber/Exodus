//! History Management System for Exodus Browser
//! 
//! This module provides comprehensive browsing history management.

use crate::config::ConfigState;
use crate::profile_stores::ProfileHistoryStores;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Active profile history store from privacy mode flag.
fn active_history<'a>(
    stores: &'a ProfileHistoryStores,
    config: &ConfigState,
) -> Result<&'a Arc<HistoryManager>, String> {
    let private = config
        .lock()
        .map_err(|e| format!("Config lock: {}", e))?
        .private_mode;
    Ok(stores.active(private))
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Entry ID
    pub id: String,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Visit timestamp
    pub visit_time: u64,
    /// Visit count
    pub visit_count: u32,
    /// Last visit timestamp
    pub last_visit: u64,
    /// Favicon URL
    pub favicon: Option<String>,
    /// Referrer
    pub referrer: Option<String>,
    /// Transition type
    pub transition_type: String,
}

impl HistoryEntry {
    pub fn new(url: String, title: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            title,
            visit_time: now,
            visit_count: 1,
            last_visit: now,
            favicon: None,
            referrer: None,
            transition_type: "typed".to_string(),
        }
    }
    
    pub fn increment_visit(&mut self) {
        self.visit_count += 1;
        self.last_visit = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }
}

/// History settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySettings {
    /// Enable history
    pub enabled: bool,
    /// Remember browsing history
    pub remember_browsing: bool,
    /// Remember download history
    pub remember_downloads: bool,
    /// Remember form data
    pub remember_form_data: bool,
    /// History retention period in days (0 = forever)
    pub retention_days: u32,
    /// Clear history on exit
    pub clear_on_exit: bool,
    /// Allow incognito history
    pub allow_incognito: bool,
}

impl Default for HistorySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            remember_browsing: true,
            remember_downloads: true,
            remember_form_data: false,
            retention_days: 90,
            clear_on_exit: false,
            allow_incognito: false,
        }
    }
}

/// History manager
pub struct HistoryManager {
    history: Arc<Mutex<Vec<HistoryEntry>>>,
    settings: Arc<Mutex<HistorySettings>>,
    storage_path: PathBuf,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            history: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(HistorySettings::default())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        manager.cleanup_old_entries()?;
        Ok(manager)
    }
    
    /// Add a history entry
    pub fn add_entry(&self, url: String, title: String) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled || !settings.remember_browsing {
            return Ok(());
        }
        
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Check if URL already exists
        if let Some(entry) = history.iter_mut().find(|e| e.url == url) {
            entry.increment_visit();
        } else {
            let entry = HistoryEntry::new(url, title);
            history.push(entry);
        }
        
        // Keep only last 10000 entries
        let len = history.len();
        if len > 10000 {
            history.drain(0..len - 10000);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a history entry
    pub fn remove_entry(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.retain(|e| e.id != id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove history by URL
    pub fn remove_by_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.retain(|e| e.url != url);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove history by domain
    pub fn remove_by_domain(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.retain(|e| !e.url.contains(domain));
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Clear all history
    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Clear history by time range
    pub fn clear_by_time_range(&self, start: u64, end: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.retain(|e| e.visit_time < start || e.visit_time > end);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get all history
    pub fn get_all(&self) -> Vec<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        history.clone()
    }
    
    /// Get history by URL
    pub fn get_by_url(&self, url: &str) -> Option<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        history.iter().find(|e| e.url == url).cloned()
    }
    
    /// Search history
    pub fn search(&self, query: &str) -> Vec<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let query_lower = query.to_lowercase();
        history.iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&query_lower) ||
                e.title.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
    
    /// Get history by time range
    pub fn get_by_time_range(&self, start: u64, end: u64) -> Vec<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        history.iter()
            .filter(|e| e.visit_time >= start && e.visit_time <= end)
            .cloned()
            .collect()
    }
    
    /// Get recent history
    pub fn get_recent(&self, limit: usize) -> Vec<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let len = history.len();
        let start = if len > limit { len - limit } else { 0 };
        history[start..].to_vec()
    }
    
    /// Get most visited
    pub fn get_most_visited(&self, limit: usize) -> Vec<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut entries: Vec<HistoryEntry> = history.clone();
        entries.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
        entries.into_iter().take(limit).collect()
    }
    
    /// Get history by domain
    pub fn get_by_domain(&self, domain: &str) -> Vec<HistoryEntry> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        history.iter()
            .filter(|e| e.url.contains(domain))
            .cloned()
            .collect()
    }
    
    /// Get history settings
    pub fn get_settings(&self) -> HistorySettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update history settings
    pub fn update_settings(&self, settings: HistorySettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get history statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), history.len() as u64);
        
        let total_visits: u64 = history.iter().map(|e| e.visit_count as u64).sum();
        stats.insert("total_visits".to_string(), total_visits);
        
        let unique_domains: std::collections::HashSet<String> = history.iter()
            .filter_map(|e| {
                if let Ok(parsed) = url::Url::parse(&e.url) {
                    parsed.host_str().map(|h| h.to_string())
                } else {
                    None
                }
            })
            .collect();
        stats.insert("unique_domains".to_string(), unique_domains.len() as u64);
        
        stats
    }
    
    /// Cleanup old entries based on retention policy
    fn cleanup_old_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if settings.retention_days == 0 {
            return Ok(()); // Keep forever
        }
        
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs() - (settings.retention_days as u64 * 86400);
        
        let mut history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let original_len = history.len();
        history.retain(|e| e.visit_time > cutoff);
        
        if history.len() != original_len {
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("history_settings.json");
        let history_path = self.storage_path.join("history.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: HistorySettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if history_path.exists() {
            let content = std::fs::read_to_string(&history_path)?;
            let history: Vec<HistoryEntry> = serde_json::from_str(&content)?;
            if let Ok(mut h) = self.history.lock() {
                *h = history;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("history_settings.json");
        let history_path = self.storage_path.join("history.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let history_content = serde_json::to_string_pretty(&*history)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&history_path, history_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add a history entry
#[tauri::command]
pub fn add_history_entry(
    url: String,
    title: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .add_entry(url, title)
        .map_err(|e| format!("Failed to add history entry: {}", e))
}

/// Remove a history entry
#[tauri::command]
pub fn remove_history_entry(
    id: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .remove_entry(&id)
        .map_err(|e| format!("Failed to remove history entry: {}", e))
}

/// Remove history by URL
#[tauri::command]
pub fn remove_history_by_url(
    url: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .remove_by_url(&url)
        .map_err(|e| format!("Failed to remove history by URL: {}", e))
}

/// Remove history by domain
#[tauri::command]
pub fn remove_history_by_domain(
    domain: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .remove_by_domain(&domain)
        .map_err(|e| format!("Failed to remove history by domain: {}", e))
}

/// Clear all history
#[tauri::command]
pub fn clear_all_history(
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .clear_all()
        .map_err(|e| format!("Failed to clear history: {}", e))
}

/// Clear history by time range
#[tauri::command]
pub fn clear_history_by_time_range(
    start: u64,
    end: u64,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .clear_by_time_range(start, end)
        .map_err(|e| format!("Failed to clear history by time range: {}", e))
}

/// Get all history
#[tauri::command]
pub fn get_all_history(
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.get_all())
}

/// Get history by URL
#[tauri::command]
pub fn get_history_by_url(
    url: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Option<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.get_by_url(&url))
}

/// Search history
#[tauri::command]
pub fn search_history(
    query: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.search(&query))
}

/// Get history by time range
#[tauri::command]
pub fn get_history_by_time_range(
    start: u64,
    end: u64,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.get_by_time_range(start, end))
}

/// Get recent history
#[tauri::command]
pub fn get_recent_history(
    limit: usize,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.get_recent(limit))
}

/// Get most visited
#[tauri::command]
pub fn get_most_visited_history(
    limit: usize,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.get_most_visited(limit))
}

/// Get history by domain
#[tauri::command]
pub fn get_history_by_domain(
    domain: String,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(active_history(&stores, &config)?.get_by_domain(&domain))
}

/// Get history settings
#[tauri::command]
pub fn get_history_settings(
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<HistorySettings, String> {
    Ok(active_history(&stores, &config)?.get_settings())
}

/// Update history settings
#[tauri::command]
pub fn update_history_settings(
    settings: HistorySettings,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_history(&stores, &config)?
        .update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get history statistics
#[tauri::command]
pub fn get_history_stats(
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<HashMap<String, u64>, String> {
    Ok(active_history(&stores, &config)?.get_stats())
}
