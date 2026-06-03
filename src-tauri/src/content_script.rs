//! Content Script Injection for Exodus Browser
//! 
//! This module provides content script injection and management capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Content script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScript {
    /// Script ID
    pub id: String,
    /// Script name
    pub name: String,
    /// Script content (JavaScript)
    pub content: String,
    /// CSS content
    pub css_content: Option<String>,
    /// URL patterns to match
    pub url_patterns: Vec<String>,
    /// Run at document start
    pub run_at_start: bool,
    /// Run at document end
    pub run_at_end: bool,
    /// Run on document idle
    pub run_at_idle: bool,
    /// Is enabled
    pub enabled: bool,
    /// Created timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub updated_at: u64,
}

impl ContentScript {
    #[allow(dead_code)]
    pub fn new(name: String, content: String, url_patterns: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            content,
            css_content: None,
            url_patterns,
            run_at_start: false,
            run_at_end: true,
            run_at_idle: false,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if this script matches a URL
    pub fn matches_url(&self, url: &str) -> bool {
        for pattern in &self.url_patterns {
            if self.matches_pattern(url, pattern) {
                return true;
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

/// Content script settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScriptSettings {
    /// Enable content scripts
    pub enable_scripts: bool,
    /// Enable CSS injection
    pub enable_css: bool,
    /// Allow inline scripts
    pub allow_inline: bool,
    /// Script execution timeout in ms
    pub execution_timeout: u64,
}

impl Default for ContentScriptSettings {
    fn default() -> Self {
        Self {
            enable_scripts: true,
            enable_css: true,
            allow_inline: false,
            execution_timeout: 5000,
        }
    }
}

/// Content script manager
pub struct ContentScriptManager {
    scripts: Arc<Mutex<HashMap<String, ContentScript>>>,
    settings: Arc<Mutex<ContentScriptSettings>>,
    storage_path: PathBuf,
}

impl ContentScriptManager {
    /// Create a new content script manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            scripts: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(ContentScriptSettings::default())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Add a content script
    pub fn add_script(&self, script: ContentScript) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        scripts.insert(script.id.clone(), script);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a content script
    pub fn remove_script(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        scripts.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get a content script by ID
    pub fn get_script(&self, id: &str) -> Option<ContentScript> {
        let scripts = self.scripts.lock().ok()?;
        scripts.get(id).cloned()
    }
    
    /// Get all content scripts
    pub fn get_all_scripts(&self) -> Vec<ContentScript> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        scripts.values().cloned().collect()
    }
    
    /// Get enabled content scripts
    pub fn get_enabled_scripts(&self) -> Vec<ContentScript> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        scripts.values()
            .filter(|s| s.enabled)
            .cloned()
            .collect()
    }
    
    /// Update a content script
    pub fn update_script(&self, id: String, script: ContentScript) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if scripts.contains_key(&id) {
            scripts.insert(id, script);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Enable a content script
    pub fn enable_script(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(script) = scripts.get_mut(id) {
            script.enabled = true;
            script.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Disable a content script
    pub fn disable_script(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(script) = scripts.get_mut(id) {
            script.enabled = false;
            script.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get scripts that should run for a URL
    pub fn get_scripts_for_url(&self, url: &str) -> Vec<ContentScript> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        scripts.values()
            .filter(|s| s.enabled && s.matches_url(url))
            .cloned()
            .collect()
    }
    
    /// Get scripts that should run at document start
    pub fn get_start_scripts(&self, url: &str) -> Vec<ContentScript> {
        self.get_scripts_for_url(url)
            .into_iter()
            .filter(|s| s.run_at_start)
            .collect()
    }
    
    /// Get scripts that should run at document end
    pub fn get_end_scripts(&self, url: &str) -> Vec<ContentScript> {
        self.get_scripts_for_url(url)
            .into_iter()
            .filter(|s| s.run_at_end)
            .collect()
    }
    
    /// Get scripts that should run at document idle
    pub fn get_idle_scripts(&self, url: &str) -> Vec<ContentScript> {
        self.get_scripts_for_url(url)
            .into_iter()
            .filter(|s| s.run_at_idle)
            .collect()
    }
    
    /// Get CSS for a URL
    pub fn get_css_for_url(&self, url: &str) -> Vec<String> {
        let scripts = self.scripts.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        scripts.values()
            .filter(|s| s.enabled && s.matches_url(url))
            .filter_map(|s| s.css_content.clone())
            .collect()
    }
    
    /// Get settings
    pub fn get_settings(&self) -> ContentScriptSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update settings
    pub fn update_settings(&self, settings: ContentScriptSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let scripts_path = self.storage_path.join("content_scripts.json");
        let settings_path = self.storage_path.join("content_script_settings.json");
        
        if scripts_path.exists() {
            let content = std::fs::read_to_string(&scripts_path)?;
            let scripts: HashMap<String, ContentScript> = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.scripts.lock() {
                *s = scripts;
            }
        }
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: ContentScriptSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let scripts_path = self.storage_path.join("content_scripts.json");
        let settings_path = self.storage_path.join("content_script_settings.json");
        
        let scripts = self.scripts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let scripts_content = serde_json::to_string_pretty(&*scripts)?;
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        
        std::fs::write(&scripts_path, scripts_content)?;
        std::fs::write(&settings_path, settings_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add content script
#[tauri::command]
pub fn add_content_script(
    script: ContentScript,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<(), String> {
    manager.add_script(script)
        .map_err(|e| format!("Failed to add script: {}", e))
}

/// Remove content script
#[tauri::command]
pub fn remove_content_script(
    id: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<(), String> {
    manager.remove_script(&id)
        .map_err(|e| format!("Failed to remove script: {}", e))
}

/// Get content script
#[tauri::command]
pub fn get_content_script(
    id: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Option<ContentScript>, String> {
    Ok(manager.get_script(&id))
}

/// Get all content scripts
#[tauri::command]
pub fn get_all_content_scripts(
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<ContentScript>, String> {
    Ok(manager.get_all_scripts())
}

/// Get enabled content scripts
#[tauri::command]
pub fn get_enabled_content_scripts(
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<ContentScript>, String> {
    Ok(manager.get_enabled_scripts())
}

/// Update content script
#[tauri::command]
pub fn update_content_script(
    id: String,
    script: ContentScript,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<(), String> {
    manager.update_script(id, script)
        .map_err(|e| format!("Failed to update script: {}", e))
}

/// Enable content script
#[tauri::command]
pub fn enable_content_script(
    id: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<(), String> {
    manager.enable_script(&id)
        .map_err(|e| format!("Failed to enable script: {}", e))
}

/// Disable content script
#[tauri::command]
pub fn disable_content_script(
    id: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<(), String> {
    manager.disable_script(&id)
        .map_err(|e| format!("Failed to disable script: {}", e))
}

/// Get scripts for URL
#[tauri::command]
pub fn get_scripts_for_url(
    url: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<ContentScript>, String> {
    Ok(manager.get_scripts_for_url(&url))
}

/// Get start scripts for URL
#[tauri::command]
pub fn get_start_scripts(
    url: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<ContentScript>, String> {
    Ok(manager.get_start_scripts(&url))
}

/// Get end scripts for URL
#[tauri::command]
pub fn get_end_scripts(
    url: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<ContentScript>, String> {
    Ok(manager.get_end_scripts(&url))
}

/// Get idle scripts for URL
#[tauri::command]
pub fn get_idle_scripts(
    url: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<ContentScript>, String> {
    Ok(manager.get_idle_scripts(&url))
}

/// Get CSS for URL
#[tauri::command]
pub fn get_css_for_url(
    url: String,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_css_for_url(&url))
}

/// Get content script settings
#[tauri::command]
pub fn get_content_script_settings(
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<ContentScriptSettings, String> {
    Ok(manager.get_settings())
}

/// Update content script settings
#[tauri::command]
pub fn update_content_script_settings(
    settings: ContentScriptSettings,
    manager: State<'_, Arc<ContentScriptManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}
