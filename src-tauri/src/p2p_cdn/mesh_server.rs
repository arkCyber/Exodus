//! Exodus P2P CDN — HTTP mesh server (serves BLAKE3-addressed blobs to peers).

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::watch;

use super::store::CdnBlobStore;

/// Running mesh HTTP server handle.
pub struct MeshServerHandle {
    pub port: u16,
    pub host: String,
    #[allow(dead_code)]
    shutdown_tx: watch::Sender<bool>,
}

impl MeshServerHandle {
    /// Bind `0.0.0.0:0` and serve blob HTTP API.
    pub async fn start(store: Arc<CdnBlobStore>) -> Result<Self, String> {
        let listener = TcpListener::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Mesh bind failed: {e}"))?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let port = addr.port();
        let host = local_lan_ip().unwrap_or_else(|| "127.0.0.1".to_string());

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            loop {
                let mut shutdown_rx = shutdown_rx.clone();
                tokio::select! {
                    accept = listener.accept() => {
                        match accept {
                            Ok((mut stream, _)) => {
                                let store = Arc::clone(&store);
                                tokio::spawn(async move {
                                    let _ = handle_connection(&mut stream, &store).await;
                                });
                            }
                            Err(e) => {
                                tracing::warn!("mesh accept error: {e}");
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });

        tracing::info!("Exodus CDN mesh listening on {host}:{port}");
        Ok(Self {
            port,
            host,
            shutdown_tx,
        })
    }

    #[allow(dead_code)]
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
    store: &CdnBlobStore,
) -> Result<(), String> {
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let path = parts[1];

    if path == "/health" {
        return write_response(stream, 200, "text/plain", b"ok").await;
    }

    if let Some(rest) = path.strip_prefix("/blobs/") {
        let rest = rest.split('?').next().unwrap_or(rest);
        if let Some((hash, tail)) = rest.split_once("/chunks/") {
            if let Ok(index) = tail.parse::<u32>() {
                return serve_chunk(stream, store, hash, index).await;
            }
        }
        if rest.ends_with("/meta") {
            let hash = rest.trim_end_matches("/meta");
            return serve_meta(stream, store, hash).await;
        }
        return serve_full_blob(stream, store, rest).await;
    }

    write_response(stream, 404, "text/plain", b"not found").await
}

async fn serve_meta(
    stream: &mut tokio::net::TcpStream,
    store: &CdnBlobStore,
    hash: &str,
) -> Result<(), String> {
    let (size, chunk_count) = match store.meta(hash) {
        Ok(m) => m,
        Err(_) => return write_response(stream, 404, "text/plain", b"blob not found").await,
    };
    let body = serde_json::json!({
        "hash": hash,
        "sizeBytes": size,
        "chunkCount": chunk_count,
    });
    let bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;
    write_response(stream, 200, "application/json", &bytes).await
}

async fn serve_chunk(
    stream: &mut tokio::net::TcpStream,
    store: &CdnBlobStore,
    hash: &str,
    index: u32,
) -> Result<(), String> {
    match store.read_chunk(hash, index) {
        Ok(bytes) => write_response(stream, 200, "application/octet-stream", &bytes).await,
        Err(_) => write_response(stream, 404, "text/plain", b"chunk not found").await,
    }
}

async fn serve_full_blob(
    stream: &mut tokio::net::TcpStream,
    store: &CdnBlobStore,
    hash: &str,
) -> Result<(), String> {
    if !store.has_complete(hash) {
        return write_response(stream, 404, "text/plain", b"blob not found").await;
    }
    let tmp = std::env::temp_dir().join(format!("exodus_mesh_{hash}.bin"));
    store.export_to_file(hash, &tmp)?;
    let bytes = std::fs::read(&tmp).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&tmp);
    write_response(stream, 200, "application/octet-stream", &bytes).await
}

async fn write_response(
    stream: &mut tokio::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stream
        .write_all(body)
        .await
        .map_err(|e| e.to_string())?;
    let _ = stream.shutdown().await;
    Ok(())
}

/// Best-effort LAN address for peer tickets (fallback 127.0.0.1).
pub fn local_lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?;
    Some(local.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Arc;

    #[tokio::test]
    async fn mesh_serves_health_and_blob() {
        let dir = std::env::temp_dir().join(format!("mesh_srv_{}", uuid::Uuid::new_v4()));
        let store = Arc::new(CdnBlobStore::new(&dir).expect("Failed to create store"));
        let src = dir.join("data.bin");
        {
            let mut f = std::fs::File::create(&src).expect("Failed to create file");
            f.write_all(b"mesh-integration-payload").expect("Failed to write data");
        }
        let (hash, _) = store.import_file(&src).expect("Failed to import file");
        let mesh = MeshServerHandle::start(Arc::clone(&store))
            .await
            .expect("mesh start");

        let client = reqwest::Client::new();
        let health = format!("http://127.0.0.1:{}/health", mesh.port);
        let body = client
            .get(&health)
            .send()
            .await
            .expect("health")
            .text()
            .await
            .expect("text");
        assert_eq!(body, "ok");

        let url = format!("http://127.0.0.1:{}/blobs/{hash}", mesh.port);
        let bytes = client
            .get(&url)
            .send()
            .await
            .expect("blob")
            .bytes()
            .await
            .expect("bytes");
        assert_eq!(bytes.as_ref(), b"mesh-integration-payload");

        mesh.shutdown();
    }
}
