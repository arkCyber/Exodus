//! Cookie Manager for Exodus Browser
//! 
//! This module provides cookie storage, retrieval, and management capabilities.

use crate::config::ConfigState;
use crate::profile_stores::ProfileCookieStores;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

fn active_cookies<'a>(
    stores: &'a ProfileCookieStores,
    config: &ConfigState,
) -> Result<&'a Arc<CookieManager>, String> {
    let private = config
        .lock()
        .map_err(|e| format!("Config lock: {}", e))?
        .private_mode;
    Ok(stores.active(private))
}

/// Cookie entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieEntry {
    /// Unique identifier
    pub id: String,
    /// Domain
    pub domain: String,
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Path
    pub path: String,
    /// Expiration timestamp (0 for session cookie)
    pub expires: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Secure flag
    pub secure: bool,
    /// HttpOnly flag
    pub http_only: bool,
    /// SameSite attribute
    pub same_site: String,
}

impl CookieEntry {
    #[allow(dead_code)]
    pub fn new(
        domain: String,
        name: String,
        value: String,
        path: String,
        expires: u64,
        secure: bool,
        http_only: bool,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            domain,
            name,
            value,
            path,
            expires,
            created_at: now,
            last_accessed: now,
            secure,
            http_only,
            same_site: "Lax".to_string(),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if self.expires == 0 {
            return false; // Session cookie
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        self.expires < now
    }
    
    #[allow(dead_code)]
    pub fn update_access(&mut self) {
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
    }
}

/// Cookie manager settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieManagerSettings {
    /// Enable cookies
    pub enable_cookies: bool,
    /// Block third-party cookies
    pub block_third_party: bool,
    /// Clear cookies on exit
    pub clear_on_exit: bool,
    /// Cookie retention period in days (0 = forever)
    pub retention_days: u32,
    /// Enable cookie blocking
    pub enable_blocking: bool,
    /// Default cookie policy
    pub default_policy: String,
}

impl Default for CookieManagerSettings {
    fn default() -> Self {
        Self {
            enable_cookies: true,
            block_third_party: false,
            clear_on_exit: false,
            retention_days: 90,
            enable_blocking: false,
            default_policy: "allow".to_string(),
        }
    }
}

/// Cookie Manager
pub struct CookieManager {
    cookies: Arc<Mutex<HashMap<String, CookieEntry>>>,
    settings: Arc<Mutex<CookieManagerSettings>>,
    blocked_domains: Arc<Mutex<HashSet<String>>>,
    storage_path: PathBuf,
}

