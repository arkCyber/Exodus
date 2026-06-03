//! Tauri commands for Clipboard Sync Service
//! 
//! These commands allow the frontend to interact with the Clipboard Sync Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{ClipboardSyncService, ClipboardSyncServiceConfig, ClipboardSyncConfig, ClipboardSyncMetadata, ClipboardItem};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Clipboard Sync Service instance
pub struct ManagedClipboardSyncService {
    service: Arc<ClipboardSyncService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedClipboardSyncService {
    pub fn new(service: ClipboardSyncService) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }
        
        self.service.start().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }
        
        self.service.stop().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
}

/// Send JSON-RPC request to Clipboard Sync Service
async fn send_clipboard_sync_request(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let socket_path_str = socket_path.to_string_lossy().to_string();
    let client = tokio::net::UnixStream::connect(&socket_path_str)
        .await
        .map_err(|e| format!("Failed to connect to Clipboard Sync Service: {}", e))?;

    let (mut reader, mut writer) = client.into_split();
    
    let request_str = serde_json::to_string(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    
    writer.write_all(request_str.as_bytes()).await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    let mut buf = [0u8; 8192];
    let n = reader.read(&mut buf).await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let response_str = String::from_utf8_lossy(&buf[..n]).to_string();
    let response: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(error) = response.get("error") {
        return Err(format!("Clipboard Sync Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Clipboard Sync Service
#[tauri::command]
pub async fn clipboard_sync_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = ClipboardSyncServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = ClipboardSyncService::new(config)
        .map_err(|e| format!("Failed to create Clipboard Sync Service: {}", e))?;
    
    let managed = ManagedClipboardSyncService::new(service);
    managed.start().await?;
    
    let _ = app.emit("clipboard-sync-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Clipboard Sync Service
#[tauri::command]
pub async fn clipboard_sync_service_stop() -> Result<(), String> {
    let config = ClipboardSyncServiceConfig::default();
    let service = ClipboardSyncService::new(config)
        .map_err(|e| format!("Failed to create Clipboard Sync Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a clipboard sync session
#[tauri::command]
pub async fn clipboard_sync_create(config: ClipboardSyncConfig) -> Result<String, String> {
    let service_config = ClipboardSyncServiceConfig::default();
    let params = json!(config);
    let result = send_clipboard_sync_request(&service_config.socket_path, "create_sync", params).await?;
    
    result.get("sync_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid sync_id response".to_string())
}

/// Stop a clipboard sync session
#[tauri::command]
pub async fn clipboard_sync_stop(sync_id: String) -> Result<(), String> {
    let config = ClipboardSyncServiceConfig::default();
    let params = json!({ "sync_id": sync_id });
    send_clipboard_sync_request(&config.socket_path, "stop_sync", params).await?;
    Ok(())
}

/// Add clipboard item
#[tauri::command]
pub async fn clipboard_sync_add_item(sync_id: String, item: ClipboardItem) -> Result<(), String> {
    let config = ClipboardSyncServiceConfig::default();
    let mut params = json!(item);
    params["sync_id"] = json!(sync_id);
    send_clipboard_sync_request(&config.socket_path, "add_clipboard_item", params).await?;
    Ok(())
}

/// Get clipboard history
#[tauri::command]
pub async fn clipboard_sync_get_history(sync_id: String, limit: Option<usize>) -> Result<Vec<ClipboardItem>, String> {
    let config = ClipboardSyncServiceConfig::default();
    let params = json!({ "sync_id": sync_id, "limit": limit });
    let result = send_clipboard_sync_request(&config.socket_path, "get_clipboard_history", params).await?;
    
    let items = result.get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid items response".to_string())?;
    
    items.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get sync info
#[tauri::command]
pub async fn clipboard_sync_get(sync_id: String) -> Result<ClipboardSyncMetadata, String> {
    let config = ClipboardSyncServiceConfig::default();
    let params = json!({ "sync_id": sync_id });
    let result = send_clipboard_sync_request(&config.socket_path, "get_sync", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse sync metadata: {}", e))
}

/// List all syncs
#[tauri::command]
pub async fn clipboard_sync_list() -> Result<Vec<ClipboardSyncMetadata>, String> {
    let config = ClipboardSyncServiceConfig::default();
    let result = send_clipboard_sync_request(&config.socket_path, "list_syncs", json!(null)).await?;
    
    let syncs = result.get("syncs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid syncs response".to_string())?;
    
    syncs.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Connect device to sync
#[tauri::command]
pub async fn clipboard_sync_connect_device(sync_id: String, device_id: String) -> Result<(), String> {
    let config = ClipboardSyncServiceConfig::default();
    let params = json!({ "sync_id": sync_id, "device_id": device_id });
    send_clipboard_sync_request(&config.socket_path, "connect_device", params).await?;
    Ok(())
}

/// Disconnect device from sync
#[tauri::command]
pub async fn clipboard_sync_disconnect_device(sync_id: String, device_id: String) -> Result<(), String> {
    let config = ClipboardSyncServiceConfig::default();
    let params = json!({ "sync_id": sync_id, "device_id": device_id });
    send_clipboard_sync_request(&config.socket_path, "disconnect_device", params).await?;
    Ok(())
}
