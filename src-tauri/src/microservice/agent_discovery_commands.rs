//! Tauri commands for Agent Discovery Service
//! 
//! These commands allow the frontend to interact with the Agent Discovery Service
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::{AgentDiscoveryService, AgentDiscoveryServiceConfig, DiscoveryCriteria, AgentRecommendation, TrendingAgent};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed Agent Discovery Service instance
pub struct ManagedAgentDiscoveryService {
    service: Arc<AgentDiscoveryService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedAgentDiscoveryService {
    pub fn new(service: AgentDiscoveryService) -> Self {
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

/// Send JSON-RPC request to Agent Discovery Service
async fn send_agent_discovery_request(
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
        .map_err(|e| format!("Failed to connect to Agent Discovery Service: {}", e))?;

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
        return Err(format!("Agent Discovery Service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Start the Agent Discovery Service
#[tauri::command]
pub async fn agent_discovery_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = AgentDiscoveryServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = AgentDiscoveryService::new(config)
        .map_err(|e| format!("Failed to create Agent Discovery Service: {}", e))?;
    
    let managed = ManagedAgentDiscoveryService::new(service);
    managed.start().await?;
    
    let _ = app.emit("agent-discovery-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Agent Discovery Service
#[tauri::command]
pub async fn agent_discovery_service_stop() -> Result<(), String> {
    let config = AgentDiscoveryServiceConfig::default();
    let service = AgentDiscoveryService::new(config)
        .map_err(|e| format!("Failed to create Agent Discovery Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Register agent for discovery
#[tauri::command]
pub async fn agent_discovery_register(
    agent_id: String,
    agent_type: String,
    capabilities: Vec<String>,
    tags: Vec<String>,
    node_id: String,
    status: String,
) -> Result<(), String> {
    let config = AgentDiscoveryServiceConfig::default();
    let params = json!({
        "agent_id": agent_id,
        "agent_type": agent_type,
        "capabilities": capabilities,
        "tags": tags,
        "node_id": node_id,
        "status": status,
        "last_seen": current_timestamp(),
        "popularity": 100
    });
    send_agent_discovery_request(&config.socket_path, "register_agent", params).await?;
    Ok(())
}

/// Update agent activity
#[tauri::command]
pub async fn agent_discovery_update_activity(agent_id: String) -> Result<(), String> {
    let config = AgentDiscoveryServiceConfig::default();
    let params = json!({ "agent_id": agent_id });
    send_agent_discovery_request(&config.socket_path, "update_agent_activity", params).await?;
    Ok(())
}

/// Discover agents based on criteria
#[tauri::command]
pub async fn agent_discovery_discover(criteria: DiscoveryCriteria) -> Result<Vec<AgentRecommendation>, String> {
    let config = AgentDiscoveryServiceConfig::default();
    let params = json!(criteria);
    let result = send_agent_discovery_request(&config.socket_path, "discover_agents", params).await?;
    
    let recommendations = result.get("recommendations")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid recommendations response".to_string())?;
    
    recommendations.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Get trending agents
#[tauri::command]
pub async fn agent_discovery_trending(time_window: String, limit: Option<usize>) -> Result<Vec<TrendingAgent>, String> {
    let config = AgentDiscoveryServiceConfig::default();
    let params = json!({ 
        "time_window": time_window,
        "limit": limit
    });
    let result = send_agent_discovery_request(&config.socket_path, "get_trending_agents", params).await?;
    
    let trending = result.get("trending")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid trending response".to_string())?;
    
    trending.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Search agents by capability
#[tauri::command]
pub async fn agent_discovery_search_capability(capability: String, limit: Option<usize>) -> Result<Vec<AgentRecommendation>, String> {
    let config = AgentDiscoveryServiceConfig::default();
    let params = json!({ 
        "capability": capability,
        "limit": limit
    });
    let result = send_agent_discovery_request(&config.socket_path, "search_by_capability", params).await?;
    
    let results = result.get("results")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid results response".to_string())?;
    
    results.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}
