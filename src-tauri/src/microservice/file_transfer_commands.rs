//! Tauri commands for File Transfer + ExodusWorkSpace + P2P CDN seeding.

use crate::exodus_workspace::{ExodusWorkspace, WORKSPACE_ROOM_ID};
use crate::file_transfer_engine::{
    ChecksumReport, FileTransferEngine, TransferDashboard, TransferEngineSettings,
};
use crate::microservice::file_transfer_hub::FileTransferHub;
use crate::microservice::{FileTransferMetadata, FileTransferService};
use crate::p2p_cdn::CdnContentKind;
use crate::p2p_cdn::P2pCdnState;
use crate::wan_relay::WanRelayConfig;
use crate::wan_relay_server::{WanRelayServerInfo, WanRelayServerState};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// Managed file transfer hub (service + engine + relay + watcher).
pub struct FileTransferState {
    pub hub: Arc<FileTransferHub>,
}

/// Workspace + transfer summary for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExodusWorkspaceInfo {
    pub root: String,
    pub shared_dir: String,
    pub inbox_dir: String,
    pub outbox_dir: String,
    pub node_id: String,
    pub room_id: String,
    pub mesh_host: Option<String>,
    pub mesh_port: Option<u16>,
    pub file_count: usize,
}

fn hub(app: &AppHandle) -> Result<Arc<FileTransferHub>, String> {
    Ok(app
        .try_state::<FileTransferState>()
        .ok_or_else(|| "File transfer hub not started".to_string())?
        .hub
        .clone())
}

fn workspace_ref(app: &AppHandle) -> Result<Arc<ExodusWorkspace>, String> {
    app.try_state::<Arc<ExodusWorkspace>>()
        .ok_or_else(|| "ExodusWorkSpace not initialized".to_string())
        .map(|s| Arc::clone(&*s))
}

/// Publish one file into workspace + CDN (shared helper).
pub async fn publish_file_to_workspace(
    app: &AppHandle,
    ws: &ExodusWorkspace,
    cdn: &P2pCdnState,
    path: &Path,
) -> Result<crate::exodus_workspace::WorkspaceFileEntry, String> {
    if !path.is_file() {
        return Err(format!("Not a file: {}", path.display()));
    }
    let _ = cdn.ensure_mesh().await;
    let (hash, _size) = cdn.store().import_file(path)?;
    let title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();
    let _ann = cdn
        .register_local_seed(
            WORKSPACE_ROOM_ID,
            &hash,
            &title,
            CdnContentKind::GenericFile,
            std::fs::metadata(path).map_err(|e| e.to_string())?.len(),
            None,
        )
        .await?;
    let entry = ws.publish_file(path, Some(hash))?;
    let _ = app.emit("exodus-workspace-file-added", &entry);
    Ok(entry)
}

/// Ensure file transfer hub exists (idempotent).
pub fn ensure_file_transfer_stack(app: &AppHandle, app_data_dir: &Path) -> Result<(), String> {
    if app.try_state::<FileTransferState>().is_some() {
        return Ok(());
    }
    let cdn = app
        .try_state::<Arc<P2pCdnState>>()
        .ok_or("P2P CDN must initialize before file transfer")?;
    let hub = FileTransferHub::bootstrap(app, app_data_dir, &cdn)?;
    app.manage(FileTransferState { hub });
    Ok(())
}

/// Start file transfer stack (in-process; no Unix socket).
#[tauri::command]
pub async fn file_transfer_service_start(app: AppHandle) -> Result<ExodusWorkspaceInfo, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    ensure_file_transfer_stack(&app, &app_data)?;
    exodus_workspace_info(app).await
}

/// Stop is a no-op for in-process service (kept for API compatibility).
#[tauri::command]
pub async fn file_transfer_service_stop(app: AppHandle) -> Result<(), String> {
    let _ = app;
    Ok(())
}

/// Workspace manifest and mesh endpoint info.
#[tauri::command]
pub async fn exodus_workspace_info(app: AppHandle) -> Result<ExodusWorkspaceInfo, String> {
    let ws = workspace_ref(&app)?;
    let manifest = ws.manifest_snapshot()?;
    let (mesh_host, mesh_port) = app
        .try_state::<Arc<P2pCdnState>>()
        .and_then(|c| c.mesh_endpoint())
        .map(|(h, p)| (Some(h), Some(p)))
        .unwrap_or((None, None));
    Ok(ExodusWorkspaceInfo {
        root: manifest.root,
        shared_dir: ws.shared_dir().to_string_lossy().to_string(),
        inbox_dir: ws.inbox_dir().to_string_lossy().to_string(),
        outbox_dir: ws.outbox_dir().to_string_lossy().to_string(),
        node_id: manifest.node_id,
        room_id: WORKSPACE_ROOM_ID.to_string(),
        mesh_host,
        mesh_port,
        file_count: manifest.files.len(),
    })
}

/// List files published in ExodusWorkSpace/shared.
#[tauri::command]
pub async fn exodus_workspace_list(app: AppHandle) -> Result<Vec<crate::exodus_workspace::WorkspaceFileEntry>, String> {
    let ws = workspace_ref(&app)?;
    ws.list_files()
}

