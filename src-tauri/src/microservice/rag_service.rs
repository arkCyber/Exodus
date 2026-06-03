//! Exodus RAG Service - Microservice for RAG database operations
//! 
//! This service provides RAG (Retrieval-Augmented Generation) functionality
//! as a standalone microservice using JSON-RPC 2.0 over Unix Domain Sockets.

use crate::rag::{RagDatabase, SessionSnapshot};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// RAG service configuration
#[derive(Debug, Clone)]
pub struct RagServiceConfig {
    pub socket_path: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for RagServiceConfig {
    fn default() -> Self {
        let socket_dir = std::env::temp_dir().join("exodus-services");
        std::fs::create_dir_all(&socket_dir).ok();
        
        let data_dir = std::env::temp_dir().join("exodus-rag-data");
        std::fs::create_dir_all(&data_dir).ok();
        
        Self {
            socket_path: socket_dir.join("rag-service.sock"),
            data_dir,
        }
    }
}

/// RAG service instance
pub struct RagService {
    config: RagServiceConfig,
    db: Arc<RagDatabase>,
    running: Arc<Mutex<bool>>,
}

impl RagService {
    pub fn new(config: RagServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db = RagDatabase::new_at(&config.data_dir)?;
        
        Ok(Self {
            config,
            db: Arc::new(db),
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut running) = self.running.lock() {
            if *running {
                return Err("Service already running".into());
            }
            *running = true;
        } else {
            return Err("Failed to lock running state".into());
        }
        
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }
        
        let listener = UnixListener::bind(&self.config.socket_path)?;
        println!("RAG service listening on: {:?}", self.config.socket_path);
        
        let db = Arc::clone(&self.db);
        let running_flag = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            while running_flag.lock().map(|r| *r).unwrap_or(false) {
                match listener.accept().await {
                    Ok((mut socket, _)) => {
                        let db = Arc::clone(&db);
                        tokio::spawn(async move {
                            let mut buf = [0u8; 8192];
                            loop {
                                match socket.read(&mut buf).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        let request = String::from_utf8_lossy(&buf[..n]).to_string();
                                        if let Ok(response) = handle_request(&db, &request) {
                                            let _ = socket.write_all(response.as_bytes()).await;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("RAG service accept error: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
        
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
    
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: serde_json::Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Handle incoming JSON-RPC requests
fn handle_request(db: &RagDatabase, request: &str) -> Result<String, String> {
    let req: JsonRpcRequest = serde_json::from_str(request)
        .map_err(|e| format!("Failed to parse request: {}", e))?;
    
    let result = match req.method.as_str() {
        "store_page" => handle_store_page(db, &req.params),
        "search_pages" => handle_search_pages(db, &req.params),
        "add_bookmark" => handle_add_bookmark(db, &req.params),
        "list_bookmarks" => handle_list_bookmarks(db),
        "record_visit" => handle_record_visit(db, &req.params),
        "search_visits" => handle_search_visits(db, &req.params),
        "save_session" => handle_save_session(db, &req.params),
        "load_session" => handle_load_session(db),
        "clear_session" => handle_clear_session(db),
        _ => Err("Unknown method".into()),
    };
    
    let response = match result {
        Ok(val) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            result: Some(val),
            error: None,
            id: req.id,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: e,
            }),
            id: req.id,
        },
    };
    
    serde_json::to_string(&response)
        .map_err(|e| format!("Failed to serialize response: {}", e))
}

/// Store a page
fn handle_store_page(db: &RagDatabase, params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let url = params["url"].as_str().ok_or("Missing url".to_string())?;
    let title = params["title"].as_str().ok_or("Missing title".to_string())?;
    let content = params["content"].as_str().ok_or("Missing content".to_string())?;
    
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    let id = rt.block_on(db.upsert_page_by_url(url.into(), title.into(), content.into()))
        .map_err(|e| format!("Failed to upsert page: {}", e))?;
    
    Ok(serde_json::json!({ "id": id }))
}

/// Search pages
fn handle_search_pages(db: &RagDatabase, params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let query = params["query"].as_str().ok_or("Missing query".to_string())?;
    
    let pages = db.search_pages(query)
        .map_err(|e| format!("Failed to search pages: {}", e))?;
    serde_json::to_value(pages)
        .map_err(|e| format!("Failed to serialize pages: {}", e))
}

/// Add a bookmark
fn handle_add_bookmark(db: &RagDatabase, params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let url = params["url"].as_str().ok_or("Missing url".to_string())?;
    let title = params["title"].as_str().ok_or("Missing title".to_string())?;
    let folder = params["folder"].as_str().unwrap_or("");
    
    let bookmark = db.add_bookmark(url.into(), title.into(), folder.into())
        .map_err(|e| format!("Failed to add bookmark: {}", e))?;
    serde_json::to_value(bookmark)
        .map_err(|e| format!("Failed to serialize bookmark: {}", e))
}

/// List bookmarks
fn handle_list_bookmarks(db: &RagDatabase) -> Result<serde_json::Value, String> {
    let bookmarks = db.list_bookmarks()
        .map_err(|e| format!("Failed to list bookmarks: {}", e))?;
    serde_json::to_value(bookmarks)
        .map_err(|e| format!("Failed to serialize bookmarks: {}", e))
}

/// Record a visit
fn handle_record_visit(db: &RagDatabase, params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let url = params["url"].as_str().ok_or("Missing url".to_string())?;
    let title = params["title"].as_str().unwrap_or("");
    
    let visit = db.record_visit(url.into(), title.into())
        .map_err(|e| format!("Failed to record visit: {}", e))?;
    serde_json::to_value(visit)
        .map_err(|e| format!("Failed to serialize visit: {}", e))
}

/// Search visits
fn handle_search_visits(db: &RagDatabase, params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let query = params["query"].as_str().ok_or("Missing query".to_string())?;
    
    let visits = db.search_visits(query)
        .map_err(|e| format!("Failed to search visits: {}", e))?;
    serde_json::to_value(visits)
        .map_err(|e| format!("Failed to serialize visits: {}", e))
}

/// Save session
fn handle_save_session(db: &RagDatabase, params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let snapshot: SessionSnapshot = serde_json::from_value(params.clone())
        .map_err(|e| format!("Failed to parse session: {}", e))?;
    
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    rt.block_on(db.save_session(snapshot))
        .map_err(|e| format!("Failed to save session: {}", e))?;
    
    Ok(serde_json::json!({ "success": true }))
}

/// Load session
fn handle_load_session(db: &RagDatabase) -> Result<serde_json::Value, String> {
    let snapshot = db.load_session()
        .map_err(|e| format!("Failed to load session: {}", e))?;
    serde_json::to_value(snapshot)
        .map_err(|e| format!("Failed to serialize session: {}", e))
}

/// Clear session
fn handle_clear_session(db: &RagDatabase) -> Result<serde_json::Value, String> {
    db.clear_session()
        .map_err(|e| format!("Failed to clear session: {}", e))?;
    Ok(serde_json::json!({ "success": true }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rag_service_config_default() {
        let config = RagServiceConfig::default();
        assert!(config.socket_path.ends_with("rag-service.sock"));
        assert!(config.data_dir.ends_with("exodus-rag-data"));
    }
    
    #[test]
    fn test_rag_service_creation() {
        let config = RagServiceConfig::default();
        let service = RagService::new(config);
        assert!(service.is_ok());
    }
}
