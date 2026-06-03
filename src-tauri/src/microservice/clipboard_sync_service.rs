//! Clipboard Sync Service - P2P clipboard synchronization
//! 
//! This service provides real-time clipboard synchronization across devices,
//! similar to biter, allowing users to share clipboard content via P2P.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

/// Clipboard item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub item_id: String,
    pub content: String,
    pub content_type: String, // "text", "image", "html", "file"
    pub source_device: String,
    pub source_user: String,
    pub timestamp: u64,
    pub size: u64,
    pub is_encrypted: bool,
}

/// Clipboard sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncConfig {
    pub sync_id: String,
    pub device_id: String,
    pub device_name: String,
    pub sync_mode: String, // "bidirectional", "send_only", "receive_only"
    pub auto_sync: bool,
    pub history_limit: u32,
}

/// Clipboard sync metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardSyncMetadata {
    pub sync_id: String,
    pub device_id: String,
    pub device_name: String,
    pub sync_mode: String,
    pub auto_sync: bool,
    pub history_limit: u32,
    pub status: String, // "active", "inactive", "paused"
    pub connected_devices: Vec<String>,
    pub last_sync: u64,
    pub created_at: u64,
}

/// Configuration for Clipboard Sync Service
#[derive(Debug, Clone)]
pub struct ClipboardSyncServiceConfig {
    pub socket_path: PathBuf,
}

impl Default for ClipboardSyncServiceConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_clipboard_sync.sock");
        Self { socket_path }
    }
}

