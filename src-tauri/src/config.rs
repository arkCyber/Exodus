//! Exodus Browser — runtime configuration (AI endpoint, model).

use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Application settings persisted for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExodusConfig {
    /// Local inference API port (Allama / Ollama-compatible). Default 11435.
    pub ai_port: u16,
    /// Model id sent to the chat completions API.
    pub ai_model: String,
    /// URL opened by the Home toolbar button.
    #[serde(default = "ExodusConfig::default_homepage_url")]
    pub homepage_url: String,
    /// Search URL template; `{query}` is replaced with the encoded search terms.
    #[serde(default = "ExodusConfig::default_search_engine_url")]
    pub search_engine_url: String,
    /// Model id for `/v1/embeddings` (Ollama: nomic-embed-text, etc.).
    #[serde(default = "ExodusConfig::default_embedding_model")]
    pub embedding_model: String,
    /// Status bar auto-clear delay in milliseconds.
    #[serde(default = "ExodusConfig::default_status_clear_ms")]
    pub status_clear_ms: u32,
    /// Spawn bundled exodus-core sidecar on app start (legacy; prefer Allama).
    #[serde(default = "ExodusConfig::default_spawn_sidecar")]
    pub spawn_sidecar: bool,
    /// Spawn Allama inference microservice on app start (Ollama replacement, port 11435).
    #[serde(default = "ExodusConfig::default_spawn_allama")]
    pub spawn_allama: bool,
    /// Force HTTPS for all navigation attempts.
    #[serde(default = "ExodusConfig::default_https_only")]
    pub https_only: bool,
    /// Enable private/incognito browsing mode (no history, no cookies).
    #[serde(default = "ExodusConfig::default_private_mode")]
    pub private_mode: bool,
    /// Block popup windows.
    #[serde(default = "ExodusConfig::default_block_popups")]
    pub block_popups: bool,
    /// Enable session restore (save/restore tabs on close/startup).
    #[serde(default = "ExodusConfig::default_session_restore")]
    pub session_restore: bool,
    /// Optional HTTPS URL returning JSON array of `StoreExtensionEntry` for remote extension catalog.
    #[serde(default)]
    pub extension_store_url: String,
    /// When true, `.crx` installs must pass CRX3 RSA signature verification (plain ZIP still allowed).
    #[serde(default = "ExodusConfig::default_require_crx_signature")]
    pub require_crx_signature: bool,
    /// When true, prompt the user before granting manifest `host_permissions` on install.
    #[serde(default = "ExodusConfig::default_confirm_host_permissions_on_install")]
    pub confirm_host_permissions_on_install: bool,
}

impl Default for ExodusConfig {
    fn default() -> Self {
        Self {
            ai_port: Self::default_ai_port(),
            ai_model: "exodus-default".to_string(),
            homepage_url: Self::default_homepage_url(),
            search_engine_url: Self::default_search_engine_url(),
            embedding_model: Self::default_embedding_model(),
            status_clear_ms: Self::default_status_clear_ms(),
            spawn_sidecar: Self::default_spawn_sidecar(),
            spawn_allama: Self::default_spawn_allama(),
            https_only: Self::default_https_only(),
            private_mode: Self::default_private_mode(),
            block_popups: Self::default_block_popups(),
            session_restore: Self::default_session_restore(),
            extension_store_url: String::new(),
            require_crx_signature: Self::default_require_crx_signature(),
            confirm_host_permissions_on_install: Self::default_confirm_host_permissions_on_install(),
        }
    }
}

/// Thread-safe config handle managed by Tauri.
pub type ConfigState = Mutex<ExodusConfig>;

impl ExodusConfig {
    /// Default homepage (new tab data URL is handled on the frontend).
    pub fn default_homepage_url() -> String {
        "https://duckduckgo.com".to_string()
    }

    /// Default DuckDuckGo search template for the omnibox.
    pub fn default_search_engine_url() -> String {
        "https://duckduckgo.com/?q={query}".to_string()
    }

    /// Default embedding model for Ollama / exodus-core.
    pub fn default_embedding_model() -> String {
        "nomic-embed-text".to_string()
    }

