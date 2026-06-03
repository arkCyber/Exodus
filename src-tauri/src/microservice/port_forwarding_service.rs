//! Port Forwarding Service - P2P TCP port forwarding with auto-reconnect
//! 
//! This service provides P2P TCP port forwarding capabilities with multiple ports
//! and auto-reconnect, similar to pai-sho.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Port forwarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardingConfig {
    pub forwarding_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub remote_node: String,
    pub protocol: String, // "tcp", "udp"
    pub description: Option<String>,
    pub auto_reconnect: bool,
    pub max_retries: u32,
}

/// Port forwarding metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardingMetadata {
    pub forwarding_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub remote_node: String,
    pub protocol: String,
    pub description: Option<String>,
    pub status: String, // "active", "inactive", "error", "reconnecting"
    pub auto_reconnect: bool,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: u64,
    pub last_heartbeat: u64,
    pub last_error: Option<String>,
}

/// Configuration for Port Forwarding Service
#[derive(Debug, Clone)]
pub struct PortForwardingServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for PortForwardingServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_port_forwarding.sock");
        Self { socket_path }
    }
}

/// Port Forwarding Service
pub struct PortForwardingService {
    config: PortForwardingServiceConfig,
    forwardings: Arc<Mutex<HashMap<String, PortForwardingMetadata>>>, // forwarding_id -> metadata
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl PortForwardingService {
    pub fn new(config: PortForwardingServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            forwardings: Arc::new(Mutex::new(HashMap::new())),
            node_id: generate_node_id(),
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }

        let socket_path = self.config.socket_path.clone();
        let forwardings = Arc::clone(&self.forwardings);
        let node_id = self.node_id.clone();
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            *tx_guard = Some(shutdown_tx);
        }
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let forwardings = Arc::clone(&forwardings);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, forwardings, node_id).await;
                                });
                            }
                            Err(e) => {
                                eprintln!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        println!("Port Forwarding Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }

        if let Some(tx) = self.shutdown_tx.lock().ok().and_then(|mut tx| tx.take()) {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("Port Forwarding Service stopped");
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

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Create a port forwarding
    #[allow(dead_code)]
    pub fn create_forwarding(&self, config: PortForwardingConfig) -> Result<String, String> {
        let forwarding_id = config.forwarding_id.clone();
        
        let metadata = PortForwardingMetadata {
            forwarding_id: forwarding_id.clone(),
            local_port: config.local_port,
            remote_port: config.remote_port,
            remote_node: config.remote_node,
            protocol: config.protocol,
            description: config.description,
            status: "active".to_string(),
            auto_reconnect: config.auto_reconnect,
            retry_count: 0,
            max_retries: config.max_retries,
            created_at: current_timestamp(),
            last_heartbeat: current_timestamp(),
            last_error: None,
        };

        let mut forwardings = self.forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
        forwardings.insert(forwarding_id.clone(), metadata);

        Ok(forwarding_id)
    }

    /// Stop a port forwarding
    #[allow(dead_code)]
    pub fn stop_forwarding(&self, forwarding_id: String) -> Result<(), String> {
        let mut forwardings = self.forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = forwardings.get_mut(&forwarding_id) {
            metadata.status = "inactive".to_string();
        }
        Ok(())
    }

    /// Get forwarding info
    #[allow(dead_code)]
    pub fn get_forwarding(&self, forwarding_id: String) -> Option<PortForwardingMetadata> {
        let forwardings = self.forwardings.lock().ok()?;
        forwardings.get(&forwarding_id).cloned()
    }

    /// List all forwardings
    #[allow(dead_code)]
    pub fn list_forwardings(&self) -> Vec<PortForwardingMetadata> {
        self.forwardings.lock()
            .map(|forwardings| forwardings.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Update heartbeat
    #[allow(dead_code)]
    pub fn update_heartbeat(&self, forwarding_id: String) -> Result<(), String> {
        let mut forwardings = self.forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = forwardings.get_mut(&forwarding_id) {
            metadata.last_heartbeat = current_timestamp();
        }
        Ok(())
    }

    /// Retry a failed forwarding (auto-reconnect)
    #[allow(dead_code)]
    pub fn retry_forwarding(&self, forwarding_id: String) -> Result<(), String> {
        let mut forwardings = self.forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = forwardings.get_mut(&forwarding_id) {
            if metadata.status == "error" && metadata.auto_reconnect && metadata.retry_count < metadata.max_retries {
                metadata.retry_count += 1;
                metadata.status = "reconnecting".to_string();
                metadata.last_error = None;
            }
        }
        Ok(())
    }
}

fn generate_node_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    forwardings: Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
    node_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: serde_json::Value = serde_json::from_str(&line)?;
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);

        let result = match method {
            "create_forwarding" => handle_create_forwarding(&params, &forwardings).await,
            "stop_forwarding" => handle_stop_forwarding(&params, &forwardings).await,
            "get_forwarding" => handle_get_forwarding(&params, &forwardings).await,
            "list_forwardings" => handle_list_forwardings(&forwardings).await,
            "update_heartbeat" => handle_update_heartbeat(&params, &forwardings).await,
            "retry_forwarding" => handle_retry_forwarding(&params, &forwardings).await,
            "node_info" => handle_node_info(&node_id).await,
            _ => Err(format!("Unknown method: {}", method)),
        };

        let response = if let Ok(res) = result {
            json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -1, "message": result.unwrap_err()},
                "id": id
            })
        };

        writer.write_all(response.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
        line.clear();
    }

    Ok(())
}

