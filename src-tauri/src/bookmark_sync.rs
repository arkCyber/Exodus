//! Bookmark Sync for Exodus Browser
//! 
//! This module provides bookmark synchronization capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    NotSynced,
    Syncing,
    Synced,
    Error,
}

impl SyncStatus {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "not_synced" => SyncStatus::NotSynced,
            "syncing" => SyncStatus::Syncing,
            "synced" => SyncStatus::Synced,
            "error" => SyncStatus::Error,
            _ => SyncStatus::NotSynced,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            SyncStatus::NotSynced => "not_synced",
            SyncStatus::Syncing => "syncing",
            SyncStatus::Synced => "synced",
            SyncStatus::Error => "error",
        }
    }
}

/// Sync bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncBookmark {
    /// Bookmark ID
    pub id: String,
    /// URL
    pub url: String,
    /// Title
    pub title: String,
    /// Parent folder ID
    pub parent_id: Option<String>,
    /// Position
    pub position: u32,
    /// Date added
    pub date_added: u64,
    /// Last modified
    pub last_modified: u64,
    /// Sync status
    pub sync_status: SyncStatus,
    /// Device ID
    pub device_id: String,
}

impl SyncBookmark {
    pub fn new(url: String, title: String, device_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            title,
            parent_id: None,
            position: 0,
            date_added: now,
            last_modified: now,
            sync_status: SyncStatus::NotSynced,
            device_id,
        }
    }
}

/// Sync folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFolder {
    /// Folder ID
    pub id: String,
    /// Title
    pub title: String,
    /// Parent folder ID
    pub parent_id: Option<String>,
    /// Position
    pub position: u32,
    /// Date added
    pub date_added: u64,
    /// Last modified
    pub last_modified: u64,
    /// Sync status
    pub sync_status: SyncStatus,
    /// Device ID
    pub device_id: String,
}

impl SyncFolder {
    pub fn new(title: String, device_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            parent_id: None,
            position: 0,
            date_added: now,
            last_modified: now,
            sync_status: SyncStatus::NotSynced,
            device_id,
        }
    }
}

/// Sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    /// Enable sync
    pub enabled: bool,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Auto-sync on change
    pub auto_sync: bool,
    /// Sync across devices
    pub sync_across_devices: bool,
    /// Conflict resolution strategy
    pub conflict_resolution: String,
    /// Last sync timestamp
    pub last_sync: u64,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            sync_interval: 300, // 5 minutes
            auto_sync: true,
            sync_across_devices: true,
            conflict_resolution: "local_wins".to_string(),
            last_sync: 0,
        }
    }
}

/// Bookmark sync manager
pub struct BookmarkSyncManager {
    bookmarks: Arc<Mutex<HashMap<String, SyncBookmark>>>,
    folders: Arc<Mutex<HashMap<String, SyncFolder>>>,
    settings: Arc<Mutex<SyncSettings>>,
    device_id: String,
    storage_path: PathBuf,
}

impl BookmarkSyncManager {
    /// Create a new bookmark sync manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let device_id = Self::get_or_create_device_id(&storage_path)?;
        
        let manager = Self {
            bookmarks: Arc::new(Mutex::new(HashMap::new())),
            folders: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(SyncSettings::default())),
            device_id,
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Get or create device ID
    fn get_or_create_device_id(storage_path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let device_id_path = storage_path.join("device_id.txt");
        
        if device_id_path.exists() {
            let content = std::fs::read_to_string(&device_id_path)?;
            Ok(content.trim().to_string())
        } else {
            let device_id = uuid::Uuid::new_v4().to_string();
            std::fs::write(&device_id_path, &device_id)?;
            Ok(device_id)
        }
    }
    
    /// Add a bookmark for sync
    pub fn add_bookmark(&self, url: String, title: String) -> Result<String, Box<dyn std::error::Error>> {
        let bookmark = SyncBookmark::new(url, title, self.device_id.clone());
        let bookmark_id = bookmark.id.clone();
        
        let mut bookmarks = self.bookmarks.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        bookmarks.insert(bookmark_id.clone(), bookmark);
        self.save_to_disk()?;
        
        if self.settings.lock()
            .map(|s| s.auto_sync)
            .unwrap_or(false) {
            self.sync()?;
        }
        
        Ok(bookmark_id)
    }
    
