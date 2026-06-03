//! Exodus Browser — Allama lifecycle (native binary or embedded gateway on port 11435).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::inference_engine::InferenceEngine;
use crate::microservice::allama_gateway::{scan_and_register_models, start_embedded_gateway, AllamaGatewayHandle};
use crate::microservice::allama_process::{
    discover_allama_binary, discover_bundle_models_dir, probe_allama_http, resolve_models_dir,
    spawn_allama_serve, ALLAMA_DEFAULT_PORT,
};
use crate::microservice::allama_service::{AllamaRuntimeMode, AllamaService, AllamaServiceConfig, AllamaServiceStatus};

/// Managed Allama stack for settings UI and auto-start.
pub struct AllamaManager {
    port: Mutex<u16>,
    models_dir: PathBuf,
    engine: Arc<InferenceEngine>,
    control: Arc<AllamaService>,
    native_child: Mutex<Option<tokio::process::Child>>,
    gateway: Mutex<Option<AllamaGatewayHandle>>,
    default_model: Mutex<String>,
}

/// Status DTO for the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllamaStatusDto {
    pub state: String,
    pub port: u16,
    pub detail: String,
    pub endpoint_online: bool,
    pub mode: String,
    pub models_registered: usize,
    pub binary_path: Option<String>,
}

impl AllamaManager {
    /// Create manager; call `start` when `spawn_enabled` is true.
    pub fn new(
        engine: Arc<InferenceEngine>,
        port: u16,
        models_dir: PathBuf,
    ) -> Self {
        let control = Arc::new(AllamaService::new(AllamaServiceConfig {
            socket_path: AllamaServiceConfig::default().socket_path,
            http_port: port,
        }));

        Self {
            port: Mutex::new(port),
            models_dir,
            engine,
            control,
            native_child: Mutex::new(None),
            gateway: Mutex::new(None),
            default_model: Mutex::new("exodus-default".to_string()),
        }
    }

    /// Update HTTP port (applied on next start/restart).
    pub fn set_port(&self, port: u16) {
        if let Ok(mut p) = self.port.lock() {
            *p = port;
        }
    }

    fn http_port(&self) -> u16 {
        self.port
            .lock()
            .map(|p| *p)
            .unwrap_or(ALLAMA_DEFAULT_PORT)
    }