impl CookieManager {
    /// Create a new cookie manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            cookies: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(CookieManagerSettings::default())),
            blocked_domains: Arc::new(Mutex::new(HashSet::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        manager.cleanup_expired_cookies()?;
        Ok(manager)
    }
    
    /// Set a cookie
    pub fn set_cookie(&self, cookie: CookieEntry) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut cookies = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            cookies.insert(cookie.id.clone(), cookie);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get cookies for a domain
    pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<CookieEntry> {
        let cookies = self.cookies.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        cookies.values()
            .filter(|c| {
                c.domain == domain || c.domain.ends_with(&format!(".{}", domain))
            })
            .filter(|c| !c.is_expired())
            .cloned()
            .collect()
    }
    
    /// Get cookie by ID
    pub fn get_cookie_by_id(&self, id: &str) -> Option<CookieEntry> {
        let cookies = self.cookies.lock().ok()?;
        cookies.get(id).cloned()
    }
    
    /// Get all cookies
    pub fn list_cookies(&self) -> Vec<CookieEntry> {
        let cookies = self.cookies.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        cookies.values()
            .filter(|c| !c.is_expired())
            .cloned()
            .collect()
    }
    
    /// Delete cookie by ID
    pub fn delete_cookie(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut cookies = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            cookies.remove(id);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Delete all cookies for a domain
    pub fn delete_cookies_for_domain(&self, domain: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let count = {
            let mut cookies = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            let mut count = 0;
            cookies.retain(|_id, c| {
                let should_delete =
                    c.domain == domain || c.domain.ends_with(&format!(".{}", domain));
                if should_delete {
                    count += 1;
                }
                !should_delete
            });
            count
        };
        self.save_to_disk()?;
        Ok(count)
    }
    
    /// Delete all cookies
    pub fn delete_all_cookies(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut cookies = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            cookies.clear();
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Search cookies by query
    pub fn search_cookies(&self, query: &str) -> Vec<CookieEntry> {
        let cookies = self.cookies.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        let query_lower = query.to_lowercase();
        
        cookies.values()
            .filter(|c| !c.is_expired())
            .filter(|c| {
                c.domain.to_lowercase().contains(&query_lower)
                    || c.name.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
    
    /// Clean up expired cookies
    pub fn cleanup_expired_cookies(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let count = {
            let mut cookies = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            let mut count = 0;
            cookies.retain(|_, c| {
                let expired = c.is_expired();
                if expired {
                    count += 1;
                }
                !expired
            });
            count
        };
        if count > 0 {
            self.save_to_disk()?;
        }
        Ok(count)
    }
    
    /// Get cookie manager settings
    pub fn get_settings(&self) -> CookieManagerSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update cookie manager settings
    pub fn update_settings(&self, settings: CookieManagerSettings) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut current = self.settings.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            *current = settings;
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Block a domain
    pub fn block_domain(&self, domain: String) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut blocked = self.blocked_domains.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            blocked.insert(domain);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Unblock a domain
    pub fn unblock_domain(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut blocked = self.blocked_domains.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            blocked.remove(domain);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Check if a domain is blocked
    pub fn is_domain_blocked(&self, domain: &str) -> bool {
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        blocked.contains(domain) || blocked.iter().any(|d| domain.ends_with(&format!(".{}", d)))
    }
    
    /// Get all blocked domains
    pub fn get_blocked_domains(&self) -> Vec<String> {
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        blocked.iter().cloned().collect()
    }
    
    /// Get cookie statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let cookies = self.cookies.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_cookies".to_string(), cookies.len() as u64);
        stats.insert("blocked_domains".to_string(), blocked.len() as u64);
        
        let session_cookies = cookies.values().filter(|c| c.expires == 0).count() as u64;
        stats.insert("session_cookies".to_string(), session_cookies);
        
        let secure_cookies = cookies.values().filter(|c| c.secure).count() as u64;
        stats.insert("secure_cookies".to_string(), secure_cookies);
        
        let http_only_cookies = cookies.values().filter(|c| c.http_only).count() as u64;
        stats.insert("http_only_cookies".to_string(), http_only_cookies);
        
        stats
    }
    
    /// Export cookies to JSON
    pub fn export_cookies(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cookies = self.cookies.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*cookies)?;
        Ok(content)
    }
    
    /// Import cookies from JSON
    pub fn import_cookies(&self, json: String) -> Result<usize, Box<dyn std::error::Error>> {
        let imported: HashMap<String, CookieEntry> = serde_json::from_str(&json)?;
        let count = {
            let mut cookies = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            let count = imported.len();
            for (id, cookie) in imported {
                cookies.insert(id, cookie);
            }
            count
        };
        self.save_to_disk()?;
        Ok(count)
    }
    
    /// Load cookies from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join("cookies.json");
        let settings_path = self.storage_path.join("cookie_settings.json");
        let blocked_path = self.storage_path.join("blocked_domains.json");
        
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)?;
            let cookies: HashMap<String, CookieEntry> = serde_json::from_str(&content)?;
            
            let mut guard = self.cookies.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            *guard = cookies;
        }
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: CookieManagerSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if blocked_path.exists() {
            let content = std::fs::read_to_string(&blocked_path)?;
            let blocked: HashSet<String> = serde_json::from_str(&content)?;
            if let Ok(mut b) = self.blocked_domains.lock() {
                *b = blocked;
            }
        }
        
        Ok(())
    }
    
    /// Save cookies to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join("cookies.json");
        let settings_path = self.storage_path.join("cookie_settings.json");
        let blocked_path = self.storage_path.join("blocked_domains.json");
        
        let cookies = self.cookies.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let blocked = self.blocked_domains.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*cookies)?;
        std::fs::write(&file_path, content)?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&settings_path, settings_content)?;
        
