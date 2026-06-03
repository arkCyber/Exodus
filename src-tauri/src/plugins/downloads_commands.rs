//! Chrome Extension API - chrome.downloads
//!
//! Provides Tauri commands for the chrome.downloads API, allowing extensions
//! to interact with the browser's download manager.

use crate::download_manager::{DownloadItem, DownloadManager, DownloadStatus};
use crate::plugins::manager::ExtensionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Download query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadQuery {
    /// Limit the number of results
    pub limit: Option<u32>,
    /// Search query
    pub query: Option<DownloadQueryOptions>,
}

/// Download query options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadQueryOptions {
    /// URL to search for
    pub url: Option<String>,
    /// Filename to search for
    pub filename: Option<String>,
    /// File size range
    pub file_size: Option<FileSizeRange>,
    /// Start time (timestamp in ms)
    pub started_after: Option<u64>,
    /// End time (timestamp in ms)
    pub ended_before: Option<u64>,
}

/// File size range
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSizeRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Download item for chrome.downloads API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItemExt {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub danger: String,
    pub mime: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub bytes_received: u64,
    pub total_bytes: Option<u64>,
    pub file_size: u64,
    pub exists: bool,
    pub state: String,
    pub paused: bool,
    pub can_resume: bool,
    pub error: Option<String>,
}

impl From<DownloadItem> for DownloadItemExt {
    fn from(item: DownloadItem) -> Self {
        Self {
            id: item.id,
            url: item.url,
            filename: item.file_name,
            danger: "safe".to_string(), // Simplified
            mime: item.mime_type.unwrap_or_default(),
            start_time: item.start_time * 1000, // Convert to ms
            end_time: item.end_time.map(|t| t * 1000),
            bytes_received: item.downloaded_bytes,
            total_bytes: Some(item.total_size),
            file_size: item.total_size,
            exists: item.status == DownloadStatus::Completed, // Simplified
            state: item.status.as_str().to_string(),
            paused: item.status == DownloadStatus::Paused,
            can_resume: item.status == DownloadStatus::Paused,
            error: item.error_message,
        }
    }
}

/// Download options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    pub url: String,
    pub filename: Option<String>,
    pub save_as: Option<bool>,
    pub conflict_action: Option<String>,
    pub method: Option<String>,
    pub headers: Option<Vec<Header>>,
}

/// Download header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub name: String,
    pub value: String,
}

/// Search downloads
#[tauri::command]
pub async fn chrome_downloads_search(
    query: DownloadQuery,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<Vec<DownloadItemExt>, String> {
    // Search downloads from download manager
    let downloads = download_manager.get_all_downloads();
    
    // Apply query filters if provided
    let filtered = if let Some(query_options) = query.query {
        downloads.into_iter()
            .filter(|item| {
                if let Some(ref url) = query_options.url {
                    if !item.url.contains(url) {
                        return false;
                    }
                }
                if let Some(ref filename) = query_options.filename {
                    if !item.file_name.contains(filename) {
                        return false;
                    }
                }
                if let Some(ref size_range) = query_options.file_size {
                    if let Some(min) = size_range.min {
                        if item.total_size < min {
                            return false;
                        }
                    }
                    if let Some(max) = size_range.max {
                        if item.total_size > max {
                            return false;
                        }
                    }
                }
                true
            })
            .collect()
    } else {
        downloads
    };
    
    // Apply limit if provided
    let limited = if let Some(limit) = query.limit {
        filtered.into_iter().take(limit as usize).collect()
    } else {
        filtered
    };
    
    // Convert to DownloadItemExt format
    let download_items: Vec<DownloadItemExt> = limited.into_iter().map(Into::into).collect();
    
    Ok(download_items)
}

/// Pause a download
#[tauri::command]
pub async fn chrome_downloads_pause(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Pause a download
    download_manager
        .pause_download(&download_id)
        .map_err(|e| format!("Failed to pause download: {}", e))
}

/// Resume a download
#[tauri::command]
pub async fn chrome_downloads_resume(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Resume a download
    download_manager
        .resume_download(&download_id)
        .map_err(|e| format!("Failed to resume download: {}", e))
}

/// Cancel a download
#[tauri::command]
pub async fn chrome_downloads_cancel(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Cancel a download
    download_manager
        .cancel_download(&download_id)
        .map_err(|e| format!("Failed to cancel download: {}", e))
}

/// Get download by ID
#[tauri::command]
pub async fn chrome_downloads_get_item(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<Option<DownloadItemExt>, String> {
    // Get download item by ID
    let download = download_manager.get_download(&download_id);
    Ok(download.map(Into::into))
}

/// Remove a download
#[tauri::command]
pub async fn chrome_downloads_remove_file(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Remove a downloaded file
    download_manager
        .remove_download(&download_id)
        .map_err(|e| format!("Failed to remove download: {}", e))
}

/// Erase download history
#[tauri::command]
pub async fn chrome_downloads_erase(
    query: DownloadQuery,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<Vec<String>, String> {
    // Erase download history matching query
    let downloads = download_manager.get_all_downloads();
    
    // Apply query filters if provided
    let filtered = if let Some(query_options) = query.query {
        downloads.into_iter()
            .filter(|item| {
                if let Some(ref url) = query_options.url {
                    if !item.url.contains(url) {
                        return false;
                    }
                }
                if let Some(ref filename) = query_options.filename {
                    if !item.file_name.contains(filename) {
                        return false;
                    }
                }
                true
            })
            .collect()
    } else {
        downloads
    };
    
    // Collect IDs to erase
    let ids_to_erase: Vec<String> = filtered.iter().map(|item| item.id.clone()).collect();
    
    // Remove each download
    for id in &ids_to_erase {
        let _ = download_manager.remove_download(id);
    }
    
    Ok(ids_to_erase)
}

/// Open a downloaded file
#[tauri::command]
pub async fn chrome_downloads_open(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Open a downloaded file
    // Get the download item to find the file path
    if let Some(download) = download_manager.get_download(&download_id) {
        // Use system open command
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&download.file_path)
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", "", &download.file_path.to_string_lossy().to_string()])
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&download.file_path)
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
        }
        Ok(())
    } else {
        Err("Download not found".to_string())
    }
}

