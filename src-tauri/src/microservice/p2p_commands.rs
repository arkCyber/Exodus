//! Tauri commands for P2P services
//! 
//! These commands allow the frontend to interact with the P2P blobs and gossip services
//! via JSON-RPC over Unix Domain Sockets.

use crate::microservice::gossip_client::gossip_json_rpc;
use crate::microservice::{P2pBlobsService, P2pBlobsConfig, BlobTicket, P2pGossipService, P2pGossipConfig, GossipMessage};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Managed P2P blobs service instance
pub struct ManagedP2pBlobsService {
    service: Arc<P2pBlobsService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedP2pBlobsService {
    pub fn new(service: P2pBlobsService) -> Self {
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

/// Managed P2P gossip service instance
pub struct ManagedP2pGossipService {
    service: Arc<P2pGossipService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedP2pGossipService {
    pub fn new(service: P2pGossipService) -> Self {
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

/// Send JSON-RPC request to P2P blobs service
async fn send_blobs_request(
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
        .map_err(|e| format!("Failed to connect to P2P blobs service: {}", e))?;

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
        return Err(format!("P2P blobs service error: {}", error));
    }
    
    response.get("result").cloned().ok_or_else(|| "No result in response".to_string())
}

/// Send JSON-RPC request to P2P gossip service.
async fn send_gossip_request(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    gossip_json_rpc(socket_path, method, params).await
}

// ============ P2P Blobs Commands ============

/// Start the P2P blobs service
#[tauri::command]
pub async fn p2p_blobs_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = P2pBlobsConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = P2pBlobsService::new(config)
        .map_err(|e| format!("Failed to create P2P blobs service: {}", e))?;
    
    let managed = ManagedP2pBlobsService::new(service);
    managed.start().await?;
    
    let _ = app.emit("p2p-blobs-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the P2P blobs service
#[tauri::command]
pub async fn p2p_blobs_service_stop() -> Result<(), String> {
    let config = P2pBlobsConfig::default();
    let service = P2pBlobsService::new(config)
        .map_err(|e| format!("Failed to create P2P blobs service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Add a blob to the service
#[tauri::command]
pub async fn p2p_blobs_add(data: String) -> Result<String, String> {
    let config = P2pBlobsConfig::default();
    let params = json!({ "data": data });
    let result = send_blobs_request(&config.socket_path, "add_blob", params).await?;
    
    result.get("hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid hash response".to_string())
}

/// Get a blob by hash
#[tauri::command]
pub async fn p2p_blobs_get(hash: String) -> Result<String, String> {
    let config = P2pBlobsConfig::default();
    let params = json!({ "hash": hash });
    let result = send_blobs_request(&config.socket_path, "get_blob", params).await?;
    
    result.get("data")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid data response".to_string())
}

/// List all blob hashes
#[tauri::command]
pub async fn p2p_blobs_list() -> Result<Vec<String>, String> {
    let config = P2pBlobsConfig::default();
    let result = send_blobs_request(&config.socket_path, "list_blobs", json!(null)).await?;
    
    result.get("blobs")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .ok_or_else(|| "Invalid blobs response".to_string())
}

/// Create a ticket for sharing a blob
#[tauri::command]
pub async fn p2p_blobs_create_ticket(hash: String, expires_at: Option<u64>) -> Result<BlobTicket, String> {
    let config = P2pBlobsConfig::default();
    let params = json!({ 
        "hash": hash,
        "expires_at": expires_at
    });
    let result = send_blobs_request(&config.socket_path, "create_ticket", params).await?;
    
    let ticket = result.get("ticket")
        .ok_or_else(|| "No ticket in response".to_string())?;
    
    Ok(BlobTicket {
        node_id: ticket.get("node_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        blob_hash: ticket.get("blob_hash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        format: ticket.get("format").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        expires_at: ticket.get("expires_at").and_then(|v| v.as_u64()),
    })
}

// ============ P2P Gossip Commands ============

/// Start the P2P gossip service
#[tauri::command]
pub async fn p2p_gossip_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = P2pGossipConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = P2pGossipService::new(config);
    
    let managed = ManagedP2pGossipService::new(service);
    managed.start().await?;
    
    let _ = app.emit("p2p-gossip-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the P2P gossip service
#[tauri::command]
pub async fn p2p_gossip_service_stop() -> Result<(), String> {
    let config = P2pGossipConfig::default();
    let service = P2pGossipService::new(config);
    service.stop().await.map_err(|e| e.to_string())
}

/// Subscribe to a topic
#[tauri::command]
pub async fn p2p_gossip_subscribe(topic: String, subscriber_id: String) -> Result<(), String> {
    let config = P2pGossipConfig::default();
    let params = json!({ 
        "topic": topic,
        "subscriber_id": subscriber_id
    });
    send_gossip_request(&config.socket_path, "subscribe", params).await?;
    Ok(())
}

/// Unsubscribe from a topic
#[tauri::command]
pub async fn p2p_gossip_unsubscribe(topic: String, subscriber_id: String) -> Result<(), String> {
    let config = P2pGossipConfig::default();
    let params = json!({ 
        "topic": topic,
        "subscriber_id": subscriber_id
    });
    send_gossip_request(&config.socket_path, "unsubscribe", params).await?;
    Ok(())
}

/// Publish a message to a topic
#[tauri::command]
pub async fn p2p_gossip_publish(topic: String, payload: serde_json::Value) -> Result<String, String> {
    let config = P2pGossipConfig::default();
    let params = json!({ 
        "topic": topic,
        "payload": payload
    });
    let result = send_gossip_request(&config.socket_path, "publish", params).await?;
    
    result.get("message_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid message_id response".to_string())
}

/// Get messages for a topic
#[tauri::command]
pub async fn p2p_gossip_get_messages(topic: String, limit: Option<usize>) -> Result<Vec<GossipMessage>, String> {
    let config = P2pGossipConfig::default();
    let params = json!({ 
        "topic": topic,
        "limit": limit
    });
    let result = send_gossip_request(&config.socket_path, "get_messages", params).await?;
    
    result.get("messages")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .ok_or_else(|| "Invalid messages response".to_string())
}

/// List all topics
#[tauri::command]
pub async fn p2p_gossip_list_topics() -> Result<Vec<String>, String> {
    let config = P2pGossipConfig::default();
    let result = send_gossip_request(&config.socket_path, "list_topics", json!(null)).await?;
    
    result.get("topics")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .ok_or_else(|| "Invalid topics response".to_string())
}

/// Get subscribers for a topic
#[tauri::command]
pub async fn p2p_gossip_get_subscribers(topic: String) -> Result<Vec<String>, String> {
    let config = P2pGossipConfig::default();
    let params = json!({ "topic": topic });
    let result = send_gossip_request(&config.socket_path, "get_subscribers", params).await?;
    
    result.get("subscribers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .ok_or_else(|| "Invalid subscribers response".to_string())
}

/// Get node info
#[tauri::command]
pub async fn p2p_gossip_node_info() -> Result<serde_json::Value, String> {
    let config = P2pGossipConfig::default();
    let result = send_gossip_request(&config.socket_path, "node_info", json!(null)).await?;
    Ok(result)
}
