//! Tauri commands for Terminal Session Service
//! 
//! These commands allow the frontend to interact with the Terminal Session Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{TerminalSessionService, TerminalSessionServiceConfig, TerminalSessionConfig, TerminalSessionMetadata, TerminalOutput};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Terminal Session Service instance
pub struct ManagedTerminalSessionService {
    service: Arc<TerminalSessionService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedTerminalSessionService {
    pub fn new(service: TerminalSessionService) -> Self {
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

/// Send JSON-RPC request to Terminal Session Service
async fn send_terminal_session_request(
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
        .map_err(|e| format!("Failed to connect to Terminal Session Service: {}", e))?;

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
        return Err(format!("Terminal Session Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Terminal Session Service
#[tauri::command]
pub async fn terminal_session_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = TerminalSessionService::new(config)
        .map_err(|e| format!("Failed to create Terminal Session Service: {}", e))?;
    
    let managed = ManagedTerminalSessionService::new(service);
    managed.start().await?;
    
    let _ = app.emit("terminal-session-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Terminal Session Service
#[tauri::command]
pub async fn terminal_session_service_stop() -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let service = TerminalSessionService::new(config)
        .map_err(|e| format!("Failed to create Terminal Session Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a terminal session
#[tauri::command]
pub async fn terminal_session_create(config: TerminalSessionConfig) -> Result<String, String> {
    let service_config = TerminalSessionServiceConfig::default();
    let params = json!(config);
    let result = send_terminal_session_request(&service_config.socket_path, "create_session", params).await?;
    
    result.get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid session_id response".to_string())
}

/// End a terminal session
#[tauri::command]
pub async fn terminal_session_end(session_id: String) -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let params = json!({ "session_id": session_id });
    send_terminal_session_request(&config.socket_path, "end_session", params).await?;
    Ok(())
}

/// Add terminal output
#[tauri::command]
pub async fn terminal_session_add_output(session_id: String, output: TerminalOutput) -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let mut params = json!(output);
    params["session_id"] = json!(session_id);
    send_terminal_session_request(&config.socket_path, "add_output", params).await?;
    Ok(())
}

/// Get terminal outputs
#[tauri::command]
pub async fn terminal_session_get_outputs(session_id: String, limit: Option<usize>) -> Result<Vec<TerminalOutput>, String> {
    let config = TerminalSessionServiceConfig::default();
    let params = json!({ "session_id": session_id, "limit": limit });
    let result = send_terminal_session_request(&config.socket_path, "get_outputs", params).await?;
    
    let outputs = result.get("outputs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid outputs response".to_string())?;
    
    outputs.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get session info
#[tauri::command]
pub async fn terminal_session_get(session_id: String) -> Result<TerminalSessionMetadata, String> {
    let config = TerminalSessionServiceConfig::default();
    let params = json!({ "session_id": session_id });
    let result = send_terminal_session_request(&config.socket_path, "get_session", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse session metadata: {}", e))
}

/// List all sessions
#[tauri::command]
pub async fn terminal_session_list() -> Result<Vec<TerminalSessionMetadata>, String> {
    let config = TerminalSessionServiceConfig::default();
    let result = send_terminal_session_request(&config.socket_path, "list_sessions", json!(null)).await?;
    
    let sessions = result.get("sessions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid sessions response".to_string())?;
    
    sessions.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Connect user to session
#[tauri::command]
pub async fn terminal_session_connect_user(session_id: String, user_id: String) -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let params = json!({ "session_id": session_id, "user_id": user_id });
    send_terminal_session_request(&config.socket_path, "connect_user", params).await?;
    Ok(())
}

/// Disconnect user from session
#[tauri::command]
pub async fn terminal_session_disconnect_user(session_id: String, user_id: String) -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let params = json!({ "session_id": session_id, "user_id": user_id });
    send_terminal_session_request(&config.socket_path, "disconnect_user", params).await?;
    Ok(())
}

/// Send command to terminal
#[tauri::command]
pub async fn terminal_session_send_command(session_id: String, command: String, user_id: String) -> Result<(), String> {
    let config = TerminalSessionServiceConfig::default();
    let params = json!({ "session_id": session_id, "command": command, "user_id": user_id });
    send_terminal_session_request(&config.socket_path, "send_command", params).await?;
    Ok(())
}
