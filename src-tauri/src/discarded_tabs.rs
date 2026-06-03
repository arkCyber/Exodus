//! Discarded tab snapshots — destroy WebView to save memory, restore on focus.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// Layout + URL captured when a tab webview is discarded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscardedTabSnapshot {
    pub url: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// In-memory registry of discarded tabs (label → snapshot).
pub struct DiscardedTabsRegistry {
    tabs: Arc<Mutex<HashMap<String, DiscardedTabSnapshot>>>,
}

impl DiscardedTabsRegistry {
    /// Create an empty discarded-tab registry.
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store snapshot for a tab label.
    pub fn insert(&self, label: String, snapshot: DiscardedTabSnapshot) {
        if let Ok(mut map) = self.tabs.lock() {
            map.insert(label, snapshot);
        }
    }

    /// Remove and return snapshot if present.
    pub fn take(&self, label: &str) -> Option<DiscardedTabSnapshot> {
        self.tabs.lock().ok()?.remove(label)
    }

    /// Whether a tab label is currently discarded.
    pub fn is_discarded(&self, label: &str) -> bool {
        self.tabs
            .lock()
            .map(|m| m.contains_key(label))
            .unwrap_or(false)
    }

    /// List discarded tab labels.
    pub fn list_labels(&self) -> Vec<String> {
        self.tabs
            .lock()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }
}

/// Discard a tab: close its webview and keep URL/layout for restore.
#[tauri::command]
pub async fn browser_discard_tab(
    app: AppHandle,
    registry: State<'_, Arc<DiscardedTabsRegistry>>,
    label: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<bool, String> {
    if let Some(wv) = app.get_webview(&label) {
        wv.close()
            .map_err(|e| format!("Close webview for discard failed: {}", e))?;
    }
    registry.insert(
        label.clone(),
        DiscardedTabSnapshot {
            url,
            x,
            y,
            width,
            height,
        },
    );
    let _ = app.emit("exodus-tab-discarded", &label);
    Ok(true)
}

/// Whether the tab webview was discarded (snapshot exists).
#[tauri::command]
pub fn browser_is_tab_discarded(
    registry: State<'_, Arc<DiscardedTabsRegistry>>,
    label: String,
) -> Result<bool, String> {
    Ok(registry.is_discarded(&label))
}

/// Remove discarded snapshot without recreating webview.
#[tauri::command]
pub fn browser_clear_discarded_tab(
    registry: State<'_, Arc<DiscardedTabsRegistry>>,
    label: String,
) -> Result<(), String> {
    let _ = registry.take(&label);
    Ok(())
}
