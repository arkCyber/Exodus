//! File Transfer Service — P2P file sharing backed by workspace + CDN blob store.
//!
//! Persists transfer metadata and file bytes under `storage_dir`; integrates with
//! [`crate::exodus_workspace::ExodusWorkSpace`] and P2P CDN seeding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// File transfer metadata (persisted).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTransferMetadata {
    pub transfer_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub blob_hash: String,
    pub chunk_count: u32,
    pub sender_id: String,
    pub receiver_id: Option<String>,
    pub status: String,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub short_code: Option<String>,
    pub retry_count: u32,
    #[serde(default)]
    pub cdn_content_hash: Option<String>,
    #[serde(default)]
    pub cdn_ticket: Option<String>,
    #[serde(default)]
    pub workspace_rel_path: Option<String>,
    #[serde(default = "default_workspace_room")]
    pub room_id: String,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default = "default_direction_upload")]
    pub direction: String,
    #[serde(default)]
    pub bytes_done: u64,
    #[serde(default)]
    pub progress_percent: f32,
    #[serde(default)]
    pub speed_bps: u64,
    #[serde(default)]
    pub checksum_verified: bool,
    #[serde(default)]
    pub last_error: Option<String>,
}

fn default_direction_upload() -> String {
    "upload".to_string()
}

fn default_workspace_room() -> String {
    crate::exodus_workspace::WORKSPACE_ROOM_ID.to_string()
}

/// File chunk descriptor (chunk bytes stored on disk).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChunk {
    pub transfer_id: String,
    pub chunk_index: u32,
    pub chunk_hash: String,
    pub blob_hash: String,
}

/// Configuration for File Transfer Service.
#[derive(Debug, Clone)]
pub struct FileTransferServiceConfig {
    pub storage_dir: PathBuf,
    pub node_id: String,
}

impl FileTransferServiceConfig {
    /// Build config under app data directory.
    pub fn under_app_data(app_data_dir: &Path, node_id: impl Into<String>) -> Self {
        Self {
            storage_dir: app_data_dir.join("file_transfers"),
            node_id: node_id.into(),
        }
    }
}

/// In-process file transfer registry with disk persistence.
pub struct FileTransferService {
    config: FileTransferServiceConfig,
    transfers: Arc<Mutex<HashMap<String, FileTransferMetadata>>>,
    chunks: Arc<Mutex<HashMap<String, Vec<FileChunk>>>>,
}

impl FileTransferService {
    /// Create service and load persisted transfers from disk.
    pub fn new(config: FileTransferServiceConfig) -> Result<Self, String> {
        fs::create_dir_all(&config.storage_dir).map_err(|e| e.to_string())?;
        let svc = Self {
            config,
            transfers: Arc::new(Mutex::new(HashMap::new())),
            chunks: Arc::new(Mutex::new(HashMap::new())),
        };
        svc.load_persisted()?;
        Ok(svc)
    }

    pub fn node_id(&self) -> &str {
        &self.config.node_id
    }

    pub fn storage_dir(&self) -> &Path {
        &self.config.storage_dir
    }

    /// Initiate transfer: copy bytes to storage, chunk on disk, return transfer id.
    pub fn initiate_transfer(
        &self,
        file_path: &Path,
        receiver_id: Option<String>,
        cdn_content_hash: Option<String>,
        cdn_ticket: Option<String>,
        workspace_rel_path: Option<String>,
    ) -> Result<FileTransferMetadata, String> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_data = fs::read(file_path).map_err(|e| format!("Failed to read file: {e}"))?;
        let file_size = file_data.len() as u64;
        let blob_hash = blake3::hash(&file_data).to_hex().to_string();
        let file_type = infer_file_type(&file_name);
        const CHUNK_SIZE: usize = 1024 * 1024;
        let chunk_count = ((file_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE).max(1) as u32;
        let transfer_id = generate_transfer_id();
        let short_code = Some(generate_short_code());
        let transfer_dir = self.transfer_dir(&transfer_id);
        fs::create_dir_all(&transfer_dir).map_err(|e| e.to_string())?;
        let data_path = transfer_dir.join("data.bin");
        fs::write(&data_path, &file_data).map_err(|e| e.to_string())?;
        let mut chunks_vec = Vec::new();
        for (i, chunk) in file_data.chunks(CHUNK_SIZE).enumerate() {
            let chunk_hash = blake3::hash(chunk).to_hex().to_string();
            let chunk_path = transfer_dir.join(format!("chunk_{i}"));
            fs::write(&chunk_path, chunk).map_err(|e| e.to_string())?;
            chunks_vec.push(FileChunk {
                transfer_id: transfer_id.clone(),
                chunk_index: i as u32,
                chunk_hash,
                blob_hash: blob_hash.clone(),
            });
        }
        let cdn_hash = cdn_content_hash.unwrap_or_else(|| blob_hash.clone());
        let metadata = FileTransferMetadata {
            transfer_id: transfer_id.clone(),
            file_name,
            file_size,
            file_type,
            blob_hash,
            chunk_count,
            sender_id: self.config.node_id.clone(),
            receiver_id,
            status: "pending".to_string(),
            created_at: current_timestamp(),
            completed_at: None,
            short_code,
            retry_count: 0,
            cdn_content_hash: Some(cdn_hash),
            cdn_ticket,
            workspace_rel_path,
            room_id: crate::exodus_workspace::WORKSPACE_ROOM_ID.to_string(),
            local_path: Some(data_path.to_string_lossy().to_string()),
            direction: "upload".to_string(),
            bytes_done: 0,
            progress_percent: 0.0,
            speed_bps: 0,
            checksum_verified: false,
            last_error: None,
        };
        {
            let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
            transfers.insert(transfer_id.clone(), metadata.clone());
        }
        {
            let mut chunks = self.chunks.lock().map_err(|e| e.to_string())?;
            chunks.insert(transfer_id, chunks_vec);
        }
        self.persist_all()?;
        Ok(metadata)
    }

