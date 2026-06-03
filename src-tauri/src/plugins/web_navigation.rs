//! Exodus Browser — chrome.webNavigation API implementation
//!
//! Provides navigation event tracking for extensions with high reliability
//! and safety guarantees following aerospace-grade standards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

/// Navigation transition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransitionType {
    Link,
    Typed,
    AutoBookmark,
    AutoSubframe,
    ManualSubframe,
    Generated,
    StartPage,
    FormSubmit,
    Reload,
    Keyword,
    KeywordGenerated,
    Other,
}

/// Navigation qualifier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransitionQualifier {
    ClientRedirect,
    ServerRedirect,
    ForwardBack,
    FromAddressBar,
}

/// Web navigation event details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationDetails {
    pub tab_id: i32,
    pub url: String,
    pub process_id: i32,
    pub frame_id: i32,
    pub parent_frame_id: i32,
    pub transition_type: Option<String>,
    pub transition_qualifiers: Option<Vec<String>>,
    pub server_redirect: Option<String>,
    pub timestamp: f64,
}

/// Frame details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameDetails {
    pub tab_id: i32,
    pub frame_id: i32,
    pub parent_frame_id: i32,
    pub url: Option<String>,
    pub error_occurred: Option<bool>,
}

/// Web navigation event listener registration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebNavigationListener {
    pub extension_id: String,
    pub event_name: String,
    pub filter: Option<WebNavigationFilter>,
}

/// Web navigation filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebNavigationFilter {
    pub url: Option<Vec<String>>,
    pub tab_id: Option<i32>,
    pub frame_id: Option<i32>,
}

/// Web navigation registry
pub struct WebNavigationRegistry {
    listeners: Arc<Mutex<HashMap<String, Vec<WebNavigationListener>>>>,
}

impl WebNavigationRegistry {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a listener for a navigation event
    pub fn add_listener(&self, listener: WebNavigationListener) -> Result<(), String> {
        let mut listeners = self.listeners.lock().map_err(|e| format!("Lock error: {}", e))?;
        let key = format!("{}:{}", listener.extension_id, listener.event_name);
        listeners.entry(key).or_insert_with(Vec::new).push(listener);
        Ok(())
    }

    /// Remove a listener
    pub fn remove_listener(&self, extension_id: &str, event_name: &str) -> Result<(), String> {
        let mut listeners = self.listeners.lock().map_err(|e| format!("Lock error: {}", e))?;
        let key = format!("{}:{}", extension_id, event_name);
        listeners.remove(&key);
        Ok(())
    }

    /// Get all listeners for an event
    pub fn get_listeners(&self, event_name: &str) -> Vec<WebNavigationListener> {
        let listeners = self.listeners.lock().ok();
        match listeners {
            Some(guard) => guard
                .values()
                .flatten()
                .filter(|l| l.event_name == event_name)
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    /// Remove all listeners for an extension
    pub fn remove_all_listeners(&self, extension_id: &str) -> Result<(), String> {
        let mut listeners = self.listeners.lock().map_err(|e| format!("Lock error: {}", e))?;
        listeners.retain(|key, _| !key.starts_with(&format!("{}:", extension_id)));
        Ok(())
    }
}

impl Default for WebNavigationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a navigation event matches a filter
pub fn matches_filter(details: &NavigationDetails, filter: &WebNavigationFilter) -> bool {
    if let Some(tab_id) = filter.tab_id {
        if details.tab_id != tab_id {
            return false;
        }
    }
    if let Some(frame_id) = filter.frame_id {
        if details.frame_id != frame_id {
            return false;
        }
    }
    if let Some(url_patterns) = &filter.url {
        if !url_patterns.is_empty() {
            let mut matched = false;
            for pattern in url_patterns {
                if url_matches_pattern(&details.url, pattern) {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return false;
            }
        }
    }
    true
}

/// Simple URL pattern matching (can be enhanced)
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    if pattern == "<all_urls>" {
        return true;
    }
    if pattern.contains("*") {
        let pattern_regex = pattern.replace(".", r"\.").replace("*", ".*");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern_regex)) {
            return re.is_match(url);
        }
    }
    url == pattern
}

/// Fire a navigation event to all registered listeners
pub fn fire_navigation_event(
    app: &tauri::AppHandle,
    registry: &WebNavigationRegistry,
    event_name: &str,
    details: NavigationDetails,
) {
    let listeners = registry.get_listeners(event_name);
    for listener in listeners {
        if let Some(filter) = &listener.filter {
            if !matches_filter(&details, filter) {
                continue;
            }
        }
        let event_name = format!("web-navigation-{}", event_name);
        let _ = app.emit(&listener.extension_id, event_name);
    }
}