    /// Update a bookmark
    pub fn update_bookmark(&self, id: String, url: Option<String>, title: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut bookmarks = self.bookmarks.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(bookmark) = bookmarks.get_mut(&id) {
            if let Some(url) = url {
                bookmark.url = url;
            }
            if let Some(title) = title {
                bookmark.title = title;
            }
            bookmark.last_modified = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            bookmark.sync_status = SyncStatus::NotSynced;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a bookmark
    pub fn remove_bookmark(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut bookmarks = self.bookmarks.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        bookmarks.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add a folder for sync
    pub fn add_folder(&self, title: String) -> Result<String, Box<dyn std::error::Error>> {
        let folder = SyncFolder::new(title, self.device_id.clone());
        let folder_id = folder.id.clone();
        
        let mut folders = self.folders.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        folders.insert(folder_id.clone(), folder);
        self.save_to_disk()?;
        
        if self.settings.lock()
            .map(|s| s.auto_sync)
            .unwrap_or(false) {
            self.sync()?;
        }
        
        Ok(folder_id)
    }
    
    /// Update a folder
    pub fn update_folder(&self, id: String, title: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut folders = self.folders.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(folder) = folders.get_mut(&id) {
            if let Some(title) = title {
                folder.title = title;
            }
            folder.last_modified = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            folder.sync_status = SyncStatus::NotSynced;
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a folder
    pub fn remove_folder(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut folders = self.folders.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        folders.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get all bookmarks
    pub fn get_all_bookmarks(&self) -> Vec<SyncBookmark> {
        let bookmarks = self.bookmarks.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        bookmarks.values().cloned().collect()
    }
    
    /// Get all folders
    pub fn get_all_folders(&self) -> Vec<SyncFolder> {
        let folders = self.folders.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        folders.values().cloned().collect()
    }
    
    /// Sync bookmarks (placeholder implementation)
    pub fn sync(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enabled {
            return Ok(());
        }
        
        // Update sync status
        let mut bookmarks = self.bookmarks.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for bookmark in bookmarks.values_mut() {
            bookmark.sync_status = SyncStatus::Syncing;
        }
        
        // Simulate sync process
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        for bookmark in bookmarks.values_mut() {
            bookmark.sync_status = SyncStatus::Synced;
        }
        
        let mut folders = self.folders.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for folder in folders.values_mut() {
            folder.sync_status = SyncStatus::Synced;
        }
        
        // Update last sync time
        let mut settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        settings.last_sync = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get sync settings
    pub fn get_settings(&self) -> SyncSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update sync settings
    pub fn update_settings(&self, settings: SyncSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get device ID
    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }
    
    /// Get sync statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let bookmarks = self.bookmarks.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        let folders = self.folders.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_bookmarks".to_string(), bookmarks.len() as u64);
        stats.insert("total_folders".to_string(), folders.len() as u64);
        stats.insert("synced_bookmarks".to_string(), bookmarks.values().filter(|b| b.sync_status == SyncStatus::Synced).count() as u64);
        stats.insert("synced_folders".to_string(), folders.values().filter(|f| f.sync_status == SyncStatus::Synced).count() as u64);
        
        stats
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("sync_settings.json");
        let bookmarks_path = self.storage_path.join("sync_bookmarks.json");
        let folders_path = self.storage_path.join("sync_folders.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: SyncSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if bookmarks_path.exists() {
            let content = std::fs::read_to_string(&bookmarks_path)?;
            let bookmarks: HashMap<String, SyncBookmark> = serde_json::from_str(&content)?;
            if let Ok(mut b) = self.bookmarks.lock() {
                *b = bookmarks;
            }
        }
        
        if folders_path.exists() {
            let content = std::fs::read_to_string(&folders_path)?;
            let folders: HashMap<String, SyncFolder> = serde_json::from_str(&content)?;
            if let Ok(mut f) = self.folders.lock() {
                *f = folders;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("sync_settings.json");
        let bookmarks_path = self.storage_path.join("sync_bookmarks.json");
        let folders_path = self.storage_path.join("sync_folders.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let bookmarks = self.bookmarks.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let folders = self.folders.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let bookmarks_content = serde_json::to_string_pretty(&*bookmarks)?;
        let folders_content = serde_json::to_string_pretty(&*folders)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&bookmarks_path, bookmarks_content)?;
        std::fs::write(&folders_path, folders_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add a bookmark for sync
#[tauri::command]
pub fn add_sync_bookmark(
    url: String,
    title: String,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<String, String> {
    manager.add_bookmark(url, title)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

/// Update a bookmark
#[tauri::command]
pub fn update_sync_bookmark(
    id: String,
    url: Option<String>,
    title: Option<String>,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    manager.update_bookmark(id, url, title)
        .map_err(|e| format!("Failed to update bookmark: {}", e))
}

/// Remove a bookmark
#[tauri::command]
pub fn remove_sync_bookmark(
    id: String,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    manager.remove_bookmark(&id)
        .map_err(|e| format!("Failed to remove bookmark: {}", e))
}

/// Add a folder for sync
#[tauri::command]
pub fn add_sync_folder(
    title: String,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<String, String> {
    manager.add_folder(title)
        .map_err(|e| format!("Failed to add folder: {}", e))
}

/// Update a folder
#[tauri::command]
pub fn update_sync_folder(
    id: String,
    title: Option<String>,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    manager.update_folder(id, title)
        .map_err(|e| format!("Failed to update folder: {}", e))
}

/// Remove a folder
#[tauri::command]
pub fn remove_sync_folder(
    id: String,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    manager.remove_folder(&id)
        .map_err(|e| format!("Failed to remove folder: {}", e))
}

/// Get all bookmarks
#[tauri::command]
pub fn get_all_sync_bookmarks(
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<Vec<SyncBookmark>, String> {
    Ok(manager.get_all_bookmarks())
}

/// Get all folders
#[tauri::command]
pub fn get_all_sync_folders(
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<Vec<SyncFolder>, String> {
    Ok(manager.get_all_folders())
}

/// Sync bookmarks
#[tauri::command]
pub fn sync_bookmarks(
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    manager.sync()
        .map_err(|e| format!("Failed to sync bookmarks: {}", e))
}

/// Get sync settings
#[tauri::command]
pub fn get_sync_settings(
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<SyncSettings, String> {
    Ok(manager.get_settings())
}

/// Update sync settings
#[tauri::command]
pub fn update_sync_settings(
    settings: SyncSettings,
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get device ID
#[tauri::command]
pub fn get_device_id(
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<String, String> {
    Ok(manager.get_device_id())
}

/// Get sync statistics
#[tauri::command]
pub fn get_sync_stats(
    manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}
