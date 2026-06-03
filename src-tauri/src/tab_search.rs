//! Tab Search functionality for Exodus Browser
//! Allows users to search through open tabs by title, URL, or content

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Tab information for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub label: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_active: bool,
    pub is_pinned: bool,
    pub is_muted: bool,
}

/// Search request for tabs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

/// Tab search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSearchResult {
    pub tabs: Vec<TabInfo>,
    pub total_count: usize,
}

/// Tab Search Manager
pub struct TabSearchManager {
    tabs: Arc<Mutex<Vec<TabInfo>>>,
}

impl TabSearchManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a tab for search
    pub fn register_tab(&self, tab: TabInfo) {
        if let Ok(mut tabs) = self.tabs.lock() {
            // Remove existing tab with same label if exists
            tabs.retain(|t| t.label != tab.label);
            tabs.push(tab);
        }
    }

    /// Unregister a tab
    pub fn unregister_tab(&self, label: &str) {
        if let Ok(mut tabs) = self.tabs.lock() {
            tabs.retain(|t| t.label != label);
        }
    }

    /// Update tab information
    pub fn update_tab(&self, label: &str, updater: impl FnOnce(&mut TabInfo)) {
        if let Ok(mut tabs) = self.tabs.lock() {
            if let Some(tab) = tabs.iter_mut().find(|t| t.label == label) {
                updater(tab);
            }
        }
    }

    /// Search tabs by query
    pub fn search_tabs(&self, request: TabSearchRequest) -> TabSearchResult {
        let tabs = self.tabs.lock();
        let query_lower = request.query.to_lowercase();
        
        let results: Vec<TabInfo> = tabs.as_ref()
            .ok()
            .map(|tabs| tabs
            .iter()
            .filter(|tab| {
                tab.title.to_lowercase().contains(&query_lower)
                    || tab.url.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect())
            .unwrap_or_default();

        let limit = request.limit.unwrap_or(results.len());
        let total_count = results.len();
        
        TabSearchResult {
            tabs: results.into_iter().take(limit).collect(),
            total_count,
        }
    }

    /// Get all registered tabs
    pub fn get_all_tabs(&self) -> Vec<TabInfo> {
        self.tabs.lock()
            .map(|tabs| tabs.clone())
            .unwrap_or_default()
    }

    /// Get a tab by label
    #[allow(dead_code)]
    pub fn get_tab(&self, label: &str) -> Option<TabInfo> {
        self.tabs.lock().ok()
            .and_then(|tabs| tabs.iter().find(|t| t.label == label).cloned())
    }
}

impl Default for TabSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to search tabs
#[tauri::command]
pub fn search_tabs(
    request: TabSearchRequest,
    manager: State<'_, Arc<TabSearchManager>>,
) -> TabSearchResult {
    manager.search_tabs(request)
}

/// Tauri command to get all tabs
#[tauri::command]
pub fn get_all_search_tabs(
    manager: State<'_, Arc<TabSearchManager>>,
) -> Vec<TabInfo> {
    manager.get_all_tabs()
}

/// Tauri command to register a tab
#[tauri::command]
pub fn register_search_tab(
    tab: TabInfo,
    manager: State<'_, Arc<TabSearchManager>>,
) {
    manager.register_tab(tab);
}

/// Tauri command to unregister a tab
#[tauri::command]
pub fn unregister_search_tab(
    label: String,
    manager: State<'_, Arc<TabSearchManager>>,
) {
    manager.unregister_tab(&label);
}

/// Tauri command to update a tab
#[tauri::command]
pub fn update_search_tab(
    label: String,
    title: Option<String>,
    url: Option<String>,
    favicon: Option<String>,
    is_active: Option<bool>,
    is_pinned: Option<bool>,
    is_muted: Option<bool>,
    manager: State<'_, Arc<TabSearchManager>>,
) {
    manager.update_tab(&label, |tab| {
        if let Some(title) = title {
            tab.title = title;
        }
        if let Some(url) = url {
            tab.url = url;
        }
        if let Some(favicon) = favicon {
            tab.favicon = Some(favicon);
        }
        if let Some(is_active) = is_active {
            tab.is_active = is_active;
        }
        if let Some(is_pinned) = is_pinned {
            tab.is_pinned = is_pinned;
        }
        if let Some(is_muted) = is_muted {
            tab.is_muted = is_muted;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_search_manager_creation() {
        let manager = TabSearchManager::new();
        let tabs = manager.get_all_tabs();
        assert!(tabs.is_empty());
    }

    #[test]
    fn test_register_and_search_tabs() {
        let manager = TabSearchManager::new();
        
        let tab1 = TabInfo {
            label: "tab1".to_string(),
            title: "Google".to_string(),
            url: "https://google.com".to_string(),
            favicon: None,
            is_active: true,
            is_pinned: false,
            is_muted: false,
        };
        
        let tab2 = TabInfo {
            label: "tab2".to_string(),
            title: "GitHub".to_string(),
            url: "https://github.com".to_string(),
            favicon: None,
            is_active: false,
            is_pinned: false,
            is_muted: false,
        };
        
        manager.register_tab(tab1);
        manager.register_tab(tab2);
        
        let request = TabSearchRequest {
            query: "google".to_string(),
            limit: None,
        };
        
        let result = manager.search_tabs(request);
        assert_eq!(result.tabs.len(), 1);
        assert_eq!(result.tabs[0].title, "Google");
    }

    #[test]
    fn test_tab_update() {
        let manager = TabSearchManager::new();
        
        let tab = TabInfo {
            label: "tab1".to_string(),
            title: "Google".to_string(),
            url: "https://google.com".to_string(),
            favicon: None,
            is_active: true,
            is_pinned: false,
            is_muted: false,
        };
        
        manager.register_tab(tab);
        
        manager.update_tab("tab1", |tab| {
            tab.title = "Google Search".to_string();
        });
        
        let updated_tab = manager.get_tab("tab1");
        assert!(updated_tab.is_some());
        assert_eq!(updated_tab.expect("Expected tab to exist").title, "Google Search");
    }

    #[test]
    fn test_unregister_tab() {
        let manager = TabSearchManager::new();
        
        let tab = TabInfo {
            label: "tab1".to_string(),
            title: "Google".to_string(),
            url: "https://google.com".to_string(),
            favicon: None,
            is_active: true,
            is_pinned: false,
            is_muted: false,
        };
        
        manager.register_tab(tab);
        assert_eq!(manager.get_all_tabs().len(), 1);
        
        manager.unregister_tab("tab1");
        assert_eq!(manager.get_all_tabs().len(), 0);
    }
}
