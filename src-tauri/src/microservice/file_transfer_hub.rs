//! Unified file transfer hub — service, engine, relay, workspace watcher.

use crate::exodus_workspace::ExodusWorkspace;
use crate::file_transfer_engine::FileTransferEngine;
use crate::microservice::file_transfer_service::{FileTransferService, FileTransferServiceConfig};
use crate::p2p_cdn::P2pCdnState;
use crate::wan_relay::WanRelayConfig;
use crate::wan_relay_server::WanRelayServerState;
use crate::workspace_watcher::WorkspaceWatcherHandle;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

/// Central hub for ExodusWorkSpace file exchange.
pub struct FileTransferHub {
    pub service: Arc<FileTransferService>,
    pub engine: Arc<FileTransferEngine>,
    pub relay: Arc<Mutex<WanRelayConfig>>,
    pub app_data_dir: PathBuf,
    pub watcher: Mutex<Option<WorkspaceWatcherHandle>>,
    reconnect_started: Mutex<bool>,
}

impl FileTransferHub {
    /// Create hub and optionally start background services.
    pub fn bootstrap(
        app: &AppHandle,
        app_data_dir: &Path,
        cdn: &Arc<P2pCdnState>,
    ) -> Result<Arc<Self>, String> {
        let node_id = cdn.node_id.clone();
        let cfg = FileTransferServiceConfig::under_app_data(app_data_dir, &node_id);
        let service = Arc::new(FileTransferService::new(cfg)?);
        let engine = Arc::new(FileTransferEngine::new(app_data_dir.to_path_buf()));
        let mut relay_cfg = WanRelayConfig::load(app_data_dir);
        relay_cfg.apply_local_relay_url();
        let _ = relay_cfg.save(app_data_dir);
        if app.try_state::<WanRelayServerState>().is_none() {
            app.manage(WanRelayServerState::new());
        }
        let relay = Arc::new(Mutex::new(relay_cfg));
        let hub = Arc::new(Self {
            service,
            engine,
            relay,
            app_data_dir: app_data_dir.to_path_buf(),
            watcher: Mutex::new(None),
            reconnect_started: Mutex::new(false),
        });
        hub.start_background(app.clone(), cdn.clone())?;
        Ok(hub)
    }

    fn start_background(&self, app: AppHandle, cdn: Arc<P2pCdnState>) -> Result<(), String> {
        self.start_wan_relay_server(&app)?;
        let relay = self.relay.lock().map_err(|e| e.to_string())?.clone();
        let ws = Arc::new(ExodusWorkspace::new(&self.app_data_dir, cdn.node_id.clone())?);
        app.manage(ws.clone());

        let engine = Arc::clone(&self.engine);
        let service = Arc::clone(&self.service);
        engine.spawn_resume_pending(
            app.clone(),
            service,
            cdn.clone(),
            relay.clone(),
            ws.inbox_dir(),
        );

        let mut started = self.reconnect_started.lock().map_err(|e| e.to_string())?;
        if !*started {
            *started = true;
            let engine_loop = Arc::clone(&self.engine);
            let app_loop = app.clone();
            let svc_loop = Arc::clone(&self.service);
            let cdn_loop = Arc::clone(&cdn);
            let relay_loop = relay.clone();
            let inbox = ws.inbox_dir();
            engine_loop.spawn_reconnect_loop(app_loop, svc_loop, cdn_loop, relay_loop, inbox);
        }

        if self.engine.settings().workspace_watch_enabled {
            let handle = crate::workspace_watcher::start_workspace_watch(app, ws, cdn)?;
            *self.watcher.lock().map_err(|e| e.to_string())? = Some(handle);
        }
        Ok(())
    }

    fn start_wan_relay_server(&self, app: &AppHandle) -> Result<(), String> {
        let cfg = self.relay.lock().map_err(|e| e.to_string())?.clone();
        if !cfg.serve_enabled {
            return Ok(());
        }
        let state = app
            .try_state::<WanRelayServerState>()
            .ok_or("WanRelayServerState not managed")?
            .inner()
            .clone();
        let bind = cfg.serve_bind.clone();
        let port = cfg.serve_port;
        let app_data = self.app_data_dir.clone();
        let relay_arc = Arc::clone(&self.relay);
        tauri::async_runtime::spawn(async move {
            match state.ensure_started(&bind, port).await {
                Ok(info) => {
                    if let Ok(mut guard) = relay_arc.lock() {
                        guard.apply_local_relay_url();
                        guard.relay_base_url = Some(info.base_url.clone());
                        guard.enabled = true;
                        let _ = guard.save(&app_data);
                    }
                    tracing::info!("WAN relay ready at {}", info.base_url);
                }
                Err(e) => tracing::warn!("WAN relay server start failed: {e}"),
            }
        });
        Ok(())
    }
}
