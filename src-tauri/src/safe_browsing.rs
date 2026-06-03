//! Safe Browsing Detection for Exodus Browser
//! 
//! This module provides malicious website detection and warning capabilities.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Threat type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThreatType {
    Malware,
    Phishing,
    UnwantedSoftware,
    SocialEngineering,
    DangerousContent,
}

impl ThreatType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "malware" => ThreatType::Malware,
            "phishing" => ThreatType::Phishing,
            "unwanted_software" => ThreatType::UnwantedSoftware,
            "social_engineering" => ThreatType::SocialEngineering,
            "dangerous_content" => ThreatType::DangerousContent,
            _ => ThreatType::DangerousContent,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            ThreatType::Malware => "malware",
            ThreatType::Phishing => "phishing",
            ThreatType::UnwantedSoftware => "unwanted_software",
            ThreatType::SocialEngineering => "social_engineering",
            ThreatType::DangerousContent => "dangerous_content",
        }
    }
}

/// Threat entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEntry {
    /// Unique identifier
    pub id: String,
    /// URL pattern
    pub url_pattern: String,
    /// Threat type
    pub threat_type: ThreatType,
    /// Severity (1-10)
    pub severity: u8,
    /// Timestamp added
    pub added_at: u64,
    /// Number of times blocked
    pub block_count: u64,
}

impl ThreatEntry {
    pub fn new(url_pattern: String, threat_type: ThreatType, severity: u8) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url_pattern,
            threat_type,
            severity,
            added_at: now,
            block_count: 0,
        }
    }
    
    pub fn matches(&self, url: &str) -> bool {
        let url_lower = url.to_lowercase();
        let pattern_lower = self.url_pattern.to_lowercase();
        
        url_lower.contains(&pattern_lower) || url_lower == pattern_lower
    }
    
    pub fn record_block(&mut self) {
        self.block_count += 1;
    }
}

/// Safe browsing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeBrowsingSettings {
    /// Enable safe browsing
    pub enabled: bool,
    /// Block malware sites
    pub block_malware: bool,
    /// Block phishing sites
    pub block_phishing: bool,
    /// Block unwanted software
    pub block_unwanted_software: bool,
    /// Show warnings instead of blocking
    pub show_warnings: bool,
    /// Allow user to proceed despite warning
    pub allow_proceed: bool,
    /// Optional remote threat list URL (JSON `{ "threats": [...] }`).
    #[serde(default)]
    pub list_url: Option<String>,
    /// Last successful online list refresh (unix seconds).
    #[serde(default)]
    pub last_list_refresh: u64,
}

impl Default for SafeBrowsingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            block_malware: true,
            block_phishing: true,
            block_unwanted_software: true,
            show_warnings: true,
            allow_proceed: true,
            list_url: None,
            last_list_refresh: 0,
        }
    }
}

/// Safe browsing manager
pub struct SafeBrowsingManager {
    threats: Arc<Mutex<HashMap<String, ThreatEntry>>>,
    settings: Arc<Mutex<SafeBrowsingSettings>>,
    blocked_domains: Arc<Mutex<HashSet<String>>>,
    storage_path: PathBuf,
}