        let blocked_content = serde_json::to_string_pretty(&*blocked)?;
        std::fs::write(&blocked_path, blocked_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Set a cookie
#[tauri::command]
pub fn set_cookie(
    cookie: CookieEntry,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_cookies(&stores, &config)?
        .set_cookie(cookie)
        .map_err(|e| format!("Failed to set cookie: {}", e))
}

/// Get cookies for a domain
#[tauri::command]
pub fn get_cookies_for_domain(
    domain: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<CookieEntry>, String> {
    Ok(active_cookies(&stores, &config)?.get_cookies_for_domain(&domain))
}

/// Get cookie by ID
#[tauri::command]
pub fn get_cookie_by_id(
    id: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Option<CookieEntry>, String> {
    Ok(active_cookies(&stores, &config)?.get_cookie_by_id(&id))
}

/// List all cookies
#[tauri::command]
pub fn list_cookies(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<CookieEntry>, String> {
    Ok(active_cookies(&stores, &config)?.list_cookies())
}

/// Delete a cookie
#[tauri::command]
pub fn delete_cookie(
    id: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_cookies(&stores, &config)?
        .delete_cookie(&id)
        .map_err(|e| format!("Failed to delete cookie: {}", e))
}

/// Delete cookies for a domain
#[tauri::command]
pub fn delete_cookies_for_domain(
    domain: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<usize, String> {
    active_cookies(&stores, &config)?
        .delete_cookies_for_domain(&domain)
        .map_err(|e| format!("Failed to delete cookies: {}", e))
}

/// Delete all cookies
#[tauri::command]
pub fn delete_all_cookies(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_cookies(&stores, &config)?
        .delete_all_cookies()
        .map_err(|e| format!("Failed to delete all cookies: {}", e))
}

/// Search cookies
#[tauri::command]
pub fn search_cookies(
    query: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<CookieEntry>, String> {
    Ok(active_cookies(&stores, &config)?.search_cookies(&query))
}

/// Cleanup expired cookies
#[tauri::command]
pub fn cleanup_expired_cookies(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<usize, String> {
    active_cookies(&stores, &config)?
        .cleanup_expired_cookies()
        .map_err(|e| format!("Failed to cleanup cookies: {}", e))
}

/// Get cookie manager settings
#[tauri::command]
pub fn get_cookie_manager_settings(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<CookieManagerSettings, String> {
    Ok(active_cookies(&stores, &config)?.get_settings())
}

/// Update cookie manager settings
#[tauri::command]
pub fn update_cookie_manager_settings(
    settings: CookieManagerSettings,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_cookies(&stores, &config)?
        .update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Block a domain
#[tauri::command]
pub fn block_cookie_domain(
    domain: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_cookies(&stores, &config)?
        .block_domain(domain)
        .map_err(|e| format!("Failed to block domain: {}", e))
}

/// Unblock a domain
#[tauri::command]
pub fn unblock_cookie_domain(
    domain: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    active_cookies(&stores, &config)?
        .unblock_domain(&domain)
        .map_err(|e| format!("Failed to unblock domain: {}", e))
}

/// Check if a domain is blocked
#[tauri::command]
pub fn is_domain_blocked(
    domain: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<bool, String> {
    Ok(active_cookies(&stores, &config)?.is_domain_blocked(&domain))
}

/// Get all blocked domains
#[tauri::command]
pub fn get_blocked_cookie_domains(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<String>, String> {
    Ok(active_cookies(&stores, &config)?.get_blocked_domains())
}

/// Get cookie statistics
#[tauri::command]
pub fn get_cookie_stats(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<HashMap<String, u64>, String> {
    Ok(active_cookies(&stores, &config)?.get_stats())
}

/// Export cookies
#[tauri::command]
pub fn export_cookies(
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<String, String> {
    active_cookies(&stores, &config)?
        .export_cookies()
        .map_err(|e| format!("Failed to export cookies: {}", e))
}

/// Import cookies
#[tauri::command]
pub fn import_cookies(
    json: String,
    stores: State<'_, ProfileCookieStores>,
    config: State<'_, ConfigState>,
) -> Result<usize, String> {
    active_cookies(&stores, &config)?
        .import_cookies(json)
        .map_err(|e| format!("Failed to import cookies: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cookie_entry_creation() {
        let cookie = CookieEntry::new(
            "example.com".to_string(),
            "session".to_string(),
            "abc123".to_string(),
            "/".to_string(),
            0,
            true,
            false,
        );
        
        assert_eq!(cookie.domain, "example.com");
        assert_eq!(cookie.name, "session");
        assert!(!cookie.is_expired());
    }
    
    #[test]
    fn test_cookie_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = CookieManager::new(temp_dir.path().to_path_buf()).expect("Failed to create CookieManager");
        
        let cookie = CookieEntry::new(
            "example.com".to_string(),
            "session".to_string(),
            "abc123".to_string(),
            "/".to_string(),
            0,
            true,
            false,
        );
        
        manager.set_cookie(cookie).expect("Failed to set cookie");
        
        let cookies = manager.get_cookies_for_domain("example.com");
        assert_eq!(cookies.len(), 1);
        
        assert_eq!(cookies[0].name, "session");
    }
    
    #[test]
    fn test_cookie_expiration() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = CookieManager::new(temp_dir.path().to_path_buf()).expect("Failed to create CookieManager");
        
        let expired_cookie = CookieEntry::new(
            "example.com".to_string(),
            "expired".to_string(),
            "value".to_string(),
            "/".to_string(),
            1000000, // Past timestamp
            true,
            false,
        );
        
        manager.set_cookie(expired_cookie).expect("Failed to set cookie");
        
        let cookies = manager.list_cookies();
        assert_eq!(cookies.len(), 0);
    }
    
    #[test]
    fn test_delete_cookies_for_domain() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = CookieManager::new(temp_dir.path().to_path_buf()).expect("Failed to create CookieManager");
        
        let cookie1 = CookieEntry::new(
            "example.com".to_string(),
            "session1".to_string(),
            "value1".to_string(),
            "/".to_string(),
            0,
            true,
            false,
        );
        
        let cookie2 = CookieEntry::new(
            "test.com".to_string(),
            "session2".to_string(),
            "value2".to_string(),
            "/".to_string(),
            0,
            true,
            false,
        );
        
        manager.set_cookie(cookie1).expect("Failed to set cookie");
        manager.set_cookie(cookie2).expect("Failed to set cookie");
        
        let count = manager.delete_cookies_for_domain("example.com").expect("Failed to delete cookies");
        assert_eq!(count, 1);
    }
}
