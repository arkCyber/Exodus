//! Exodus Browser — Chrome Storage API Tauri commands
//!
//! Provides Tauri commands for chrome.storage.local and chrome.storage.session

use serde_json::{Map, Value};
use tauri::State;

use super::error::PluginError;
use super::manager::ExtensionState;
use super::storage::ExtensionSessionStorage;

/// Get bytes in use for local storage
#[tauri::command]
pub fn extension_storage_get_bytes_in_use(
    extension_id: String,
    keys: Option<Vec<String>>,
    state: State<'_, ExtensionState>,
) -> Result<usize, String> {
    let mgr = state.lock().map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr
        .permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    mgr.storage()
        .get_bytes_in_use(&extension_id, &perms, keys)
        .map_err(|e: PluginError| e.to_string())
}

/// Get session storage
#[tauri::command]
pub fn extension_storage_session_get(
    extension_id: String,
    keys: Option<Vec<String>>,
    state: State<'_, ExtensionState>,
) -> Result<Map<String, Value>, String> {
    let mgr = state.lock().map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr.permissions_for(&extension_id).map_err(|e: PluginError| e.to_string())?;
    let session_storage = ExtensionSessionStorage::new();
    session_storage
        .get(&extension_id, &perms, keys)
        .map_err(|e: PluginError| e.to_string())
}

/// Set session storage
#[tauri::command]
pub fn extension_storage_session_set(
    extension_id: String,
    items: Map<String, Value>,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mgr = state.lock().map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr.permissions_for(&extension_id).map_err(|e: PluginError| e.to_string())?;
    let session_storage = ExtensionSessionStorage::new();
    session_storage
        .set(&extension_id, &perms, items)
        .map_err(|e: PluginError| e.to_string())
}

/// Remove from session storage
#[tauri::command]
pub fn extension_storage_session_remove(
    extension_id: String,
    keys: Vec<String>,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mgr = state.lock().map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr.permissions_for(&extension_id).map_err(|e: PluginError| e.to_string())?;
    let session_storage = ExtensionSessionStorage::new();
    session_storage
        .remove(&extension_id, &perms, keys)
        .map_err(|e: PluginError| e.to_string())
}

/// Clear session storage
#[tauri::command]
pub fn extension_storage_session_clear(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mgr = state.lock().map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr.permissions_for(&extension_id).map_err(|e: PluginError| e.to_string())?;
    let session_storage = ExtensionSessionStorage::new();
    session_storage
        .clear(&extension_id, &perms)
        .map_err(|e: PluginError| e.to_string())
}

/// Get bytes in use for session storage
#[tauri::command]
pub fn extension_storage_session_get_bytes_in_use(
    extension_id: String,
    keys: Option<Vec<String>>,
    state: State<'_, ExtensionState>,
) -> Result<usize, String> {
    let mgr = state.lock().map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr.permissions_for(&extension_id).map_err(|e: PluginError| e.to_string())?;
    let session_storage = ExtensionSessionStorage::new();
    session_storage
        .get_bytes_in_use(&extension_id, &perms, keys)
        .map_err(|e: PluginError| e.to_string())
}
