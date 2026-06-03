//! Exodus WAN relay HTTP server — proxies mesh blob requests across NAT.
//!
//! Endpoint: `GET /exodus-mesh/fetch?host=H&port=P&path=/blobs/{hash}/...`
//! Forwards to `http://{host}:{port}{path}` on the peer mesh HTTP server.

use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// Running relay server handle.
pub struct WanRelayServerHandle {
    pub port: u16,
    pub bind_host: String,
    pub base_url: String,
    shutdown_tx: watch::Sender<bool>,
}

/// Query parameters for mesh fetch proxy.
#[derive(Debug, Deserialize)]
pub struct MeshFetchQuery {
    pub host: String,
    pub port: u16,
    pub path: String,
}

/// Relay server status for UI / diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WanRelayServerInfo {
    pub running: bool,
    pub port: u16,
    pub base_url: String,
    pub bind_host: String,
}

/// Start relay on `bind_host:port` (idempotent per process).
pub async fn start_relay_server(bind_host: &str, port: u16) -> Result<WanRelayServerHandle, String> {
    let addr: SocketAddr = format!("{bind_host}:{port}")
        .parse()
        .map_err(|e| format!("Invalid relay bind address: {e}"))?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("WAN relay bind failed on {addr}: {e}"))?;
    let bound = listener.local_addr().map_err(|e| e.to_string())?;
    let port = bound.port();
    let bind_host = bound.ip().to_string();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/exodus-mesh/fetch", get(mesh_fetch_handler));

    let shutdown_rx_serve = shutdown_rx.clone();
    tauri::async_runtime::spawn(async move {
        let serve = axum::serve(listener, router).with_graceful_shutdown(async move {
            let mut rx = shutdown_rx_serve;
            loop {
                if *rx.borrow() {
                    break;
                }
                if rx.changed().await.is_err() {
                    break;
                }
            }
        });
        if let Err(e) = serve.await {
            tracing::warn!("WAN relay server stopped: {e}");
        }
    });

    let base_url = format!("http://{bind_host}:{port}");
    tracing::info!("Exodus WAN relay listening on {base_url}");
    Ok(WanRelayServerHandle {
        port,
        bind_host: bind_host.clone(),
        base_url,
        shutdown_tx,
    })
}

/// Managed relay server (optional singleton).
#[derive(Clone)]
pub struct WanRelayServerState {
    handle: Arc<tokio::sync::Mutex<Option<WanRelayServerHandle>>>,
}

