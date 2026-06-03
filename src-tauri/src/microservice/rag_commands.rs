//! Tauri commands for RAG service management

use crate::microservice::{RagService, RagServiceConfig, ServiceRegistry, ServiceInfo};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// RAG service instance managed by Tauri
pub struct ManagedRagService {
    service: Arc<RagService>,
    running: Arc<std::sync::Mutex<bool>>,
}

impl ManagedRagService {
    pub fn new(data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = RagServiceConfig::default();
        config.data_dir = data_dir;
        
        let service = Arc::new(RagService::new(config)?);
        
        Ok(Self {
            service,
            running: Arc::new(std::sync::Mutex::new(false)),
        })
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
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
        
        self.service.stop().await.map_err(|e| e.to_string())
    }
    
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
    
    pub fn socket_path(&self) -> PathBuf {
        self.service.socket_path().clone()
    }
}

/// Send JSON-RPC request to RAG service
async fn send_rag_request(socket_path: &PathBuf, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    
    let mut socket = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to RAG service: {}", e))?;
    
    socket.write_all(request.to_string().as_bytes())
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    let mut response_buf = Vec::new();
    let mut temp_buf = [0u8; 8192];
    
    loop {
        match socket.read(&mut temp_buf).await {
            Ok(0) => break,
            Ok(n) => response_buf.extend_from_slice(&temp_buf[..n]),
            Err(e) => return Err(format!("Failed to read response: {}", e)),
        }
    }
    
    let response: serde_json::Value = serde_json::from_slice(&response_buf)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(error) = response.get("error") {
        return Err(format!("RAG service error: {}", error));
    }
    
    response.get("result")
        .cloned()
        .ok_or_else(|| "No result in response".to_string())
}

/// Start RAG service
#[tauri::command]
pub async fn rag_service_start(
    data_dir: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = ManagedRagService::new(PathBuf::from(data_dir))
        .map_err(|e| format!("Failed to create RAG service: {}", e))?;
    
    service.start().await?;
    
    let socket_path = service.socket_path();
    
    // Register in service registry
    let service_info = ServiceInfo::new(
        "rag-service",
        socket_path.to_string_lossy().to_string(),
        std::process::id(),
    );
    
    registry.register(service_info)
        .map_err(|e| format!("Failed to register service: {}", e))?;
    
    Ok(socket_path.to_string_lossy().to_string())
}

/// Stop RAG service
#[tauri::command]
pub async fn rag_service_stop(
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<bool, String> {
    registry.unregister("rag-service")
        .map_err(|e| format!("Failed to unregister service: {}", e))?;
    
    Ok(true)
}

/// Store a page via RAG service
#[tauri::command]
pub async fn rag_store_page(
    url: String,
    title: String,
    content: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get("rag-service")
        .map_err(|e| format!("RAG service not found: {}", e))?
        .ok_or("RAG service not registered")?;
    
    let socket_path = PathBuf::from(service.socket_path);
    
    let params = serde_json::json!({
        "url": url,
        "title": title,
        "content": content
    });
    
    let result = send_rag_request(&socket_path, "store_page", params).await?;
    
    result.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid response".to_string())
}

/// Search pages via RAG service
#[tauri::command]
pub async fn rag_search_pages(
    query: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get("rag-service")
        .map_err(|e| format!("RAG service not found: {}", e))?
        .ok_or("RAG service not registered")?;
    
    let socket_path = PathBuf::from(service.socket_path);
    
    let params = serde_json::json!({ "query": query });
    
    let result = send_rag_request(&socket_path, "search_pages", params).await?;
    Ok(result.to_string())
}

/// Add bookmark via RAG service
#[tauri::command]
pub async fn rag_add_bookmark(
    url: String,
    title: String,
    folder: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get("rag-service")
        .map_err(|e| format!("RAG service not found: {}", e))?
        .ok_or("RAG service not registered")?;
    
    let socket_path = PathBuf::from(service.socket_path);
    
    let params = serde_json::json!({
        "url": url,
        "title": title,
        "folder": folder
    });
    
    let result = send_rag_request(&socket_path, "add_bookmark", params).await?;
    Ok(result.to_string())
}

/// List bookmarks via RAG service
#[tauri::command]
pub async fn rag_list_bookmarks(
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get("rag-service")
        .map_err(|e| format!("RAG service not found: {}", e))?
        .ok_or("RAG service not registered")?;
    
    let socket_path = PathBuf::from(service.socket_path);
    
    let result = send_rag_request(&socket_path, "list_bookmarks", serde_json::json!(null)).await?;
    Ok(result.to_string())
}

/// Record visit via RAG service
#[tauri::command]
pub async fn rag_record_visit(
    url: String,
    title: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get("rag-service")
        .map_err(|e| format!("RAG service not found: {}", e))?
        .ok_or("RAG service not registered")?;
    
    let socket_path = PathBuf::from(service.socket_path);
    
    let params = serde_json::json!({
        "url": url,
        "title": title
    });
    
    let result = send_rag_request(&socket_path, "record_visit", params).await?;
    Ok(result.to_string())
}

/// Search visits via RAG service
#[tauri::command]
pub async fn rag_search_visits(
    query: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get("rag-service")
        .map_err(|e| format!("RAG service not found: {}", e))?
        .ok_or("RAG service not registered")?;
    
    let socket_path = PathBuf::from(service.socket_path);
    
    let params = serde_json::json!({ "query": query });
    
    let result = send_rag_request(&socket_path, "search_visits", params).await?;
    Ok(result.to_string())
}
