//! Search Engine Management for Exodus Browser
//! 
//! This module provides search engine management and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use std::time::Duration;
/// Search engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEngine {
    /// Engine ID
    pub id: String,
    /// Engine name
    pub name: String,
    /// Search URL template
    pub search_url: String,
    /// Suggestion URL template
    pub suggestion_url: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
    /// Is default
    pub is_default: bool,
    /// Is built-in
    pub is_builtin: bool,
    /// Created timestamp
    pub created_at: u64,
}

impl SearchEngine {
    pub fn new(name: String, search_url: String, is_builtin: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            search_url,
            suggestion_url: None,
            icon_url: None,
            is_default: false,
            is_builtin,
            created_at: now,
        }
    }
    
    pub fn build_search_url(&self, query: &str) -> String {
        // Simple URL encoding
        let encoded = query
            .chars()
            .map(|c| match c {
                ' ' => "+".to_string(),
                c if c.is_ascii_alphanumeric() => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect::<String>();
        self.search_url.replace("{q}", &encoded)
    }
}

/// Search engine settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEngineSettings {
    /// Default engine ID
    pub default_engine_id: String,
    /// Enable suggestions
    pub enable_suggestions: bool,
    /// Suggestion limit
    pub suggestion_limit: usize,
    /// Search in new tab
    pub search_in_new_tab: bool,
    /// Show search bar on new tab
    pub show_search_bar: bool,
}

impl Default for SearchEngineSettings {
    fn default() -> Self {
        Self {
            default_engine_id: String::new(),
            enable_suggestions: true,
            suggestion_limit: 10,
            search_in_new_tab: false,
            show_search_bar: true,
        }
    }
}

/// Search engine manager
pub struct SearchEngineManager {
    engines: Arc<Mutex<HashMap<String, SearchEngine>>>,
    settings: Arc<Mutex<SearchEngineSettings>>,
    storage_path: PathBuf,
}

