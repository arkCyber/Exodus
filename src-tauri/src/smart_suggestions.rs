//! Smart Suggestions for Exodus Browser
//! 
//! This module provides intelligent URL and search suggestions based on
//! browsing history, bookmarks, and popular sites.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use std::time::Duration;
/// Suggestion type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuggestionType {
    History,
    Bookmark,
    Search,
    Popular,
}

/// Suggestion entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique identifier
    pub id: String,
    /// Suggestion text
    pub text: String,
    /// Suggestion URL
    pub url: String,
    /// Suggestion type
    pub suggestion_type: SuggestionType,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f64,
    /// Number of times visited
    pub visit_count: u32,
    /// Last visited timestamp
    pub last_visited: u64,
    /// Favicon URL (optional)
    pub favicon_url: Option<String>,
}

impl Suggestion {
    pub fn new(text: String, url: String, suggestion_type: SuggestionType) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            url,
            suggestion_type,
            relevance: 0.5,
            visit_count: 0,
            last_visited: now,
            favicon_url: None,
        }
    }
    
    pub fn calculate_relevance(&mut self, query: &str) {
        let query_lower = query.to_lowercase();
        let text_lower = self.text.to_lowercase();
        
        // Exact match gets highest relevance
        if text_lower == query_lower {
            self.relevance = 1.0;
            return;
        }
        
        // Starts with query gets high relevance
        if text_lower.starts_with(&query_lower) {
            self.relevance = 0.8;
            return;
        }
        
        // Contains query gets medium relevance
        if text_lower.contains(&query_lower) {
            self.relevance = 0.6;
            return;
        }
        
        // Fuzzy match based on character similarity
        let similarity = Self::calculate_similarity(&text_lower, &query_lower);
        self.relevance = similarity * 0.5;
    }
    
    fn calculate_similarity(s1: &str, s2: &str) -> f64 {
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }
        
        let longer = if s1.len() > s2.len() { s1 } else { s2 };
        let shorter = if s1.len() > s2.len() { s2 } else { s1 };
        
        let longer_len = longer.len();
        let _shorter_len = shorter.len();
        
        if longer_len == 0 {
            return 1.0;
        }
        
        let mut matches = 0;
        for (i, c) in shorter.chars().enumerate() {
            if longer.chars().nth(i) == Some(c) {
                matches += 1;
            }
        }
        
        matches as f64 / longer_len as f64
    }
    
    #[allow(dead_code)]
    pub fn record_visit(&mut self) {
        self.visit_count += 1;
        self.last_visited = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        // Increase relevance based on visit count
        self.relevance = (self.relevance + 0.1).min(1.0);
    }
}

/// Smart suggestions manager
pub struct SmartSuggestionsManager {
    suggestions: Arc<Mutex<HashMap<String, Suggestion>>>,
    history: Arc<Mutex<Vec<String>>>,
    bookmarks: Arc<Mutex<Vec<String>>>,
    storage_path: PathBuf,
}

