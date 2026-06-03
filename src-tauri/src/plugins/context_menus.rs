//! Exodus Browser — chrome.contextMenus API implementation
//!
//! Provides context menu functionality for extensions with high reliability
//! and safety guarantees following aerospace-grade standards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

/// Context menu item type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContextItemType {
    Normal,
    Checkbox,
    Radio,
    Separator,
}

impl Default for ContextItemType {
    fn default() -> Self {
        Self::Normal
    }
}

/// Context menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuItem {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub contexts: Vec<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub document_url_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub target_url_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub item_type: ContextItemType,
    #[serde(default)]
    pub checked: bool,
}

/// Context menu registry
pub struct ContextMenuRegistry {
    items: Arc<Mutex<HashMap<String, ContextMenuItem>>>,
}

impl ContextMenuRegistry {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a context menu item
    pub fn create(&self, item: ContextMenuItem) -> Result<(), String> {
        let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        items.insert(item.id.clone(), item);
        Ok(())
    }

    /// Update a context menu item
    pub fn update(&self, id: &str, update_properties: ContextMenuItem) -> Result<(), String> {
        let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        if items.contains_key(id) {
            items.insert(id.to_string(), update_properties);
            Ok(())
        } else {
            Err("Item not found".to_string())
        }
    }

    /// Remove a context menu item
    pub fn remove(&self, id: &str) -> Result<(), String> {
        let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        if items.remove(id).is_some() {
            Ok(())
        } else {
            Err("Item not found".to_string())
        }
    }

    /// Remove all context menu items for an extension
    pub fn remove_all(&self, extension_id: &str) -> Result<(), String> {
        let mut items = self.items.lock().map_err(|e| format!("Lock error: {}", e))?;
        items.retain(|id, _| !id.starts_with(&format!("{}:", extension_id)));
        Ok(())
    }

    /// Get all context menu items for an extension
    pub fn get_all(&self, extension_id: &str) -> Vec<ContextMenuItem> {
        let items = self.items.lock().ok();
        match items {
            Some(guard) => guard
                .values()
                .filter(|item| item.id.starts_with(&format!("{}:", extension_id)))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    /// Get a specific context menu item
    pub fn get(&self, id: &str) -> Option<ContextMenuItem> {
        let items = self.items.lock().ok();
        match items {
            Some(guard) => guard.get(id).cloned(),
            None => None,
        }
    }
}

impl Default for ContextMenuRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply flushed context menu ops from a content/background webview outbox.
pub fn apply_context_menu_flush_ops(
    registry: &ContextMenuRegistry,
    ops: &[super::chrome_bridge::ContextMenuFlushOp],
) {
    for op in ops {
        let ext_id = &op.extension_id;
        match op.op.as_str() {
            "create" => {
                if let Some(mut item) = op.item.clone() {
                    item.id = format!("{}:{}", ext_id, item.id);
                    let _ = registry.create(item);
                }
            }
            "update" => {
                if let (Some(id), Some(mut item)) = (&op.id, op.item.clone()) {
                    let prefixed = format!("{}:{}", ext_id, id);
                    item.id = prefixed.clone();
                    let _ = registry.update(&prefixed, item);
                }
            }
            "remove" => {
                if let Some(id) = &op.id {
                    let prefixed = format!("{}:{}", ext_id, id);
                    let _ = registry.remove(&prefixed);
                }
            }
            "removeAll" => {
                let _ = registry.remove_all(ext_id);
            }
            _ => {}
        }
    }
}

/// Host-visible context menu row (extension + item).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostContextMenuEntry {
    pub extension_id: String,
    pub extension_name: String,
    pub item: ContextMenuItem,
}

/// Whether a menu item should appear for the given host context and page URL.
pub fn item_visible_for_host(item: &ContextMenuItem, host_context: &str, page_url: &str) -> bool {
    if !item.visible || !item.enabled {
        return false;
    }
    if matches!(item.item_type, ContextItemType::Separator) {
        return true;
    }
    if !item.contexts.is_empty() && !item.contexts.iter().any(|c| c == host_context) {
        return false;
    }
    if let Some(patterns) = &item.document_url_patterns {
        if !patterns.is_empty() {
            let mut matched = false;
            for pat in patterns {
                if super::match_patterns::url_matches_pattern(page_url, pat) {
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
