//! Tauri commands for Collaborative Editing Service
//! 
//! These commands allow the frontend to interact with the Collaborative Editing Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{CollaborativeEditingService, CollaborativeEditingServiceConfig, DocumentConfig, DocumentMetadata, TextOperation, CursorPosition};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Collaborative Editing Service instance
pub struct ManagedCollaborativeEditingService {
    service: Arc<CollaborativeEditingService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedCollaborativeEditingService {
    pub fn new(service: CollaborativeEditingService) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        self.service.start().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), String> {
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }
        
        self.service.stop().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

/// Send JSON-RPC request to Collaborative Editing Service
async fn send_collaborative_editing_request(
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
        .map_err(|e| format!("Failed to connect to Collaborative Editing Service: {}", e))?;

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
        return Err(format!("Collaborative Editing Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Collaborative Editing Service
#[tauri::command]
pub async fn collaborative_editing_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = CollaborativeEditingServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = CollaborativeEditingService::new(config)
        .map_err(|e| format!("Failed to create Collaborative Editing Service: {}", e))?;
    
    let managed = ManagedCollaborativeEditingService::new(service);
    managed.start().await?;
    
    let _ = app.emit("collaborative-editing-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Collaborative Editing Service
#[tauri::command]
pub async fn collaborative_editing_service_stop() -> Result<(), String> {
    let config = CollaborativeEditingServiceConfig::default();
    let service = CollaborativeEditingService::new(config)
        .map_err(|e| format!("Failed to create Collaborative Editing Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a document
#[tauri::command]
pub async fn collaborative_create_document(config: DocumentConfig) -> Result<String, String> {
    let service_config = CollaborativeEditingServiceConfig::default();
    let params = json!(config);
    let result = send_collaborative_editing_request(&service_config.socket_path, "create_document", params).await?;
    
    result.get("document_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid document_id response".to_string())
}

/// Open a document
#[tauri::command]
pub async fn collaborative_open_document(document_id: String, user_id: String) -> Result<(), String> {
    let config = CollaborativeEditingServiceConfig::default();
    let params = json!({ "document_id": document_id, "user_id": user_id });
    send_collaborative_editing_request(&config.socket_path, "open_document", params).await?;
    Ok(())
}

/// Close a document
#[tauri::command]
pub async fn collaborative_close_document(document_id: String, user_id: String) -> Result<(), String> {
    let config = CollaborativeEditingServiceConfig::default();
    let params = json!({ "document_id": document_id, "user_id": user_id });
    send_collaborative_editing_request(&config.socket_path, "close_document", params).await?;
    Ok(())
}

/// Apply text operation
#[tauri::command]
pub async fn collaborative_apply_operation(document_id: String, operation: TextOperation) -> Result<(), String> {
    let config = CollaborativeEditingServiceConfig::default();
    let mut params = json!(operation);
    params["document_id"] = json!(document_id);
    send_collaborative_editing_request(&config.socket_path, "apply_operation", params).await?;
    Ok(())
}

/// Get document info
#[tauri::command]
pub async fn collaborative_get_document(document_id: String) -> Result<DocumentMetadata, String> {
    let config = CollaborativeEditingServiceConfig::default();
    let params = json!({ "document_id": document_id });
    let result = send_collaborative_editing_request(&config.socket_path, "get_document", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse document metadata: {}", e))
}

/// List all documents
#[tauri::command]
pub async fn collaborative_list_documents() -> Result<Vec<DocumentMetadata>, String> {
    let config = CollaborativeEditingServiceConfig::default();
    let result = send_collaborative_editing_request(&config.socket_path, "list_documents", json!(null)).await?;
    
    let documents = result.get("documents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid documents response".to_string())?;
    
    documents.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Update cursor position
#[tauri::command]
pub async fn collaborative_update_cursor(cursor: CursorPosition) -> Result<(), String> {
    let config = CollaborativeEditingServiceConfig::default();
    let params = json!(cursor);
    send_collaborative_editing_request(&config.socket_path, "update_cursor", params).await?;
    Ok(())
}

/// Get cursors for a document
#[tauri::command]
pub async fn collaborative_get_cursors(document_id: String) -> Result<Vec<CursorPosition>, String> {
    let config = CollaborativeEditingServiceConfig::default();
    let params = json!({ "document_id": document_id });
    let result = send_collaborative_editing_request(&config.socket_path, "get_cursors", params).await?;
    
    let cursors = result.get("cursors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid cursors response".to_string())?;
    
    cursors.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get operations for a document
#[tauri::command]
pub async fn collaborative_get_operations(document_id: String, since_version: Option<u64>) -> Result<Vec<TextOperation>, String> {
    let config = CollaborativeEditingServiceConfig::default();
    let params = json!({ "document_id": document_id, "since_version": since_version });
    let result = send_collaborative_editing_request(&config.socket_path, "get_operations", params).await?;
    
    let operations = result.get("operations")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid operations response".to_string())?;
    
    operations.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}
