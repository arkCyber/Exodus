//! Tauri commands for AI Agent Service
//! 
//! These commands allow the frontend to interact with the AI Agent Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{AiAgentService, AiAgentServiceConfig, AgentIdentity, AgentMessage, AgentPresence};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed AI Agent Service instance
pub struct ManagedAiAgentService {
    service: Arc<AiAgentService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedAiAgentService {
    pub fn new(service: AiAgentService) -> Self {
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

/// Send JSON-RPC request to AI Agent Service
async fn send_ai_agent_request(
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
        .map_err(|e| format!("Failed to connect to AI Agent Service: {}", e))?;

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
        return Err(format!("AI Agent Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the AI Agent Service
#[tauri::command]
pub async fn ai_agent_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = AiAgentService::new(config);
    
    let managed = ManagedAiAgentService::new(service);
    managed.start().await?;
    
    let _ = app.emit("ai-agent-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the AI Agent Service
#[tauri::command]
pub async fn ai_agent_service_stop() -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let service = AiAgentService::new(config);
    service.stop().await.map_err(|e| e.to_string())
}

/// Register an AI agent
#[tauri::command]
pub async fn ai_agent_register(agent: AgentIdentity) -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let params = json!(agent);
    send_ai_agent_request(&config.socket_path, "register_agent", params).await?;
    Ok(())
}

/// Unregister an AI agent
#[tauri::command]
pub async fn ai_agent_unregister(agent_id: String) -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ "agent_id": agent_id });
    send_ai_agent_request(&config.socket_path, "unregister_agent", params).await?;
    Ok(())
}

/// Get AI agent identity
#[tauri::command]
pub async fn ai_agent_get(agent_id: String) -> Result<AgentIdentity, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ "agent_id": agent_id });
    let result = send_ai_agent_request(&config.socket_path, "get_agent", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse agent identity: {}", e))
}

/// List all AI agents
#[tauri::command]
pub async fn ai_agent_list() -> Result<Vec<AgentIdentity>, String> {
    let config = AiAgentServiceConfig::default();
    let result = send_ai_agent_request(&config.socket_path, "list_agents", json!(null)).await?;
    
    let agents = result.get("agents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid agents response".to_string())?;
    
    agents.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Find agents by capability
#[tauri::command]
pub async fn ai_agent_find_by_capability(capability: String) -> Result<Vec<AgentIdentity>, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ "capability": capability });
    let result = send_ai_agent_request(&config.socket_path, "find_agents_by_capability", params).await?;
    
    let agents = result.get("agents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid agents response".to_string())?;
    
    agents.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Find agents by type
#[tauri::command]
pub async fn ai_agent_find_by_type(agent_type: String) -> Result<Vec<AgentIdentity>, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ "agent_type": agent_type });
    let result = send_ai_agent_request(&config.socket_path, "find_agents_by_type", params).await?;
    
    let agents = result.get("agents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid agents response".to_string())?;
    
    agents.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Update agent status
#[tauri::command]
pub async fn ai_agent_update_status(agent_id: String, status: String) -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ 
        "agent_id": agent_id,
        "status": status
    });
    send_ai_agent_request(&config.socket_path, "update_agent_status", params).await?;
    Ok(())
}

/// Link agent to public account
#[tauri::command]
pub async fn ai_agent_link_to_public_account(agent_id: String, account_id: String) -> Result<bool, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ 
        "agent_id": agent_id,
        "account_id": account_id
    });
    let result = send_ai_agent_request(&config.socket_path, "link_to_public_account", params).await?;
    
    result.get("linked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid linked response".to_string())
}

/// Unlink agent from public account
#[tauri::command]
pub async fn ai_agent_unlink_from_public_account(agent_id: String) -> Result<bool, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ "agent_id": agent_id });
    let result = send_ai_agent_request(&config.socket_path, "unlink_from_public_account", params).await?;
    
    result.get("unlinked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid unlinked response".to_string())
}

/// Get agents linked to a public account
#[tauri::command]
pub async fn ai_agent_get_agents_by_public_account(account_id: String) -> Result<Vec<AgentIdentity>, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_ai_agent_request(&config.socket_path, "get_agents_by_public_account", params).await?;
    
    let agents = result.get("agents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid agents response".to_string())?;
    
    agents.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Send message to agent
#[tauri::command]
pub async fn ai_agent_send_message(message: AgentMessage) -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let params = json!(message);
    send_ai_agent_request(&config.socket_path, "send_message", params).await?;
    Ok(())
}

/// Get messages for an agent
#[tauri::command]
pub async fn ai_agent_get_messages(agent_id: String, limit: Option<usize>) -> Result<Vec<AgentMessage>, String> {
    let config = AiAgentServiceConfig::default();
    let params = json!({ 
        "agent_id": agent_id,
        "limit": limit
    });
    let result = send_ai_agent_request(&config.socket_path, "get_messages", params).await?;
    
    let messages = result.get("messages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid messages response".to_string())?;
    
    messages.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Broadcast presence update
#[tauri::command]
pub async fn ai_agent_broadcast_presence(presence: AgentPresence) -> Result<(), String> {
    let config = AiAgentServiceConfig::default();
    let params = json!(presence);
    send_ai_agent_request(&config.socket_path, "broadcast_presence", params).await?;
    Ok(())
}