impl WanRelayServerState {
    pub fn new() -> Self {
        Self {
            handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Start relay if not already running.
    pub async fn ensure_started(&self, bind_host: &str, port: u16) -> Result<WanRelayServerInfo, String> {
        let mut guard = self.handle.lock().await;
        if let Some(h) = guard.as_ref() {
            return Ok(WanRelayServerInfo {
                running: true,
                port: h.port,
                base_url: h.base_url.clone(),
                bind_host: h.bind_host.clone(),
            });
        }
        let h = start_relay_server(bind_host, port).await?;
        let info = WanRelayServerInfo {
            running: true,
            port: h.port,
            base_url: h.base_url.clone(),
            bind_host: h.bind_host.clone(),
        };
        *guard = Some(h);
        Ok(info)
    }

    pub async fn info(&self) -> WanRelayServerInfo {
        let guard = self.handle.lock().await;
        if let Some(h) = guard.as_ref() {
            WanRelayServerInfo {
                running: true,
                port: h.port,
                base_url: h.base_url.clone(),
                bind_host: h.bind_host.clone(),
            }
        } else {
            WanRelayServerInfo {
                running: false,
                port: 0,
                base_url: String::new(),
                bind_host: String::new(),
            }
        }
    }
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn mesh_fetch_handler(Query(q): Query<MeshFetchQuery>) -> Response {
    match proxy_mesh_fetch(&q).await {
        Ok(resp) => resp,
        Err((status, msg)) => (status, msg).into_response(),
    }
}

async fn proxy_mesh_fetch(q: &MeshFetchQuery) -> Result<Response, (StatusCode, String)> {
    validate_mesh_fetch_query(q)?;
    let path = normalize_mesh_path(&q.path);
    let upstream = format!("http://{}:{}{}", q.host.trim(), q.port, path);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3600))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let resp = client
        .get(&upstream)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Upstream fetch failed: {e}")))?;

    let status =
        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let headers = resp.headers().clone();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Upstream body read failed: {e}")))?;

    let mut out = Response::builder().status(status);
    if let Some(ct) = headers.get(header::CONTENT_TYPE) {
        if let Ok(v) = axum::http::HeaderValue::from_bytes(ct.as_bytes()) {
            out = out.header(header::CONTENT_TYPE, v);
        }
    }
    if let Some(cl) = headers.get(header::CONTENT_LENGTH) {
        if let Ok(v) = axum::http::HeaderValue::from_bytes(cl.as_bytes()) {
            out = out.header(header::CONTENT_LENGTH, v);
        }
    }
    out.body(axum::body::Body::from(bytes))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

fn validate_mesh_fetch_query(q: &MeshFetchQuery) -> Result<(), (StatusCode, String)> {
    let host = q.host.trim();
    if host.is_empty() || host.len() > 253 {
        return Err((StatusCode::BAD_REQUEST, "Invalid host".into()));
    }
    if q.port == 0 {
        return Err((StatusCode::BAD_REQUEST, "Invalid port".into()));
    }
    let path = q.path.trim();
    if !path.starts_with("/blobs/") {
        return Err((
            StatusCode::BAD_REQUEST,
            "path must start with /blobs/".into(),
        ));
    }
    if path.contains("..") {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".into()));
    }
    Ok(())
}

fn normalize_mesh_path(path: &str) -> String {
    let p = path.trim();
    if p.starts_with('/') {
        p.to_string()
    } else {
        format!("/{p}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_mesh_path() {
        let ok = MeshFetchQuery {
            host: "10.0.0.5".into(),
            port: 7878,
            path: "/blobs/abc/meta".into(),
        };
        assert!(validate_mesh_fetch_query(&ok).is_ok());

        let bad = MeshFetchQuery {
            host: "10.0.0.5".into(),
            port: 7878,
            path: "/etc/passwd".into(),
        };
        assert!(validate_mesh_fetch_query(&bad).is_err());
    }

    #[test]
    fn normalizes_path() {
        assert_eq!(normalize_mesh_path("blobs/x"), "/blobs/x");
        assert_eq!(normalize_mesh_path("/blobs/x"), "/blobs/x");
    }

    #[test]
    fn validate_empty_host() {
        let query = MeshFetchQuery {
            host: "".into(),
            port: 7878,
            path: "/blobs/abc".into(),
        };
        assert!(validate_mesh_fetch_query(&query).is_err());
    }

    #[test]
    fn validate_oversized_host() {
        let query = MeshFetchQuery {
            host: "a".repeat(254).into(),
            port: 7878,
            path: "/blobs/abc".into(),
        };
        assert!(validate_mesh_fetch_query(&query).is_err());
    }

    #[test]
    fn validate_zero_port() {
        let query = MeshFetchQuery {
            host: "10.0.0.5".into(),
            port: 0,
            path: "/blobs/abc".into(),
        };
        assert!(validate_mesh_fetch_query(&query).is_err());
    }

    #[test]
    fn validate_path_traversal_attack() {
        let query = MeshFetchQuery {
            host: "10.0.0.5".into(),
            port: 7878,
            path: "/blobs/../../etc/passwd".into(),
        };
        assert!(validate_mesh_fetch_query(&query).is_err());
    }

    #[test]
    fn test_wan_relay_server_state_creation() {
        let state = WanRelayServerState::new();
        // If we got here, creation succeeded
        assert!(true);
    }

    #[test]
    fn test_wan_relay_server_info_when_not_running() {
        let state = WanRelayServerState::new();
        let info = tokio::runtime::Runtime::new()
            .expect("Failed to create runtime")
            .block_on(state.info());
        
        assert!(!info.running);
        assert_eq!(info.port, 0);
        assert!(info.base_url.is_empty());
        assert!(info.bind_host.is_empty());
    }

    #[test]
    fn test_wan_relay_server_info_serialization() {
        let info = WanRelayServerInfo {
            running: true,
            port: 8790,
            base_url: "http://127.0.0.1:8790".to_string(),
            bind_host: "127.0.0.1".to_string(),
        };
        
        let json = serde_json::to_string(&info).expect("Failed to serialize info");
        let deserialized: WanRelayServerInfo = serde_json::from_str(&json).expect("Failed to deserialize info");
        
        assert_eq!(deserialized.running, true);
        assert_eq!(deserialized.port, 8790);
        assert_eq!(deserialized.base_url, "http://127.0.0.1:8790");
    }
}