    /// Auto-start Allama when enabled in settings.
    pub fn spawn_if_enabled(self: &Arc<Self>, app: &AppHandle, enabled: bool) {
        if !enabled {
            println!("[Allama] auto-start disabled in settings");
            return;
        }
        let mgr = Arc::clone(self);
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = mgr.start().await {
                eprintln!("[Allama] auto-start failed: {e}");
            } else {
                let _ = app.emit("allama-started", mgr.status_dto().await);
            }
        });
    }

    /// Start Allama: native binary if present, else embedded HTTP gateway.
    pub async fn start(self: &Arc<Self>) -> Result<(), String> {
        let mut registered =
            scan_and_register_models(self.engine.as_ref(), &self.models_dir).await;
        if registered == 0 {
            if let Some(bundle) = discover_bundle_models_dir() {
                registered +=
                    scan_and_register_models(self.engine.as_ref(), &bundle).await;
            }
        }
        if registered == 0 {
            let _ = self
                .engine
                .add_model(crate::inference_engine::ModelInfo {
                    name: "exodus-default".to_string(),
                    path: std::path::PathBuf::from("builtin"),
                    size_bytes: 0,
                    quantization: "stub".to_string(),
                    parameters: "n/a".to_string(),
                    context_length: 2048,
                    loaded: false,
                    backend: crate::inference_engine::BackendType::Allama,
                })
                .await;
            registered = 1;
        }
        if registered > 0 {
            if let Some(first) = self.engine.list_models().await.into_iter().next() {
                if let Ok(mut name) = self.default_model.lock() {
                    *name = first.name;
                }
            }
        }

        let _ = self.control.start().await.map_err(|e| e.to_string())?;

        if let Some(binary) = discover_allama_binary() {
            println!("[Allama] spawning native binary: {}", binary.display());
            let port = self.http_port();
            let child = spawn_allama_serve(&binary, port, &self.models_dir).await?;
            if let Ok(mut slot) = self.native_child.lock() {
                *slot = Some(child);
            }
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            if probe_allama_http(port).await {
                self.engine.set_embedded_gateway_active(false);
                self.engine.set_allama_http_port(Some(port)).await;
                self.publish_status(AllamaRuntimeMode::NativeBinary, Some(binary.display().to_string()), registered)
                    .await;
                return Ok(());
            }
            println!("[Allama] native process did not respond; falling back to embedded gateway");
            if let Ok(mut slot) = self.native_child.lock() {
                slot.take();
            }
        }

        let default_model = self
            .default_model
            .lock()
            .map(|n| n.clone())
            .unwrap_or_else(|_| "exodus-default".to_string());

        let port = self.http_port();
        let handle = start_embedded_gateway(
            Arc::clone(&self.engine),
            port,
            default_model,
        )
        .await?;

        if let Ok(mut gw) = self.gateway.lock() {
            *gw = Some(handle);
        }

        self.engine.set_embedded_gateway_active(true);
        self.engine.set_allama_http_port(Some(port)).await;

        self.publish_status(AllamaRuntimeMode::EmbeddedGateway, None, registered)
            .await;
        Ok(())
    }

    /// Stop Allama HTTP and control plane.
    pub async fn stop(&self) -> Result<(), String> {
        if let Ok(mut gw) = self.gateway.lock() {
            if let Some(handle) = gw.take() {
                handle.stop();
            }
        }
        if let Ok(mut child) = self.native_child.lock() {
            child.take();
        }
        self.engine.set_embedded_gateway_active(false);
        self.engine.set_allama_http_port(None).await;
        self.control.stop().await.map_err(|e| e.to_string())?;
        self.publish_status(AllamaRuntimeMode::Stopped, None, 0).await;
        Ok(())
    }

    /// Restart the Allama stack.
    pub async fn restart(self: &Arc<Self>) -> Result<(), String> {
        self.stop().await?;
        self.start().await
    }

    /// HTTP health probe on the configured port.
    pub async fn http_online(&self) -> bool {
        probe_allama_http(self.http_port()).await
    }

    /// Build status for UI.
    pub async fn status_dto(&self) -> AllamaStatusDto {
        let snap = self.control.get_status();
        let port = self.http_port();
        let online = probe_allama_http(port).await;
        let state = match snap.mode {
            AllamaRuntimeMode::NativeBinary => "native",
            AllamaRuntimeMode::EmbeddedGateway => "embedded",
            AllamaRuntimeMode::Stopped => "stopped",
        };
        AllamaStatusDto {
            state: state.to_string(),
            port,
            detail: match snap.mode {
                AllamaRuntimeMode::NativeBinary => {
                    "Allama native server (Ollama-compatible API)".to_string()
                }
                AllamaRuntimeMode::EmbeddedGateway => {
                    "Exodus embedded Allama gateway (inference engine)".to_string()
                }
                AllamaRuntimeMode::Stopped => "Allama is stopped".to_string(),
            },
            endpoint_online: online,
            mode: state.to_string(),
            models_registered: snap.models_registered,
            binary_path: snap.binary_path,
        }
    }

    pub fn port(&self) -> u16 {
        self.http_port()
    }

    pub fn control_service(&self) -> Arc<AllamaService> {
        Arc::clone(&self.control)
    }

    async fn publish_status(
        &self,
        mode: AllamaRuntimeMode,
        binary_path: Option<String>,
        models_registered: usize,
    ) {
        let port = self.http_port();
        let online = probe_allama_http(port).await;
        self.control.set_status(AllamaServiceStatus {
            running: mode != AllamaRuntimeMode::Stopped,
            mode,
            http_port: port,
            http_online: online,
            binary_path,
            models_registered,
        });
    }
}

impl Default for AllamaManager {
    fn default() -> Self {
        Self::new(
            Arc::new(InferenceEngine::new()),
            ALLAMA_DEFAULT_PORT,
            resolve_models_dir(None),
        )
    }
}
