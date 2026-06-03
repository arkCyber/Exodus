//! Background file transfer engine — resume, throttle, checksum reports, auto-reconnect.
//!
//! Jobs run in Tokio tasks and persist state so transfers continue while the Tauri app runs
//! (including when the user navigates away from the WorkSpace panel).

use crate::exodus_workspace::WORKSPACE_ROOM_ID;
use crate::microservice::file_transfer_service::{FileTransferMetadata, FileTransferService};
use crate::p2p_cdn::{
    fetch_from_mesh_peers, start_cdn_download, tickets_from_peers, CdnContentKind, ExodusCdnTicket,
    P2pCdnState,
};
use crate::wan_relay::WanRelayConfig;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

/// Engine-wide settings (persisted).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEngineSettings {
    /// Max bytes per second (0 = unlimited).
    pub throttle_bytes_per_sec: u64,
    pub auto_reconnect: bool,
    pub background_jobs: bool,
    pub workspace_watch_enabled: bool,
}

impl Default for TransferEngineSettings {
    fn default() -> Self {
        Self {
            throttle_bytes_per_sec: 0,
            auto_reconnect: true,
            background_jobs: true,
            workspace_watch_enabled: true,
        }
    }
}

/// Per-chunk checksum entry for integrity reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkChecksumEntry {
    pub index: u32,
    pub hash: String,
    pub size_bytes: u64,
}

/// Checksum report written after transfer completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecksumReport {
    pub algorithm: String,
    pub file_hash: String,
    pub file_size: u64,
    pub chunk_count: u32,
    pub chunks: Vec<ChunkChecksumEntry>,
    pub destination_verified: bool,
    pub verified_at: u64,
    pub mismatch_chunks: Vec<u32>,
}

/// Resume checkpoint for interrupted downloads.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransferResumeState {
    pub completed_chunks: Vec<u32>,
    pub bytes_done: u64,
    pub last_peer_attempt: Option<String>,
}

/// Live progress event for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgressEvent {
    pub transfer_id: String,
    pub status: String,
    pub progress_percent: f32,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub speed_bps: u64,
    pub direction: String,
    pub checksum_verified: bool,
    pub last_error: Option<String>,
}

/// Dashboard aggregate for UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferDashboard {
    pub transfers: Vec<FileTransferMetadata>,
    pub settings: TransferEngineSettings,
    pub relay_enabled: bool,
    pub workspace_watch_active: bool,
    pub active_background_jobs: u32,
}

/// Background transfer engine.
pub struct FileTransferEngine {
    settings: Arc<Mutex<TransferEngineSettings>>,
    active_jobs: Arc<Mutex<u32>>,
    app_data_dir: PathBuf,
}

