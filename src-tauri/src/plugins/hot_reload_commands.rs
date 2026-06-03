//! Exodus Browser — Extension Hot Reload Tauri commands
//!
//! Provides Tauri commands for extension hot-reload functionality

use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};

use super::hot_reload::HotReloadRegistry;

/// Enable hot reload for extensions
#[tauri::command]
pub fn extension_hot_reload_enable(
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.enable()
}

/// Disable hot reload for extensions
#[tauri::command]
pub fn extension_hot_reload_disable(
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.disable()
}

/// Check if hot reload is enabled
#[tauri::command]
pub fn extension_hot_reload_is_enabled(
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<bool, String> {
    let registry = state.inner();
    Ok(registry.is_enabled())
}

/// Start watching an extension for changes
#[tauri::command]
pub fn extension_hot_reload_watch(
    extension_id: String,
    extension_path: String,
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<(), String> {
    let registry = state.inner();
    let path = Path::new(&extension_path);
    registry.watch_extension(&extension_id, path)
}

/// Stop watching an extension
#[tauri::command]
pub fn extension_hot_reload_unwatch(
    extension_id: String,
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.unwatch_extension(&extension_id)
}

/// Get list of watched extensions
#[tauri::command]
pub fn extension_hot_reload_get_watched(
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<Vec<String>, String> {
    let registry = state.inner();
    Ok(registry.get_watched_extensions())
}

/// Start hot reload polling (runs in background)
#[tauri::command]
pub fn extension_hot_reload_start_polling(
    app: AppHandle,
    state: State<'_, Arc<HotReloadRegistry>>,
) -> Result<(), String> {
    super::hot_reload::start_hot_reload_polling(state.inner().clone(), app)
}
