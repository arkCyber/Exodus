//! Exodus Browser — exodus-core sidecar process lifecycle.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

use crate::config::ExodusConfig;

/// Managed sidecar process and status for the settings UI.
pub struct SidecarManager {
    state: Arc<Mutex<SidecarRuntimeState>>,
    child: Mutex<Option<CommandChild>>,
}

#[derive(Debug, Clone, Default)]
enum SidecarRuntimeState {
    #[default]
    Disabled,
    BinaryNotFound,
    SpawnFailed(String),
    Running,
    Exited(Option<i32>),
}

impl SidecarRuntimeState {
    /// Mark process as exited (used when the sidecar channel closes).
    fn mark_exited(state: &Arc<Mutex<SidecarRuntimeState>>, code: Option<i32>) {
        if let Ok(mut s) = state.lock() {
            if matches!(*s, SidecarRuntimeState::Running) {
                *s = SidecarRuntimeState::Exited(code);
            }
        }
    }
}

/// Sidecar status returned to the frontend (settings panel).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarStatusDto {
    pub state: String,
    pub port: u16,
    pub detail: String,
    pub endpoint_online: bool,
}

impl SidecarManager {
    /// Create manager and optionally spawn the sidecar from app setup.
    pub fn new(app: &AppHandle, spawn_enabled: bool, port: u16) -> Self {
        let manager = Self {
            state: Arc::new(Mutex::new(SidecarRuntimeState::Disabled)),
            child: Mutex::new(None),
        };
        if spawn_enabled {
            if let Err(e) = manager.spawn_internal(app, port) {
                eprintln!("[Exodus] Sidecar setup: {}", e);
            }
        } else {
            if let Ok(mut s) = manager.state.lock() {
                *s = SidecarRuntimeState::Disabled;
            }
            println!("[Exodus] Sidecar auto-spawn disabled in settings");
        }
        manager
    }

    fn set_state(&self, state: SidecarRuntimeState) {
        if let Ok(mut s) = self.state.lock() {
            *s = state;
        }
    }

    fn spawn_internal(&self, app: &AppHandle, port: u16) -> Result<(), String> {
        let sidecar_command = app
            .shell()
            .sidecar("exodus-core")
            .map_err(|e| {
                let msg = format!("Sidecar binary not found: {}", e);
                self.set_state(SidecarRuntimeState::BinaryNotFound);
                msg
            })?;

        let sidecar_command = sidecar_command.args(["--port", &port.to_string()]);

        match sidecar_command.spawn() {
            Ok((mut rx, child)) => {
                self.set_state(SidecarRuntimeState::Running);
                if let Ok(mut slot) = self.child.lock() {
                    *slot = Some(child);
                }
                let state_watch = Arc::clone(&self.state);
                tauri::async_runtime::spawn(async move {
                    while let Some(event) = rx.recv().await {
                        match event {
                            CommandEvent::Stdout(line) => {
                                let log = String::from_utf8_lossy(&line);
                                println!("[Exodus Core STDOUT] {}", log.trim());
                            }
                            CommandEvent::Stderr(line) => {
                                let log = String::from_utf8_lossy(&line);
                                eprintln!("[Exodus Core STDERR] {}", log.trim());
                            }
                            CommandEvent::Terminated(payload) => {
                                SidecarRuntimeState::mark_exited(&state_watch, payload.code);
                                println!(
                                    "[Exodus Core] Engine terminated with status: {:?}",
                                    payload.code
                                );
                            }
                            _ => {}
                        }
                    }
                    SidecarRuntimeState::mark_exited(&state_watch, None);
                });
                println!("[Exodus] Sidecar spawned on port {}", port);
                Ok(())
            }
            Err(e) => {
                let msg = format!("Failed to spawn sidecar: {}", e);
                self.set_state(SidecarRuntimeState::SpawnFailed(msg.clone()));
                Err(msg)
            }
        }
    }

    fn kill_child(&self) {
        if let Ok(mut slot) = self.child.lock() {
            if let Some(child) = slot.take() {
                let _ = child.kill();
            }
        }
    }

    /// Stop and start sidecar using current config port.
    pub fn restart(&self, app: &AppHandle, port: u16, spawn_enabled: bool) -> Result<(), String> {
        self.kill_child();
        if !spawn_enabled {
            self.set_state(SidecarRuntimeState::Disabled);
            return Ok(());
        }
        self.spawn_internal(app, port)
    }

    /// Build DTO for the settings UI.
    pub async fn status_dto(&self, port: u16) -> SidecarStatusDto {
        let (state, detail) = match self.state.lock() {
            Ok(s) => match &*s {
                SidecarRuntimeState::Disabled => (
                    "disabled",
                    "Sidecar auto-start is off. Use Ollama on the same port or enable below."
                        .to_string(),
                ),
                SidecarRuntimeState::BinaryNotFound => (
                    "not_found",
                    "exodus-core binary missing. See src-tauri/binaries/README.md".to_string(),
                ),
                SidecarRuntimeState::SpawnFailed(msg) => ("spawn_failed", msg.clone()),
                SidecarRuntimeState::Running => {
                    ("running", format!("Process running on port {}", port))
                }
                SidecarRuntimeState::Exited(code) => (
                    "exited",
                    format!("Sidecar exited (code {:?})", code),
                ),
            },
            Err(_) => ("unknown", "Could not read sidecar state".to_string()),
        };

        let endpoint_online = probe_models_endpoint(port).await;

        SidecarStatusDto {
            state: state.to_string(),
            port,
            detail,
            endpoint_online,
        }
    }
}

/// Probe OpenAI-compatible `/v1/models` on the inference port.
pub async fn probe_models_endpoint(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/v1/models", port);
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client.get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Return sidecar + endpoint status for settings.
#[tauri::command]
pub async fn get_sidecar_status(
    sidecar: tauri::State<'_, SidecarManager>,
    config: tauri::State<'_, Mutex<ExodusConfig>>,
) -> Result<SidecarStatusDto, String> {
    let port = config
        .lock()
        .map_err(|e| format!("Config lock error: {}", e))?
        .ai_port;
    Ok(sidecar.status_dto(port).await)
}

/// Restart exodus-core sidecar (e.g. after changing AI port).
#[tauri::command]
pub async fn restart_sidecar(
    app: AppHandle,
    sidecar: tauri::State<'_, SidecarManager>,
    config: tauri::State<'_, Mutex<ExodusConfig>>,
) -> Result<SidecarStatusDto, String> {
    let (port, spawn_enabled) = {
        let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
        (cfg.ai_port, cfg.spawn_sidecar)
    };
    sidecar.restart(&app, port, spawn_enabled)?;
    Ok(sidecar.status_dto(port).await)
}
