//! User Script Support for Exodus Browser
//! 
//! This module provides user script (Tampermonkey/Greasemonkey style) support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// User script metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScriptMetadata {
    /// Script name
    pub name: String,
    /// Script version
    pub version: String,
    /// Script description
    pub description: Option<String>,
    /// Script author
    pub author: Option<String>,
    /// Script namespace
    pub namespace: Option<String>,
    /// Script homepage
    pub homepage: Option<String>,
    /// Script icon
    pub icon: Option<String>,
    /// Update URL
    pub update_url: Option<String>,
    /// Download URL
    pub download_url: Option<String>,
    /// Support URL
    pub support_url: Option<String>,
}

/// User script match pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPattern {
    /// Include pattern
    pub include: Vec<String>,
    /// Exclude pattern
    pub exclude: Vec<String>,
    /// Match pattern
    pub match_pattern: Vec<String>,
}

/// User script grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserScriptGrant {
    None,
    GMGetValue(String),
    GMSetValue(String),
    GMDeleteValue(String),
    GMListValues,
    GMAddStyle,
    GMDeleteStyle,
    GMNotification,
    GMXmlHttpRequest,
    GMOpenInTab,
    GMSetClipboard,
    GMGetResourceText,
}

/// User script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScript {
    /// Script ID
    pub id: String,
    /// Script metadata
    pub metadata: UserScriptMetadata,
    /// Script content
    pub content: String,
    /// Match patterns
    pub match_patterns: MatchPattern,
    /// Run at document start
    pub run_at: String,
    /// Grants required
    pub grants: Vec<UserScriptGrant>,
    /// Resources
    pub resources: HashMap<String, String>,
    /// Is enabled
    pub enabled: bool,
    /// Created timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub updated_at: u64,
    /// Last update check timestamp
    pub last_update_check: Option<u64>,
}

impl UserScript {
    pub fn new(name: String, content: String, include_patterns: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: UserScriptMetadata {
                name,
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                namespace: None,
                homepage: None,
                icon: None,
                update_url: None,
                download_url: None,
                support_url: None,
            },
            content,
            match_patterns: MatchPattern {
                include: include_patterns,
                exclude: vec![],
                match_pattern: vec![],
            },
            run_at: "document-end".to_string(),
            grants: vec![],
            resources: HashMap::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
            last_update_check: None,
        }
    }
    
    /// Check if this script matches a URL
    pub fn matches_url(&self, url: &str) -> bool {
        // Check include patterns
        for pattern in &self.match_patterns.include {
            if self.matches_pattern(url, pattern) {
                // Check exclude patterns
                let excluded = self.match_patterns.exclude.iter()
                    .any(|p| self.matches_pattern(url, p));
                if !excluded {
                    return true;
                }
            }
        }
        
        // Check match patterns
        for pattern in &self.match_patterns.match_pattern {
            if self.matches_pattern(url, pattern) {
                let excluded = self.match_patterns.exclude.iter()
                    .any(|p| self.matches_pattern(url, p));
                if !excluded {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn matches_pattern(&self, url: &str, pattern: &str) -> bool {
        // Simple wildcard matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return url.starts_with(prefix) && url.ends_with(suffix);
            }
        }
        
        url == pattern || url.starts_with(pattern)
    }
}

/// User script settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScriptSettings {
    /// Enable user scripts
    pub enable_scripts: bool,
    /// Auto-update scripts
    pub auto_update: bool,
    /// Update check interval in hours
    pub update_interval: u64,
    /// Allow unsafe grants
    pub allow_unsafe_grants: bool,
    /// Script execution timeout in ms
    pub execution_timeout: u64,
}

impl Default for UserScriptSettings {
    fn default() -> Self {
        Self {
            enable_scripts: true,
            auto_update: false,
            update_interval: 24,
            allow_unsafe_grants: false,
            execution_timeout: 10000,
        }
    }
}

/// User script manager
pub struct UserScriptManager {
    scripts: Arc<Mutex<HashMap<String, UserScript>>>,
    settings: Arc<Mutex<UserScriptSettings>>,
    script_values: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    storage_path: PathBuf,
}

