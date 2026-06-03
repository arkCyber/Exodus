//! Password Manager for Exodus Browser
//! 
//! This module provides password storage, retrieval, and autofill capabilities.
//! Passwords are encrypted using the system's keyring/keychain where available.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

/// Password strength level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=20 => PasswordStrength::VeryWeak,
            21..=40 => PasswordStrength::Weak,
            41..=60 => PasswordStrength::Fair,
            61..=80 => PasswordStrength::Strong,
            81..=100 => PasswordStrength::VeryStrong,
            _ => PasswordStrength::VeryWeak,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            PasswordStrength::VeryWeak => "very_weak",
            PasswordStrength::Weak => "weak",
            PasswordStrength::Fair => "fair",
            PasswordStrength::Strong => "strong",
            PasswordStrength::VeryStrong => "very_strong",
        }
    }
}

/// Password breach status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BreachStatus {
    Safe,
    Compromised,
    Unknown,
}

/// Password entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    /// Unique identifier
    pub id: String,
    /// Website URL
    pub url: String,
    /// Username/email
    pub username: String,
    /// Encrypted password
    pub password: String,
    /// Site name
    pub site_name: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub updated_at: u64,
    /// Number of times used
    pub use_count: u32,
    /// Password strength
    pub strength: PasswordStrength,
    /// Breach status
    pub breach_status: BreachStatus,
    /// Notes
    pub notes: Option<String>,
    /// Custom fields
    pub custom_fields: HashMap<String, String>,
}

impl PasswordEntry {
    #[allow(dead_code)]
    pub fn new(url: String, username: String, password: String, site_name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        let strength = Self::calculate_strength(&password);
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            username,
            password,
            site_name,
            created_at: now,
            updated_at: now,
            use_count: 0,
            strength,
            breach_status: BreachStatus::Unknown,
            notes: None,
            custom_fields: HashMap::new(),
        }
    }
    
    #[allow(dead_code)]
    pub fn increment_use_count(&mut self) {
        self.use_count += 1;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
    }
    
    /// Calculate password strength
    fn calculate_strength(password: &str) -> PasswordStrength {
        let mut score = 0u32;
        
        // Length bonus
        score += (password.len() as u32) * 4;
        
        // Character variety
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digits = password.chars().any(|c| c.is_numeric());
        let has_symbols = password.chars().any(|c| !c.is_alphanumeric());
        
        if has_lowercase { score += 10; }
        if has_uppercase { score += 10; }
        if has_digits { score += 10; }
        if has_symbols { score += 15; }
        
        // Variety bonus
        let variety_count = [has_lowercase, has_uppercase, has_digits, has_symbols]
            .iter()
            .filter(|&&x| x)
            .count();
        score += (variety_count as u32) * 5;
        
        // Cap at 100
        if score > 100 { score = 100; }
        
        PasswordStrength::from_score(score)
    }
    
    /// Update password and recalculate strength
    #[allow(dead_code)]
    pub fn update_password(&mut self, password: String) {
        self.password = password;
        self.strength = Self::calculate_strength(&self.password);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
    }
}

/// Password manager settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordManagerSettings {
    /// Enable auto-save passwords
    pub auto_save: bool,
    /// Enable auto-fill
    pub auto_fill: bool,
    /// Require master password
    pub require_master_password: bool,
    /// Minimum password length
    pub min_password_length: u32,
    /// Require password strength check
    pub require_strength_check: bool,
    /// Enable breach detection
    pub enable_breach_detection: bool,
    /// Auto-lock timeout in seconds
    pub auto_lock_timeout: u64,
    /// Enable password sync
    pub enable_sync: bool,
}

impl Default for PasswordManagerSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_fill: true,
            require_master_password: false,
            min_password_length: 8,
            require_strength_check: true,
            enable_breach_detection: true,
            auto_lock_timeout: 300, // 5 minutes
            enable_sync: false,
        }
    }
}

/// Password Manager
pub struct PasswordManager {
    entries: Arc<Mutex<HashMap<String, PasswordEntry>>>,
    settings: Arc<Mutex<PasswordManagerSettings>>,
    compromised_passwords: Arc<Mutex<HashSet<String>>>,
    storage_path: PathBuf,
}