impl SearchEngineManager {
    /// Create a new search engine manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            engines: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(SearchEngineSettings::default())),
            storage_path,
        };
        
        manager.initialize_builtin_engines()?;
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Initialize built-in search engines
    fn initialize_builtin_engines(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut engines = self.engines.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Google
        let google = SearchEngine::new(
            "Google".to_string(),
            "https://www.google.com/search?q={q}".to_string(),
            true,
        );
        engines.insert(google.id.clone(), google);
        
        // Bing
        let bing = SearchEngine::new(
            "Bing".to_string(),
            "https://www.bing.com/search?q={q}".to_string(),
            true,
        );
        engines.insert(bing.id.clone(), bing);
        
        // DuckDuckGo
        let duckduckgo = SearchEngine::new(
            "DuckDuckGo".to_string(),
            "https://duckduckgo.com/?q={q}".to_string(),
            true,
        );
        engines.insert(duckduckgo.id.clone(), duckduckgo);
        
        // Yahoo
        let yahoo = SearchEngine::new(
            "Yahoo".to_string(),
            "https://search.yahoo.com/search?p={q}".to_string(),
            true,
        );
        engines.insert(yahoo.id.clone(), yahoo);
        
        // Baidu
        let baidu = SearchEngine::new(
            "Baidu".to_string(),
            "https://www.baidu.com/s?wd={q}".to_string(),
            true,
        );
        engines.insert(baidu.id.clone(), baidu);
        
        Ok(())
    }
    
    /// Add a custom search engine
    pub fn add_engine(&self, name: String, search_url: String, suggestion_url: Option<String>, icon_url: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
        let mut engine = SearchEngine::new(name, search_url, false);
        engine.suggestion_url = suggestion_url;
        engine.icon_url = icon_url;
        
        let engine_id = engine.id.clone();
        
        let mut engines = self.engines.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        engines.insert(engine_id.clone(), engine);
        self.save_to_disk()?;
        Ok(engine_id)
    }
    
    /// Remove a search engine
    pub fn remove_engine(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut engines = self.engines.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(engine) = engines.get(id) {
            if engine.is_builtin {
                return Err("Cannot remove built-in search engine".into());
            }
        }
        
        engines.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Update a search engine
    pub fn update_engine(&self, id: String, name: Option<String>, search_url: Option<String>, suggestion_url: Option<String>, icon_url: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut engines = self.engines.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(engine) = engines.get_mut(&id) {
            if let Some(name) = name {
                engine.name = name;
            }
            if let Some(search_url) = search_url {
                engine.search_url = search_url;
            }
            if let Some(suggestion_url) = suggestion_url {
                engine.suggestion_url = Some(suggestion_url);
            }
            if let Some(icon_url) = icon_url {
                engine.icon_url = Some(icon_url);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Set default search engine
    pub fn set_default_engine(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut engines = self.engines.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Clear existing default
        for engine in engines.values_mut() {
            engine.is_default = false;
        }
        
        // Set new default
        if let Some(engine) = engines.get_mut(id) {
            engine.is_default = true;
            
            let mut settings = self.settings.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            settings.default_engine_id = id.to_string();
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get default search engine
    pub fn get_default_engine(&self) -> Option<SearchEngine> {
        let engines = self.engines.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        engines.values()
            .find(|e| e.is_default)
            .cloned()
            .or_else(|| {
                // Fallback to first engine if no default set
                engines.values().next().cloned()
            })
    }
    
    /// Get all search engines
    pub fn get_all_engines(&self) -> Vec<SearchEngine> {
        let engines = self.engines.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        engines.values().cloned().collect()
    }
    
    /// Get a search engine by ID
    pub fn get_engine(&self, id: &str) -> Option<SearchEngine> {
        let engines = self.engines.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        engines.get(id).cloned()
    }
    
    /// Search using default engine
    pub fn search(&self, query: String) -> Option<String> {
        if let Some(engine) = self.get_default_engine() {
            Some(engine.build_search_url(&query))
        } else {
            None
        }
    }
    
    /// Search using specific engine
    pub fn search_with_engine(&self, engine_id: &str, query: String) -> Option<String> {
        if let Some(engine) = self.get_engine(engine_id) {
            Some(engine.build_search_url(&query))
        } else {
            None
        }
    }
    
    /// Get search engine settings
    pub fn get_settings(&self) -> SearchEngineSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update search engine settings
    pub fn update_settings(&self, settings: SearchEngineSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("search_engine_settings.json");
        let engines_path = self.storage_path.join("search_engines.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: SearchEngineSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if engines_path.exists() {
            let content = std::fs::read_to_string(&engines_path)?;
            let engines: HashMap<String, SearchEngine> = serde_json::from_str(&content)?;
            if let Ok(mut e) = self.engines.lock() {
                *e = engines;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("search_engine_settings.json");
        let engines_path = self.storage_path.join("search_engines.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let engines = self.engines.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let engines_content = serde_json::to_string_pretty(&*engines)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&engines_path, engines_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add a custom search engine
#[tauri::command]
pub fn add_search_engine(
    name: String,
    search_url: String,
    suggestion_url: Option<String>,
    icon_url: Option<String>,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<String, String> {
    manager.add_engine(name, search_url, suggestion_url, icon_url)
        .map_err(|e| format!("Failed to add search engine: {}", e))
}

/// Remove a search engine
#[tauri::command]
pub fn remove_search_engine(
    id: String,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<(), String> {
    manager.remove_engine(&id)
        .map_err(|e| format!("Failed to remove search engine: {}", e))
}

/// Update a search engine
#[tauri::command]
pub fn update_search_engine(
    id: String,
    name: Option<String>,
    search_url: Option<String>,
    suggestion_url: Option<String>,
    icon_url: Option<String>,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<(), String> {
    manager.update_engine(id, name, search_url, suggestion_url, icon_url)
        .map_err(|e| format!("Failed to update search engine: {}", e))
}

/// Set default search engine
#[tauri::command]
pub fn set_default_search_engine(
    id: String,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<(), String> {
    manager.set_default_engine(&id)
        .map_err(|e| format!("Failed to set default search engine: {}", e))
}

/// Get default search engine
#[tauri::command]
pub fn get_default_search_engine(
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<Option<SearchEngine>, String> {
    Ok(manager.get_default_engine())
}

/// Get all search engines
#[tauri::command]
pub fn get_all_search_engines(
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<Vec<SearchEngine>, String> {
    Ok(manager.get_all_engines())
}

/// Get a search engine by ID
#[tauri::command]
pub fn get_search_engine(
    id: String,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<Option<SearchEngine>, String> {
    Ok(manager.get_engine(&id))
}

/// Search using default engine
#[tauri::command]
pub fn search(
    query: String,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<Option<String>, String> {
    Ok(manager.search(query))
}

/// Search using specific engine
#[tauri::command]
pub fn search_with_engine(
    engine_id: String,
    query: String,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<Option<String>, String> {
    Ok(manager.search_with_engine(&engine_id, query))
}

/// Get search engine settings
#[tauri::command]
pub fn get_search_engine_settings(
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<SearchEngineSettings, String> {
    Ok(manager.get_settings())
}

/// Update search engine settings
#[tauri::command]
pub fn update_search_engine_settings(
    settings: SearchEngineSettings,
    manager: State<'_, Arc<SearchEngineManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}
