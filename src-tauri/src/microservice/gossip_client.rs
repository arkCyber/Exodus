//! Exodus Browser — JSON-RPC client for the P2P gossip microservice (Unix socket).

use std::path::Path;

use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Send a JSON-RPC 2.0 request to the gossip service socket.
pub async fn gossip_json_rpc(
    socket_path: &Path,
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
        .map_err(|e| format!("Failed to connect to P2P gossip service: {e}"))?;

    let (mut reader, mut writer) = client.into_split();

    let request_str =
        serde_json::to_string(&request).map_err(|e| format!("Failed to serialize request: {e}"))?;

    writer
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;

    let mut buf = [0u8; 8192];
    let n = reader
        .read(&mut buf)
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    let response_str = String::from_utf8_lossy(&buf[..n]).to_string();
    let response: serde_json::Value =
        serde_json::from_str(&response_str).map_err(|e| format!("Failed to parse response: {e}"))?;

    if let Some(error) = response.get("error") {
        return Err(format!("P2P gossip service error: {error}"));
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| "No result in response".to_string())
}
