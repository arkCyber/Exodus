//! Exodus Browser — Chrome Context Menu API Tauri commands
//!
//! Provides Tauri commands for chrome.contextMenus API

use tauri::{AppHandle, Manager, State};

use super::context_menus::{
    item_visible_for_host, ContextMenuItem, ContextMenuRegistry, HostContextMenuEntry,
};
use super::manager::ExtensionState;
use super::runtime;

/// Create a context menu item
#[tauri::command]
pub fn extension_context_menus_create(
    extension_id: String,
    item: ContextMenuItem,
    state: State<'_, ContextMenuRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    // Prefix the ID with extension_id to avoid conflicts
    let mut prefixed_item = item;
    prefixed_item.id = format!("{}:{}", extension_id, prefixed_item.id);
    registry.create(prefixed_item)
}

/// Update a context menu item
#[tauri::command]
pub fn extension_context_menus_update(
    extension_id: String,
    id: String,
    update_properties: ContextMenuItem,
    state: State<'_, ContextMenuRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    let prefixed_id = format!("{}:{}", extension_id, id);
    let mut prefixed_item = update_properties;
    prefixed_item.id = prefixed_id.clone();
    registry.update(&prefixed_id, prefixed_item)
}

/// Remove a context menu item
#[tauri::command]
pub fn extension_context_menus_remove(
    extension_id: String,
    id: String,
    state: State<'_, ContextMenuRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    let prefixed_id = format!("{}:{}", extension_id, id);
    registry.remove(&prefixed_id)
}

/// Remove all context menu items for an extension
#[tauri::command]
pub fn extension_context_menus_remove_all(
    extension_id: String,
    state: State<'_, ContextMenuRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.remove_all(&extension_id)
}

/// Get all context menu items for an extension
#[tauri::command]
pub fn extension_context_menus_get_all(
    extension_id: String,
    state: State<'_, ContextMenuRegistry>,
) -> Result<Vec<ContextMenuItem>, String> {
    let registry = state.inner();
    Ok(registry.get_all(&extension_id))
}

/// List context menu items from all enabled extensions for the host UI.
#[tauri::command]
pub fn extension_context_menus_list_host(
    page_url: String,
    host_context: String,
    menu_reg: State<'_, ContextMenuRegistry>,
    ext_state: State<'_, ExtensionState>,
) -> Result<Vec<HostContextMenuEntry>, String> {
    let mgr = ext_state
        .inner()
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let registry = menu_reg.inner();
    let mut out = Vec::new();
    for ext in mgr.list() {
        if !ext.enabled {
            continue;
        }
        for item in registry.get_all(&ext.id) {
            if item_visible_for_host(&item, &host_context, &page_url) {
                out.push(HostContextMenuEntry {
                    extension_id: ext.id.clone(),
                    extension_name: ext.name.clone(),
                    item,
                });
            }
        }
    }
    Ok(out)
}

/// Fire `chrome.contextMenus.onClicked` in the extension background service worker.
#[tauri::command]
pub fn extension_context_menu_clicked(
    app: AppHandle,
    extension_id: String,
    menu_item_id: String,
    page_url: String,
) -> Result<(), String> {
    runtime::deliver_context_menu_click(&app, &extension_id, &menu_item_id, &page_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_item_serialization() {
        let item = ContextMenuItem {
            id: "test-item".to_string(),
            title: "Test Menu Item".to_string(),
            contexts: vec!["all".to_string()],
            parent_id: None,
            document_url_patterns: None,
            target_url_patterns: None,
            enabled: true,
            visible: true,
            item_type: crate::plugins::context_menus::ContextItemType::Normal,
            checked: false,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test Menu Item"));
    }

    #[test]
    fn test_context_menu_item_with_parent() {
        let item = ContextMenuItem {
            id: "child-item".to_string(),
            title: "Child Item".to_string(),
            contexts: vec!["link".to_string()],
            parent_id: Some("parent-item".to_string()),
            document_url_patterns: None,
            target_url_patterns: None,
            enabled: true,
            visible: true,
            item_type: crate::plugins::context_menus::ContextItemType::Normal,
            checked: false,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Child Item"));
        assert!(json.contains("parent-item"));
    }

    #[test]
    fn test_context_menu_item_separator() {
        let item = ContextMenuItem {
            id: "separator".to_string(),
            title: "".to_string(),
            contexts: vec!["all".to_string()],
            parent_id: None,
            document_url_patterns: None,
            target_url_patterns: None,
            enabled: true,
            visible: true,
            item_type: crate::plugins::context_menus::ContextItemType::Separator,
            checked: false,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("separator"));
    }

    #[test]
    fn test_context_menu_item_with_url_patterns() {
        let item = ContextMenuItem {
            id: "pattern-item".to_string(),
            title: "Pattern Item".to_string(),
            contexts: vec!["selection".to_string()],
            parent_id: None,
            document_url_patterns: Some(vec!["https://*.example.com/*".to_string()]),
            target_url_patterns: None,
            enabled: true,
            visible: true,
            item_type: crate::plugins::context_menus::ContextItemType::Normal,
            checked: false,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("https://*.example.com/*"));
    }

    #[test]
    fn test_host_context_menu_entry_serialization() {
        let entry = HostContextMenuEntry {
            extension_id: "test-ext".to_string(),
            extension_name: "Test Extension".to_string(),
            item: ContextMenuItem {
                id: "menu-item".to_string(),
                title: "Menu Item".to_string(),
                contexts: vec!["all".to_string()],
                parent_id: None,
                document_url_patterns: None,
                target_url_patterns: None,
                enabled: true,
                visible: true,
                item_type: crate::plugins::context_menus::ContextItemType::Normal,
                checked: false,
            },
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Test Extension"));
        assert!(json.contains("menu-item"));
    }
}