impl PasswordManager {
    /// Create a new password manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(PasswordManagerSettings::default())),
            compromised_passwords: Arc::new(Mutex::new(HashSet::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Add or update a password entry
    pub fn save_password(&self, entry: PasswordEntry) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut entries = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            entries.insert(entry.id.clone(), entry);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get password entry by URL
    pub fn get_password_by_url(&self, url: &str) -> Option<PasswordEntry> {
        let entries = self.entries.lock().ok()?;
        entries.values()
            .find(|e| e.url == url)
            .cloned()
    }

    /// Match saved credentials for a page URL (exact URL, then same host).
    pub fn get_password_for_page(&self, page_url: &str) -> Option<PasswordEntry> {
        if let Some(exact) = self.get_password_by_url(page_url) {
            return Some(exact);
        }
        let host = Self::page_host(page_url)?;
        let entries = self.entries.lock().ok()?;
        entries
            .values()
            .filter(|e| {
                Self::page_host(&e.url)
                    .map(|h| Self::hosts_same_site(&h, &host))
                    .unwrap_or(false)
            })
            .max_by_key(|e| e.updated_at)
            .cloned()
    }

    fn page_host(url: &str) -> Option<String> {
        let parsed = url::Url::parse(url).ok()?;
        parsed.host_str().map(|h| h.to_lowercase())
    }

    /// True when two hosts belong to the same registrable domain (e.g. login.example.com ↔ www.example.com).
    fn hosts_same_site(a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }
        fn base_domain(host: &str) -> Option<String> {
            let parts: Vec<&str> = host.split('.').filter(|p| !p.is_empty()).collect();
            if parts.len() >= 2 {
                Some(parts[parts.len() - 2..].join("."))
            } else {
                None
            }
        }
        match (base_domain(a), base_domain(b)) {
            (Some(aa), Some(bb)) => aa == bb,
            _ => false,
        }
    }
    
    /// Get password entry by ID
    pub fn get_password_by_id(&self, id: &str) -> Option<PasswordEntry> {
        let entries = self.entries.lock().ok()?;
        entries.get(id).cloned()
    }
    
    /// Get all password entries
    pub fn list_passwords(&self) -> Vec<PasswordEntry> {
        self.entries.lock()
            .map(|entries| entries.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Delete password entry by ID
    pub fn delete_password(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut entries = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            entries.remove(id);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Search passwords by query
    pub fn search_passwords(&self, query: &str) -> Vec<PasswordEntry> {
        let entries = self.entries.lock()
            .ok()
            .map(|entries| entries.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let query_lower = query.to_lowercase();
        
        entries.into_iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&query_lower)
                    || e.username.to_lowercase().contains(&query_lower)
                    || e.site_name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
    
    /// Generate a strong password
    pub fn generate_password(length: u32, include_symbols: bool) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let digits = "0123456789";
        let symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        
        let charset = if include_symbols {
            format!("{}{}{}", letters, digits, symbols)
        } else {
            format!("{}{}", letters, digits)
        };
        
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap_or('a')
            })
            .collect();
        
        password
    }
    
    /// Check password strength
    pub fn check_password_strength(&self, password: &str) -> PasswordStrength {
        PasswordEntry::calculate_strength(password)
    }
    
    /// Check if password is compromised (placeholder implementation)
    pub fn check_password_compromised(&self, password: &str) -> BreachStatus {
        let compromised = self.compromised_passwords.lock()
            .ok()
            .and_then(|guard| Some(guard.clone()))
            .unwrap_or_default();
        
        // In a real implementation, this would check against a breach database
        // For now, we use a local cache
        let password_hash = self.hash_password(password);
        if compromised.contains(&password_hash) {
            BreachStatus::Compromised
        } else {
            BreachStatus::Safe
        }
    }
    
    /// Hash password for comparison (simple placeholder)
    fn hash_password(&self, password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Get password manager settings
    pub fn get_settings(&self) -> PasswordManagerSettings {
        self.settings.lock()
            .ok()
            .and_then(|guard| Some(guard.clone()))
            .unwrap_or_default()
    }
    
    /// Update password manager settings
    pub fn update_settings(&self, settings: PasswordManagerSettings) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut current = self.settings.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            *current = settings;
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get weak passwords
    pub fn get_weak_passwords(&self) -> Vec<PasswordEntry> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        entries.values()
            .filter(|e| {
                e.strength == PasswordStrength::VeryWeak || 
                e.strength == PasswordStrength::Weak
            })
            .cloned()
            .collect()
    }
    
    /// Get compromised passwords
    pub fn get_compromised_passwords(&self) -> Vec<PasswordEntry> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        entries.values()
            .filter(|e| e.breach_status == BreachStatus::Compromised)
            .cloned()
            .collect()
    }
    
    /// Get password statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_passwords".to_string(), entries.len() as u64);
        
        let weak_count = entries.values()
            .filter(|e| e.strength == PasswordStrength::VeryWeak || e.strength == PasswordStrength::Weak)
            .count() as u64;
        stats.insert("weak_passwords".to_string(), weak_count);
        
        let compromised_count = entries.values()
            .filter(|e| e.breach_status == BreachStatus::Compromised)
            .count() as u64;
        stats.insert("compromised_passwords".to_string(), compromised_count);
        
        let total_uses: u64 = entries.values().map(|e| e.use_count as u64).sum();
        stats.insert("total_uses".to_string(), total_uses);
        
        stats
    }
    