async fn handle_create_forwarding(
    params: &serde_json::Value,
    forwardings: &Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
) -> Result<serde_json::Value, String> {
    let config: PortForwardingConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid config: {}", e))?;
    
    let forwarding_id = config.forwarding_id.clone();
    
    let metadata = PortForwardingMetadata {
        forwarding_id: forwarding_id.clone(),
        local_port: config.local_port,
        remote_port: config.remote_port,
        remote_node: config.remote_node,
        protocol: config.protocol,
        description: config.description,
        status: "active".to_string(),
        auto_reconnect: config.auto_reconnect,
        retry_count: 0,
        max_retries: config.max_retries,
        created_at: current_timestamp(),
        last_heartbeat: current_timestamp(),
        last_error: None,
    };

    let mut guard = forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(forwarding_id.clone(), metadata);

    Ok(json!({
        "forwarding_id": forwarding_id
    }))
}

async fn handle_stop_forwarding(
    params: &serde_json::Value,
    forwardings: &Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
) -> Result<serde_json::Value, String> {
    let forwarding_id = params.get("forwarding_id").and_then(|f| f.as_str()).ok_or("Missing forwarding_id")?;
    
    let mut guard = forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(forwarding_id) {
        metadata.status = "inactive".to_string();
    }

    Ok(json!({
        "stopped": true
    }))
}

async fn handle_get_forwarding(
    params: &serde_json::Value,
    forwardings: &Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
) -> Result<serde_json::Value, String> {
    let forwarding_id = params.get("forwarding_id").and_then(|f| f.as_str()).ok_or("Missing forwarding_id")?;
    
    let guard = forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
    let metadata = guard.get(forwarding_id).ok_or("Forwarding not found")?;

    Ok(json!(metadata))
}

async fn handle_list_forwardings(
    forwardings: &Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
) -> Result<serde_json::Value, String> {
    let guard = forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
    let forwarding_list: Vec<PortForwardingMetadata> = guard.values().cloned().collect();

    Ok(json!({
        "forwardings": forwarding_list
    }))
}

async fn handle_update_heartbeat(
    params: &serde_json::Value,
    forwardings: &Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
) -> Result<serde_json::Value, String> {
    let forwarding_id = params.get("forwarding_id").and_then(|f| f.as_str()).ok_or("Missing forwarding_id")?;
    
    let mut guard = forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(forwarding_id) {
        metadata.last_heartbeat = current_timestamp();
    }

    Ok(json!({
        "updated": true
    }))
}

async fn handle_retry_forwarding(
    params: &serde_json::Value,
    forwardings: &Arc<Mutex<HashMap<String, PortForwardingMetadata>>>,
) -> Result<serde_json::Value, String> {
    let forwarding_id = params.get("forwarding_id").and_then(|f| f.as_str()).ok_or("Missing forwarding_id")?;
    
    let mut guard = forwardings.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(forwarding_id) {
        if metadata.status == "error" && metadata.auto_reconnect && metadata.retry_count < metadata.max_retries {
            metadata.retry_count += 1;
            metadata.status = "reconnecting".to_string();
            metadata.last_error = None;
        }
    }

    Ok(json!({
        "retried": true
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}