    pub fn get_transfer(&self, transfer_id: &str) -> Option<FileTransferMetadata> {
        self.transfers.lock().ok()?.get(transfer_id).cloned()
    }

    pub fn list_transfers(&self) -> Vec<FileTransferMetadata> {
        self.transfers
            .lock()
            .map(|t| t.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_chunks(&self, transfer_id: &str) -> Vec<FileChunk> {
        self.chunks
            .lock()
            .ok()
            .and_then(|c| c.get(transfer_id).cloned())
            .unwrap_or_default()
    }

    /// Read chunk bytes from disk.
    pub fn read_chunk_bytes(&self, transfer_id: &str, chunk_index: u32) -> Result<Vec<u8>, String> {
        let path = self
            .transfer_dir(transfer_id)
            .join(format!("chunk_{chunk_index}"));
        fs::read(&path).map_err(|e| format!("Read chunk: {e}"))
    }

    /// Register a background download job (metadata only; engine fills bytes).
    pub fn register_download(
        &self,
        file_name: String,
        file_size: u64,
        cdn_content_hash: String,
    ) -> Result<FileTransferMetadata, String> {
        let transfer_id = generate_transfer_id();
        let blob_hash = cdn_content_hash.clone();
        let metadata = FileTransferMetadata {
            transfer_id: transfer_id.clone(),
            file_name,
            file_size,
            file_type: "application/octet-stream".to_string(),
            blob_hash,
            chunk_count: ((file_size / (1024 * 1024)) + 1).max(1) as u32,
            sender_id: self.config.node_id.clone(),
            receiver_id: None,
            status: "pending".to_string(),
            created_at: current_timestamp(),
            completed_at: None,
            short_code: Some(generate_short_code()),
            retry_count: 0,
            cdn_content_hash: Some(cdn_content_hash),
            cdn_ticket: None,
            workspace_rel_path: None,
            room_id: crate::exodus_workspace::WORKSPACE_ROOM_ID.to_string(),
            local_path: None,
            direction: "download".to_string(),
            bytes_done: 0,
            progress_percent: 0.0,
            speed_bps: 0,
            checksum_verified: false,
            last_error: None,
        };
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        transfers.insert(transfer_id.clone(), metadata.clone());
        self.persist_all()?;
        Ok(metadata)
    }

    pub fn update_progress(
        &self,
        transfer_id: &str,
        progress_percent: f32,
        bytes_done: u64,
        bytes_total: u64,
    ) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(m) = transfers.get_mut(transfer_id) {
            m.progress_percent = progress_percent;
            m.bytes_done = bytes_done;
            if bytes_total > 0 {
                m.file_size = bytes_total;
            }
        }
        self.persist_all()
    }

    pub fn update_speed(&self, transfer_id: &str, speed_bps: u64) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(m) = transfers.get_mut(transfer_id) {
            m.speed_bps = speed_bps;
        }
        self.persist_all()
    }

