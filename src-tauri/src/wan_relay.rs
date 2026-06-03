//! WAN relay configuration — fetch mesh blobs via an HTTP relay when LAN peers fail.
//!
//! Relay URL pattern: `{base}/exodus-mesh/fetch?host=H&port=P&path=/blobs/{hash}`

use serde::{Deserialize, Serialize};
use std::path::Path;

fn default_serve_enabled() -> bool {
    true
}

fn default_serve_port() -> u16 {
    8790
}

fn default_serve_bind() -> String {
    "127.0.0.1".to_string()
}

/// Persisted WAN relay settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WanRelayConfig {
    /// Use relay URLs when downloading (client).
    pub enabled: bool,
    #[serde(default)]
    pub relay_base_url: Option<String>,
    /// Run embedded Exodus WAN relay HTTP server.
    #[serde(default = "default_serve_enabled")]
    pub serve_enabled: bool,
    #[serde(default = "default_serve_port")]
    pub serve_port: u16,
    #[serde(default = "default_serve_bind")]
    pub serve_bind: String,
}

impl Default for WanRelayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            relay_base_url: None,
            serve_enabled: default_serve_enabled(),
            serve_port: default_serve_port(),
            serve_bind: default_serve_bind(),
        }
    }
}

impl WanRelayConfig {
    /// Load from `{app_data}/wan_relay.json` or defaults.
    pub fn load(app_data_dir: &Path) -> Self {
        let path = app_data_dir.join("wan_relay.json");
        if !path.exists() {
            return Self::default();
        }
        fs_read_json(&path).unwrap_or_default()
    }

    /// Default local relay base URL after server start.
    pub fn local_base_url(&self) -> String {
        format!("http://{}:{}", self.serve_bind.trim(), self.serve_port)
    }

    /// Ensure client `relay_base_url` points at local server when serving.
    pub fn apply_local_relay_url(&mut self) {
        if self.serve_enabled {
            let base = self.local_base_url();
            if self.relay_base_url.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                self.relay_base_url = Some(base);
            }
            self.enabled = true;
        }
    }

    /// Save relay settings.
    pub fn save(&self, app_data_dir: &Path) -> Result<(), String> {
        let path = app_data_dir.join("wan_relay.json");
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }

    /// Build relay proxy URL for a mesh path (e.g. `/blobs/{hash}/meta`).
    pub fn proxy_url(&self, host: &str, port: u16, mesh_path: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let base = self.relay_base_url.as_ref()?.trim().trim_end_matches('/');
        if base.is_empty() {
            return None;
        }
        let path = if mesh_path.starts_with('/') {
            mesh_path.to_string()
        } else {
            format!("/{mesh_path}")
        };
        Some(format!(
            "{base}/exodus-mesh/fetch?host={host}&port={port}&path={}",
            urlencoding_encode_path(&path)
        ))
    }
}

fn urlencoding_encode_path(path: &str) -> String {
    url::form_urlencoded::byte_serialize(path.as_bytes()).collect()
}

fn fs_read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_proxy_url() {
        let cfg = WanRelayConfig {
            enabled: true,
            relay_base_url: Some("https://relay.example.com".into()),
            ..Default::default()
        };
        let u = cfg.proxy_url("1.2.3.4", 9000, "/blobs/abc/meta").expect("url");
        assert!(u.contains("relay.example.com"));
        assert!(u.contains("host=1.2.3.4"));
        assert!(u.contains("port=9000"));
    }
}
