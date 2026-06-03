//! Tauri commands for native plugin management.

use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, State};

use super::native_plugin::{NativePluginInfo, NativePluginManager, PermissionSet};

/// List all loaded native plugins.
#[allow(dead_code)]
#[tauri::command]
pub fn native_plugin_list(
    manager: State<'_, std::sync::Arc<std::sync::Mutex<NativePluginManager>>>,
) -> Result<Vec<NativePluginInfo>, String> {
    let mgr = manager.lock().map_err(|e| e.to_string())?;
    Ok(mgr.list_plugins())
}

/// Load a native plugin from a dynamic library path.
#[allow(dead_code)]
#[tauri::command]
pub fn native_plugin_load(
    manager: State<'_, std::sync::Arc<std::sync::Mutex<NativePluginManager>>>,
    path: String,
) -> Result<String, String> {
    let mut mgr = manager.lock().map_err(|e| e.to_string())?;
    let path_buf = PathBuf::from(path);
    mgr.load_plugin(path_buf).map_err(|e| e.to_string())
}

/// Unload a native plugin by ID.
#[allow(dead_code)]
#[tauri::command]
pub fn native_plugin_unload(
    manager: State<'_, std::sync::Arc<std::sync::Mutex<NativePluginManager>>>,
    id: String,
) -> Result<(), String> {
    let mut mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.unload_plugin(&id).map_err(|e| e.to_string())
}

/// Enable a native plugin.
#[allow(dead_code)]
#[tauri::command]
pub fn native_plugin_enable(
    manager: State<'_, std::sync::Arc<std::sync::Mutex<NativePluginManager>>>,
    id: String,
) -> Result<(), String> {
    let mut mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.enable_plugin(&id).map_err(|e| e.to_string())
}

/// Disable a native plugin.
#[allow(dead_code)]
#[tauri::command]
pub fn native_plugin_disable(
    manager: State<'_, std::sync::Arc<std::sync::Mutex<NativePluginManager>>>,
    id: String,
) -> Result<(), String> {
    let mut mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.disable_plugin(&id).map_err(|e| e.to_string())
}

/// Send a message to a native plugin.
#[allow(dead_code)]
#[tauri::command]
pub fn native_plugin_send_message(
    manager: State<'_, std::sync::Arc<std::sync::Mutex<NativePluginManager>>>,
    id: String,
    message: Value,
) -> Result<Value, String> {
    let mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.send_message(&id, message).map_err(|e| e.to_string())
}

/// Request permissions for a native plugin.
#[allow(dead_code)]
#[tauri::command]
pub async fn native_plugin_request_permissions(
    _app: AppHandle,
    _id: String,
    _permissions: PermissionSet,
) -> Result<bool, String> {
    // In production, this would show a permission dialog to the user
    // For now, we auto-approve in development
    Ok(true)
}
