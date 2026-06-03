//! Download Manager Improvements for Exodus Browser
//! 
//! This module provides advanced download management capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => DownloadStatus::Pending,
            "downloading" => DownloadStatus::Downloading,
            "paused" => DownloadStatus::Paused,
            "completed" => DownloadStatus::Completed,
            "failed" => DownloadStatus::Failed,
            "cancelled" => DownloadStatus::Cancelled,
            _ => DownloadStatus::Pending,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            DownloadStatus::Pending => "pending",
            DownloadStatus::Downloading => "downloading",
            DownloadStatus::Paused => "paused",
            DownloadStatus::Completed => "completed",
            DownloadStatus::Failed => "failed",
            DownloadStatus::Cancelled => "cancelled",
        }
    }
}

/// Download item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    /// Download ID
    pub id: String,
    /// URL
    pub url: String,
    /// File name
    pub file_name: String,
    /// File path
    pub file_path: PathBuf,
    /// Total size in bytes
    pub total_size: u64,
    /// Downloaded bytes
    pub downloaded_bytes: u64,
    /// Status
    pub status: DownloadStatus,
    /// Download speed in bytes per second
    pub speed: u64,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp
    pub end_time: Option<u64>,
    /// Error message
    pub error_message: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Referrer URL
    pub referrer: Option<String>,
}

impl DownloadItem {
    pub fn new(url: String, file_name: String, file_path: PathBuf) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            file_name,
            file_path,
            total_size: 0,
            downloaded_bytes: 0,
            status: DownloadStatus::Pending,
            speed: 0,
            start_time: now,
            end_time: None,
            error_message: None,
            mime_type: None,
            referrer: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn progress(&self) -> f32 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f32 / self.total_size as f32) * 100.0
        }
    }
    
    pub fn is_complete(&self) -> bool {
        self.status == DownloadStatus::Completed
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, DownloadStatus::Pending | DownloadStatus::Downloading)
    }
}

/// Download settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSettings {
    /// Default download directory
    pub default_directory: PathBuf,
    /// Ask for download location
    pub ask_for_location: bool,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Auto-resume interrupted downloads
    pub auto_resume: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Download speed limit in bytes per second (0 = unlimited)
    pub speed_limit: u64,
    /// Clear completed downloads on exit
    pub clear_completed_on_exit: bool,
    /// Show download notifications
    pub show_notifications: bool,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            default_directory: PathBuf::from("~/Downloads"),
            ask_for_location: false,
            max_concurrent_downloads: 3,
            auto_resume: true,
            max_retry_attempts: 3,
            speed_limit: 0,
            clear_completed_on_exit: false,
            show_notifications: true,
        }
    }
}