impl SafeBrowsingManager {
    /// Create a new safe browsing manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            threats: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(SafeBrowsingSettings::default())),
            blocked_domains: Arc::new(Mutex::new(HashSet::new())),
            storage_path,
        };
        
        manager.load_default_threats()?;
        manager.load_threats_from_disk()?;
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Load default threat patterns
    fn load_default_threats(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_threats = vec![
            // Known phishing patterns
            ("login-update-account.com", ThreatType::Phishing, 8),
            ("secure-login-verify.com", ThreatType::Phishing, 8),
            ("account-verification-required.com", ThreatType::Phishing, 7),
            ("verify-your-identity.com", ThreatType::Phishing, 7),
            ("security-alert-notification.com", ThreatType::Phishing, 6),
            
            // Known malware patterns
            ("free-download-crack.com", ThreatType::Malware, 9),
            ("keygen-download.com", ThreatType::Malware, 9),
            ("cracked-software-free.com", ThreatType::Malware, 8),
            ("serial-number-generator.com", ThreatType::Malware, 8),
            
            // Unwanted software patterns
            ("free-pc-cleaner-download.com", ThreatType::UnwantedSoftware, 7),
            ("system-optimizer-free.com", ThreatType::UnwantedSoftware, 6),
            ("driver-update-free.com", ThreatType::UnwantedSoftware, 6),
            // Common phishing TLD patterns (local list; not a substitute for Google Safe Browsing)
            ("login-secure-verify.net", ThreatType::Phishing, 8),
            ("paypal-security-check.com", ThreatType::Phishing, 8),
            ("apple-id-locked.com", ThreatType::Phishing, 7),
            ("microsoft-account-alert.com", ThreatType::Phishing, 7),
            ("wallet-connect-verify.com", ThreatType::SocialEngineering, 8),
            ("metamask-sync.com", ThreatType::SocialEngineering, 8),
        ];
        
        let mut threats = self.threats.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for (pattern, threat_type, severity) in default_threats {
            let threat = ThreatEntry::new(pattern.to_string(), threat_type, severity);
            threats.insert(threat.id.clone(), threat);
        }
        
        Ok(())
    }
    
    /// Check if a URL is safe
    pub fn check_url(&self, url: &str) -> Option<ThreatEntry> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled {
            return None;
        }
        
        let threats = self.threats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        for threat in threats.values() {
            if threat.matches(url) {
                let should_block = match threat.threat_type {
                    ThreatType::Malware => settings.block_malware,
                    ThreatType::Phishing => settings.block_phishing,
                    ThreatType::UnwantedSoftware => settings.block_unwanted_software,
                    ThreatType::SocialEngineering => settings.block_phishing,
                    ThreatType::DangerousContent => true,
                };
                
                if should_block {
                    return Some(threat.clone());
                }
            }
        }
        
        None
    }
    
    /// Block a URL
    pub fn block_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(threat) = self.check_url(url) {
            let mut threats = self.threats.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if let Some(threat) = threats.get_mut(&threat.id) {
                threat.record_block();
            }
            
            let mut blocked = self.blocked_domains.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if let Some(domain) = Self::extract_domain(url) {
                blocked.insert(domain);
            }
            
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return Some(host.to_string());
            }
        }
        None
    }
    
    /// Get all threats
    pub fn get_threats(&self) -> Vec<ThreatEntry> {
        let threats = self.threats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        threats.values().cloned().collect()
    }
    
    /// Add a custom threat
    pub fn add_threat(&self, threat: ThreatEntry) -> Result<(), Box<dyn std::error::Error>> {
        let mut threats = self.threats.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        threats.insert(threat.id.clone(), threat);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a threat
    pub fn remove_threat(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut threats = self.threats.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        threats.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get safe browsing settings
    pub fn get_settings(&self) -> SafeBrowsingSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update safe browsing settings
    pub fn update_settings(&self, settings: SafeBrowsingSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get blocked domains
    pub fn get_blocked_domains(&self) -> HashSet<String> {
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        blocked.clone()
    }
    
    /// Clear blocked domains
    pub fn clear_blocked_domains(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_domains.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        blocked.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get blocking statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let threats = self.threats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        for threat in threats.values() {
            stats.insert(threat.threat_type.as_str().to_string(), threat.block_count);
        }
        
        stats
    }
    
    /// Refresh threat list from configured URL or explicit override.
    pub async fn refresh_online_list(&self, url_override: Option<String>) -> Result<usize, String> {
        let fetch_url = {
            let settings = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            url_override
                .filter(|u| u.starts_with("http"))
                .or_else(|| settings.list_url.clone())
        };
        let Some(fetch_url) = fetch_url else {
            return Err("No Safe Browsing list URL configured".to_string());
        };
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .map_err(|e| format!("HTTP client: {}", e))?;
        let response = client
            .get(&fetch_url)
            .send()
            .await
            .map_err(|e| format!("Fetch list: {}", e))?;
        if !response.status().is_success() {
            return Err(format!("List HTTP {}", response.status()));
        }
        let text = response
            .text()
            .await
            .map_err(|e| format!("Read body: {}", e))?;
        let count = self.merge_threats_json(&text)?;
        {
            let mut settings = self.settings.lock().map_err(|e| format!("Lock: {}", e))?;
            settings.last_list_refresh = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        self.save_to_disk()
            .map_err(|e| format!("Save settings: {}", e))?;
        self.save_threats_to_disk()
            .map_err(|e| format!("Save threats: {}", e))?;
        Ok(count)
    }

    fn merge_threats_json(&self, json: &str) -> Result<usize, String> {
        #[derive(Deserialize)]
        struct RemoteThreat {
            url_pattern: String,
            threat_type: String,
            #[serde(default = "default_severity")]
            severity: u8,
        }
        #[derive(Deserialize)]
        struct RemoteFile {
            threats: Vec<RemoteThreat>,
        }
        fn default_severity() -> u8 {
            7
        }
        let file: RemoteFile =
            serde_json::from_str(json).map_err(|e| format!("Parse threats JSON: {}", e))?;
        let mut threats = self.threats.lock().map_err(|e| format!("Lock: {}", e))?;
        let before = threats.len();
        for entry in file.threats {
            let threat = ThreatEntry::new(
                entry.url_pattern,
                ThreatType::from_str(&entry.threat_type),
                entry.severity,
            );
            threats.insert(threat.id.clone(), threat);
        }
        Ok(threats.len().saturating_sub(before))
    }

    fn threats_path(&self) -> PathBuf {
        self.storage_path.join("threats.json")
    }

    fn load_threats_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.threats_path();
        if !path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&path)?;
        let list: Vec<ThreatEntry> = serde_json::from_str(&content)?;
        let mut threats = self.threats.lock().map_err(|e| format!("Lock: {}", e))?;
        for t in list {
            threats.insert(t.id.clone(), t);
        }
        Ok(())
    }

    fn save_threats_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let threats = self.threats.lock().map_err(|e| format!("Lock: {}", e))?;
        let list: Vec<ThreatEntry> = threats.values().cloned().collect();
        let content = serde_json::to_string_pretty(&list)?;
        std::fs::write(self.threats_path(), content)?;
        Ok(())
    }

    /// Load settings from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("safe_browsing_settings.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let loaded: SafeBrowsingSettings = serde_json::from_str(&content)?;
            if let Ok(mut guard) = self.settings.lock() {
                *guard = loaded;
            }
        }
        
        Ok(())
    }
    
    /// Save settings to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("safe_browsing_settings.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Check if a URL is safe
