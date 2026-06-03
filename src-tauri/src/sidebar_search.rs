//! Exodus Browser — Sidebar Search functionality
//!
//! Provides search engine management and search history for sidebar search.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Search engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEngine {
    pub id: String,
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
}

/// Search history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryEntry {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub engine: String,
}

/// Sidebar search state
pub struct SidebarSearchState {
    db: Arc<Db>,
    engines: Arc<Mutex<Vec<SearchEngine>>>,
    default_engine: Arc<Mutex<String>>,
}

impl SidebarSearchState {
    /// Create sidebar search state with sled database
    pub fn new(db_path: &Path) -> Result<Self, String> {
        let db = sled::open(db_path).map_err(|e| format!("Failed to open sidebar search DB: {}", e))?;
        let engines = Self::load_engines(&db)?;
        let default_engine = Self::load_default_engine(&db)?;
        
        Ok(Self {
            db: Arc::new(db),
            engines: Arc::new(Mutex::new(engines)),
            default_engine: Arc::new(Mutex::new(default_engine)),
        })
    }

    /// Load engines from database
    fn load_engines(db: &Db) -> Result<Vec<SearchEngine>, String> {
        let mut engines = Vec::new();
        if let Some(value) = db.get(b"engines").map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(loaded) = bincode::deserialize::<Vec<SearchEngine>>(&value) {
                engines = loaded;
            }
        }
        
        // Add default engines if empty
        if engines.is_empty() {
            engines = vec![
                SearchEngine {
                    id: "duckduckgo".to_string(),
                    name: "DuckDuckGo".to_string(),
                    url: "https://duckduckgo.com/?q={query}".to_string(),
                    icon: None,
                },
                SearchEngine {
                    id: "google".to_string(),
                    name: "Google".to_string(),
                    url: "https://www.google.com/search?q={query}".to_string(),
                    icon: None,
                },
                SearchEngine {
                    id: "bing".to_string(),
                    name: "Bing".to_string(),
                    url: "https://www.bing.com/search?q={query}".to_string(),
                    icon: None,
                },
            ];
        }
        Ok(engines)
    }

    /// Load default engine from database
    fn load_default_engine(db: &Db) -> Result<String, String> {
        if let Some(value) = db.get(b"default_engine").map_err(|e| format!("DB get error: {}", e))? {
            if let Ok(engine) = String::from_utf8(value.to_vec()) {
                return Ok(engine);
            }
        }
        Ok("duckduckgo".to_string())
    }

    /// Get all search engines
    pub fn get_engines(&self) -> Result<Vec<SearchEngine>, String> {
        let engines = self.engines.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(engines.clone())
    }

    /// Add custom search engine
    pub fn add_engine(&self, engine: SearchEngine) -> Result<(), String> {
        let mut engines = self.engines.lock().map_err(|e| format!("Lock error: {}", e))?;
        engines.push(engine.clone());
        
        let engines_vec = engines.clone();
        let value = bincode::serialize(&engines_vec).map_err(|e| format!("Serialization error: {}", e))?;
        self.db.insert(b"engines", value)
            .map_err(|e| format!("DB insert error: {}", e))?;
        
        Ok(())
    }

    /// Remove search engine
    pub fn remove_engine(&self, engine_id: &str) -> Result<(), String> {
        let mut engines = self.engines.lock().map_err(|e| format!("Lock error: {}", e))?;
        engines.retain(|e| e.id != engine_id);
        
        let engines_vec = engines.clone();
        let value = bincode::serialize(&engines_vec).map_err(|e| format!("Serialization error: {}", e))?;
        self.db.insert(b"engines", value)
            .map_err(|e| format!("DB insert error: {}", e))?;
        
        Ok(())
    }

    /// Get default search engine
    pub fn get_default_engine(&self) -> Result<String, String> {
        let engine = self.default_engine.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(engine.clone())
    }

    /// Set default search engine
    pub fn set_default_engine(&self, engine_id: String) -> Result<(), String> {
        let mut default = self.default_engine.lock().map_err(|e| format!("Lock error: {}", e))?;
        *default = engine_id.clone();
        
        self.db.insert(b"default_engine", engine_id.as_bytes())
            .map_err(|e| format!("DB insert error: {}", e))?;
        
        Ok(())
    }

    /// Add to search history
    pub fn add_to_history(&self, query: String, engine: String) -> Result<(), String> {
        let entry = SearchHistoryEntry {
            query: query.clone(),
            timestamp: Utc::now(),
            engine,
        };
        
        let key = format!("history:{}", Uuid::new_v4());
        let value = bincode::serialize(&entry).map_err(|e| format!("Serialization error: {}", e))?;
        
        self.db.insert(key.as_bytes(), value)
            .map_err(|e| format!("DB insert error: {}", e))?;
        
        Ok(())
    }

    /// Get search history
    pub fn get_history(&self, limit: usize) -> Result<Vec<SearchHistoryEntry>, String> {
        let mut entries = Vec::new();
        for item_result in self.db.iter() {
            let (key, value) = item_result.map_err(|e| format!("DB iteration error: {}", e))?;
            if key.starts_with(b"history:") {
                if let Ok(entry) = bincode::deserialize::<SearchHistoryEntry>(&value) {
                    entries.push(entry);
                }
            }
        }
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);
        Ok(entries)
    }

    /// Clear search history
    pub fn clear_history(&self) -> Result<(), String> {
        let keys_to_remove: Vec<Vec<u8>> = self.db
            .iter()
            .filter_map(|item| item.ok())
            .filter(|(key, _)| key.starts_with(b"history:"))
            .map(|(key, _)| key.to_vec())
            .collect();
        
        for key in keys_to_remove {
            self.db.remove(&key).map_err(|e| format!("DB remove error: {}", e))?;
        }
        
        Ok(())
    }

    /// Remove specific query from history
    pub fn remove_from_history(&self, query: &str) -> Result<(), String> {
        let keys_to_remove: Vec<Vec<u8>> = self.db
            .iter()
            .filter_map(|item| item.ok())
            .filter(|(key, _)| key.starts_with(b"history:"))
            .filter_map(|(key, value)| {
                if let Ok(entry) = bincode::deserialize::<SearchHistoryEntry>(&value) {
                    if entry.query == query {
                        Some(key.to_vec())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        for key in keys_to_remove {
            self.db.remove(&key).map_err(|e| format!("DB remove error: {}", e))?;
        }
        
        Ok(())
    }

    /// Get search suggestions based on query
    pub fn get_suggestions(&self, query: &str, limit: usize) -> Result<Vec<String>, String> {
        let history = self.get_history(100)?;
        let query_lower = query.to_lowercase();
        
        let suggestions: Vec<String> = history
            .iter()
            .filter(|entry| entry.query.to_lowercase().contains(&query_lower))
            .map(|entry| entry.query.clone())
            .take(limit)
            .collect();
        
        Ok(suggestions)
    }
}
