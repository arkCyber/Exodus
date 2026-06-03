//! Exodus Browser — Allama microservice control plane (JSON-RPC over Unix socket).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

/// Allama microservice configuration.
#[derive(Debug, Clone)]
pub struct AllamaServiceConfig {
    pub socket_path: PathBuf,
    pub http_port: u16,
}

impl Default for AllamaServiceConfig {
    fn default() -> Self {
        let socket_dir = std::env::temp_dir().join("exodus-services");
        std::fs::create_dir_all(&socket_dir).ok();
        Self {
            socket_path: socket_dir.join("allama-service.sock"),
            http_port: super::allama_process::ALLAMA_DEFAULT_PORT,
        }
    }
}

/// Runtime mode reported to clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AllamaRuntimeMode {
    Stopped,
    NativeBinary,
    EmbeddedGateway,
}

/// Allama service status snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllamaServiceStatus {
    pub running: bool,
    pub mode: AllamaRuntimeMode,
    pub http_port: u16,
    pub http_online: bool,
    pub binary_path: Option<String>,
    pub models_registered: usize,
}

/// UDS control microservice (start/stop/health for the Allama stack).
pub struct AllamaService {
    config: AllamaServiceConfig,
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<AllamaServiceStatus>>,
}

impl AllamaService {
    /// Create a new Allama control service.
    pub fn new(config: AllamaServiceConfig) -> Self {
        Self {
            config,
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(AllamaServiceStatus {
                running: false,
                mode: AllamaRuntimeMode::Stopped,
                http_port: AllamaServiceConfig::default().http_port,
                http_online: false,
                binary_path: None,
                models_registered: 0,
            })),
        }
    }

    /// Update shared status (called by AllamaManager).
    pub fn set_status(&self, status: AllamaServiceStatus) {
        if let Ok(mut s) = self.status.lock() {
            *s = status;
        }
    }

    /// Read current status.
    pub fn get_status(&self) -> AllamaServiceStatus {
        self.status.lock().map(|s| s.clone()).unwrap_or_else(|_| AllamaServiceStatus {
            running: false,
            mode: AllamaRuntimeMode::Stopped,
            http_port: self.config.http_port,
            http_online: false,
            binary_path: None,
            models_registered: 0,
        })
    }

    /// Start JSON-RPC listener on the Unix socket.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.lock().map_err(|e| format!("Lock error: {e}"))?;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        let listener = UnixListener::bind(&self.config.socket_path)?;
        println!(
            "[Allama] control service listening on {:?}",
            self.config.socket_path
        );

        let running_flag = Arc::clone(&self.running);
        let status_arc = Arc::clone(&self.status);

        tokio::spawn(async move {
            while *running_flag.lock().unwrap_or_else(|_| panic!("Lock error")) {
                match listener.accept().await {
                    Ok((mut socket, _)) => {
                        let status_arc = Arc::clone(&status_arc);
                        tokio::spawn(async move {
                            let mut buf = [0u8; 16384];
                            loop {
                                match socket.read(&mut buf).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        let request =
                                            String::from_utf8_lossy(&buf[..n]).to_string();
                                        if let Ok(response) =
                                            handle_request(&status_arc, &request)
                                        {
                                            let _ = socket.write_all(response.as_bytes()).await;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("[Allama] UDS accept error: {e}");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the control service.
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.lock().map_err(|e| format!("Lock error: {e}"))?;
        *running = false;
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }
        Ok(())
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    #[allow(dead_code)]
    params: Option<serde_json::Value>,
    id: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    id: serde_json::Value,
}

fn handle_request(status: &Arc<Mutex<AllamaServiceStatus>>, raw: &str) -> Result<String, String> {
    let req: JsonRpcRequest =
        serde_json::from_str(raw).map_err(|e| format!("Invalid JSON-RPC request: {e}"))?;
    let snapshot = status.lock().map(|s| s.clone()).unwrap_or_else(|_| {
        AllamaServiceStatus {
            running: false,
            mode: AllamaRuntimeMode::Stopped,
            http_port: 11435,
            http_online: false,
            binary_path: None,
            models_registered: 0,
        }
    });

    let result = match req.method.as_str() {
        "health" | "ping" => serde_json::json!({ "ok": true, "service": "allama" }),
        "get_status" => serde_json::to_value(&snapshot)
            .map_err(|e| format!("Serialize status failed: {e}"))?,
        "get_port" => serde_json::json!({ "port": snapshot.http_port }),
        _ => {
            return Ok(serde_json::to_string(&JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(serde_json::json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", req.method)
                })),
                id: req.id,
            })
            .map_err(|e| format!("Serialize error response failed: {e}"))?);
        }
    };

    serde_json::to_string(&JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: req.id,
    })
    .map_err(|e| format!("Serialize response failed: {e}"))
}

/// Send a JSON-RPC request to the Allama control socket.
pub async fn send_allama_rpc(
    socket_path: &PathBuf,
    method: &str,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": null,
        "id": 1
    });

    let mut socket = tokio::net::UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Allama UDS connect failed: {e}"))?;

    socket
        .write_all(request.to_string().as_bytes())
        .await
        .map_err(|e| format!("Allama UDS write failed: {e}"))?;

    let mut buf = vec![0u8; 16384];
    let n = socket
        .read(&mut buf)
        .await
        .map_err(|e| format!("Allama UDS read failed: {e}"))?;

    let response: serde_json::Value = serde_json::from_slice(&buf[..n])
        .map_err(|e| format!("Allama UDS parse failed: {e}"))?;

    if let Some(err) = response.get("error") {
        return Err(format!("Allama RPC error: {err}"));
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| "No result in Allama RPC response".to_string())
}
