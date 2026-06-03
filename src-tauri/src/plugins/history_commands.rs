//! Chrome Extension API - chrome.history
//!
//! Provides Tauri commands for the chrome.history API, allowing extensions
//! to interact with the browser's history.

use crate::config::ConfigState;
use crate::history_manager::HistoryEntry;
use crate::plugins::manager::ExtensionManager;
use crate::profile_stores::ProfileHistoryStores;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Active profile history store from privacy mode flag.
fn active_history<'a>(
    stores: &'a ProfileHistoryStores,
    config: &ConfigState,
) -> Result<&'a Arc<crate::history_manager::HistoryManager>, String> {
    let private = config
        .lock()
        .map_err(|e| format!("Config lock: {}", e))?
        .private_mode;
    Ok(stores.active(private))
}

/// History query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
    /// Text to search for in history
    #[serde(default)]
    pub text: Option<String>,
    /// Maximum number of results
    #[serde(default)]
    pub max_results: Option<u32>,
    /// End time (timestamp in ms)
    #[serde(default)]
    pub end_time: Option<u64>,
    /// Start time (timestamp in ms)
    #[serde(default)]
    pub start_time: Option<u64>,
}

/// History item for chrome.history API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub last_visit_time: u64,
    pub visit_count: u32,
    pub typed_count: u32,
}

impl From<HistoryEntry> for HistoryItem {
    fn from(entry: HistoryEntry) -> Self {
        Self {
            id: entry.id,
            url: entry.url,
            title: entry.title,
            last_visit_time: entry.last_visit * 1000, // Convert to ms
            visit_count: entry.visit_count,
            typed_count: entry.visit_count, // Simplified
        }
    }
}

/// Search history
#[tauri::command]
pub async fn chrome_history_search(
    query: HistoryQuery,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<HistoryItem>, String> {
    // Get history manager
    let history_manager = active_history(&stores, &config)?;
    
    // Search based on query
    let results = if let Some(text) = query.text {
        history_manager.search(&text)
    } else {
        history_manager.get_all()
    };
    
    // Apply time range filters if specified
    let filtered = if let Some(start_time) = query.start_time {
        let start = start_time / 1000; // Convert ms to seconds
        let end = query.end_time.unwrap_or(u64::MAX) / 1000;
        results.into_iter()
            .filter(|e| e.visit_time >= start && e.visit_time <= end)
            .collect()
    } else {
        results
    };
    
    // Apply max results limit
    let limited = if let Some(max_results) = query.max_results {
        filtered.into_iter().take(max_results as usize).collect()
    } else {
        filtered
    };
    
    // Convert to HistoryItem format
    let history_items: Vec<HistoryItem> = limited.into_iter().map(Into::into).collect();
    
    Ok(history_items)
}

/// Add a history item
#[tauri::command]
pub async fn chrome_history_add_item(
    item: HistoryItem,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    // Convert to HistoryEntry and add to history
    let history_manager = active_history(&stores, &config)?;
    
    history_manager
        .add_entry(item.url, item.title)
        .map_err(|e| format!("Failed to add history item: {}", e))
}

/// Delete a history item
#[tauri::command]
pub async fn chrome_history_delete_item(
    item_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    // Delete history item by ID
    let history_manager = active_history(&stores, &config)?;
    
    history_manager
        .remove_entry(&item_id)
        .map_err(|e| format!("Failed to delete history item: {}", e))
}

/// Delete all history
#[tauri::command]
pub async fn chrome_history_delete_all(
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<(), String> {
    // Delete all history
    let history_manager = active_history(&stores, &config)?;
    
    history_manager
        .clear_all()
        .map_err(|e| format!("Failed to delete all history: {}", e))
}

/// Get history item by URL
#[tauri::command]
pub async fn chrome_history_get_url(
    url: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Option<HistoryItem>, String> {
    // Get history item by URL
    let history_manager = active_history(&stores, &config)?;
    
    let entry = history_manager.get_by_url(&url);
    Ok(entry.map(Into::into))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_query_deserialization() {
        let query_json = r#"{"text":"test","maxResults":10}"#;
        let query: HistoryQuery = serde_json::from_str(query_json).unwrap();
        assert_eq!(query.text, Some("test".to_string()));
        assert_eq!(query.max_results, Some(10));
    }

    #[test]
    fn test_history_item_serialization() {
        let item = HistoryItem {
            id: "test-id".to_string(),
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            last_visit_time: 1234567890,
            visit_count: 5,
            typed_count: 3,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_history_item_all_fields() {
        let item = HistoryItem {
            id: "history-123".to_string(),
            url: "https://github.com".to_string(),
            title: "GitHub".to_string(),
            last_visit_time: 9999999999,
            visit_count: 100,
            typed_count: 50,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("GitHub"));
        assert!(json.contains("9999999999"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_history_query_with_start_time() {
        let query = HistoryQuery {
            text: Some("search".to_string()),
            start_time: Some(1234567890),
            end_time: Some(1234567900),
            max_results: Some(20),
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("search"));
        assert!(json.contains("startTime"));
        assert!(json.contains("endTime"));
    }

    #[test]
    fn test_history_query_default() {
        let query = HistoryQuery::default();
        assert!(query.text.is_none());
        assert!(query.start_time.is_none());
        assert!(query.end_time.is_none());
        assert!(query.max_results.is_none());
    }

    #[test]
    fn test_history_item_camel_case() {
        let item = HistoryItem {
            id: "test".to_string(),
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            last_visit_time: 1234567890,
            visit_count: 5,
            typed_count: 3,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("lastVisitTime"));
        assert!(json.contains("visitCount"));
        assert!(json.contains("typedCount"));
    }
}
