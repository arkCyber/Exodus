//! Exodus Browser — file download manager with progress events and resume support.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use chrono::Utc;
use futures_util::StreamExt;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::download_manager::{DownloadManager, DownloadStatus};

/// Progress payload emitted to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressPayload {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub received: u64,
    pub total: u64,
    pub progress: f32,
}

/// Completion payload emitted when a download finishes.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadDonePayload {
    pub id: String,
    pub path: String,
    pub filename: String,
}

/// Error payload emitted when a download fails (includes id for targeted UI updates).
#[derive(Debug, Clone, Serialize)]
pub struct DownloadErrorPayload {
    pub id: String,
    pub message: String,
}

/// Emit a download error event to the frontend.
fn emit_download_error(app: &AppHandle, id: &str, message: &str) {
    let _ = app.emit(
        "exodus-download-error",
        DownloadErrorPayload {
            id: id.to_string(),
            message: message.to_string(),
        },
    );
}

/// Log module load with UTC timestamp (import point).
fn log_downloads_init() {
    println!(
        "[{}] exodus downloads module ready",
        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC")
    );
}

/// Derive a filename from URL path or use a fallback.
fn filename_from_url(url: &str, override_name: Option<String>) -> String {
    if let Some(name) = override_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(idx) = url.rfind('/') {
        let last = url[idx + 1..].split('?').next().unwrap_or("");
        if !last.is_empty() && last.contains('.') {
            return last.to_string();
        }
    }
    format!("download-{}.bin", Utc::now().timestamp())
}

/// Download a URL to the system downloads directory; stream progress via events.
#[tauri::command]
pub async fn download_url(
    app: AppHandle,
    id: String,
    url: String,
    filename: Option<String>,
    dm: State<'_, Arc<DownloadManager>>,
) -> Result<String, String> {
    log_downloads_init();

    let downloads_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Downloads dir error: {}", e))?;

    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Create downloads dir failed: {}", e))?;

    let name = filename_from_url(&url, filename);
    let dest: PathBuf = downloads_dir.join(&name);

    let _ = dm.register_download_with_id(&id, &url, &name, dest.clone());

    let mut received: u64 = if dest.exists() {
        std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let mut request = client.get(&url);
    if received > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={}-", received));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        let msg = format!("Download HTTP {}", response.status());
        emit_download_error(&app, &id, &msg);
        let _ = dm.mark_failed(&id, msg.clone());
        return Err(msg);
    }

    let total = response.content_length().unwrap_or(0).saturating_add(received);
    let mut stream = response.bytes_stream();
    let mut file = if received > 0 {
        OpenOptions::new()
            .append(true)
            .open(&dest)
            .map_err(|e| {
                let msg = format!("Open file for resume failed: {}", e);
                emit_download_error(&app, &id, &msg);
                msg
            })?
    } else {
        std::fs::File::create(&dest).map_err(|e| {
            let msg = format!("Create file failed: {}", e);
            emit_download_error(&app, &id, &msg);
            msg
        })?
    };

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| {
            let msg = format!("Download stream error: {}", e);
            emit_download_error(&app, &id, &msg);
            msg
        })?;
        received += bytes.len() as u64;
        file.write_all(&bytes).map_err(|e| {
            let msg = format!("Write file failed: {}", e);
            emit_download_error(&app, &id, &msg);
            msg
        })?;

        let progress = if total > 0 {
            (received as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        let _ = dm.update_progress(&id, received, total, 0);

        let _ = app.emit(
            "exodus-download-progress",
            DownloadProgressPayload {
                id: id.clone(),
                url: url.clone(),
                filename: name.clone(),
                received,
                total,
                progress,
            },
        );
    }

    file.flush()
        .map_err(|e| format!("Flush file failed: {}", e))?;

    let path_str = dest.to_string_lossy().to_string();
    let _ = dm.complete_download_record(&id, received);

    let _ = app.emit(
        "exodus-download-done",
        DownloadDonePayload {
            id,
            path: path_str.clone(),
            filename: name,
        },
    );

    Ok(path_str)
}

#[test]
fn filename_from_url_uses_path() {
    let name = filename_from_url("https://example.com/files/report.pdf", None);
    assert_eq!(name, "report.pdf");
}

#[test]
fn filename_from_url_override() {
    let name = filename_from_url("https://example.com/x", Some("custom.zip".into()));
    assert_eq!(name, "custom.zip");
}

#[test]
fn download_error_payload_serializes_id_and_message() {
    let payload = DownloadErrorPayload {
        id: "abc123".into(),
        message: "Download HTTP 404".into(),
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    assert!(json.contains("\"id\":\"abc123\""));
    assert!(json.contains("\"message\":\"Download HTTP 404\""));
}

/// Open the OS downloads folder in the system file manager.
#[tauri::command]
pub fn open_downloads_folder(app: AppHandle) -> Result<(), String> {
    let downloads_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Downloads dir error: {}", e))?;

    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Create downloads dir failed: {}", e))?;

    tauri_plugin_opener::open_path(&downloads_dir, None::<&str>)
        .map_err(|e| format!("Open downloads folder failed: {}", e))
}

/// Reveal a downloaded file in the system file manager (Finder on macOS).
#[tauri::command]
pub fn reveal_download(path: String) -> Result<(), String> {
    let file = PathBuf::from(path.trim());
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&file)
            .spawn()
            .map_err(|e| format!("reveal_download failed: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(parent) = file.parent() {
            tauri_plugin_opener::open_path(parent, None::<&str>)
                .map_err(|e| format!("reveal_download failed: {}", e))?;
        } else {
            tauri_plugin_opener::open_path(&file, None::<&str>)
                .map_err(|e| format!("reveal_download failed: {}", e))?;
        }
        Ok(())
    }
}

/// Open a completed download with the default application.
#[tauri::command]
pub fn open_download(path: String) -> Result<(), String> {
    let file = PathBuf::from(path.trim());
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()));
    }
    tauri_plugin_opener::open_path(&file, None::<&str>)
        .map_err(|e| format!("open_download failed: {}", e))
}

/// Map persisted download rows for the downloads panel.
#[tauri::command]
pub fn list_persisted_downloads(dm: State<'_, Arc<DownloadManager>>) -> Result<Vec<serde_json::Value>, String> {
    let rows: Vec<serde_json::Value> = dm
        .get_all_downloads()
        .into_iter()
        .map(|d| {
            let progress = if d.total_size > 0 {
                (d.downloaded_bytes as f32 / d.total_size as f32) * 100.0
            } else {
                0.0
            };
            let status = match d.status {
                DownloadStatus::Completed => "completed",
                DownloadStatus::Failed => "failed",
                DownloadStatus::Downloading => "downloading",
                DownloadStatus::Pending => "pending",
                DownloadStatus::Paused => "pending",
                DownloadStatus::Cancelled => "failed",
            };
            serde_json::json!({
                "id": d.id,
                "url": d.url,
                "filename": d.file_name,
                "path": d.file_path.to_string_lossy(),
                "status": status,
                "progress": progress,
                "received": d.downloaded_bytes,
                "total": d.total_size,
            })
        })
        .collect();
    Ok(rows)
}