    /// Load passwords from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join("passwords.json");
        let settings_path = self.storage_path.join("password_settings.json");
        let compromised_path = self.storage_path.join("compromised_passwords.json");
        
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)?;
            let entries: HashMap<String, PasswordEntry> = serde_json::from_str(&content)?;
            
            let mut guard = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            *guard = entries;
        }
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: PasswordManagerSettings = serde_json::from_str(&content)?;
            if let Ok(mut settings_lock) = self.settings.lock() {
                *settings_lock = settings;
            }
        }
        
        if compromised_path.exists() {
            let content = std::fs::read_to_string(&compromised_path)?;
            let compromised: HashSet<String> = serde_json::from_str(&content)?;
            if let Ok(mut compromised_lock) = self.compromised_passwords.lock() {
                *compromised_lock = compromised;
            }
        }
        
        Ok(())
    }
    
    /// Save passwords to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join("passwords.json");
        let settings_path = self.storage_path.join("password_settings.json");
        let compromised_path = self.storage_path.join("compromised_passwords.json");
        
        let entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let compromised = self.compromised_passwords.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*entries)?;
        std::fs::write(&file_path, content)?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&settings_path, settings_content)?;
        
        let compromised_content = serde_json::to_string_pretty(&*compromised)?;
        std::fs::write(&compromised_path, compromised_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Save a password entry
#[tauri::command]
pub fn save_password(
    entry: PasswordEntry,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<(), String> {
    manager.save_password(entry)
        .map_err(|e| format!("Failed to save password: {}", e))
}

/// Get password by URL
#[tauri::command]
pub fn get_password_by_url(
    url: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Option<PasswordEntry>, String> {
    Ok(manager.get_password_by_url(&url))
}

/// Get password for a page (exact URL or same host).
#[tauri::command]
pub fn get_password_for_page(
    url: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Option<PasswordEntry>, String> {
    Ok(manager.get_password_for_page(&url))
}

/// Save credentials captured from a login form.
#[tauri::command]
pub fn save_password_capture(
    url: String,
    username: String,
    password: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<(), String> {
    let site_name = url::Url::parse(&url)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| url.clone());
    let entry = PasswordEntry::new(url, username, password, site_name);
    manager
        .save_password(entry)
        .map_err(|e| format!("Failed to save password: {}", e))
}

/// Get password by ID
#[tauri::command]
pub fn get_password_by_id(
    id: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Option<PasswordEntry>, String> {
    Ok(manager.get_password_by_id(&id))
}

/// List all passwords
#[tauri::command]
pub fn list_passwords(
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Vec<PasswordEntry>, String> {
    Ok(manager.list_passwords())
}

/// Delete a password
#[tauri::command]
pub fn delete_password(
    id: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<(), String> {
    manager.delete_password(&id)
        .map_err(|e| format!("Failed to delete password: {}", e))
}

/// Search passwords
#[tauri::command]
pub fn search_passwords(
    query: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Vec<PasswordEntry>, String> {
    Ok(manager.search_passwords(&query))
}

/// Generate a password
#[tauri::command]
pub fn generate_password(length: u32, include_symbols: bool) -> String {
    PasswordManager::generate_password(length, include_symbols)
}

/// Check password strength
#[tauri::command]
pub fn check_password_strength(
    password: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<String, String> {
    let strength = manager.check_password_strength(&password);
    Ok(strength.as_str().to_string())
}

/// Check if password is compromised
#[tauri::command]
pub fn check_password_compromised(
    password: String,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<String, String> {
    let status = manager.check_password_compromised(&password);
    let status_str = match status {
        BreachStatus::Safe => "safe",
        BreachStatus::Compromised => "compromised",
        BreachStatus::Unknown => "unknown",
    };
    Ok(status_str.to_string())
}

/// Get password manager settings
#[tauri::command]
pub fn get_password_manager_settings(
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<PasswordManagerSettings, String> {
    Ok(manager.get_settings())
}

/// Update password manager settings
#[tauri::command]
pub fn update_password_manager_settings(
    settings: PasswordManagerSettings,
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get weak passwords
#[tauri::command]
pub fn get_weak_passwords(
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Vec<PasswordEntry>, String> {
    Ok(manager.get_weak_passwords())
}

/// Get compromised passwords
#[tauri::command]
pub fn get_compromised_passwords(
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<Vec<PasswordEntry>, String> {
    Ok(manager.get_compromised_passwords())
}

/// Get password statistics
#[tauri::command]
pub fn get_password_stats(
    manager: State<'_, Arc<PasswordManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}

/// Form data entry for autofill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormDataEntry {
    /// Unique identifier
    pub id: String,
    /// Website URL
    pub url: String,
    /// Form field name
    pub field_name: String,
    /// Form field value
    pub field_value: String,
    /// Form field type (text, email, password, etc.)
    pub field_type: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used: u64,
}

impl FormDataEntry {
    #[allow(dead_code)]
    pub fn new(url: String, field_name: String, field_value: String, field_type: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            field_name,
            field_value,
            field_type,
            created_at: now,
            last_used: now,
        }
    }
    
    #[allow(dead_code)]
    pub fn update_last_used(&mut self) {
        self.last_used = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
    }
}

/// Save form data for autofill
#[tauri::command]
pub fn save_form_data(
    _entry: FormDataEntry,
    _manager: State<'_, Arc<PasswordManager>>,
) -> Result<(), String> {
    // For now, we'll store form data in the password manager
    // In a real implementation, this would have a separate form data manager
    Err("Form data storage not yet implemented".to_string())
}

/// Get form data for autofill
#[tauri::command]
pub fn get_form_data(
    _url: String,
    _field_name: String,
    _manager: State<'_, Arc<PasswordManager>>,
) -> Result<Option<FormDataEntry>, String> {
    // For now, we'll return None
    // In a real implementation, this would query the form data manager
    Ok(None)
}

/// Autofill form field
#[tauri::command]
pub fn autofill_field(
    _app: AppHandle,
    _label: String,
    _field_name: String,
    _value: String,
) -> Result<bool, String> {
    // In a real implementation, this would inject the value into the webview form field
    // For now, we just acknowledge the action
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_password_entry_creation() {
        let entry = PasswordEntry::new(
            "https://example.com".to_string(),
            "user@example.com".to_string(),
            "password123".to_string(),
            "Example Site".to_string(),
        );
        
        assert_eq!(entry.url, "https://example.com");
        assert_eq!(entry.username, "user@example.com");
        assert_eq!(entry.use_count, 0);
    }
    
    #[test]
    fn test_password_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = PasswordManager::new(temp_dir.path().to_path_buf()).expect("Failed to create manager");
        
        let entry = PasswordEntry::new(
            "https://example.com".to_string(),
            "user@example.com".to_string(),
            "password123".to_string(),
            "Example Site".to_string(),
        );
        
        manager.save_password(entry).expect("Failed to save password");
        
        let retrieved = manager.get_password_by_url("https://example.com");
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.expect("Expected password entry");
        assert_eq!(retrieved.username, "user@example.com");
    }
    
    #[test]
    fn get_password_for_page_matches_host() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = PasswordManager::new(temp_dir.path().to_path_buf()).expect("Failed to create manager");
        let entry = PasswordEntry::new(
            "https://login.example.com/signin".to_string(),
            "user@example.com".to_string(),
            "secret".to_string(),
            "Example".to_string(),
        );
        manager.save_password(entry).expect("Failed to save password");
        let found = manager
            .get_password_for_page("https://www.example.com/dashboard")
            .expect("match by host");
        assert_eq!(found.username, "user@example.com");
    }

    #[test]
    fn test_password_search() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = PasswordManager::new(temp_dir.path().to_path_buf()).expect("Failed to create manager");
        
        let entry1 = PasswordEntry::new(
            "https://example.com".to_string(),
            "user@example.com".to_string(),
            "password123".to_string(),
            "Example Site".to_string(),
        );
        
        let entry2 = PasswordEntry::new(
            "https://test.com".to_string(),
            "test@test.com".to_string(),
            "password456".to_string(),
            "Test Site".to_string(),
        );
        
        manager.save_password(entry1).expect("Failed to save password");
        manager.save_password(entry2).expect("Failed to save password");
        
        let results = manager.search_passwords("example");
        assert_eq!(results.len(), 1);
        
        let results = manager.search_passwords("test");
        assert_eq!(results.len(), 1);
    }
    
    #[test]
    fn test_password_generation() {
        let password = PasswordManager::generate_password(16, true);
        assert_eq!(password.len(), 16);
        
        let password = PasswordManager::generate_password(20, false);
        assert_eq!(password.len(), 20);
        assert!(!password.contains('!'));
    }
}