/// Show a downloaded file in file manager
#[tauri::command]
pub async fn chrome_downloads_show(
    download_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Show a downloaded file in file manager
    // Get the download item to find the file path
    if let Some(download) = download_manager.get_download(&download_id) {
        // Use system show in folder command
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .args(&["-R", &download.file_path.to_string_lossy().to_string()])
                .spawn()
                .map_err(|e| format!("Failed to show file: {}", e))?;
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .args(&["/select,", &download.file_path.to_string_lossy().to_string()])
                .spawn()
                .map_err(|e| format!("Failed to show file: {}", e))?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(download.file_path.parent().unwrap_or(&download.file_path))
                .spawn()
                .map_err(|e| format!("Failed to show file: {}", e))?;
        }
        Ok(())
    } else {
        Err("Download not found".to_string())
    }
}

/// Show default downloads folder
#[tauri::command]
pub async fn chrome_downloads_show_default_folder(
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Show default downloads folder
    // Get download settings to find the default directory
    let settings = download_manager.get_settings();
    let default_dir = &settings.default_directory;
    
    // Use system show in folder command
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(default_dir)
            .spawn()
            .map_err(|e| format!("Failed to show folder: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(default_dir)
            .spawn()
            .map_err(|e| format!("Failed to show folder: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(default_dir)
            .spawn()
            .map_err(|e| format!("Failed to show folder: {}", e))?;
    }
    Ok(())
}

/// Set download options
#[tauri::command]
pub async fn chrome_downloads_set_ui_options(
    options: serde_json::Value,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    download_manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    // Set download UI options
    // For now, this is a no-op as the download manager doesn't have UI options
    // In a full implementation, this would configure download behavior
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_query_deserialization() {
        let query_json = r#"{"limit":10,"query":{"url":"https://example.com"}}"#;
        let query: DownloadQuery = serde_json::from_str(query_json).unwrap();
        assert_eq!(query.limit, Some(10));
        assert!(query.query.is_some());
    }

    #[test]
    fn test_download_options_deserialization() {
        let options_json = r#"{"url":"https://example.com","filename":"test.pdf"}"#;
        let options: DownloadOptions = serde_json::from_str(options_json).unwrap();
        assert_eq!(options.url, "https://example.com");
        assert_eq!(options.filename, Some("test.pdf".to_string()));
    }

    #[test]
    fn test_download_item_ext_serialization() {
        let item = DownloadItemExt {
            id: "test-id".to_string(),
            url: "https://example.com/file.pdf".to_string(),
            filename: "file.pdf".to_string(),
            danger: "safe".to_string(),
            mime: "application/pdf".to_string(),
            start_time: 1234567890,
            end_time: Some(1234567900),
            bytes_received: 1024,
            total_bytes: Some(2048),
            file_size: 2048,
            exists: true,
            paused: false,
            state: "complete".to_string(),
            can_resume: false,
            error: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("file.pdf"));
    }

    #[test]
    fn test_download_query_with_all_fields() {
        let query = DownloadQuery {
            limit: Some(50),
            query: Some(DownloadQueryOptions {
                url: Some("https://example.com".to_string()),
                filename: Some("test.pdf".to_string()),
                file_size: None,
                started_after: Some(1234567890),
                ended_before: Some(1234567900),
            }),
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_download_options_all_fields() {
        let options = DownloadOptions {
            url: "https://example.com/file.zip".to_string(),
            filename: Some("download.zip".to_string()),
            save_as: Some(true),
            conflict_action: Some("uniquify".to_string()),
            method: Some("POST".to_string()),
            headers: None,
        };
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("download.zip"));
        assert!(json.contains("uniquify"));
    }

    #[test]
    fn test_download_item_incomplete() {
        let item = DownloadItemExt {
            id: "incomplete-id".to_string(),
            url: "https://example.com/large.zip".to_string(),
            filename: "large.zip".to_string(),
            danger: "safe".to_string(),
            mime: "application/zip".to_string(),
            start_time: 1234567890,
            end_time: None,
            bytes_received: 512000,
            total_bytes: Some(1024000),
            file_size: 512000,
            exists: true,
            paused: true,
            state: "in_progress".to_string(),
            can_resume: true,
            error: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("in_progress"));
        assert!(json.contains("paused"));
    }

    #[test]
    fn test_download_item_with_error() {
        let item = DownloadItemExt {
            id: "error-id".to_string(),
            url: "https://example.com/missing.pdf".to_string(),
            filename: "missing.pdf".to_string(),
            danger: "safe".to_string(),
            mime: "application/pdf".to_string(),
            start_time: 1234567890,
            end_time: Some(1234567900),
            bytes_received: 0,
            total_bytes: Some(1024),
            file_size: 0,
            exists: false,
            paused: false,
            state: "interrupted".to_string(),
            can_resume: false,
            error: Some("Network error".to_string()),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("interrupted"));
        assert!(json.contains("Network error"));
    }
}