impl FileTransferEngine {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let settings_path = app_data_dir.join("transfer_engine_settings.json");
        let settings = if settings_path.exists() {
            std::fs::read_to_string(&settings_path)
                .ok()
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_default()
        } else {
            TransferEngineSettings::default()
        };
        Self {
            settings: Arc::new(Mutex::new(settings)),
            active_jobs: Arc::new(Mutex::new(0)),
            app_data_dir,
        }
    }

    pub fn settings(&self) -> TransferEngineSettings {
        self.settings.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn set_throttle(&self, bytes_per_sec: u64) -> Result<(), String> {
        {
            let mut s = self.settings.lock().map_err(|e| e.to_string())?;
            s.throttle_bytes_per_sec = bytes_per_sec;
        }
        self.persist_settings()
    }

    pub fn set_auto_reconnect(&self, enabled: bool) -> Result<(), String> {
        {
            let mut s = self.settings.lock().map_err(|e| e.to_string())?;
            s.auto_reconnect = enabled;
        }
        self.persist_settings()
    }

    pub fn active_job_count(&self) -> u32 {
        self.active_jobs.lock().map(|n| *n).unwrap_or(0)
    }

    /// Periodic reconnect loop for paused/failed downloads.
    pub fn spawn_reconnect_loop(
        self: Arc<Self>,
        app: AppHandle,
        service: Arc<FileTransferService>,
        cdn: Arc<P2pCdnState>,
        relay: WanRelayConfig,
        inbox_dir: PathBuf,
    ) {
        let engine = Arc::clone(&self);
        tauri::async_runtime::spawn(async move {
            loop {
                sleep(Duration::from_secs(45)).await;
                if !engine.settings().auto_reconnect {
                    continue;
                }
                Arc::clone(&engine).spawn_resume_pending(
                    app.clone(),
                    Arc::clone(&service),
                    Arc::clone(&cdn),
                    relay.clone(),
                    inbox_dir.clone(),
                );
            }
        });
    }

    fn persist_settings(&self) -> Result<(), String> {
        let s = self.settings.lock().map_err(|e| e.to_string())?;
        let path = self.app_data_dir.join("transfer_engine_settings.json");
        std::fs::write(path, serde_json::to_string_pretty(&*s).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }

    fn bump_jobs(&self, delta: i32) {
        if let Ok(mut n) = self.active_jobs.lock() {
            if delta < 0 {
                *n = n.saturating_sub((-delta) as u32);
            } else {
                *n = n.saturating_add(delta as u32);
            }
        }
    }

    /// Build checksum report from on-disk chunks and verify file hash.
    pub fn build_checksum_report(
        service: &FileTransferService,
        transfer_id: &str,
    ) -> Result<ChecksumReport, String> {
        let meta = service
            .get_transfer(transfer_id)
            .ok_or_else(|| format!("Transfer not found: {transfer_id}"))?;
        let chunks = service.get_chunks(transfer_id);
        let mut entries = Vec::new();
        let mut hasher = Hasher::new();
        for ch in &chunks {
            let data = service.read_chunk_bytes(transfer_id, ch.chunk_index)?;
            hasher.update(&data);
            entries.push(ChunkChecksumEntry {
                index: ch.chunk_index,
                hash: ch.chunk_hash.clone(),
                size_bytes: data.len() as u64,
            });
        }
        let computed = hasher.finalize().to_hex().to_string();
        let mut mismatch = Vec::new();
        for ch in &chunks {
            let data = service.read_chunk_bytes(transfer_id, ch.chunk_index)?;
            let h = blake3::hash(&data).to_hex().to_string();
            if h != ch.chunk_hash {
                mismatch.push(ch.chunk_index);
            }
        }
        let verified = computed == meta.blob_hash && mismatch.is_empty();
        let report = ChecksumReport {
            algorithm: "blake3".into(),
            file_hash: meta.blob_hash.clone(),
            file_size: meta.file_size,
            chunk_count: meta.chunk_count,
            chunks: entries,
            destination_verified: verified,
            verified_at: now_secs(),
            mismatch_chunks: mismatch,
        };
        let path = service
            .storage_dir()
            .join(transfer_id)
            .join("checksum_report.json");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(report)
    }

    /// Spawn background download with resume + throttle + checksum verification.
    pub fn spawn_background_download(
        self: Arc<Self>,
        app: AppHandle,
        service: Arc<FileTransferService>,
        cdn: Arc<P2pCdnState>,
        relay: WanRelayConfig,
        transfer_id: String,
        content_hash: String,
        dest: PathBuf,
    ) {
        self.bump_jobs(1);
        let engine = Arc::clone(&self);
        tauri::async_runtime::spawn(async move {
            let result = engine
                .run_download_job(
                    &app,
                    &service,
                    Arc::clone(&cdn),
                    &relay,
                    &transfer_id,
                    &content_hash,
                    &dest,
                )
                .await;
            if let Err(e) = result {
                let _ = service.update_status(&transfer_id, "failed");
                engine.update_meta_error(&service, &transfer_id, &e);
            }
            engine.bump_jobs(-1);
        });
    }

    /// Resume pending / failed downloads on app startup.
    pub fn spawn_resume_pending(
        self: Arc<Self>,
        app: AppHandle,
        service: Arc<FileTransferService>,
        cdn: Arc<P2pCdnState>,
        relay: WanRelayConfig,
        inbox_dir: PathBuf,
    ) {
        let settings = self.settings();
        if !settings.auto_reconnect || !settings.background_jobs {
            return;
        }
        let pending: Vec<_> = service
            .list_transfers()
            .into_iter()
            .filter(|t| {
                t.direction == "download"
                    && (t.status == "pending" || t.status == "paused" || t.status == "failed")
            })
            .collect();
        for meta in pending {
            if let Some(hash) = meta.cdn_content_hash.clone() {
                let dest = inbox_dir.join(&meta.file_name);
                Arc::clone(&self).spawn_background_download(
                    app.clone(),
                    Arc::clone(&service),
                    Arc::clone(&cdn),
                    relay.clone(),
                    meta.transfer_id.clone(),
                    hash,
                    dest,
                );
            }
        }
    }

    async fn run_download_job(
        &self,
        app: &AppHandle,
        service: &FileTransferService,
        cdn: Arc<P2pCdnState>,
        relay: &WanRelayConfig,
        transfer_id: &str,
        content_hash: &str,
        dest: &Path,
    ) -> Result<(), String> {
        let _ = service.update_status(transfer_id, "transferring");
        self.emit_progress(app, service, transfer_id);

        if cdn.store().has_complete(content_hash) {
            return self
                .finish_download_success(app, service, &cdn, transfer_id, dest, content_hash)
                .await;
        }

        let _ = cdn.pull_external_gossip(WORKSPACE_ROOM_ID).await;
        let _ = cdn.sync_gossip();

        if let Ok(()) = Self::try_mesh_full_download(&cdn, content_hash, dest).await
        {
            return self
                .finish_download_success(app, service, &cdn, transfer_id, dest, content_hash)
                .await;
        }

        let meta_for_cdn = service.get_transfer(transfer_id);
        if let Some(meta) = meta_for_cdn {
            let room = meta.room_id.clone();
            let title = meta.file_name.clone();
            if let Ok(()) = Self::try_p2p_cdn_download(
                app,
                Arc::clone(&cdn),
                &room,
                content_hash,
                &title,
                dest,
            )
            .await
            {
                return self
                    .finish_download_success(app, service, &cdn, transfer_id, dest, content_hash)
                    .await;
            }
        }

        let peers = cdn.list_peers(content_hash)?;
        if peers.is_empty() {
            return Err("No peers available; sync gossip or configure WAN relay".into());
        }

        let mut resume = load_resume(service, transfer_id);
        let throttle = self.settings().throttle_bytes_per_sec;
        let tickets: Vec<_> = tickets_from_peers(&peers);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3600))
            .build()
            .map_err(|e| e.to_string())?;

        let meta = service.get_transfer(transfer_id).ok_or("missing meta")?;
        let total = meta.file_size.max(1);
        let chunk_count = meta.chunk_count.max(1);
        let mut completed: HashSet<u32> = resume.completed_chunks.iter().copied().collect();

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let start = Instant::now();
        let mut last_emit = Instant::now();

        for index in 0..chunk_count {
            if completed.contains(&index) {
                continue;
            }
            let mut fetched = false;
            let mut last_err = String::new();
            for ticket in &tickets {
                let urls = mesh_fetch_urls(ticket, content_hash, index, relay);
                for url in urls {
                    match client.get(&url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                            apply_throttle(throttle, bytes.len()).await;
                            let chunk_path = dest.with_extension(format!("part_{index:06}"));
                            std::fs::write(&chunk_path, &bytes).map_err(|e| e.to_string())?;
                            completed.insert(index);
                            resume.completed_chunks.push(index);
                            resume.bytes_done = resume.bytes_done.saturating_add(bytes.len() as u64);
                            save_resume(service, transfer_id, &resume)?;
                            fetched = true;
                            break;
                        }
                        Ok(resp) => last_err = format!("HTTP {}", resp.status()),
                        Err(e) => last_err = e.to_string(),
                    }
                }
                if fetched {
                    break;
                }
            }
            if !fetched {
                let _ = service.update_status(transfer_id, "paused");
                self.update_meta_error(service, transfer_id, &last_err);
                self.emit_progress(app, service, transfer_id);
                return Err(format!("Chunk {index} failed: {last_err}"));
            }

            let pct = (completed.len() as f32 / chunk_count as f32) * 100.0;
            let elapsed = start.elapsed().as_secs_f64().max(0.001);
            let speed = (resume.bytes_done as f64 / elapsed) as u64;
            self.set_progress(service, transfer_id, pct, resume.bytes_done, total)?;
            self.set_speed(service, transfer_id, speed)?;
            if last_emit.elapsed() > Duration::from_millis(400) {
                self.emit_progress(app, service, transfer_id);
                last_emit = Instant::now();
            }
        }

        assemble_parts(dest, chunk_count)?;
        self.finish_download_success(app, service, &cdn, transfer_id, dest, content_hash)
            .await
    }

    async fn try_mesh_full_download(
        cdn: &P2pCdnState,
        content_hash: &str,
        dest: &Path,
    ) -> Result<(), String> {
        let peers = cdn.list_peers(content_hash)?;
        let external: Vec<_> = peers
            .into_iter()
            .filter(|p| p.node_id != cdn.node_id)
            .collect();
        if external.is_empty() {
            return Err("No external mesh peers".into());
        }
        fetch_from_mesh_peers(content_hash, &external, dest, cdn.store()).await?;
        Ok(())
    }

    async fn try_p2p_cdn_download(
        app: &AppHandle,
        cdn: Arc<P2pCdnState>,
        room_id: &str,
        content_hash: &str,
        title: &str,
        dest: &Path,
    ) -> Result<(), String> {
        let work_dir = dest
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| dest.to_path_buf());
        std::fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;
        let job = start_cdn_download(
            app.clone(),
            cdn.clone(),
            room_id.to_string(),
            content_hash.to_string(),
            title.to_string(),
            CdnContentKind::GenericFile,
            None,
            work_dir,
        )
        .await?;
        if job.status != "completed" {
            return Err(
                job.error
                    .unwrap_or_else(|| format!("p2p_cdn_download status: {}", job.status)),
            );
        }
        if cdn.store().has_complete(content_hash) {
            cdn.store().export_to_file(content_hash, dest)?;
            return Ok(());
        }
        if let Some(path) = job.local_path {
            std::fs::copy(&path, dest).map_err(|e| e.to_string())?;
            return Ok(());
        }
        Err("p2p_cdn_download completed without local file".into())
    }

    async fn finish_download_success(
        &self,
        app: &AppHandle,
        service: &FileTransferService,
        cdn: &P2pCdnState,
        transfer_id: &str,
        dest: &Path,
        content_hash: &str,
    ) -> Result<(), String> {
        if !dest.exists() {
            if cdn.store().has_complete(content_hash) {
                cdn.store().export_to_file(content_hash, dest)?;
            } else {
                return Err("Download finished but destination file missing".into());
            }
        }
        let file_bytes = std::fs::read(dest).map_err(|e| e.to_string())?;
        let (hash, _) = cdn.store().put_bytes(&file_bytes)?;
        if hash != content_hash {
            return Err(format!("Checksum mismatch expected {content_hash} got {hash}"));
        }
        service.update_status(transfer_id, "completed")?;
        let total = service
            .get_transfer(transfer_id)
            .map(|m| m.file_size)
            .unwrap_or(file_bytes.len() as u64);
        self.set_progress(service, transfer_id, 100.0, total, total)?;
        let report = Self::build_checksum_report(service, transfer_id)?;
        self.mark_checksum(service, transfer_id, report.destination_verified)?;
        self.emit_progress(app, service, transfer_id);
        Ok(())
    }

    fn emit_progress(&self, app: &AppHandle, service: &FileTransferService, transfer_id: &str) {
        if let Some(meta) = service.get_transfer(transfer_id) {
            let ev = TransferProgressEvent {
                transfer_id: transfer_id.to_string(),
                status: meta.status.clone(),
                progress_percent: meta.progress_percent,
                bytes_done: meta.bytes_done,
                bytes_total: meta.file_size,
                speed_bps: meta.speed_bps,
                direction: meta.direction.clone(),
                checksum_verified: meta.checksum_verified,
                last_error: meta.last_error.clone(),
            };
            let _ = app.emit("file-transfer-progress", &ev);
        }
    }

    fn set_progress(
        &self,
        service: &FileTransferService,
        id: &str,
        pct: f32,
        done: u64,
        total: u64,
    ) -> Result<(), String> {
        service.update_progress(id, pct, done, total)
    }

    fn set_speed(&self, service: &FileTransferService, id: &str, bps: u64) -> Result<(), String> {
        service.update_speed(id, bps)
    }

    fn update_meta_error(&self, service: &FileTransferService, id: &str, err: &str) {
        let _ = service.update_last_error(id, err);
    }

    fn mark_checksum(&self, service: &FileTransferService, id: &str, ok: bool) -> Result<(), String> {
        service.set_checksum_verified(id, ok)
    }
}