/// Initiate transfer: workspace publish + CDN seed + transfer registry.
#[tauri::command]
pub async fn file_transfer_initiate(
    app: AppHandle,
    file_path: String,
    receiver_id: Option<String>,
) -> Result<FileTransferMetadata, String> {
    let h = hub(&app)?;
    let ws = workspace_ref(&app)?;
    let cdn = app
        .try_state::<Arc<P2pCdnState>>()
        .ok_or("P2P CDN not ready")?;
    let path = PathBuf::from(&file_path);
    let ws_entry = publish_file_to_workspace(&app, &ws, &cdn, &path).await?;
    let meta = h.service.initiate_transfer(
        &path,
        receiver_id,
        ws_entry.content_hash.clone(),
        None,
        Some(ws_entry.relative_path),
    )?;
    let _ = app.emit("file-transfer-initiated", &meta);
    Ok(meta)
}

/// Native file picker (system dialog).
#[tauri::command]
pub async fn file_transfer_pick_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .add_filter("All files", &["*"])
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

/// Dashboard: all transfers + engine settings.
#[tauri::command]
pub async fn file_transfer_dashboard(app: AppHandle) -> Result<TransferDashboard, String> {
    let h = hub(&app)?;
    let relay_enabled = h.relay.lock().map_err(|e| e.to_string())?.enabled;
    let workspace_watch_active = h.watcher.lock().map_err(|e| e.to_string())?.is_some();
    Ok(TransferDashboard {
        transfers: h.service.list_transfers(),
        settings: h.engine.settings(),
        relay_enabled,
        workspace_watch_active,
        active_background_jobs: h.engine.active_job_count(),
    })
}

#[tauri::command]
pub async fn file_transfer_set_throttle(app: AppHandle, bytes_per_sec: u64) -> Result<(), String> {
    hub(&app)?.engine.set_throttle(bytes_per_sec)
}

#[tauri::command]
pub async fn file_transfer_set_auto_reconnect(app: AppHandle, enabled: bool) -> Result<(), String> {
    hub(&app)?.engine.set_auto_reconnect(enabled)
}

#[tauri::command]
pub async fn file_transfer_set_relay_config(
    app: AppHandle,
    enabled: bool,
    relay_base_url: Option<String>,
) -> Result<(), String> {
    let h = hub(&app)?;
    let mut cfg = h.relay.lock().map_err(|e| e.to_string())?.clone();
    cfg.enabled = enabled;
    if let Some(url) = relay_base_url {
        cfg.relay_base_url = Some(url);
    }
    cfg.save(&h.app_data_dir)?;
    *h.relay.lock().map_err(|e| e.to_string())? = cfg;
    Ok(())
}

/// Configure embedded WAN relay HTTP server (local `/exodus-mesh/fetch`).
#[tauri::command]
pub async fn file_transfer_set_relay_serve(
    app: AppHandle,
    serve_enabled: bool,
    serve_port: Option<u16>,
    serve_bind: Option<String>,
) -> Result<WanRelayServerInfo, String> {
    let h = hub(&app)?;
    let mut cfg = h.relay.lock().map_err(|e| e.to_string())?.clone();
    cfg.serve_enabled = serve_enabled;
    if let Some(p) = serve_port {
        cfg.serve_port = p;
    }
    if let Some(b) = serve_bind {
        cfg.serve_bind = b;
    }
    cfg.apply_local_relay_url();
    cfg.save(&h.app_data_dir)?;
    *h.relay.lock().map_err(|e| e.to_string())? = cfg.clone();

    let state = app
        .try_state::<WanRelayServerState>()
        .ok_or("WAN relay server state not initialized")?;
    if serve_enabled {
        state
            .ensure_started(&cfg.serve_bind, cfg.serve_port)
            .await
    } else {
        Ok(state.info().await)
    }
}

/// WAN relay HTTP server status (embedded or external).
#[tauri::command]
pub async fn wan_relay_server_info(app: AppHandle) -> Result<WanRelayServerInfo, String> {
    let state = app
        .try_state::<WanRelayServerState>()
        .ok_or_else(|| "WAN relay server state not initialized".to_string())?;
    Ok(state.info().await)
}

/// Start background download with resume + checksum (runs while app is open).
#[tauri::command]
pub async fn file_transfer_start_background_download(
    app: AppHandle,
    content_hash: String,
    file_name: String,
) -> Result<FileTransferMetadata, String> {
    let h = hub(&app)?;
    let ws = workspace_ref(&app)?;
    let cdn = app
        .try_state::<Arc<P2pCdnState>>()
        .ok_or("P2P CDN not ready")?
        .inner()
        .clone();
    let _ = cdn.ensure_mesh().await;
    let _ = cdn.pull_external_gossip(WORKSPACE_ROOM_ID).await;
    let _ = cdn.sync_gossip();
    let size = cdn
        .store()
        .meta(&content_hash)
        .map(|(s, _)| s)
        .unwrap_or(0);
    let meta = h
        .service
        .register_download(file_name.clone(), size.max(1), content_hash.clone())?;
    let dest = ws.inbox_dir().join(&file_name);
    let relay = h.relay.lock().map_err(|e| e.to_string())?.clone();
    Arc::clone(&h.engine).spawn_background_download(
        app,
        Arc::clone(&h.service),
        cdn,
        relay,
        meta.transfer_id.clone(),
        content_hash,
        dest,
    );
    Ok(meta)
}