impl SmartSuggestionsManager {
    /// Create a new smart suggestions manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            suggestions: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            bookmarks: Arc::new(Mutex::new(Vec::new())),
            storage_path,
        };
        
        manager.load_popular_suggestions()?;
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Load popular site suggestions
    fn load_popular_suggestions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let popular_sites = vec![
            ("Google", "https://www.google.com"),
            ("YouTube", "https://www.youtube.com"),
            ("Facebook", "https://www.facebook.com"),
            ("Twitter", "https://www.twitter.com"),
            ("Amazon", "https://www.amazon.com"),
            ("Wikipedia", "https://www.wikipedia.org"),
            ("Reddit", "https://www.reddit.com"),
            ("GitHub", "https://github.com"),
            ("Stack Overflow", "https://stackoverflow.com"),
            ("LinkedIn", "https://www.linkedin.com"),
        ];
        
        let mut suggestions = self.suggestions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for (name, url) in popular_sites {
            let suggestion = Suggestion::new(name.to_string(), url.to_string(), SuggestionType::Popular);
            suggestions.insert(suggestion.id.clone(), suggestion);
        }
        
        Ok(())
    }
    
    /// Get suggestions for a query
    pub fn get_suggestions(&self, query: &str, limit: usize) -> Vec<Suggestion> {
        let suggestions = self.suggestions.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut results: Vec<Suggestion> = suggestions.values()
            .map(|s| s.clone())
            .collect();
        
        // Calculate relevance for each suggestion
        for suggestion in &mut results {
            suggestion.calculate_relevance(query);
        }
        
        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance.partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.visit_count.cmp(&a.visit_count))
                .then_with(|| b.last_visited.cmp(&a.last_visited))
        });
        
        // Filter out low relevance suggestions
        results.retain(|s| s.relevance > 0.3);
        
        // Limit results
        results.truncate(limit);
        
        results
    }
    
    /// Add a history entry
    pub fn add_history(&self, url: String, title: String) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut history = self.history.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            history.push(url.clone());
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        {
            let mut suggestions = self.suggestions.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            let suggestion = Suggestion::new(title, url, SuggestionType::History);
            suggestions.insert(suggestion.id.clone(), suggestion);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add a bookmark
    pub fn add_bookmark(&self, url: String, title: String) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut bookmarks = self.bookmarks.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            bookmarks.push(url.clone());
        }
        {
            let mut suggestions = self.suggestions.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            let suggestion = Suggestion::new(title, url, SuggestionType::Bookmark);
            suggestions.insert(suggestion.id.clone(), suggestion);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a bookmark
    pub fn remove_bookmark(&self, url: String) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut bookmarks = self.bookmarks.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            bookmarks.retain(|u| u != &url);
        }
        {
            let mut suggestions = self.suggestions.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            suggestions.retain(|_, s| {
                !(s.suggestion_type == SuggestionType::Bookmark && s.url == url)
            });
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get browsing history
    pub fn get_history(&self, limit: usize) -> Vec<String> {
        let history = self.history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Get bookmarks
    pub fn get_bookmarks(&self) -> Vec<String> {
        let bookmarks = self.bookmarks.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        bookmarks.clone()
    }
    
    /// Clear browsing history
    pub fn clear_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut history = self.history.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            history.clear();
        }
        {
            let mut suggestions = self.suggestions.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            suggestions.retain(|_, s| s.suggestion_type != SuggestionType::History);
        }
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let history_path = self.storage_path.join("history.json");
        let bookmarks_path = self.storage_path.join("bookmarks.json");
        
        if history_path.exists() {
            let content = std::fs::read_to_string(&history_path)?;
            let history: Vec<String> = serde_json::from_str(&content)?;
            if let Ok(mut h) = self.history.lock() {
                *h = history;
            }
        }
        
        if bookmarks_path.exists() {
            let content = std::fs::read_to_string(&bookmarks_path)?;
            let bookmarks: Vec<String> = serde_json::from_str(&content)?;
            if let Ok(mut b) = self.bookmarks.lock() {
                *b = bookmarks;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let history_path = self.storage_path.join("history.json");
        let bookmarks_path = self.storage_path.join("bookmarks.json");
        
        let history = self.history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let bookmarks = self.bookmarks.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        std::fs::write(&history_path, serde_json::to_string(&*history)?)?;
        std::fs::write(&bookmarks_path, serde_json::to_string(&*bookmarks)?)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get suggestions for a query
#[tauri::command]
pub fn get_suggestions(
    query: String,
    limit: usize,
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<Vec<Suggestion>, String> {
    Ok(manager.get_suggestions(&query, limit))
}

/// Add history entry for suggestions
#[tauri::command]
pub fn add_suggestion_history_entry(
    url: String,
    title: String,
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<(), String> {
    manager.add_history(url, title)
        .map_err(|e| format!("Failed to add history: {}", e))
}

/// Add bookmark
#[tauri::command]
pub fn add_suggestion_bookmark(
    url: String,
    title: String,
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<(), String> {
    manager.add_bookmark(url, title)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

/// Remove bookmark
#[tauri::command]
pub fn remove_suggestion_bookmark(
    url: String,
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<(), String> {
    manager.remove_bookmark(url)
        .map_err(|e| format!("Failed to remove bookmark: {}", e))
}

/// Get browsing history
#[tauri::command]
pub fn get_browsing_history(
    limit: usize,
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_history(limit))
}

/// Get bookmarks
#[tauri::command]
pub fn get_bookmarks(
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_bookmarks())
}

/// Clear browsing history
#[tauri::command]
pub fn clear_browsing_history(
    manager: State<'_, Arc<SmartSuggestionsManager>>,
) -> Result<(), String> {
    manager.clear_history()
        .map_err(|e| format!("Failed to clear history: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new(
            "Google".to_string(),
            "https://www.google.com".to_string(),
            SuggestionType::Popular,
        );
        
        assert_eq!(suggestion.text, "Google");
        assert_eq!(suggestion.url, "https://www.google.com");
        assert_eq!(suggestion.suggestion_type, SuggestionType::Popular);
    }
    
    #[test]
    fn test_suggestion_relevance() {
        let mut suggestion = Suggestion::new(
            "Google Search".to_string(),
            "https://www.google.com".to_string(),
            SuggestionType::History,
        );
        
        suggestion.calculate_relevance("google");
        assert!(suggestion.relevance > 0.5);
        
        suggestion.calculate_relevance("google search");
        assert!(suggestion.relevance > 0.8);
    }
    
    #[test]
    fn test_smart_suggestions_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = SmartSuggestionsManager::new(temp_dir.path().to_path_buf()).expect("Failed to create manager");
        
        manager.add_history(
            "https://example.com".to_string(),
            "Example Site".to_string(),
        ).expect("Failed to add history");
        
        let suggestions = manager.get_suggestions("example", 10);
        assert!(!suggestions.is_empty());
    }
}
