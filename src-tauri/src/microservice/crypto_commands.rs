//! Tauri commands for crypto-service microservice
//! 
//! These commands allow the frontend to interact with the crypto microservice
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{CryptoService, CryptoServiceConfig, ServiceInfo, extract_12_digit_from_hex, format_12_digit_with_hyphens};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed crypto service instance
pub struct ManagedCryptoService {
    service: Arc<CryptoService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedCryptoService {
    pub fn new(service: CryptoService) -> Self {
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

/// Send JSON-RPC request to crypto service
async fn send_crypto_request(
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
        .map_err(|e| format!("Failed to connect to crypto service: {}", e))?;

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
        return Err(format!("Crypto service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the crypto service
#[tauri::command]
pub async fn crypto_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = CryptoServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = CryptoService::new(config)
        .map_err(|e| format!("Failed to create crypto service: {}", e))?;
    
    let managed = ManagedCryptoService::new(service);
    managed.start().await?;
    
    let service_info = ServiceInfo::new(
        "crypto-service",
        socket_path.to_string_lossy().to_string(),
        std::process::id(),
    );
    
    let _ = app.emit("crypto-service-started", service_info);
    
    Ok(())
}

/// Stop the crypto service
#[tauri::command]
pub async fn crypto_service_stop() -> Result<(), String> {
    let config = CryptoServiceConfig::default();
    let service = CryptoService::new(config)
        .map_err(|e| format!("Failed to create crypto service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Hash data
#[tauri::command]
pub async fn crypto_hash(data: String) -> Result<String, String> {
    let config = CryptoServiceConfig::default();
    let params = json!({ "data": data });
    let result = send_crypto_request(&config.socket_path, "hash", params).await?;
    
    result.get("hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid hash response".to_string())
}

/// Generate UUID
#[tauri::command]
pub async fn crypto_uuid_generate() -> Result<String, String> {
    let config = CryptoServiceConfig::default();
    let result = send_crypto_request(&config.socket_path, "uuid_generate", json!(null)).await?;
    
    result.get("uuid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid UUID response".to_string())
}

/// Extract 12-digit number from NodeID hex string
#[tauri::command]
pub async fn crypto_extract_12_digit(node_id_hex: String) -> Result<u64, String> {
    extract_12_digit_from_hex(&node_id_hex)
        .map_err(|e| format!("[{}] {}", e.error_code(), e))
}

/// Get formatted 12-digit ID with hyphens (4-4-4 format)
#[tauri::command]
pub async fn crypto_get_formatted_id(node_id_hex: String) -> Result<String, String> {
    let num = extract_12_digit_from_hex(&node_id_hex)
        .map_err(|e| format!("[{}] {}", e.error_code(), e))?;
    let formatted = format_12_digit_with_hyphens(num);
    Ok(formatted)
}

/// Generate QR code for 12-digit number
#[tauri::command]
pub async fn crypto_generate_qr_code(digit_id: String) -> Result<String, String> {
    let config = CryptoServiceConfig::default();
    let params = json!({ "digit_id": digit_id });
    let result = send_crypto_request(&config.socket_path, "generate_qr_code", params).await?;
    
    result.get("qr_code")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid qr_code response".to_string())
}

/// Parse QR code data and extract 12-digit number
#[tauri::command]
pub async fn crypto_parse_qr_code(qr_data: String) -> Result<String, String> {
    let config = CryptoServiceConfig::default();
    let params = json!({ "qr_data": qr_data });
    let result = send_crypto_request(&config.socket_path, "parse_qr_code", params).await?;
    
    result.get("digit_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid digit_id response".to_string())
}