/// Clipboard Sync Service
pub struct ClipboardSyncService {
    config: ClipboardSyncServiceConfig,
    syncs: Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>, // sync_id -> metadata
    clipboard_history: Arc<Mutex<HashMap<String, Vec<ClipboardItem>>>>, // sync_id -> history
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl ClipboardSyncService {
    pub fn new(config: ClipboardSyncServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            syncs: Arc::new(Mutex::new(HashMap::new())),
            clipboard_history: Arc::new(Mutex::new(HashMap::new())),
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
        let syncs = Arc::clone(&self.syncs);
        let clipboard_history = Arc::clone(&self.clipboard_history);
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
                                let syncs = Arc::clone(&syncs);
                                let clipboard_history = Arc::clone(&clipboard_history);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, syncs, clipboard_history, node_id).await;
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

        println!("Clipboard Sync Service started on {:?}", socket_path);
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

        println!("Clipboard Sync Service stopped");
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

    /// Create a clipboard sync session
    #[allow(dead_code)]
    pub fn create_sync(&self, config: ClipboardSyncConfig) -> Result<String, String> {
        let sync_id = config.sync_id.clone();
        
        let metadata = ClipboardSyncMetadata {
            sync_id: sync_id.clone(),
            device_id: config.device_id,
            device_name: config.device_name,
            sync_mode: config.sync_mode,
            auto_sync: config.auto_sync,
            history_limit: config.history_limit,
            status: "active".to_string(),
            connected_devices: Vec::new(),
            last_sync: current_timestamp(),
            created_at: current_timestamp(),
        };

        let mut syncs = self.syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
        syncs.insert(sync_id.clone(), metadata);

        Ok(sync_id)
    }

    /// Stop a clipboard sync session
    #[allow(dead_code)]
    pub fn stop_sync(&self, sync_id: String) -> Result<(), String> {
        let mut syncs = self.syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = syncs.get_mut(&sync_id) {
            metadata.status = "inactive".to_string();
        }
        Ok(())
    }

    /// Add clipboard item
    #[allow(dead_code)]
    pub fn add_clipboard_item(&self, sync_id: String, item: ClipboardItem) -> Result<(), String> {
        let mut clipboard_history = self.clipboard_history.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut syncs = self.syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        // Get history limit
        let history_limit = if let Some(metadata) = syncs.get(&sync_id) {
            metadata.history_limit
        } else {
            100
        };
        
        clipboard_history.entry(sync_id.clone()).or_insert_with(Vec::new).push(item.clone());
        
        // Enforce history limit
        if let Some(history) = clipboard_history.get_mut(&sync_id) {
            if history.len() > history_limit as usize {
                history.drain(0..history.len() - history_limit as usize);
            }
        }
        
        // Update last sync timestamp
        if let Some(metadata) = syncs.get_mut(&sync_id) {
            metadata.last_sync = current_timestamp();
        }
        
        Ok(())
    }

    /// Get clipboard history
    #[allow(dead_code)]
    pub fn get_clipboard_history(&self, sync_id: String, limit: Option<usize>) -> Vec<ClipboardItem> {
        self.clipboard_history.lock()
            .map(|clipboard_history| {
                let history = clipboard_history.get(&sync_id).cloned().unwrap_or_default();
                
                if let Some(limit) = limit {
                    history.into_iter().rev().take(limit).collect()
                } else {
                    history.into_iter().rev().collect()
                }
            })
            .unwrap_or_default()
    }

    /// Get sync info
    #[allow(dead_code)]
    pub fn get_sync(&self, sync_id: String) -> Option<ClipboardSyncMetadata> {
        let syncs = self.syncs.lock().ok()?;
        syncs.get(&sync_id).cloned()
    }

    /// List all syncs
    #[allow(dead_code)]
    pub fn list_syncs(&self) -> Vec<ClipboardSyncMetadata> {
        self.syncs.lock()
            .map(|syncs| syncs.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Connect device to sync
    #[allow(dead_code)]
    pub fn connect_device(&self, sync_id: String, device_id: String) -> Result<(), String> {
        let mut syncs = self.syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = syncs.get_mut(&sync_id) {
            if !metadata.connected_devices.contains(&device_id) {
                metadata.connected_devices.push(device_id);
            }
        }
        Ok(())
    }

    /// Disconnect device from sync
    #[allow(dead_code)]
    pub fn disconnect_device(&self, sync_id: String, device_id: String) -> Result<(), String> {
        let mut syncs = self.syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(metadata) = syncs.get_mut(&sync_id) {
            metadata.connected_devices.retain(|id| id != &device_id);
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

async fn handle_client(
    stream: tokio::net::UnixStream,
    syncs: Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
    clipboard_history: Arc<Mutex<HashMap<String, Vec<ClipboardItem>>>>,
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
            "create_sync" => handle_create_sync(&params, &syncs).await,
            "stop_sync" => handle_stop_sync(&params, &syncs).await,
            "add_clipboard_item" => handle_add_clipboard_item(&params, &clipboard_history, &syncs).await,
            "get_clipboard_history" => handle_get_clipboard_history(&params, &clipboard_history).await,
            "get_sync" => handle_get_sync(&params, &syncs).await,
            "list_syncs" => handle_list_syncs(&syncs).await,
            "connect_device" => handle_connect_device(&params, &syncs).await,
            "disconnect_device" => handle_disconnect_device(&params, &syncs).await,
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

async fn handle_create_sync(
    params: &serde_json::Value,
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let config: ClipboardSyncConfig = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid config: {}", e))?;
    
    let sync_id = config.sync_id.clone();
    
    let metadata = ClipboardSyncMetadata {
        sync_id: sync_id.clone(),
        device_id: config.device_id,
        device_name: config.device_name,
        sync_mode: config.sync_mode,
        auto_sync: config.auto_sync,
        history_limit: config.history_limit,
        status: "active".to_string(),
        connected_devices: Vec::new(),
        last_sync: current_timestamp(),
        created_at: current_timestamp(),
    };

    let mut guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(sync_id.clone(), metadata);

    Ok(json!({
        "sync_id": sync_id
    }))
}

async fn handle_stop_sync(
    params: &serde_json::Value,
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let sync_id = params.get("sync_id").and_then(|s| s.as_str()).ok_or("Missing sync_id")?;
    
    let mut guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(sync_id) {
        metadata.status = "inactive".to_string();
    }

    Ok(json!({
        "stopped": true
    }))
}

async fn handle_add_clipboard_item(
    params: &serde_json::Value,
    clipboard_history: &Arc<Mutex<HashMap<String, Vec<ClipboardItem>>>>,
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let sync_id = params.get("sync_id").and_then(|s| s.as_str()).ok_or("Missing sync_id")?;
    let item: ClipboardItem = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid item: {}", e))?;
    
    let mut history_guard = clipboard_history.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut syncs_guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let history_limit = if let Some(metadata) = syncs_guard.get(sync_id) {
        metadata.history_limit
    } else {
        100
    };
    
    history_guard.entry(sync_id.to_string()).or_insert_with(Vec::new).push(item.clone());
    
    if let Some(history) = history_guard.get_mut(sync_id) {
        if history.len() > history_limit as usize {
            history.drain(0..history.len() - history_limit as usize);
        }
    }
    
    if let Some(metadata) = syncs_guard.get_mut(sync_id) {
        metadata.last_sync = current_timestamp();
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_get_clipboard_history(
    params: &serde_json::Value,
    clipboard_history: &Arc<Mutex<HashMap<String, Vec<ClipboardItem>>>>,
) -> Result<serde_json::Value, String> {
    let sync_id = params.get("sync_id").and_then(|s| s.as_str()).ok_or("Missing sync_id")?;
    let limit = params.get("limit").and_then(|l| l.as_u64());
    
    let guard = clipboard_history.lock().map_err(|e| format!("Lock error: {}", e))?;
    let history = guard.get(sync_id).cloned().unwrap_or_default();
    
    let items = if let Some(limit) = limit {
        history.into_iter().rev().take(limit as usize).collect::<Vec<_>>()
    } else {
        history.into_iter().rev().collect()
    };

    Ok(json!({
        "items": items
    }))
}

async fn handle_get_sync(
    params: &serde_json::Value,
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let sync_id = params.get("sync_id").and_then(|s| s.as_str()).ok_or("Missing sync_id")?;
    
    let guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    let metadata = guard.get(sync_id).ok_or("Sync not found")?;

    Ok(json!(metadata))
}

async fn handle_list_syncs(
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    let sync_list: Vec<ClipboardSyncMetadata> = guard.values().cloned().collect();

    Ok(json!({
        "syncs": sync_list
    }))
}

async fn handle_connect_device(
    params: &serde_json::Value,
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let sync_id = params.get("sync_id").and_then(|s| s.as_str()).ok_or("Missing sync_id")?;
    let device_id = params.get("device_id").and_then(|d| d.as_str()).ok_or("Missing device_id")?;
    
    let mut guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(sync_id) {
        if !metadata.connected_devices.contains(&device_id.to_string()) {
            metadata.connected_devices.push(device_id.to_string());
        }
    }

    Ok(json!({
        "connected": true
    }))
}

async fn handle_disconnect_device(
    params: &serde_json::Value,
    syncs: &Arc<Mutex<HashMap<String, ClipboardSyncMetadata>>>,
) -> Result<serde_json::Value, String> {
    let sync_id = params.get("sync_id").and_then(|s| s.as_str()).ok_or("Missing sync_id")?;
    let device_id = params.get("device_id").and_then(|d| d.as_str()).ok_or("Missing device_id")?;
    
    let mut guard = syncs.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(metadata) = guard.get_mut(sync_id) {
        metadata.connected_devices.retain(|id| id != &device_id.to_string());
    }

    Ok(json!({
        "disconnected": true
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}