#[tauri::command]
pub fn check_url_safe(
    url: String,
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<Option<ThreatEntry>, String> {
    Ok(manager.check_url(&url))
}

/// Block a URL
#[tauri::command]
pub fn block_malicious_url(
    url: String,
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<(), String> {
    manager.block_url(&url)
        .map_err(|e| format!("Failed to block URL: {}", e))
}

/// Get all threats
#[tauri::command]
pub fn get_threats(
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<Vec<ThreatEntry>, String> {
    Ok(manager.get_threats())
}

/// Add a custom threat
#[tauri::command]
pub fn add_threat(
    threat: ThreatEntry,
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<(), String> {
    manager.add_threat(threat)
        .map_err(|e| format!("Failed to add threat: {}", e))
}

/// Remove a threat
#[tauri::command]
pub fn remove_threat(
    id: String,
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<(), String> {
    manager.remove_threat(&id)
        .map_err(|e| format!("Failed to remove threat: {}", e))
}

/// Get safe browsing settings
#[tauri::command]
pub fn get_safe_browsing_settings(
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<SafeBrowsingSettings, String> {
    Ok(manager.get_settings())
}

/// Update safe browsing settings
#[tauri::command]
pub fn update_safe_browsing_settings(
    settings: SafeBrowsingSettings,
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get blocked domains
#[tauri::command]
pub fn get_safe_browsing_blocked_domains(
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_blocked_domains().into_iter().collect())
}

/// Clear blocked domains
#[tauri::command]
pub fn clear_safe_browsing_blocked_domains(
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<(), String> {
    manager.clear_blocked_domains()
        .map_err(|e| format!("Failed to clear blocked domains: {}", e))
}

/// Get blocking statistics
#[tauri::command]
pub fn get_safe_browsing_stats(
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}

/// Refresh Safe Browsing threat list from `list_url` in settings or `url` argument.
#[tauri::command]
pub async fn refresh_safe_browsing_list(
    url: Option<String>,
    manager: State<'_, Arc<SafeBrowsingManager>>,
) -> Result<usize, String> {
    manager.refresh_online_list(url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_url_blocks_known_phishing_pattern() {
        let dir = std::env::temp_dir().join(format!("exodus_sb_{}", uuid::Uuid::new_v4()));
        let mgr = SafeBrowsingManager::new(dir).expect("manager");
        let threat = mgr
            .check_url("https://login-update-account.com/signin")
            .expect("threat");
        assert_eq!(threat.threat_type, ThreatType::Phishing);
    }

    #[test]
    fn check_url_allows_benign_host() {
        let dir = std::env::temp_dir().join(format!("exodus_sb_ok_{}", uuid::Uuid::new_v4()));
        let mgr = SafeBrowsingManager::new(dir).expect("manager");
        assert!(mgr.check_url("https://example.com").is_none());
    }
}