    /// Default status bar message duration (ms).
    pub fn default_status_clear_ms() -> u32 {
        4000
    }

    /// Default Allama port (Ollama replacement).
    pub fn default_ai_port() -> u16 {
        11435
    }

    /// Allama microservice is the default local LLM backend.
    pub fn default_spawn_allama() -> bool {
        true
    }

    /// Legacy exodus-core sidecar is off when Allama is primary.
    pub fn default_spawn_sidecar() -> bool {
        false
    }

    /// Session restore is enabled by default.
    pub fn default_session_restore() -> bool {
        true
    }

    /// Config file name inside the app data directory.
    pub const FILE_NAME: &'static str = "exodus_config.json";

    /// Load config from disk or return defaults if missing.
    pub fn load_from(data_dir: &Path) -> Self {
        let path = data_dir.join(Self::FILE_NAME);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    /// Persist config to the app data directory.
    pub fn save_to(&self, data_dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(data_dir).map_err(|e| format!("Create config dir failed: {}", e))?;
        let path = data_dir.join(Self::FILE_NAME);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize config failed: {}", e))?;
        std::fs::write(&path, json).map_err(|e| format!("Write config failed: {}", e))?;
        Ok(())
    }

    /// Base URL for OpenAI-compatible chat completions.
    pub fn chat_completions_url(&self) -> String {
        format!(
            "http://127.0.0.1:{}/v1/chat/completions",
            self.ai_port
        )
    }

    /// Base URL for OpenAI-compatible embeddings.
    #[allow(dead_code)]
    pub fn embeddings_url(&self) -> String {
        format!("http://127.0.0.1:{}/v1/embeddings", self.ai_port)
    }

    /// CLI port argument for exodus-core sidecar.
    #[allow(dead_code)]
    pub fn sidecar_port_arg(&self) -> String {
        self.ai_port.to_string()
    }

    /// Default HTTPS-only mode (enabled for new installs — Brave/Chrome-style safety).
    pub fn default_https_only() -> bool {
        true
    }

    /// Default private mode (disabled by default).
    pub fn default_private_mode() -> bool {
        false
    }

    /// Default block popups (enabled for new installs).
    pub fn default_block_popups() -> bool {
        true
    }

    /// Default require CRX signature (disabled by default).
    pub fn default_require_crx_signature() -> bool {
        false
    }

    /// Default confirm host permissions on install (disabled by default).
    pub fn default_confirm_host_permissions_on_install() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_homepage_and_search() {
        let cfg = ExodusConfig::default();
        assert_eq!(cfg.homepage_url, ExodusConfig::default_homepage_url());
        assert!(cfg.search_engine_url.contains("{query}"));
        assert_eq!(cfg.embedding_model, ExodusConfig::default_embedding_model());
    }

    #[test]
    fn load_from_missing_file_uses_defaults() {
        let dir = std::env::temp_dir().join(format!("exodus_cfg_{}", std::process::id()));
        let cfg = ExodusConfig::load_from(&dir);
        assert_eq!(cfg.ai_port, 11435);
    }

    #[test]
    fn default_config_has_status_clear_and_sidecar_spawn() {
        let cfg = ExodusConfig::default();
        assert_eq!(cfg.status_clear_ms, ExodusConfig::default_status_clear_ms());
        assert!(!cfg.spawn_sidecar);
        assert!(cfg.spawn_allama);
    }

    #[test]
    fn default_privacy_options() {
        let cfg = ExodusConfig::default();
        assert_eq!(cfg.https_only, ExodusConfig::default_https_only());
        assert_eq!(cfg.private_mode, ExodusConfig::default_private_mode());
        assert_eq!(cfg.block_popups, ExodusConfig::default_block_popups());
        assert!(cfg.session_restore);
    }

    #[test]
    fn save_and_load_privacy_flags_roundtrip() {
        let dir = std::env::temp_dir().join(format!("exodus_cfg_priv_{}", std::process::id()));
        let cfg = ExodusConfig {
            https_only: true,
            private_mode: true,
            block_popups: false,
            session_restore: false,
            ..Default::default()
        };
        cfg.save_to(&dir).expect("save");
        let loaded = ExodusConfig::load_from(&dir);
        assert!(loaded.https_only);
        assert!(loaded.private_mode);
        assert!(!loaded.block_popups);
        assert!(!loaded.session_restore);
    }

    #[test]
    fn test_chat_completions_url() {
        let cfg = ExodusConfig {
            ai_port: 11435,
            ..Default::default()
        };
        let url = cfg.chat_completions_url();
        assert_eq!(url, "http://127.0.0.1:11435/v1/chat/completions");
    }

    #[test]
    fn test_embeddings_url() {
        let cfg = ExodusConfig {
            ai_port: 11435,
            ..Default::default()
        };
        let url = cfg.embeddings_url();
        assert_eq!(url, "http://127.0.0.1:11435/v1/embeddings");
    }

    #[test]
    fn test_sidecar_port_arg() {
        let cfg = ExodusConfig {
            ai_port: 11435,
            ..Default::default()
        };
        let arg = cfg.sidecar_port_arg();
        assert_eq!(arg, "11435");
    }

    #[test]
    fn test_save_and_load_full_config() {
        let dir = std::env::temp_dir().join(format!("exodus_cfg_full_{}", std::process::id()));
        let cfg = ExodusConfig {
            ai_port: 12345,
            ai_model: "test-model".to_string(),
            homepage_url: "https://example.com".to_string(),
            search_engine_url: "https://example.com/search?q={query}".to_string(),
            embedding_model: "test-embed".to_string(),
            status_clear_ms: 5000,
            spawn_sidecar: true,
            spawn_allama: false,
            https_only: false,
            private_mode: true,
            block_popups: true,
            session_restore: false,
            extension_store_url: "https://extensions.example.com".to_string(),
            require_crx_signature: true,
            confirm_host_permissions_on_install: true,
        };
        cfg.save_to(&dir).expect("save");
        let loaded = ExodusConfig::load_from(&dir);
        assert_eq!(loaded.ai_port, 12345);
        assert_eq!(loaded.ai_model, "test-model");
        assert_eq!(loaded.homepage_url, "https://example.com");
        assert_eq!(loaded.embedding_model, "test-embed");
        assert_eq!(loaded.status_clear_ms, 5000);
        assert!(loaded.spawn_sidecar);
        assert!(!loaded.spawn_allama);
        assert!(!loaded.https_only);
        assert!(loaded.private_mode);
        assert!(loaded.block_popups);
        assert!(!loaded.session_restore);
        assert_eq!(loaded.extension_store_url, "https://extensions.example.com");
        assert!(loaded.require_crx_signature);
        assert!(loaded.confirm_host_permissions_on_install);
    }

    #[test]
    fn test_default_values() {
        assert_eq!(ExodusConfig::default_ai_port(), 11435);
        assert_eq!(ExodusConfig::default_homepage_url(), "https://duckduckgo.com");
        assert_eq!(ExodusConfig::default_search_engine_url(), "https://duckduckgo.com/?q={query}");
        assert_eq!(ExodusConfig::default_embedding_model(), "nomic-embed-text");
        assert_eq!(ExodusConfig::default_status_clear_ms(), 4000);
        assert!(ExodusConfig::default_spawn_allama());
        assert!(!ExodusConfig::default_spawn_sidecar());
        assert!(ExodusConfig::default_session_restore());
        assert!(ExodusConfig::default_https_only());
        assert!(!ExodusConfig::default_private_mode());
        assert!(ExodusConfig::default_block_popups());
        assert!(!ExodusConfig::default_require_crx_signature());
        assert!(!ExodusConfig::default_confirm_host_permissions_on_install());
    }

    #[test]
    fn test_file_name_constant() {
        assert_eq!(ExodusConfig::FILE_NAME, "exodus_config.json");
    }

    #[test]
    fn test_config_serialization() {
        let cfg = ExodusConfig::default();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let deserialized: ExodusConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cfg.ai_port, deserialized.ai_port);
        assert_eq!(cfg.ai_model, deserialized.ai_model);
        assert_eq!(cfg.homepage_url, deserialized.homepage_url);
    }
}
