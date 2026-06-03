//! Privacy Dashboard for Exodus Browser
//! 
//! This module provides privacy protection statistics and management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use std::time::Duration;
/// Privacy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyStats {
    /// Total trackers blocked
    pub trackers_blocked: u64,
    /// Total cookies blocked
    pub cookies_blocked: u64,
    /// Total fingerprinting attempts blocked
    pub fingerprinting_blocked: u64,
    /// Total malicious sites blocked
    pub malicious_sites_blocked: u64,
    /// Total data saved (bytes)
    pub data_saved: u64,
    /// Total time saved (seconds)
    pub time_saved: u64,
    /// Start timestamp
    pub start_timestamp: u64,
}

impl Default for PrivacyStats {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            trackers_blocked: 0,
            cookies_blocked: 0,
            fingerprinting_blocked: 0,
            malicious_sites_blocked: 0,
            data_saved: 0,
            time_saved: 0,
            start_timestamp: now,
        }
    }
}

/// Privacy event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Description
    pub description: String,
    /// Domain
    pub domain: String,
    /// Timestamp
    pub timestamp: u64,
}

impl PrivacyEvent {
    pub fn new(event_type: String, description: String, domain: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            description,
            domain,
            timestamp: now,
        }
    }
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Enable tracking protection
    pub tracking_protection_enabled: bool,
    /// Enable cookie blocking
    pub cookie_blocking_enabled: bool,
    /// Enable fingerprinting protection
    pub fingerprinting_protection_enabled: bool,
    /// Enable safe browsing
    pub safe_browsing_enabled: bool,
    /// Send do not track signal
    pub send_do_not_track: bool,
    /// Clear browsing data on exit
    pub clear_data_on_exit: bool,
    /// Block third-party cookies
    pub block_third_party_cookies: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            tracking_protection_enabled: true,
            cookie_blocking_enabled: true,
            fingerprinting_protection_enabled: true,
            safe_browsing_enabled: true,
            send_do_not_track: true,
            clear_data_on_exit: false,
            block_third_party_cookies: true,
        }
    }
}

/// Privacy dashboard manager
pub struct PrivacyDashboardManager {
    stats: Arc<Mutex<PrivacyStats>>,
    events: Arc<Mutex<Vec<PrivacyEvent>>>,
    settings: Arc<Mutex<PrivacySettings>>,
    storage_path: PathBuf,
}

impl PrivacyDashboardManager {
    /// Create a new privacy dashboard manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            stats: Arc::new(Mutex::new(PrivacyStats::default())),
            events: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(PrivacySettings::default())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Record tracker blocked
    pub fn record_tracker_blocked(&self, domain: String) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.trackers_blocked += 1;
        
        let event = PrivacyEvent::new(
            "tracker_blocked".to_string(),
            format!("Blocked tracker from {}", domain),
            domain,
        );
        
        let mut events = self.events.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        events.push(event);
        
