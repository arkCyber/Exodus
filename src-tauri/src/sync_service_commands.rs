//! Exodus Browser — Cloud Sync Service Tauri commands
//!
//! Provides Tauri commands for cloud synchronization

use tauri::State;

use super::auth::Tokens;
use super::sync_service::{SyncService, SyncResult, SyncChange};

/// Set authentication tokens for sync
#[tauri::command]
pub fn sync_set_tokens(
    tokens: Tokens,
    state: State<'_, SyncService>,
) -> Result<(), String> {
    state.set_tokens(tokens);
    Ok(())
}

/// Set encryption key for sync
#[tauri::command]
pub fn sync_set_encryption_key(
    key: String,
    state: State<'_, SyncService>,
) -> Result<(), String> {
    state.set_encryption_key(key);
    Ok(())
}

/// Sync bookmarks
#[tauri::command]
pub async fn cloud_sync_bookmarks(
    local_bookmarks: Vec<serde_json::Value>,
    state: State<'_, SyncService>,
) -> Result<SyncResult, String> {
    state.sync_bookmarks(local_bookmarks).await
        .map_err(|e| e.to_string())
}

/// Sync history
#[tauri::command]
pub async fn cloud_sync_history(
    local_history: Vec<serde_json::Value>,
    state: State<'_, SyncService>,
) -> Result<SyncResult, String> {
    state.sync_history(local_history).await
        .map_err(|e| e.to_string())
}

/// Full sync
#[tauri::command]
pub async fn cloud_sync_full(
    state: State<'_, SyncService>,
) -> Result<SyncResult, String> {
    state.full_sync().await
        .map_err(|e| e.to_string())
}

/// Push changes
#[tauri::command]
pub async fn cloud_sync_push_changes(
    changes: Vec<SyncChange>,
    state: State<'_, SyncService>,
) -> Result<(), String> {
    state.push_changes(changes).await
        .map_err(|e| e.to_string())
}

/// Pull changes
#[tauri::command]
pub async fn cloud_sync_pull_changes(
    since: String, // ISO 8601 datetime string
    state: State<'_, SyncService>,
) -> Result<Vec<SyncChange>, String> {
    let since_dt = chrono::DateTime::parse_from_rfc3339(&since)
        .map_err(|e| format!("Invalid datetime: {}", e))?
        .with_timezone(&chrono::Utc);
    
    state.pull_changes(since_dt).await
        .map_err(|e| e.to_string())
}
