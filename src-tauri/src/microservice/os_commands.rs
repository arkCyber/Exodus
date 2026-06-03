//! Tauri commands for os-service microservice
//! 
//! These commands allow the frontend to interact with the OS microservice
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{OsService, OsServiceConfig, ServiceInfo};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed OS service instance
pub struct ManagedOsService {
    service: Arc<OsService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedOsService {
    pub fn new(service: OsService) -> Self {
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

/// Send JSON-RPC request to OS service
async fn send_os_request(
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
        .map_err(|e| format!("Failed to connect to OS service: {}", e))?;

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
        return Err(format!("OS service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the OS service
#[tauri::command]
pub async fn os_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = OsServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = OsService::new(config)
        .map_err(|e| format!("Failed to create OS service: {}", e))?;
    
    let managed = ManagedOsService::new(service);
    managed.start().await?;
    
    let service_info = ServiceInfo::new(
        "os-service",
        socket_path.to_string_lossy().to_string(),
        std::process::id(),
    );
    
    let _ = app.emit("os-service-started", service_info);
    
    Ok(())
}

/// Stop the OS service
#[tauri::command]
pub async fn os_service_stop() -> Result<(), String> {
    let config = OsServiceConfig::default();
    let service = OsService::new(config)
        .map_err(|e| format!("Failed to create OS service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Get platform information
#[tauri::command]
pub async fn os_get_platform() -> Result<String, String> {
    let config = OsServiceConfig::default();
    let result = send_os_request(&config.socket_path, "get_platform", json!(null)).await?;
    
    result.get("platform")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid platform response".to_string())
}

/// Get architecture
#[tauri::command]
pub async fn os_get_arch() -> Result<String, String> {
    let config = OsServiceConfig::default();
    let result = send_os_request(&config.socket_path, "get_arch", json!(null)).await?;
    
    result.get("arch")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid arch response".to_string())
}

/// Get temp directory
#[tauri::command]
pub async fn os_get_temp_dir() -> Result<String, String> {
    let config = OsServiceConfig::default();
    let result = send_os_request(&config.socket_path, "get_temp_dir", json!(null)).await?;
    
    result.get("temp_dir")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid temp_dir response".to_string())
}

/// Get home directory
#[tauri::command]
pub async fn os_get_home_dir() -> Result<String, String> {
    let config = OsServiceConfig::default();
    let result = send_os_request(&config.socket_path, "get_home_dir", json!(null)).await?;
    
    result.get("home_dir")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid home_dir response".to_string())
}

/// Check if path exists
#[tauri::command]
pub async fn os_path_exists(path: String) -> Result<bool, String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path });
    let result = send_os_request(&config.socket_path, "path_exists", params).await?;
    
    result.get("exists")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid exists response".to_string())
}

/// Read file
#[tauri::command]
pub async fn os_read_file(path: String) -> Result<String, String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path });
    let result = send_os_request(&config.socket_path, "read_file", params).await?;
    
    result.get("content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid content response".to_string())
}

/// Write file
#[tauri::command]
pub async fn os_write_file(path: String, content: String) -> Result<(), String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path, "content": content });
    send_os_request(&config.socket_path, "write_file", params).await?;
    Ok(())
}

/// Delete file
#[tauri::command]
pub async fn os_delete_file(path: String) -> Result<(), String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path });
    send_os_request(&config.socket_path, "delete_file", params).await?;
    Ok(())
}

/// List directory
#[tauri::command]
pub async fn os_list_dir(path: String) -> Result<Vec<serde_json::Value>, String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path });
    let result = send_os_request(&config.socket_path, "list_dir", params).await?;
    
    result.get("entries")
        .and_then(|v| v.as_array())
        .map(|arr| arr.clone())
        .ok_or_else(|| "Invalid entries response".to_string())
}

/// Create directory
#[tauri::command]
pub async fn os_create_dir(path: String, recursive: bool) -> Result<(), String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path, "recursive": recursive });
    send_os_request(&config.socket_path, "create_dir", params).await?;
    Ok(())
}

/// Remove directory
#[tauri::command]
pub async fn os_remove_dir(path: String, recursive: bool) -> Result<(), String> {
    let config = OsServiceConfig::default();
    let params = json!({ "path": path, "recursive": recursive });
    send_os_request(&config.socket_path, "remove_dir", params).await?;
    Ok(())
}
