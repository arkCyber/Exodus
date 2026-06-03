//! Bookmark Suggestions System for Exodus Browser
//!
//! This module provides intelligent bookmark suggestions based on:
//! - Usage frequency
//! - AI-powered contextual recommendations
//! - Duplicate detection
//! - Smart categorization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sled::Db;

/// Bookmark usage tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkUsage {
    pub bookmark_id: String,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub first_accessed: DateTime<Utc>,
}

/// Bookmark suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkSuggestion {
    pub bookmark_id: String,
    pub title: String,
    pub url: String,
    pub score: f32,
    pub reason: SuggestionReason,
}

/// Reason for suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SuggestionReason {
    Frequent,
    Recent,
    Contextual,
    Related,
}

/// Bookmark suggestions manager
pub struct BookmarkSuggestionsManager {
    db: Arc<Db>,
    usage_tracker: Arc<Mutex<HashMap<String, BookmarkUsage>>>,
}

impl BookmarkSuggestionsManager {
    /// Create a new bookmark suggestions manager
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = sled::open(db_path)
            .map_err(|e| format!("Failed to open suggestions DB: {}", e))?;
        
        let usage_tracker = Self::load_usage(&db)?;
        
        Ok(Self {
            db: Arc::new(db),
            usage_tracker: Arc::new(Mutex::new(usage_tracker)),
        })
    }
    
    /// Load usage data from database
    fn load_usage(db: &Db) -> Result<HashMap<String, BookmarkUsage>, String> {
        let mut usage_map = HashMap::new();
        
        if let Some(value) = db.get(b"bookmark_usage")
            .map_err(|e| format!("DB get error: {}", e))? 
        {
            if let Ok(loaded) = bincode::deserialize::<HashMap<String, BookmarkUsage>>(&value) {
                usage_map = loaded;
            }
        }
        
        Ok(usage_map)
    }
    
    /// Save usage data to database
    fn save_usage(&self) -> Result<(), String> {
        let usage = self.usage_tracker.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let serialized = bincode::serialize(&*usage)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        self.db.insert(b"bookmark_usage", serialized)
            .map_err(|e| format!("DB insert error: {}", e))?;
        
        Ok(())
    }
    
    /// Record bookmark access
    pub fn record_access(&self, bookmark_id: String) -> Result<(), String> {
        let mut usage = self.usage_tracker.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let now = Utc::now();
        
        if let Some(entry) = usage.get_mut(&bookmark_id) {
            entry.access_count += 1;
            entry.last_accessed = now;
        } else {
            usage.insert(bookmark_id.clone(), BookmarkUsage {
                bookmark_id: bookmark_id.clone(),
                access_count: 1,
                last_accessed: now,
                first_accessed: now,
            });
        }
        
        drop(usage);
        self.save_usage()?;
        
        Ok(())
    }
    
    /// Get frequent bookmarks
    pub fn get_frequent_bookmarks(&self, limit: usize) -> Vec<BookmarkUsage> {
        let usage = match self.usage_tracker.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Lock error: {}", e);
                return Vec::new();
            }
        };
        
        let mut items: Vec<_> = usage.values().cloned().collect();
        items.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        items.truncate(limit);
        
        items
    }
    
    /// Get recently accessed bookmarks
    pub fn get_recent_bookmarks(&self, limit: usize) -> Vec<BookmarkUsage> {
        let usage = match self.usage_tracker.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Lock error: {}", e);
                return Vec::new();
            }
        };
        
        let mut items: Vec<_> = usage.values().cloned().collect();
        items.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        items.truncate(limit);
        
        items
    }
    
    /// Get bookmark usage by ID
    pub fn get_usage(&self, bookmark_id: &str) -> Option<BookmarkUsage> {
        let usage = self.usage_tracker.lock().ok()?;
        usage.get(bookmark_id).cloned()
    }
    
    /// Clear usage data
    pub fn clear_usage(&self) -> Result<(), String> {
        let mut usage = self.usage_tracker.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        usage.clear();
        drop(usage);
        self.save_usage()?;
        
        Ok(())
    }
}

/// Duplicate bookmark detector
pub struct DuplicateDetector {
    url_map: Arc<Mutex<HashMap<String, Vec<String>>>>, // url -> bookmark_ids
}

impl DuplicateDetector {
    /// Create a new duplicate detector
    pub fn new() -> Self {
        Self {
            url_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Add a bookmark
    pub fn add_bookmark(&self, bookmark_id: String, url: String) {
        let mut url_map = match self.url_map.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Lock error: {}", e);
                return;
            }
        };
        
        url_map.entry(url).or_insert_with(Vec::new).push(bookmark_id);
    }
    
    /// Remove a bookmark
    pub fn remove_bookmark(&self, bookmark_id: &str, url: &str) {
        let mut url_map = match self.url_map.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Lock error: {}", e);
                return;
            }
        };
        
        if let Some(ids) = url_map.get_mut(url) {
            ids.retain(|id| id != bookmark_id);
            if ids.is_empty() {
                url_map.remove(url);
            }
        }
    }
    
    /// Find duplicate bookmarks
    pub fn find_duplicates(&self) -> Vec<(String, Vec<String>)> {
        let url_map = match self.url_map.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Lock error: {}", e);
                return Vec::new();
            }
        };
        
        url_map.iter()
            .filter(|(_, ids)| ids.len() > 1)
            .map(|(url, ids)| (url.clone(), ids.clone()))
            .collect()
    }
    
    /// Check if URL has duplicates
    pub fn has_duplicates(&self, url: &str) -> bool {
        let url_map = match self.url_map.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Lock error: {}", e);
                return false;
            }
        };
        
        url_map.get(url).map_or(false, |ids| ids.len() > 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_usage_tracking() {
        let manager = BookmarkSuggestionsManager::new("/tmp/test_suggestions")
            .expect("Failed to create manager");
        
        manager.record_access("bookmark1".to_string())
            .expect("Failed to record access");
        
        let usage = manager.get_usage("bookmark1");
        assert!(usage.is_some());
        assert_eq!(usage.unwrap().access_count, 1);
    }
    
    #[test]
    fn test_frequent_bookmarks() {
        let manager = BookmarkSuggestionsManager::new("/tmp/test_frequent")
            .expect("Failed to create manager");
        
        manager.record_access("bookmark1".to_string())
            .expect("Failed to record access");
        manager.record_access("bookmark1".to_string())
            .expect("Failed to record access");
        manager.record_access("bookmark2".to_string())
            .expect("Failed to record access");
        
        let frequent = manager.get_frequent_bookmarks(10);
        assert_eq!(frequent.len(), 2);
        assert_eq!(frequent[0].bookmark_id, "bookmark1");
        assert_eq!(frequent[0].access_count, 2);
    }
    
    #[test]
    fn test_duplicate_detection() {
        let detector = DuplicateDetector::new();
        
        detector.add_bookmark("bookmark1".to_string(), "https://example.com".to_string());
        detector.add_bookmark("bookmark2".to_string(), "https://example.com".to_string());
        detector.add_bookmark("bookmark3".to_string(), "https://other.com".to_string());
        
        let duplicates = detector.find_duplicates();
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].0, "https://example.com");
        assert!(detector.has_duplicates("https://example.com"));
        assert!(!detector.has_duplicates("https://other.com"));
    }
}