    pub fn update_last_error(&self, transfer_id: &str, err: &str) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(m) = transfers.get_mut(transfer_id) {
            m.last_error = Some(err.to_string());
        }
        self.persist_all()
    }

    pub fn set_checksum_verified(&self, transfer_id: &str, verified: bool) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(m) = transfers.get_mut(transfer_id) {
            m.checksum_verified = verified;
            if verified {
                m.status = "completed".to_string();
                m.completed_at = Some(current_timestamp());
                m.progress_percent = 100.0;
            }
        }
        self.persist_all()
    }

    pub fn update_status(&self, transfer_id: &str, status: &str) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(metadata) = transfers.get_mut(transfer_id) {
            metadata.status = status.to_string();
            if status == "completed" {
                metadata.completed_at = Some(current_timestamp());
            }
            if status == "failed" && metadata.retry_count < 3 {
                metadata.retry_count += 1;
                metadata.status = "pending".to_string();
            }
        }
        self.persist_all()
    }

    pub fn retry_transfer(&self, transfer_id: &str) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(metadata) = transfers.get_mut(transfer_id) {
            if metadata.status == "failed" {
                metadata.status = "pending".to_string();
                metadata.retry_count += 1;
            }
        }
        self.persist_all()
    }

    pub fn cancel_transfer(&self, transfer_id: &str) -> Result<(), String> {
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        if let Some(metadata) = transfers.get_mut(transfer_id) {
            metadata.status = "cancelled".to_string();
        }
        self.persist_all()
    }

    pub fn resolve_by_short_code(&self, short_code: &str) -> Option<FileTransferMetadata> {
        let transfers = self.transfers.lock().ok()?;
        transfers
            .values()
            .find(|t| t.short_code.as_deref() == Some(short_code))
            .cloned()
    }

    pub fn generate_qr_data(&self, transfer_id: &str) -> Result<serde_json::Value, String> {
        let transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        let metadata = transfers.get(transfer_id).ok_or("Transfer not found")?;
        Ok(serde_json::json!({
            "transferId": metadata.transfer_id,
            "shortCode": metadata.short_code,
            "fileName": metadata.file_name,
            "fileSize": metadata.file_size,
            "fileType": metadata.file_type,
            "blobHash": metadata.blob_hash,
            "cdnContentHash": metadata.cdn_content_hash,
            "cdnTicket": metadata.cdn_ticket,
            "senderId": metadata.sender_id,
            "roomId": metadata.room_id,
            "expiresAt": metadata.created_at + 3600
        }))
    }

    pub fn resolve_conflict(&self, file_name: &str, existing_files: &[String]) -> String {
        if !existing_files.contains(&file_name.to_string()) {
            return file_name.to_string();
        }
        let (stem, ext) = split_name_ext(file_name);
        let mut counter = 1u32;
        loop {
            let new_name = if ext.is_empty() {
                format!("{stem}_{counter}")
            } else {
                format!("{stem}_{counter}.{ext}")
            };
            if !existing_files.contains(&new_name) {
                return new_name;
            }
            counter += 1;
        }
    }

    fn transfer_dir(&self, transfer_id: &str) -> PathBuf {
        self.config.storage_dir.join(transfer_id)
    }

    fn persist_all(&self) -> Result<(), String> {
        let transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        let list: Vec<_> = transfers.values().cloned().collect();
        let path = self.config.storage_dir.join("transfers.json");
        let raw = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?;
        fs::write(&path, raw).map_err(|e| e.to_string())
    }

    fn load_persisted(&self) -> Result<(), String> {
        let path = self.config.storage_dir.join("transfers.json");
        if !path.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let list: Vec<FileTransferMetadata> =
            serde_json::from_str(&raw).unwrap_or_else(|_| Vec::new());
        let mut transfers = self.transfers.lock().map_err(|e| e.to_string())?;
        for m in list {
            transfers.insert(m.transfer_id.clone(), m);
        }
        Ok(())
    }
}

fn infer_file_type(file_name: &str) -> String {
    let lower = file_name.to_lowercase();
    if lower.ends_with(".txt") {
        "text/plain".to_string()
    } else if lower.ends_with(".pdf") {
        "application/pdf".to_string()
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".to_string()
    } else if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".mp4") {
        "video/mp4".to_string()
    } else if lower.ends_with(".mp3") {
        "audio/mpeg".to_string()
    } else if lower.ends_with(".zip") {
        "application/zip".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

fn split_name_ext(name: &str) -> (String, String) {
    match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), e.to_string()),
        _ => (name.to_string(), String::new()),
    }
}

fn generate_transfer_id() -> String {
    format!(
        "transfer_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")[..16].to_string()
    )
}

fn generate_short_code() -> String {
    use rand::Rng;
    format!("{:06}", rand::thread_rng().gen_range(0..1_000_000))
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initiate_persists_and_reads_chunks() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cfg = FileTransferServiceConfig::under_app_data(tmp.path(), "ft-node");
        let svc = FileTransferService::new(cfg).expect("service");
        let src = tmp.path().join("data.bin");
        fs::write(&src, b"hello exodus file transfer").expect("write");
        let meta = svc
            .initiate_transfer(&src, None, None, None, None)
            .expect("initiate");
        assert_eq!(meta.status, "pending");
        let loaded = FileTransferService::new(FileTransferServiceConfig::under_app_data(
            tmp.path(),
            "ft-node",
        ))
        .expect("reload");
        let got = loaded.get_transfer(&meta.transfer_id).expect("get");
        assert_eq!(got.file_name, meta.file_name);
        let bytes = loaded
            .read_chunk_bytes(&meta.transfer_id, 0)
            .expect("chunk");
        assert_eq!(bytes, b"hello exodus file transfer");
    }

    #[test]
    fn test_infer_file_type() {
        assert_eq!(infer_file_type("test.txt"), "text/plain");
        assert_eq!(infer_file_type("test.pdf"), "application/pdf");
    }
}
