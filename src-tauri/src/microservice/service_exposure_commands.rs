//! Tauri commands for Service Exposure Service
//! 
//! These commands allow the frontend to interact with the Service Exposure Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{ServiceExposureService, ServiceExposureServiceConfig, ServiceExposureConfig, ServiceExposureMetadata};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Service Exposure Service instance
pub struct ManagedServiceExposureService {
    service: Arc<ServiceExposureService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedServiceExposureService {
    pub fn new(service: ServiceExposureService) -> Self {
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

/// Send JSON-RPC request to Service Exposure Service
async fn send_service_exposure_request(
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
        .map_err(|e| format!("Failed to connect to Service Exposure Service: {}", e))?;

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
        return Err(format!("Service Exposure Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Service Exposure Service
#[tauri::command]
pub async fn service_exposure_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = ServiceExposureServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = ServiceExposureService::new(config)
        .map_err(|e| format!("Failed to create Service Exposure Service: {}", e))?;
    
    let managed = ManagedServiceExposureService::new(service);
    managed.start().await?;
    
    let _ = app.emit("service-exposure-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Service Exposure Service
#[tauri::command]
pub async fn service_exposure_service_stop() -> Result<(), String> {
    let config = ServiceExposureServiceConfig::default();
    let service = ServiceExposureService::new(config)
        .map_err(|e| format!("Failed to create Service Exposure Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Expose a local service
#[tauri::command]
pub async fn service_exposure_expose(config: ServiceExposureConfig) -> Result<String, String> {
    let service_config = ServiceExposureServiceConfig::default();
    let params = json!(config);
    let result = send_service_exposure_request(&service_config.socket_path, "expose_service", params).await?;
    
    result.get("service_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid service_id response".to_string())
}

/// Stop exposing a service
#[tauri::command]
pub async fn service_exposure_stop(service_id: String) -> Result<(), String> {
    let config = ServiceExposureServiceConfig::default();
    let params = json!({ "service_id": service_id });
    send_service_exposure_request(&config.socket_path, "stop_exposure", params).await?;
    Ok(())
}

/// Get exposed service info
#[tauri::command]
pub async fn service_exposure_get(service_id: String) -> Result<ServiceExposureMetadata, String> {
    let config = ServiceExposureServiceConfig::default();
    let params = json!({ "service_id": service_id });
    let result = send_service_exposure_request(&config.socket_path, "get_service", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse service metadata: {}", e))
}

/// List all exposed services
#[tauri::command]
pub async fn service_exposure_list() -> Result<Vec<ServiceExposureMetadata>, String> {
    let config = ServiceExposureServiceConfig::default();
    let result = send_service_exposure_request(&config.socket_path, "list_services", json!(null)).await?;
    
    let services = result.get("services")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid services response".to_string())?;
    
    services.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Update service heartbeat
#[tauri::command]
pub async fn service_exposure_update_heartbeat(service_id: String) -> Result<(), String> {
    let config = ServiceExposureServiceConfig::default();
    let params = json!({ "service_id": service_id });
    send_service_exposure_request(&config.socket_path, "update_heartbeat", params).await?;
    Ok(())
}
