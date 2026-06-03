//! Tauri commands for Port Forwarding Service
//! 
//! These commands allow the frontend to interact with the Port Forwarding Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{PortForwardingService, PortForwardingServiceConfig, PortForwardingConfig, PortForwardingMetadata};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Port Forwarding Service instance
pub struct ManagedPortForwardingService {
    service: Arc<PortForwardingService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedPortForwardingService {
    pub fn new(service: PortForwardingService) -> Self {
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

/// Send JSON-RPC request to Port Forwarding Service
async fn send_port_forwarding_request(
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
        .map_err(|e| format!("Failed to connect to Port Forwarding Service: {}", e))?;

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
        return Err(format!("Port Forwarding Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Port Forwarding Service
#[tauri::command]
pub async fn port_forwarding_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = PortForwardingServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = PortForwardingService::new(config)
        .map_err(|e| format!("Failed to create Port Forwarding Service: {}", e))?;
    
    let managed = ManagedPortForwardingService::new(service);
    managed.start().await?;
    
    let _ = app.emit("port-forwarding-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Port Forwarding Service
#[tauri::command]
pub async fn port_forwarding_service_stop() -> Result<(), String> {
    let config = PortForwardingServiceConfig::default();
    let service = PortForwardingService::new(config)
        .map_err(|e| format!("Failed to create Port Forwarding Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a port forwarding
#[tauri::command]
pub async fn port_forwarding_create(config: PortForwardingConfig) -> Result<String, String> {
    let service_config = PortForwardingServiceConfig::default();
    let params = json!(config);
    let result = send_port_forwarding_request(&service_config.socket_path, "create_forwarding", params).await?;
    
    result.get("forwarding_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid forwarding_id response".to_string())
}

/// Stop a port forwarding
#[tauri::command]
pub async fn port_forwarding_stop(forwarding_id: String) -> Result<(), String> {
    let config = PortForwardingServiceConfig::default();
    let params = json!({ "forwarding_id": forwarding_id });
    send_port_forwarding_request(&config.socket_path, "stop_forwarding", params).await?;
    Ok(())
}

/// Get forwarding info
#[tauri::command]
pub async fn port_forwarding_get(forwarding_id: String) -> Result<PortForwardingMetadata, String> {
    let config = PortForwardingServiceConfig::default();
    let params = json!({ "forwarding_id": forwarding_id });
    let result = send_port_forwarding_request(&config.socket_path, "get_forwarding", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse forwarding metadata: {}", e))
}

/// List all forwardings
#[tauri::command]
pub async fn port_forwarding_list() -> Result<Vec<PortForwardingMetadata>, String> {
    let config = PortForwardingServiceConfig::default();
    let result = send_port_forwarding_request(&config.socket_path, "list_forwardings", json!(null)).await?;
    
    let forwardings = result.get("forwardings")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid forwardings response".to_string())?;
    
    forwardings.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Update forwarding heartbeat
#[tauri::command]
pub async fn port_forwarding_update_heartbeat(forwarding_id: String) -> Result<(), String> {
    let config = PortForwardingServiceConfig::default();
    let params = json!({ "forwarding_id": forwarding_id });
    send_port_forwarding_request(&config.socket_path, "update_heartbeat", params).await?;
    Ok(())
}

/// Retry a failed forwarding (auto-reconnect)
#[tauri::command]
pub async fn port_forwarding_retry(forwarding_id: String) -> Result<(), String> {
    let config = PortForwardingServiceConfig::default();
    let params = json!({ "forwarding_id": forwarding_id });
    send_port_forwarding_request(&config.socket_path, "retry_forwarding", params).await?;
    Ok(())
}