fn mesh_fetch_urls(ticket: &ExodusCdnTicket, hash: &str, index: u32, relay: &WanRelayConfig) -> Vec<String> {
    let mut out = Vec::new();
    let direct = format!(
        "{}/blobs/{hash}/chunks/{index:06}",
        ticket.base_url()
    );
    out.push(direct);
    if let Some(proxy) = relay.proxy_url(&ticket.host, ticket.port, &format!("/blobs/{hash}/chunks/{index:06}")) {
        out.push(proxy);
    }
    out
}

async fn apply_throttle(throttle_bps: u64, nbytes: usize) {
    if throttle_bps == 0 {
        return;
    }
    let delay_ms = ((nbytes as u64) * 1000) / throttle_bps.max(1);
    if delay_ms > 0 {
        sleep(Duration::from_millis(delay_ms.min(500))).await;
    }
}

fn assemble_parts(dest: &Path, chunk_count: u32) -> Result<(), String> {
    use std::io::Write;
    let mut out = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    for index in 0..chunk_count {
        let part = dest.with_extension(format!("part_{index:06}"));
        if part.exists() {
            let data = std::fs::read(&part).map_err(|e| e.to_string())?;
            out.write_all(&data).map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(part);
        }
    }
    Ok(())
}

fn resume_path(service: &FileTransferService, transfer_id: &str) -> PathBuf {
    service.storage_dir().join(transfer_id).join("resume.json")
}

fn load_resume(service: &FileTransferService, transfer_id: &str) -> TransferResumeState {
    let path = resume_path(service, transfer_id);
    if !path.exists() {
        return TransferResumeState::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or_default()
}

fn save_resume(service: &FileTransferService, transfer_id: &str, state: &TransferResumeState) -> Result<(), String> {
    let path = resume_path(service, transfer_id);
    std::fs::write(path, serde_json::to_string_pretty(state).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