        // Keep only last 1000 events
        let len = events.len();
        if len > 1000 {
            events.drain(0..len - 1000);
        }
    }
    
    /// Record cookie blocked
    pub fn record_cookie_blocked(&self, domain: String) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.cookies_blocked += 1;
        
        let event = PrivacyEvent::new(
            "cookie_blocked".to_string(),
            format!("Blocked cookie from {}", domain),
            domain,
        );
        
        let mut events = self.events.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        events.push(event);
        
        let len = events.len();
        if len > 1000 {
            events.drain(0..len - 1000);
        }
    }
    
    /// Record fingerprinting blocked
    pub fn record_fingerprinting_blocked(&self, domain: String) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.fingerprinting_blocked += 1;
        
        let event = PrivacyEvent::new(
            "fingerprinting_blocked".to_string(),
            format!("Blocked fingerprinting attempt from {}", domain),
            domain,
        );
        
        let mut events = self.events.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        events.push(event);
        
        let len = events.len();
        if len > 1000 {
            events.drain(0..len - 1000);
        }
    }
    
    /// Record malicious site blocked
    pub fn record_malicious_site_blocked(&self, domain: String) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.malicious_sites_blocked += 1;
        
        let event = PrivacyEvent::new(
            "malicious_site_blocked".to_string(),
            format!("Blocked malicious site: {}", domain),
            domain,
        );
        
        let mut events = self.events.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        events.push(event);
        
        let len = events.len();
        if len > 1000 {
            events.drain(0..len - 1000);
        }
    }
    
    /// Get privacy stats saved
    pub fn record_data_saved(&self, bytes: u64) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.data_saved += bytes;
    }
    
    /// Record time saved
    pub fn record_time_saved(&self, seconds: u64) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.time_saved += seconds;
    }
    
    /// Get privacy statistics
    pub fn get_stats(&self) -> PrivacyStats {
        let stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.clone()
    }
    
    /// Get privacy events
    pub fn get_events(&self, limit: usize) -> Vec<PrivacyEvent> {
        let events = self.events.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let len = events.len();
        let start = if len > limit { len - limit } else { 0 };
        events[start..].to_vec()
    }
    
    /// Clear events
    pub fn clear_events(&self) {
        let mut events = self.events.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        events.clear();
    }
    
    /// Get privacy settings
    pub fn get_settings(&self) -> PrivacySettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update privacy settings
    pub fn update_settings(&self, settings: PrivacySettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stats = self.stats.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *stats = PrivacyStats::default();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get privacy score (0-100)
    pub fn get_privacy_score(&self) -> u8 {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut score = 0u8;
        
        if settings.tracking_protection_enabled { score += 20; }
        if settings.cookie_blocking_enabled { score += 15; }
        if settings.fingerprinting_protection_enabled { score += 15; }
        if settings.safe_browsing_enabled { score += 15; }
        if settings.send_do_not_track { score += 10; }
        if settings.block_third_party_cookies { score += 15; }
        if settings.clear_data_on_exit { score += 10; }
        
        score
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("privacy_settings.json");
        let stats_path = self.storage_path.join("privacy_stats.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: PrivacySettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if stats_path.exists() {
            let content = std::fs::read_to_string(&stats_path)?;
            let stats: PrivacyStats = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.stats.lock() {
                *s = stats;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("privacy_settings.json");
        let stats_path = self.storage_path.join("privacy_stats.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let stats = self.stats.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let stats_content = serde_json::to_string_pretty(&*stats)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&stats_path, stats_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Record tracker blocked
#[tauri::command]
pub fn record_tracker_blocked(
    domain: String,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.record_tracker_blocked(domain);
    Ok(())
}

/// Record cookie blocked
#[tauri::command]
pub fn record_cookie_blocked(
    domain: String,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.record_cookie_blocked(domain);
    Ok(())
}

/// Record fingerprinting blocked
#[tauri::command]
pub fn record_fingerprinting_blocked(
    domain: String,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.record_fingerprinting_blocked(domain);
    Ok(())
}

/// Record malicious site blocked
#[tauri::command]
pub fn record_malicious_site_blocked(
    domain: String,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.record_malicious_site_blocked(domain);
    Ok(())
}

/// Record data saved
#[tauri::command]
pub fn record_data_saved(
    bytes: u64,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.record_data_saved(bytes);
    Ok(())
}

/// Record time saved
#[tauri::command]
pub fn record_time_saved(
    seconds: u64,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.record_time_saved(seconds);
    Ok(())
}

/// Get privacy statistics
#[tauri::command]
pub fn get_privacy_stats(
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<PrivacyStats, String> {
    Ok(manager.get_stats())
}

/// Get privacy events
#[tauri::command]
pub fn get_privacy_events(
    limit: usize,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<Vec<PrivacyEvent>, String> {
    Ok(manager.get_events(limit))
}

/// Clear privacy events
#[tauri::command]
pub fn clear_privacy_events(
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.clear_events();
    Ok(())
}

/// Get privacy settings
#[tauri::command]
pub fn get_privacy_dashboard_settings(
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<PrivacySettings, String> {
    Ok(manager.get_settings())
}

/// Update privacy settings
#[tauri::command]
pub fn update_privacy_settings(
    settings: PrivacySettings,
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Reset privacy statistics
#[tauri::command]
pub fn reset_privacy_stats(
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<(), String> {
    manager.reset_stats()
        .map_err(|e| format!("Failed to reset stats: {}", e))
}

/// Get privacy score
#[tauri::command]
pub fn get_privacy_score(
    manager: State<'_, Arc<PrivacyDashboardManager>>,
) -> Result<u8, String> {
    Ok(manager.get_privacy_score())
}
