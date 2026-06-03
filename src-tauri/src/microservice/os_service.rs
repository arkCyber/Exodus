//! Exodus OS Service - Microservice for operating system operations
//! 
//! This service provides OS-level functionality as a standalone microservice
//! using JSON-RPC 2.0 over Unix Domain Sockets.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// OS service configuration
#[derive(Debug, Clone)]
pub struct OsServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for OsServiceConfig {
    fn default() -> Self {
        let socket_dir = std::env::temp_dir().join("exodus-services");
        std::fs::create_dir_all(&socket_dir).ok();
        
        Self {
            socket_path: socket_dir.join("os-service.sock"),
        }
    }
}

/// OS service instance
pub struct OsService {
    config: OsServiceConfig,
    running: Arc<Mutex<bool>>,
}

impl OsService {
    pub fn new(config: OsServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut running) = self.running.lock() {
            if *running {
                return Err("Service already running".into());
            }
            *running = true;
        }
        
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }
        
        let listener = UnixListener::bind(&self.config.socket_path)?;
        println!("OS service listening on: {:?}", self.config.socket_path);
        
        let running_flag = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            while running_flag.lock().as_ref().map(|r| **r).unwrap_or(false) {
                match listener.accept().await {
                    Ok((mut socket, _)) => {
                        tokio::spawn(async move {
                            let mut buf = [0u8; 8192];
                            loop {
                                match socket.read(&mut buf).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        let request = String::from_utf8_lossy(&buf[..n]).to_string();
                                        if let Ok(response) = handle_request(&request) {
                                            let _ = socket.write_all(response.as_bytes()).await;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("OS service accept error: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
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
    
    #[allow(dead_code)]
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
fn handle_request(request: &str) -> Result<String, String> {
    let req: JsonRpcRequest = serde_json::from_str(request)
        .map_err(|e| format!("Failed to parse request: {}", e))?;
    
    let result = match req.method.as_str() {
        "get_platform" => handle_get_platform(),
        "get_arch" => handle_get_arch(),
        "get_temp_dir" => handle_get_temp_dir(),
        "get_home_dir" => handle_get_home_dir(),
        "path_exists" => handle_path_exists(&req.params),
        "read_file" => handle_read_file(&req.params),
        "write_file" => handle_write_file(&req.params),
        "delete_file" => handle_delete_file(&req.params),
        "list_dir" => handle_list_dir(&req.params),
        "create_dir" => handle_create_dir(&req.params),
        "remove_dir" => handle_remove_dir(&req.params),
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

/// Get platform information
fn handle_get_platform() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "platform": std::env::consts::OS,
        "family": if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "unix" }
    }))
}

/// Get architecture
fn handle_get_arch() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "arch": std::env::consts::ARCH }))
}

/// Get temp directory
fn handle_get_temp_dir() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "temp_dir": std::env::temp_dir().to_string_lossy().to_string() }))
}

/// Get home directory
fn handle_get_home_dir() -> Result<serde_json::Value, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    Ok(serde_json::json!({ "home_dir": home }))
}

/// Check if path exists
fn handle_path_exists(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    
    let exists = std::path::Path::new(path).exists();
    
    Ok(serde_json::json!({ "exists": exists }))
}

/// Read file
fn handle_read_file(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    Ok(serde_json::json!({ "content": content }))
}

/// Write file
fn handle_write_file(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    let content = params["content"].as_str().ok_or("Missing content".to_string())?;
    
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(serde_json::json!({ "success": true }))
}

/// Delete file
fn handle_delete_file(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    
    std::fs::remove_file(path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;
    
    Ok(serde_json::json!({ "success": true }))
}

/// List directory
fn handle_list_dir(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            serde_json::json!({
                "name": entry.file_name().to_string_lossy().to_string(),
                "is_dir": is_dir,
                "size": size
            })
        })
        .collect::<Vec<_>>();
    
    Ok(serde_json::json!({ "entries": entries }))
}

/// Create directory
fn handle_create_dir(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    let recursive = params["recursive"].as_bool().unwrap_or(true);
    
    if recursive {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    } else {
        std::fs::create_dir(path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    Ok(serde_json::json!({ "success": true }))
}

/// Remove directory
fn handle_remove_dir(params: &Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let params = params.as_ref().ok_or("Missing params".to_string())?;
    let path = params["path"].as_str().ok_or("Missing path".to_string())?;
    let recursive = params["recursive"].as_bool().unwrap_or(true);
    
    if recursive {
        std::fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove directory: {}", e))?;
    } else {
        std::fs::remove_dir(path)
            .map_err(|e| format!("Failed to remove directory: {}", e))?;
    }
    
    Ok(serde_json::json!({ "success": true }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_os_service_config_default() {
        let config = OsServiceConfig::default();
        assert!(config.socket_path.ends_with("os-service.sock"));
    }
    
    #[test]
    fn test_os_service_creation() {
        let config = OsServiceConfig::default();
        let service = OsService::new(config);
        assert!(service.is_ok());
    }
    
    #[test]
    fn test_get_platform() {
        let result = handle_get_platform();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_get_arch() {
        let result = handle_get_arch();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_get_temp_dir() {
        let result = handle_get_temp_dir();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_get_home_dir() {
        let result = handle_get_home_dir();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_path_exists() {
        let params = serde_json::json!({ "path": "/tmp" });
        let result = handle_path_exists(&Some(params));
        assert!(result.is_ok());
    }
}
