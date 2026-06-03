//! Exodus P2P CDN — parallel download orchestration (peers first, HTTP fallback).

use std::path::PathBuf;
use std::sync::Arc;

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use super::swarm::P2pCdnState;
use super::types::{CdnContentKind, CdnDownloadJob, CdnDownloadProgress, CdnPeerSource};

/// Start a CDN download: local hit → parallel peers → HTTP seed for the swarm.
pub async fn start_cdn_download(
    app: AppHandle,
    hub: Arc<P2pCdnState>,
    room_id: String,
    content_hash: String,
    title: String,
    kind: CdnContentKind,
    http_url: Option<String>,
    downloads_dir: PathBuf,
) -> Result<CdnDownloadJob, String> {
    let job_id = format!("cdn-{}", Uuid::new_v4());
    let mut job = CdnDownloadJob {
        job_id: job_id.clone(),
        content_hash: content_hash.clone(),
        title: title.clone(),
        status: "pending".into(),
        progress_percent: 0.0,
        bytes_done: 0,
        bytes_total: 0,
        source: "init".into(),
        peer_count: 0,
        local_path: None,
        error: None,
    };
    hub.upsert_job(job.clone())?;

    if hub.store().has_complete(&content_hash) {
        let dest = downloads_dir.join(safe_filename(&title, &content_hash));
        let bytes = hub
            .store()
            .export_to_file(&content_hash, &dest)
            .map_err(|e| e.to_string())?;
        job.status = "completed".into();
        job.source = "local_cache".into();
        job.progress_percent = 100.0;
        job.bytes_done = bytes;
        job.bytes_total = bytes;
        job.local_path = Some(dest.to_string_lossy().to_string());
        hub.upsert_job(job.clone())?;
        emit_progress(&app, &job);
        return Ok(job);
    }

    let peers = hub.list_peers(&content_hash)?;
    let external_peers: Vec<CdnPeerSource> = peers
        .into_iter()
        .filter(|p| p.node_id != hub.node_id)
        .collect();
    job.peer_count = external_peers.len() as u32;

    if !external_peers.is_empty() {
        job.status = "downloading".into();
        job.source = "p2p_peers".into();
        hub.upsert_job(job.clone())?;
        emit_progress(&app, &job);

        if let Ok(bytes) = try_fetch_from_peers(&hub, &content_hash, &external_peers, &downloads_dir, &title).await
        {
            job.status = "completed".into();
            job.progress_percent = 100.0;
            job.bytes_done = bytes;
            job.bytes_total = bytes;
            job.local_path = Some(
                downloads_dir
                    .join(safe_filename(&title, &content_hash))
                    .to_string_lossy()
                    .to_string(),
            );
            hub.upsert_job(job.clone())?;
            let _ = hub
                .register_local_seed(
                    &room_id,
                    &content_hash,
                    &title,
                    kind,
                    bytes,
                    http_url.clone(),
                )
                .await;
            emit_progress(&app, &job);
            return Ok(job);
        }
    }

    let url = http_url.ok_or_else(|| {
        "No peers available and no HTTP source URL for fallback".to_string()
    })?;

    job.status = "downloading".into();
    job.source = "http_origin".into();
    hub.upsert_job(job.clone())?;
    emit_progress(&app, &job);

    let dest = downloads_dir.join(safe_filename(&title, &content_hash));
    let (bytes_done, bytes_total) = http_download_with_progress(&app, &job_id, &content_hash, &url, &dest).await?;

    let (hash, size) = hub.store().import_file(&dest)?;
    if hash != content_hash {
        tracing::warn!(
            "CDN hash mismatch expected={} got={} — using computed hash",
            content_hash,
            hash
        );
    }

    let _ = hub
        .register_local_seed(&room_id, &hash, &title, kind, size, Some(url))
        .await;

    job.status = "completed".into();
    job.source = "http_then_seed".into();
    job.progress_percent = 100.0;
    job.bytes_done = bytes_done;
    job.bytes_total = bytes_total.max(size);
    job.local_path = Some(dest.to_string_lossy().to_string());
    hub.upsert_job(job.clone())?;
    emit_progress(&app, &job);
    Ok(job)
}

/// Attempt parallel peer fetch (iroh-blobs adapter hook; dev: shared-store simulation).
async fn try_fetch_from_peers(
    hub: &P2pCdnState,
    content_hash: &str,
    peers: &[CdnPeerSource],
    downloads_dir: &PathBuf,
    title: &str,
) -> Result<u64, String> {
    use super::iroh_adapter::PeerBlobTransport;
    let transport = PeerBlobTransport::new(hub.node_id.clone());
    transport
        .download_from_peers(content_hash, peers, downloads_dir, title, hub.store())
        .await
}

async fn http_download_with_progress(
    app: &AppHandle,
    job_id: &str,
    content_hash: &str,
    url: &str,
    dest: &std::path::Path,
) -> Result<(u64, u64), String> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("HTTP status {}", response.status()));
    }
    let total = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut done = 0u64;
    use std::io::Write;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {e}"))?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        done += chunk.len() as u64;
        let pct = if total > 0 {
            (done as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        emit_progress(
            app,
            &CdnDownloadJob {
                job_id: job_id.to_string(),
                content_hash: content_hash.to_string(),
                title: String::new(),
                status: "downloading".into(),
                progress_percent: pct,
                bytes_done: done,
                bytes_total: total,
                source: "http_origin".into(),
                peer_count: 0,
                local_path: None,
                error: None,
            },
        );
    }
    Ok((done, total.max(done)))
}

fn emit_progress(app: &AppHandle, job: &CdnDownloadJob) {
    let payload = CdnDownloadProgress {
        job_id: job.job_id.clone(),
        content_hash: job.content_hash.clone(),
        progress_percent: job.progress_percent,
        bytes_done: job.bytes_done,
        bytes_total: job.bytes_total,
        source: job.source.clone(),
        peer_count: job.peer_count,
    };
    let _ = app.emit("exodus-p2p-cdn-progress", payload);
}

fn safe_filename(title: &str, hash: &str) -> String {
    let base: String = title
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
            c
        } else {
            '_'
        })
        .take(80)
        .collect();
    let short_hash = if hash.len() > 8 { &hash[..8] } else { hash };
    format!("{base}-{short_hash}.bin")
}