/// Download manager
pub struct DownloadManager {
    downloads: Arc<Mutex<HashMap<String, DownloadItem>>>,
    settings: Arc<Mutex<DownloadSettings>>,
    active_downloads: Arc<Mutex<usize>>,
    storage_path: PathBuf,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            downloads: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(DownloadSettings::default())),
            active_downloads: Arc::new(Mutex::new(0)),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Start a download
    pub fn start_download(&self, url: String, file_name: String, file_path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut active = self.active_downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if *active >= settings.max_concurrent_downloads {
            return Err("Maximum concurrent downloads reached".into());
        }
        
        let download = DownloadItem::new(url, file_name, file_path);
        let download_id = download.id.clone();
        
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        downloads.insert(download_id.clone(), download);
        *active += 1;
        
        self.save_to_disk()?;
        Ok(download_id)
    }
    
    /// Pause a download
    pub fn pause_download(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(download) = downloads.get_mut(id) {
            if download.is_active() {
                download.status = DownloadStatus::Paused;
                download.speed = 0;
                
                let mut active = self.active_downloads.lock()
                    .unwrap_or_else(|_| panic!("Lock error"));
                *active = active.saturating_sub(1);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Resume a download
    pub fn resume_download(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut active = self.active_downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if *active >= settings.max_concurrent_downloads {
            return Err("Maximum concurrent downloads reached".into());
        }
        
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(download) = downloads.get_mut(id) {
            if download.status == DownloadStatus::Paused {
                download.status = DownloadStatus::Downloading;
                *active += 1;
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Cancel a download
    pub fn cancel_download(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(download) = downloads.get_mut(id) {
            if download.is_active() {
                download.status = DownloadStatus::Cancelled;
                download.speed = 0;
                
                let mut active = self.active_downloads.lock()
                    .unwrap_or_else(|_| panic!("Lock error"));
                *active = active.saturating_sub(1);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a download
    pub fn remove_download(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(download) = downloads.get(id) {
            if download.is_active() {
                return Err("Cannot remove active download".into());
            }
        }
        
        downloads.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Update download progress
    pub fn update_progress(&self, id: &str, downloaded_bytes: u64, total_size: u64, speed: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(download) = downloads.get_mut(id) {
            download.downloaded_bytes = downloaded_bytes;
            download.total_size = total_size;
            download.speed = speed;
            
            if downloaded_bytes >= total_size && total_size > 0 {
                download.status = DownloadStatus::Completed;
                download.end_time = Some(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
                
                let mut active = self.active_downloads.lock()
                    .unwrap_or_else(|_| panic!("Lock error"));
                *active = active.saturating_sub(1);
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Mark download as failed
    pub fn mark_failed(&self, id: &str, error_message: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(download) = downloads.get_mut(id) {
            download.status = DownloadStatus::Failed;
            download.error_message = Some(error_message);
            download.speed = 0;
            
            let mut active = self.active_downloads.lock()
                .unwrap_or_else(|_| panic!("Lock error"));
            *active = active.saturating_sub(1);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Register or update an active download with a stable frontend id.
    pub fn register_download_with_id(
        &self,
        id: &str,
        url: &str,
        file_name: &str,
        file_path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self
            .downloads
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let mut item = DownloadItem::new(url.to_string(), file_name.to_string(), file_path);
        item.id = id.to_string();
        item.status = DownloadStatus::Downloading;
        downloads.insert(id.to_string(), item);
        drop(downloads);
        self.save_to_disk()?;
        Ok(())
    }

    /// Mark download completed on disk.
    pub fn complete_download_record(
        &self,
        id: &str,
        total_size: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self
            .downloads
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        if let Some(download) = downloads.get_mut(id) {
            download.downloaded_bytes = total_size.max(download.downloaded_bytes);
            download.total_size = total_size.max(download.total_size);
            download.status = DownloadStatus::Completed;
            download.end_time = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }
        drop(downloads);
        self.save_to_disk()?;
        Ok(())
    }

    /// Get download
    pub fn get_download(&self, id: &str) -> Option<DownloadItem> {
        let downloads = self.downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        downloads.get(id).cloned()
    }
    
    /// Get all downloads
    pub fn get_all_downloads(&self) -> Vec<DownloadItem> {
        let downloads = self.downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        downloads.values().cloned().collect()
    }
    
    /// Get active downloads
    pub fn get_active_downloads(&self) -> Vec<DownloadItem> {
        let downloads = self.downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        downloads.values()
            .filter(|d| d.is_active())
            .cloned()
            .collect()
    }
    
    /// Get completed downloads
    pub fn get_completed_downloads(&self) -> Vec<DownloadItem> {
        let downloads = self.downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        downloads.values()
            .filter(|d| d.is_complete())
            .cloned()
            .collect()
    }
    
    /// Clear completed downloads
    pub fn clear_completed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        downloads.retain(|_, d| !d.is_complete());
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Clear all downloads
    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        downloads.clear();
        *self.active_downloads.lock().unwrap() = 0;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get download settings
    pub fn get_settings(&self) -> DownloadSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update download settings
    pub fn update_settings(&self, settings: DownloadSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get download statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let downloads = self.downloads.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total".to_string(), downloads.len() as u64);
        stats.insert("active".to_string(), downloads.values().filter(|d| d.is_active()).count() as u64);
        stats.insert("completed".to_string(), downloads.values().filter(|d| d.is_complete()).count() as u64);
        stats.insert("failed".to_string(), downloads.values().filter(|d| d.status == DownloadStatus::Failed).count() as u64);
        
        let total_bytes: u64 = downloads.values()
            .filter(|d| d.is_complete())
            .map(|d| d.total_size)
            .sum();
        stats.insert("total_bytes_downloaded".to_string(), total_bytes);
        
        stats
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("download_settings.json");
        let downloads_path = self.storage_path.join("downloads.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: DownloadSettings = serde_json::from_str(&content)?;
            *self.settings.lock().unwrap() = settings;
        }
        
        if downloads_path.exists() {
            let content = std::fs::read_to_string(&downloads_path)?;
            let downloads: HashMap<String, DownloadItem> = serde_json::from_str(&content)?;
            *self.downloads.lock().unwrap() = downloads;
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("download_settings.json");
        let downloads_path = self.storage_path.join("downloads.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let downloads = self.downloads.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let downloads_content = serde_json::to_string_pretty(&*downloads)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&downloads_path, downloads_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Start a download
#[tauri::command]
pub fn start_download(
    url: String,
    file_name: String,
    file_path: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<String, String> {
    let path = PathBuf::from(file_path);
    manager.start_download(url, file_name, path)
        .map_err(|e| format!("Failed to start download: {}", e))
}

/// Pause a download
#[tauri::command]
pub fn pause_download(
    id: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.pause_download(&id)
        .map_err(|e| format!("Failed to pause download: {}", e))
}

/// Resume a download
#[tauri::command]
pub fn resume_download(
    id: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.resume_download(&id)
        .map_err(|e| format!("Failed to resume download: {}", e))
}

/// Cancel a download
#[tauri::command]
pub fn cancel_download(
    id: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.cancel_download(&id)
        .map_err(|e| format!("Failed to cancel download: {}", e))
}

/// Remove a download
#[tauri::command]
pub fn remove_download(
    id: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.remove_download(&id)
        .map_err(|e| format!("Failed to remove download: {}", e))
}

/// Update download progress
#[tauri::command]
pub fn update_download_progress(
    id: String,
    downloaded_bytes: u64,
    total_size: u64,
    speed: u64,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.update_progress(&id, downloaded_bytes, total_size, speed)
        .map_err(|e| format!("Failed to update progress: {}", e))
}

/// Mark download as failed
#[tauri::command]
pub fn mark_download_failed(
    id: String,
    error_message: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.mark_failed(&id, error_message)
        .map_err(|e| format!("Failed to mark as failed: {}", e))
}

/// Get download
#[tauri::command]
pub fn get_download(
    id: String,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<Option<DownloadItem>, String> {
    Ok(manager.get_download(&id))
}

/// Get all downloads
#[tauri::command]
pub fn get_all_downloads(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<Vec<DownloadItem>, String> {
    Ok(manager.get_all_downloads())
}

/// Get active downloads
#[tauri::command]
pub fn get_active_downloads(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<Vec<DownloadItem>, String> {
    Ok(manager.get_active_downloads())
}

/// Get completed downloads
#[tauri::command]
pub fn get_completed_downloads(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<Vec<DownloadItem>, String> {
    Ok(manager.get_completed_downloads())
}

/// Clear completed downloads
#[tauri::command]
pub fn clear_completed_downloads(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.clear_completed()
        .map_err(|e| format!("Failed to clear completed downloads: {}", e))
}

/// Clear all downloads
#[tauri::command]
pub fn clear_all_downloads(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.clear_all()
        .map_err(|e| format!("Failed to clear all downloads: {}", e))
}

/// Get download settings
#[tauri::command]
pub fn get_download_settings(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<DownloadSettings, String> {
    Ok(manager.get_settings())
}

/// Update download settings
#[tauri::command]
pub fn update_download_settings(
    settings: DownloadSettings,
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get download statistics
#[tauri::command]
pub fn get_download_stats(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}
