//! Exodus Browser — tab registry for Web Extension `chrome.tabs` API.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tab snapshot exposed to extensions (maps UI tab + webview label).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionTabInfo {
    pub id: String,
    pub chrome_tab_id: i64,
    pub index: i64,
    pub webview_label: String,
    pub url: String,
    pub title: String,
    pub active: bool,
}

/// Thread-safe tab list synced from the frontend.
#[derive(Clone, Default)]
pub struct TabRegistry(pub Arc<Mutex<Vec<ExtensionTabInfo>>>);

impl TabRegistry {
    /// Replace the tab list (called from UI on tab changes).
    pub fn sync(&self, tabs: Vec<ExtensionTabInfo>) {
        if let Ok(mut guard) = self.0.lock() {
            *guard = tabs;
        }
    }

    /// Find tab by Chrome numeric id used in shims.
    pub fn find_by_chrome_id(&self, chrome_tab_id: i64) -> Option<ExtensionTabInfo> {
        let guard = self.0.lock().ok()?;
        guard
            .iter()
            .find(|t| t.chrome_tab_id == chrome_tab_id)
            .cloned()
    }

    /// Query tabs (subset of Chrome tabs.query).
    pub fn query(&self, query: &Value) -> Vec<ExtensionTabInfo> {
        let guard = match self.0.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let active_only = query
            .get("active")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let current_window = query
            .get("currentWindow")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        guard
            .iter()
            .filter(|t| {
                if active_only && !t.active {
                    return false;
                }
                if current_window && !t.active {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    /// JSON array for injection into content scripts.
    pub fn inject_json(&self) -> String {
        let guard = match self.0.lock() {
            Ok(g) => g,
            Err(_) => return "[]".to_string(),
        };
        serde_json::to_string(&*guard).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn query_active_tab() {
        let reg = TabRegistry::default();
        reg.sync(vec![
            ExtensionTabInfo {
                id: "a".into(),
                chrome_tab_id: 1,
                index: 0,
                webview_label: "exodus-tab-a".into(),
                url: "https://a.com".into(),
                title: "A".into(),
                active: false,
            },
            ExtensionTabInfo {
                id: "b".into(),
                chrome_tab_id: 2,
                index: 1,
                webview_label: "exodus-tab-b".into(),
                url: "https://b.com".into(),
                title: "B".into(),
                active: true,
            },
        ]);
        let hits = reg.query(&json!({ "active": true }));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chrome_tab_id, 2);
    }
}
