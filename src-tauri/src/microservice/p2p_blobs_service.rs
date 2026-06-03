//! P2P Blobs Service - Data sharing microservice inspired by iroh-blobs
//! 
//! This service provides data block sharing with BLAKE3 hash verification,
//! similar to iroh-blobs but adapted for Exodus microservice architecture.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Configuration for P2P Blobs Service
#[derive(Debug, Clone)]
pub struct P2pBlobsConfig {
    pub socket_path: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for P2pBlobsConfig {
    fn default() -> Self {
        let mut socket_path = std::env::temp_dir();
        socket_path.push("exodus_p2p_blobs.sock");
        
        let mut data_dir = std::env::temp_dir();
        data_dir.push("exodus_p2p_blobs_data");
        
        Self { socket_path, data_dir }
    }
}

/// Blob metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BlobMetadata {
    pub hash: String,
    pub size: u64,
    pub format: String,
    pub created_at: u64,
}

/// Blob ticket for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobTicket {
    pub node_id: String,
    pub blob_hash: String,
    pub format: String,
    pub expires_at: Option<u64>,
}

/// P2P Blobs Service
pub struct P2pBlobsService {
    config: P2pBlobsConfig,
    blobs: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
}

impl P2pBlobsService {
    pub fn new(config: P2pBlobsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.data_dir)?;
        
        Ok(Self {
            config,
            blobs: Arc::new(Mutex::new(HashMap::new())),
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
        let blobs = Arc::clone(&self.blobs);
        let _running = Arc::clone(&self.running);
        
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
                                let blobs = Arc::clone(&blobs);
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, blobs).await;
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

        println!("P2P Blobs Service started on {:?}", socket_path);
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

        println!("P2P Blobs Service stopped");
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

    /// Add a blob and return its hash
    #[allow(dead_code)]
    pub fn add_blob(&self, data: Vec<u8>) -> Result<String, String> {
        let hash = blake3::hash(&data).to_hex().to_string();
        let mut blobs = self.blobs.lock().map_err(|e| format!("Lock error: {}", e))?;
        blobs.insert(hash.clone(), data);
        Ok(hash)
    }

    /// Get a blob by hash
    #[allow(dead_code)]
    pub fn get_blob(&self, hash: &str) -> Option<Vec<u8>> {
        let blobs = self.blobs.lock().ok()?;
        blobs.get(hash).cloned()
    }

    /// List all blob hashes
    #[allow(dead_code)]
    pub fn list_blobs(&self) -> Vec<String> {
        self.blobs.lock()
            .map(|blobs| blobs.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Create a ticket for sharing a blob
    #[allow(dead_code)]
    pub fn create_ticket(&self, hash: &str, expires_at: Option<u64>) -> Result<BlobTicket, String> {
        let blobs = self.blobs.lock().map_err(|e| format!("Lock error: {}", e))?;
        if !blobs.contains_key(hash) {
            return Err("Blob not found".to_string());
        }

        let node_id = generate_node_id();
        Ok(BlobTicket {
            node_id,
            blob_hash: hash.to_string(),
            format: "raw".to_string(),
            expires_at,
        })
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    blobs: Arc<Mutex<HashMap<String, Vec<u8>>>>,
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
            "add_blob" => handle_add_blob(&params, &blobs).await,
            "get_blob" => handle_get_blob(&params, &blobs).await,
            "list_blobs" => handle_list_blobs(&blobs).await,
            "create_ticket" => handle_create_ticket(&params, &blobs).await,
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

async fn handle_add_blob(
    params: &serde_json::Value,
    blobs: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
) -> Result<serde_json::Value, String> {
    let data_base64 = params.get("data").and_then(|d| d.as_str()).ok_or("Missing data")?;
    let data = STANDARD.decode(data_base64).map_err(|e| format!("Base64 decode error: {}", e))?;
    
    let hash = blake3::hash(&data).to_hex().to_string();
    let mut guard = blobs.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(hash.clone(), data);

    Ok(json!({
        "hash": hash
    }))
}

async fn handle_get_blob(
    params: &serde_json::Value,
    blobs: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
) -> Result<serde_json::Value, String> {
    let hash = params.get("hash").and_then(|h| h.as_str()).ok_or("Missing hash")?;
    let guard = blobs.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    guard.get(hash)
        .map(|data| json!({
            "data": STANDARD.encode(data),
            "hash": hash
        }))
        .ok_or_else(|| "Blob not found".to_string())
}

async fn handle_list_blobs(
    blobs: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
) -> Result<serde_json::Value, String> {
    let guard = blobs.lock().map_err(|e| format!("Lock error: {}", e))?;
    let hashes: Vec<String> = guard.keys().cloned().collect();
    
    Ok(json!({
        "blobs": hashes
    }))
}

async fn handle_create_ticket(
    params: &serde_json::Value,
    blobs: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
) -> Result<serde_json::Value, String> {
    let hash = params.get("hash").and_then(|h| h.as_str()).ok_or("Missing hash")?;
    let expires_at = params.get("expires_at").and_then(|e| e.as_u64());
    
    let guard = blobs.lock().map_err(|e| format!("Lock error: {}", e))?;
    if !guard.contains_key(hash) {
        return Err("Blob not found".to_string());
    }

    let node_id = generate_node_id();
    Ok(json!({
        "ticket": {
            "node_id": node_id,
            "blob_hash": hash,
            "format": "raw",
            "expires_at": expires_at
        }
    }))
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("node_{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_blob() {
        let config = P2pBlobsConfig::default();
        let service = P2pBlobsService::new(config).expect("Failed to create service");
        
        let data = b"Hello, world!".to_vec();
        let hash = service.add_blob(data.clone()).expect("Failed to add blob");
        
        let retrieved = service.get_blob(&hash).expect("Failed to get blob");
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_list_blobs() {
        let config = P2pBlobsConfig::default();
        let service = P2pBlobsService::new(config).expect("Failed to create service");
        
        service.add_blob(b"blob1".to_vec()).expect("Failed to add blob1");
        service.add_blob(b"blob2".to_vec()).expect("Failed to add blob2");
        
        let blobs = service.list_blobs();
        assert_eq!(blobs.len(), 2);
    }

    #[tokio::test]
    async fn test_create_ticket() {
        let config = P2pBlobsConfig::default();
        let service = P2pBlobsService::new(config).expect("Failed to create service");
        
        let data = b"test data".to_vec();
        let hash = service.add_blob(data).expect("Failed to add blob");
        
        let ticket = service.create_ticket(&hash, None).expect("Failed to create ticket");
        assert_eq!(ticket.blob_hash, hash);
        assert!(!ticket.node_id.is_empty());
    }
}
