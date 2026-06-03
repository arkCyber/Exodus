//! Chrome Extension API - chrome.topSites
//!
//! Provides Tauri commands for the chrome.topSites API, allowing extensions
//! to interact with the browser's most visited sites.

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

/// Most visited site
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MostVisitedURL {
    pub url: String,
    pub title: String,
    pub favicon_url: Option<String>,
}

/// Get most visited sites
#[tauri::command]
pub async fn chrome_topsites_get(
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<MostVisitedURL>, String> {
    // Get most visited sites from history manager
    let history_manager = active_history(&stores, &config)?;
    
    // Get most visited history entries (default limit: 12)
    let most_visited = history_manager.get_most_visited(12);
    
    // Convert to MostVisitedURL format
    let top_sites: Vec<MostVisitedURL> = most_visited
        .into_iter()
        .map(|entry| MostVisitedURL {
            url: entry.url.clone(),
            title: entry.title.clone(),
            favicon_url: entry.favicon.clone(),
        })
        .collect();
    
    Ok(top_sites)
}

/// Get most visited sites with options
#[tauri::command]
pub async fn chrome_topsites_get_with_options(
    options: TopSitesOptions,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    stores: State<'_, ProfileHistoryStores>,
    config: State<'_, ConfigState>,
) -> Result<Vec<MostVisitedURL>, String> {
    // Get most visited sites with filtering options
    let history_manager = active_history(&stores, &config)?;
    
    // Get the number of results from options (default: 12)
    let num_results = options.num_results.unwrap_or(12) as usize;
    
    // Get most visited history entries
    let most_visited = history_manager.get_most_visited(num_results);
    
    // Convert to MostVisitedURL format
    let top_sites: Vec<MostVisitedURL> = most_visited
        .into_iter()
        .map(|entry| MostVisitedURL {
            url: entry.url.clone(),
            title: entry.title.clone(),
            favicon_url: entry.favicon.clone(),
        })
        .collect();
    
    Ok(top_sites)
}

/// Top sites options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopSitesOptions {
    /// Number of results to return
    #[serde(default)]
    pub num_results: Option<u32>,
    /// Whether to include thumbnails
    #[serde(default)]
    pub include_thumbnails: Option<bool>,
    /// Whether to include favicons
    #[serde(default)]
    pub include_favicons: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_most_visited_url_serialization() {
        let site = MostVisitedURL {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            favicon_url: Some("https://example.com/favicon.ico".to_string()),
        };
        let json = serde_json::to_string(&site).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("Example"));
    }

    #[test]
    fn test_topsites_options_deserialization() {
        let options_json = r#"{"numResults":10,"includeThumbnails":true}"#;
        let options: TopSitesOptions = serde_json::from_str(options_json).unwrap();
        assert_eq!(options.num_results, Some(10));
        assert_eq!(options.include_thumbnails, Some(true));
    }

    #[test]
    fn test_most_visited_url_without_favicon() {
        let site = MostVisitedURL {
            url: "https://github.com".to_string(),
            title: "GitHub".to_string(),
            favicon_url: None,
        };
        let json = serde_json::to_string(&site).unwrap();
        assert!(json.contains("GitHub"));
        assert!(json.contains("url"));
    }

    #[test]
    fn test_topsites_options_all_fields() {
        let options = TopSitesOptions {
            num_results: Some(20),
            include_thumbnails: Some(false),
            include_favicons: Some(true),
        };
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("numResults"));
        assert!(json.contains("includeThumbnails"));
        assert!(json.contains("includeFavicons"));
    }

    #[test]
    fn test_topsites_options_default() {
        let options = TopSitesOptions::default();
        assert!(options.num_results.is_none());
        assert!(options.include_thumbnails.is_none());
        assert!(options.include_favicons.is_none());
    }

    #[test]
    fn test_most_visited_url_camel_case() {
        let site = MostVisitedURL {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            favicon_url: Some("https://example.com/favicon.ico".to_string()),
        };
        let json = serde_json::to_string(&site).unwrap();
        assert!(json.contains("faviconUrl"));
    }
}