#[tauri::command]
pub async fn file_transfer_verify_checksum(
    app: AppHandle,
    transfer_id: String,
) -> Result<ChecksumReport, String> {
    let h = hub(&app)?;
    let report = FileTransferEngine::build_checksum_report(&h.service, &transfer_id)?;
    h.service
        .set_checksum_verified(&transfer_id, report.destination_verified)?;
    let _ = app;
    Ok(report)
}

#[tauri::command]
pub async fn exodus_workspace_watch_start(app: AppHandle) -> Result<(), String> {
    let h = hub(&app)?;
    let ws = workspace_ref(&app)?;
    let cdn = app
        .try_state::<Arc<P2pCdnState>>()
        .ok_or("P2P CDN not ready")?
        .inner()
        .clone();
    let handle = crate::workspace_watcher::start_workspace_watch(app, ws, cdn)?;
    *h.watcher.lock().map_err(|e| e.to_string())? = Some(handle);
    Ok(())
}

#[tauri::command]
pub async fn exodus_workspace_watch_stop(app: AppHandle) -> Result<(), String> {
    let h = hub(&app)?;
    if let Some(w) = h.watcher.lock().map_err(|e| e.to_string())?.take() {
        w.stop();
    }
    Ok(())
}

#[tauri::command]
pub async fn file_transfer_get(
    app: AppHandle,
    transfer_id: String,
) -> Result<FileTransferMetadata, String> {
    hub(&app)?
        .service
        .get_transfer(&transfer_id)
        .ok_or_else(|| format!("Transfer not found: {transfer_id}"))
}

#[tauri::command]
pub async fn file_transfer_list(app: AppHandle) -> Result<Vec<FileTransferMetadata>, String> {
    Ok(hub(&app)?.service.list_transfers())
}

#[tauri::command]
pub async fn file_transfer_get_chunks(
    app: AppHandle,
    transfer_id: String,
) -> Result<Vec<crate::microservice::FileChunk>, String> {
    Ok(hub(&app)?.service.get_chunks(&transfer_id))
}

#[tauri::command]
pub async fn file_transfer_update_status(
    app: AppHandle,
    transfer_id: String,
    status: String,
) -> Result<(), String> {
    hub(&app)?.service.update_status(&transfer_id, &status)
}

#[tauri::command]
pub async fn file_transfer_cancel(app: AppHandle, transfer_id: String) -> Result<(), String> {
    hub(&app)?.service.cancel_transfer(&transfer_id)
}

#[tauri::command]
pub async fn file_transfer_generate_qr_data(
    app: AppHandle,
    transfer_id: String,
) -> Result<serde_json::Value, String> {
    hub(&app)?.service.generate_qr_data(&transfer_id)
}

#[tauri::command]
pub async fn file_transfer_resolve_by_short_code(
    app: AppHandle,
    short_code: String,
) -> Result<FileTransferMetadata, String> {
    hub(&app)?
        .service
        .resolve_by_short_code(&short_code)
        .ok_or_else(|| format!("No transfer for code: {short_code}"))
}

#[tauri::command]
pub async fn file_transfer_retry(app: AppHandle, transfer_id: String) -> Result<(), String> {
    hub(&app)?.service.retry_transfer(&transfer_id)
}

#[tauri::command]
pub async fn file_transfer_resolve_conflict(
    app: AppHandle,
    file_name: String,
    existing_files: Vec<String>,
) -> Result<String, String> {
    Ok(hub(&app)?
        .service
        .resolve_conflict(&file_name, &existing_files))
}

/// Receive file by CDN hash into workspace inbox (peer pull simulation / local complete blob).
#[tauri::command]
pub async fn file_transfer_receive_to_inbox(
    app: AppHandle,
    content_hash: String,
    file_name: String,
) -> Result<String, String> {
    let ws = workspace_ref(&app)?;
    let cdn = app
        .try_state::<Arc<P2pCdnState>>()
        .ok_or("P2P CDN not ready")?;
    if !cdn.store().has_complete(&content_hash) {
        return Err("Blob not available locally; run p2p_cdn_download first".into());
    }
    let inbox = ws.inbox_dir();
    let dest = inbox.join(&file_name);
    let dest = if dest.exists() {
        let mut n = 1;
        loop {
            let candidate = inbox.join(format!("{file_name}.{n}"));
            if !candidate.exists() {
                break candidate;
            }
            n += 1;
        }
    } else {
        dest
    };
    cdn.store()
        .export_to_file(&content_hash, &dest)
        .map_err(|e| e.to_string())?;
    let _ = ws.publish_file(&dest, Some(content_hash))?;
    Ok(dest.to_string_lossy().to_string())
}
