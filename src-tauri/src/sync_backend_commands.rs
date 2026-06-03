//! Exodus Browser — Cross-Device Sync Backend Tauri commands
//!
//! Provides Tauri commands for sync backend functionality

use tauri::State;

use super::sync_backend::{DeviceInfo, SyncBackendState, SyncConflict, SyncDataItem, UserAccount};

/// Create user account
#[tauri::command]
pub fn sync_create_user(
    email: String,
    display_name: String,
    state: State<'_, SyncBackendState>,
) -> Result<UserAccount, String> {
    state.create_user(email, display_name)
}

/// Get user account
#[tauri::command]
pub fn sync_get_user(
    user_id: String,
    state: State<'_, SyncBackendState>,
) -> Result<Option<UserAccount>, String> {
    Ok(state.get_user(&user_id))
}

/// Get user by email
#[tauri::command]
pub fn sync_get_user_by_email(
    email: String,
    state: State<'_, SyncBackendState>,
) -> Result<Option<UserAccount>, String> {
    Ok(state.get_user_by_email(&email))
}

/// Register device for user
#[tauri::command]
pub fn sync_register_device(
    user_id: String,
    device_name: String,
    device_type: String,
    os: String,
    state: State<'_, SyncBackendState>,
) -> Result<DeviceInfo, String> {
    state.register_device(user_id, device_name, device_type, os)
}

/// Get devices for user
#[tauri::command]
pub fn sync_get_user_devices(
    user_id: String,
    state: State<'_, SyncBackendState>,
) -> Result<Vec<DeviceInfo>, String> {
    state.get_user_devices(&user_id)
}

/// Upload sync data
#[tauri::command]
pub fn sync_upload_data(
    data: SyncDataItem,
    state: State<'_, SyncBackendState>,
) -> Result<(), String> {
    state.upload_sync_data(data)
}

/// Download sync data for user
#[tauri::command]
pub fn sync_download_data(
    user_id: String,
    data_type: Option<String>,
    state: State<'_, SyncBackendState>,
) -> Result<Vec<SyncDataItem>, String> {
    state.download_sync_data(&user_id, data_type)
}

/// Detect conflicts
#[tauri::command]
pub fn sync_detect_conflicts(
    user_id: String,
    state: State<'_, SyncBackendState>,
) -> Result<Vec<SyncConflict>, String> {
    state.detect_conflicts(&user_id)
}

/// Resolve conflict
#[tauri::command]
pub fn sync_resolve_conflict(
    conflict_id: String,
    keep_local: bool,
    state: State<'_, SyncBackendState>,
) -> Result<(), String> {
    state.resolve_conflict(&conflict_id, keep_local)
}

/// Get conflicts for user
#[tauri::command]
pub fn sync_get_user_conflicts(
    user_id: String,
    state: State<'_, SyncBackendState>,
) -> Result<Vec<SyncConflict>, String> {
    state.get_user_conflicts(&user_id)
}

/// Update last sync time
#[tauri::command]
pub fn sync_update_last_sync(
    user_id: String,
    state: State<'_, SyncBackendState>,
) -> Result<(), String> {
    state.update_last_sync(&user_id)
}