impl UserScriptManager {
    /// Create a new user script manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            scripts: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(UserScriptSettings::default())),
            script_values: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Add a user script
    pub fn add_script(&self, script: UserScript) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        scripts.insert(script.id.clone(), script);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a user script
    pub fn remove_script(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        scripts.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get a user script by ID
    pub fn get_script(&self, id: &str) -> Option<UserScript> {
        let scripts = self.scripts.lock().ok()?;
        scripts.get(id).cloned()
    }
    
    /// Get all user scripts
    pub fn get_all_scripts(&self) -> Vec<UserScript> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        scripts.values().cloned().collect()
    }
    
    /// Get enabled user scripts
    pub fn get_enabled_scripts(&self) -> Vec<UserScript> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        scripts.values()
            .filter(|s| s.enabled)
            .cloned()
            .collect()
    }
    
    /// Update a user script
    pub fn update_script(&self, id: String, script: UserScript) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if scripts.contains_key(&id) {
            scripts.insert(id, script);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Enable a user script
    pub fn enable_script(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(script) = scripts.get_mut(id) {
            script.enabled = true;
            script.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Disable a user script
    pub fn disable_script(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(script) = scripts.get_mut(id) {
            script.enabled = false;
            script.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get scripts that should run for a URL
    pub fn get_scripts_for_url(&self, url: &str) -> Vec<UserScript> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        scripts.values()
            .filter(|s| s.enabled && s.matches_url(url))
            .cloned()
            .collect()
    }
    
    /// Set script value
    pub fn set_script_value(&self, script_id: String, key: String, value: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut values = self.script_values.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        values.entry(script_id).or_insert_with(HashMap::new).insert(key, value);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get script value
    pub fn get_script_value(&self, script_id: &str, key: &str) -> Option<String> {
        let values = self.script_values.lock().ok()?;
        values.get(script_id)?.get(key).cloned()
    }
    
    /// Delete script value
    pub fn delete_script_value(&self, script_id: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut values = self.script_values.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(script_values) = values.get_mut(script_id) {
            script_values.remove(key);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// List script values
    pub fn list_script_values(&self, script_id: &str) -> HashMap<String, String> {
        let values = self.script_values.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        values.get(script_id).cloned().unwrap_or_default()
    }
    
    /// Get settings
    pub fn get_settings(&self) -> UserScriptSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update settings
    pub fn update_settings(&self, settings: UserScriptSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Import script from file
    pub fn import_script(&self, content: String) -> Result<String, Box<dyn std::error::Error>> {
        // Parse user script metadata from content
        let name = Self::parse_script_name(&content);
        let include_patterns = Self::parse_include_patterns(&content);
        
        let script = UserScript::new(name, content, include_patterns);
        let script_id = script.id.clone();
        
        self.add_script(script)?;
        Ok(script_id)
    }
    
    /// Export script to string
    pub fn export_script(&self, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let script = self.get_script(id)
            .ok_or("Script not found")?;
        Ok(script.content)
    }
    
    fn parse_script_name(content: &str) -> String {
        // Simple parsing of @name directive
        for line in content.lines() {
            if line.trim().starts_with("// @name") {
                if let Some(name) = line.splitn(2, "@name").nth(1) {
                    return name.trim().to_string();
                }
            }
        }
        "Unnamed Script".to_string()
    }
    
    fn parse_include_patterns(content: &str) -> Vec<String> {
        let mut patterns = vec![];
        
        for line in content.lines() {
            if line.trim().starts_with("// @include") {
                if let Some(pattern) = line.splitn(2, "@include").nth(1) {
                    patterns.push(pattern.trim().to_string());
                }
            }
        }
        
        if patterns.is_empty() {
            patterns.push("*".to_string());
        }
        
        patterns
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let scripts_path = self.storage_path.join("user_scripts.json");
        let settings_path = self.storage_path.join("user_script_settings.json");
        let values_path = self.storage_path.join("user_script_values.json");
        
        if scripts_path.exists() {
            let content = std::fs::read_to_string(&scripts_path)?;
            let scripts: HashMap<String, UserScript> = serde_json::from_str(&content)?;
            *self.scripts.lock().unwrap() = scripts;
        }
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: UserScriptSettings = serde_json::from_str(&content)?;
            *self.settings.lock().unwrap() = settings;
        }
        
        if values_path.exists() {
            let content = std::fs::read_to_string(&values_path)?;
            let values: HashMap<String, HashMap<String, String>> = serde_json::from_str(&content)?;
            *self.script_values.lock().unwrap() = values;
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let scripts_path = self.storage_path.join("user_scripts.json");
        let settings_path = self.storage_path.join("user_script_settings.json");
        let values_path = self.storage_path.join("user_script_values.json");
        
        let scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let values = self.script_values.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let scripts_content = serde_json::to_string_pretty(&*scripts)?;
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let values_content = serde_json::to_string_pretty(&*values)?;
        
        std::fs::write(&scripts_path, scripts_content)?;
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&values_path, values_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add user script
#[tauri::command]
pub fn add_user_script(
    script: UserScript,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.add_script(script)
        .map_err(|e| format!("Failed to add script: {}", e))
}

/// Remove user script
#[tauri::command]
pub fn remove_user_script(
    id: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.remove_script(&id)
        .map_err(|e| format!("Failed to remove script: {}", e))
}

/// Get user script
#[tauri::command]
pub fn get_user_script(
    id: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<Option<UserScript>, String> {
    Ok(manager.get_script(&id))
}

/// Get all user scripts
#[tauri::command]
pub fn get_all_user_scripts(
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<Vec<UserScript>, String> {
    Ok(manager.get_all_scripts())
}

/// Get enabled user scripts
#[tauri::command]
pub fn get_enabled_user_scripts(
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<Vec<UserScript>, String> {
    Ok(manager.get_enabled_scripts())
}

/// Update user script
#[tauri::command]
pub fn update_user_script(
    id: String,
    script: UserScript,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.update_script(id, script)
        .map_err(|e| format!("Failed to update script: {}", e))
}

/// Enable user script
#[tauri::command]
pub fn enable_user_script(
    id: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.enable_script(&id)
        .map_err(|e| format!("Failed to enable script: {}", e))
}

/// Disable user script
#[tauri::command]
pub fn disable_user_script(
    id: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.disable_script(&id)
        .map_err(|e| format!("Failed to disable script: {}", e))
}

/// Get scripts for URL
#[tauri::command]
pub fn get_user_scripts_for_url(
    url: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<Vec<UserScript>, String> {
    Ok(manager.get_scripts_for_url(&url))
}

/// Set script value
#[tauri::command]
pub fn set_user_script_value(
    script_id: String,
    key: String,
    value: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.set_script_value(script_id, key, value)
        .map_err(|e| format!("Failed to set value: {}", e))
}

/// Get script value
#[tauri::command]
pub fn get_user_script_value(
    script_id: String,
    key: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<Option<String>, String> {
    Ok(manager.get_script_value(&script_id, &key))
}

/// Delete script value
#[tauri::command]
pub fn delete_user_script_value(
    script_id: String,
    key: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.delete_script_value(&script_id, &key)
        .map_err(|e| format!("Failed to delete value: {}", e))
}

/// List script values
#[tauri::command]
pub fn list_user_script_values(
    script_id: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<HashMap<String, String>, String> {
    Ok(manager.list_script_values(&script_id))
}

/// Import user script
#[tauri::command]
pub fn import_user_script(
    content: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<String, String> {
    manager.import_script(content)
        .map_err(|e| format!("Failed to import script: {}", e))
}

/// Export user script
#[tauri::command]
pub fn export_user_script(
    id: String,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<String, String> {
    manager.export_script(&id)
        .map_err(|e| format!("Failed to export script: {}", e))
}

/// Get user script settings
#[tauri::command]
pub fn get_user_script_settings(
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<UserScriptSettings, String> {
    Ok(manager.get_settings())
}

/// Update user script settings
#[tauri::command]
pub fn update_user_script_settings(
    settings: UserScriptSettings,
    manager: State<'_, Arc<UserScriptManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}
