//! Tauri commands for AI Model Service
//! 
//! These commands allow the frontend to interact with the AI Model Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{AiModelService, AiModelServiceConfig, AiModelMetadata, ModelRegistration, ModelSearchQuery};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed AI Model Service instance
pub struct ManagedAiModelService {
    service: Arc<AiModelService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedAiModelService {
    pub fn new(service: AiModelService) -> Self {
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

/// Send JSON-RPC request to AI Model Service
async fn send_ai_model_request(
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
        .map_err(|e| format!("Failed to connect to AI Model Service: {}", e))?;

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
        return Err(format!("AI Model Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the AI Model Service
#[tauri::command]
pub async fn ai_model_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = AiModelServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = AiModelService::new(config);
    
    let managed = ManagedAiModelService::new(service);
    managed.start().await?;
    
    let _ = app.emit("ai-model-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the AI Model Service
#[tauri::command]
pub async fn ai_model_service_stop() -> Result<(), String> {
    let config = AiModelServiceConfig::default();
    let service = AiModelService::new(config);
    service.stop().await.map_err(|e| e.to_string())
}

/// Register an AI model
#[tauri::command]
pub async fn ai_model_register(registration: ModelRegistration) -> Result<(), String> {
    let config = AiModelServiceConfig::default();
    let params = json!(registration);
    send_ai_model_request(&config.socket_path, "register_model", params).await?;
    Ok(())
}

/// Unregister an AI model
#[tauri::command]
pub async fn ai_model_unregister(model_id: String) -> Result<(), String> {
    let config = AiModelServiceConfig::default();
    let params = json!({ "model_id": model_id });
    send_ai_model_request(&config.socket_path, "unregister_model", params).await?;
    Ok(())
}

/// Get AI model metadata
#[tauri::command]
pub async fn ai_model_get(model_id: String) -> Result<AiModelMetadata, String> {
    let config = AiModelServiceConfig::default();
    let params = json!({ "model_id": model_id });
    let result = send_ai_model_request(&config.socket_path, "get_model", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse model metadata: {}", e))
}

/// Search AI models
#[tauri::command]
pub async fn ai_model_search(query: ModelSearchQuery) -> Result<Vec<AiModelMetadata>, String> {
    let config = AiModelServiceConfig::default();
    let params = json!(query);
    let result = send_ai_model_request(&config.socket_path, "search_models", params).await?;
    
    let results = result.get("results")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid results response".to_string())?;
    
    results.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// List all AI models
#[tauri::command]
pub async fn ai_model_list() -> Result<Vec<AiModelMetadata>, String> {
    let config = AiModelServiceConfig::default();
    let result = send_ai_model_request(&config.socket_path, "list_models", json!(null)).await?;
    
    let models = result.get("models")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid models response".to_string())?;
    
    models.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get models from a specific node
#[tauri::command]
pub async fn ai_model_get_node_models(node_id: String) -> Result<Vec<AiModelMetadata>, String> {
    let config = AiModelServiceConfig::default();
    let params = json!({ "node_id": node_id });
    let result = send_ai_model_request(&config.socket_path, "get_node_models", params).await?;
    
    let models = result.get("models")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid models response".to_string())?;
    
    models.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// List all nodes
#[tauri::command]
pub async fn ai_model_list_nodes() -> Result<Vec<String>, String> {
    let config = AiModelServiceConfig::default();
    let result = send_ai_model_request(&config.socket_path, "list_nodes", json!(null)).await?;
    
    result.get("nodes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .ok_or_else(|| "Invalid nodes response".to_string())
}

/// Get node info
#[tauri::command]
pub async fn ai_model_node_info() -> Result<serde_json::Value, String> {
    let config = AiModelServiceConfig::default();
    let result = send_ai_model_request(&config.socket_path, "node_info", json!(null)).await?;
    Ok(result)
}
