//! Local Service Exposure Service - Expose local services to the internet via P2P
//! 
//! This service provides capabilities to expose local services to the internet
//! using P2P CDN and port forwarding, similar to Datum and malai.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Service exposure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceExposureConfig {
    pub service_id: String,
    pub local_port: u16,
    pub service_name: String,
    pub service_type: String, // "http", "tcp", "ssh", "custom"
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: u64,
}

/// Service exposure metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceExposureMetadata {
    pub service_id: String,
    pub local_port: u16,
    pub service_name: String,
    pub service_type: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub node_id: String,
    pub tunnel_port: Option<u16>,
    pub status: String, // "active", "inactive", "error"
    pub created_at: u64,
    pub last_heartbeat: u64,
}

/// Configuration for Service Exposure Service
#[derive(Debug, Clone)]
pub struct ServiceExposureServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for ServiceExposureServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_service_exposure.sock");
        Self { socket_path }
    }
}

/// Service Exposure Service
pub struct ServiceExposureService {
    config: ServiceExposureServiceConfig,
    services: Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>, // service_id -> metadata
    port_mapping: Arc<Mutex<HashMap<u16, u16>>>, // local_port -> tunnel_port
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl ServiceExposureService {
    pub fn new(config: ServiceExposureServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            services: Arc::new(Mutex::new(HashMap::new())),
            port_mapping: Arc::new(Mutex::new(HashMap::new())),
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
        let services = Arc::clone(&self.services);
        let port_mapping = Arc::clone(&self.port_mapping);
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
                                let services = Arc::clone(&services);
                                let port_mapping = Arc::clone(&port_mapping);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, services, port_mapping, node_id).await;
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

        println!("Service Exposure Service started on {:?}", socket_path);
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

        println!("Service Exposure Service stopped");
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

    /// Expose a local service
    #[allow(dead_code)]
    pub fn expose_service(&self, config: ServiceExposureConfig) -> Result<String, String> {
        let service_id = config.service_id.clone();
        let local_port = config.local_port;
        
        // Allocate tunnel port
        let tunnel_port = allocate_tunnel_port(local_port);
        
        let metadata = ServiceExposureMetadata {
            service_id: service_id.clone(),
            local_port,
            service_name: config.service_name,
            service_type: config.service_type,
            description: config.description,
            is_public: config.is_public,
            node_id: self.node_id.clone(),
            tunnel_port: Some(tunnel_port),
            status: "active".to_string(),
            created_at: current_timestamp(),
            last_heartbeat: current_timestamp(),
        };

        let mut services = self.services.lock().map_err(|e| format!("Lock error: {}", e))?;
        services.insert(service_id.clone(), metadata);
        drop(services);

        let mut port_mapping = self.port_mapping.lock().map_err(|e| format!("Lock error: {}", e))?;
        port_mapping.insert(local_port, tunnel_port);
        drop(port_mapping);

        Ok(service_id)
    }

    /// Stop exposing a service
    #[allow(dead_code)]
    pub fn stop_exposure(&self, service_id: String) -> Result<(), String> {
        let mut services = self.services.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = services.get_mut(&service_id) {
            metadata.status = "inactive".to_string();
            if let Some(_tunnel_port) = metadata.tunnel_port {
                let mut port_mapping = self.port_mapping.lock().map_err(|e| format!("Lock error: {}", e))?;
                port_mapping.remove(&metadata.local_port);
            }
        }
        Ok(())
    }

    /// Get exposed service info
    #[allow(dead_code)]
    pub fn get_service(&self, service_id: String) -> Option<ServiceExposureMetadata> {
        let services = self.services.lock().ok()?;
        services.get(&service_id).cloned()
    }

    /// List all exposed services
    #[allow(dead_code)]
    pub fn list_services(&self) -> Vec<ServiceExposureMetadata> {
        self.services.lock()
            .map(|services| services.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Update heartbeat
    #[allow(dead_code)]
    pub fn update_heartbeat(&self, service_id: String) -> Result<(), String> {
        let mut services = self.services.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = services.get_mut(&service_id) {
            metadata.last_heartbeat = current_timestamp();
        }
        Ok(())
    }
}

fn generate_node_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

fn allocate_tunnel_port(local_port: u16) -> u16 {
    // Simple port allocation strategy
    // In a real implementation, this would use a proper port range
    local_port + 10000
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    services: Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>,
    port_mapping: Arc<Mutex<HashMap<u16, u16>>>,
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
            "expose_service" => handle_expose_service(&params, &services, &port_mapping, &node_id).await,
            "stop_exposure" => handle_stop_exposure(&params, &services, &port_mapping).await,
            "get_service" => handle_get_service(&params, &services).await,
            "list_services" => handle_list_services(&services).await,
            "update_heartbeat" => handle_update_heartbeat(&params, &services).await,
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

async fn handle_expose_service(
    params: &serde_json::Value,
    services: &Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>,
    port_mapping: &Arc<Mutex<HashMap<u16, u16>>>,
    node_id: &str,
) -> Result<serde_json::Value, String> {
    let config: ServiceExposureConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid config: {}", e))?;
    
    let service_id = config.service_id.clone();
    let local_port = config.local_port;
    
    let tunnel_port = allocate_tunnel_port(local_port);
    
    let metadata = ServiceExposureMetadata {
        service_id: service_id.clone(),
        local_port,
        service_name: config.service_name,
        service_type: config.service_type,
        description: config.description,
        is_public: config.is_public,
        node_id: node_id.to_string(),
        tunnel_port: Some(tunnel_port),
        status: "active".to_string(),
        created_at: current_timestamp(),
        last_heartbeat: current_timestamp(),
    };

    let mut guard = services.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(service_id.clone(), metadata);
    drop(guard);

    let mut port_guard = port_mapping.lock().map_err(|e| format!("Lock error: {}", e))?;
    port_guard.insert(local_port, tunnel_port);
    drop(port_guard);

    Ok(json!({
        "service_id": service_id,
        "tunnel_port": tunnel_port
    }))
}

async fn handle_stop_exposure(
    params: &serde_json::Value,
    services: &Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>,
    port_mapping: &Arc<Mutex<HashMap<u16, u16>>>,
) -> Result<serde_json::Value, String> {
    let service_id = params.get("service_id").and_then(|s| s.as_str()).ok_or("Missing service_id")?;
    
    let mut guard = services.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(service_id) {
        metadata.status = "inactive".to_string();
        if let Some(_tunnel_port) = metadata.tunnel_port {
            let mut port_guard = port_mapping.lock().map_err(|e| format!("Lock error: {}", e))?;
            port_guard.remove(&metadata.local_port);
        }
    }

    Ok(json!({
        "stopped": true
    }))
}

async fn handle_get_service(
    params: &serde_json::Value,
    services: &Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>,
) -> Result<serde_json::Value, String> {
    let service_id = params.get("service_id").and_then(|s| s.as_str()).ok_or("Missing service_id")?;
    
    let guard = services.lock().map_err(|e| format!("Lock error: {}", e))?;
    let metadata = guard.get(service_id).ok_or("Service not found")?;

    Ok(json!(metadata))
}

async fn handle_list_services(
    services: &Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>,
) -> Result<serde_json::Value, String> {
    let guard = services.lock().map_err(|e| format!("Lock error: {}", e))?;
    let service_list: Vec<ServiceExposureMetadata> = guard.values().cloned().collect();

    Ok(json!({
        "services": service_list
    }))
}

async fn handle_update_heartbeat(
    params: &serde_json::Value,
    services: &Arc<Mutex<HashMap<String, ServiceExposureMetadata>>>,
) -> Result<serde_json::Value, String> {
    let service_id = params.get("service_id").and_then(|s| s.as_str()).ok_or("Missing service_id")?;
    
    let mut guard = services.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(service_id) {
        metadata.last_heartbeat = current_timestamp();
    }

    Ok(json!({
        "updated": true
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}
