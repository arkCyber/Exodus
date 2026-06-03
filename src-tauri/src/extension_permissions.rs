//! Extension Permissions System for Exodus Browser
//! 
//! This module provides fine-grained permission control for extensions.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Permission type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionType {
    /// Access to tabs
    Tabs,
    /// Access to bookmarks
    Bookmarks,
    /// Access to history
    History,
    /// Access to cookies
    Cookies,
    /// Access to storage
    Storage,
    /// Access to notifications
    Notifications,
    /// Access to web requests
    WebRequest,
    /// Access to web navigation
    WebNavigation,
    /// Access to runtime
    Runtime,
    /// Access to active tab
    ActiveTab,
    /// Access to background
    Background,
    /// Access to devtools
    DevTools,
    /// Access to geolocation
    Geolocation,
    /// Access to camera
    Camera,
    /// Access to microphone
    Microphone,
    /// Access to clipboard
    Clipboard,
}

impl PermissionType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tabs" => PermissionType::Tabs,
            "bookmarks" => PermissionType::Bookmarks,
            "history" => PermissionType::History,
            "cookies" => PermissionType::Cookies,
            "storage" => PermissionType::Storage,
            "notifications" => PermissionType::Notifications,
            "webrequest" => PermissionType::WebRequest,
            "webnavigation" => PermissionType::WebNavigation,
            "runtime" => PermissionType::Runtime,
            "activetab" => PermissionType::ActiveTab,
            "background" => PermissionType::Background,
            "devtools" => PermissionType::DevTools,
            "geolocation" => PermissionType::Geolocation,
            "camera" => PermissionType::Camera,
            "microphone" => PermissionType::Microphone,
            "clipboard" => PermissionType::Clipboard,
            _ => PermissionType::Storage,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            PermissionType::Tabs => "tabs",
            PermissionType::Bookmarks => "bookmarks",
            PermissionType::History => "history",
            PermissionType::Cookies => "cookies",
            PermissionType::Storage => "storage",
            PermissionType::Notifications => "notifications",
            PermissionType::WebRequest => "webRequest",
            PermissionType::WebNavigation => "webNavigation",
            PermissionType::Runtime => "runtime",
            PermissionType::ActiveTab => "activeTab",
            PermissionType::Background => "background",
            PermissionType::DevTools => "devtools",
            PermissionType::Geolocation => "geolocation",
            PermissionType::Camera => "camera",
            PermissionType::Microphone => "microphone",
            PermissionType::Clipboard => "clipboard",
        }
    }
}

/// Permission grant
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// Extension ID
    pub extension_id: String,
    /// Permission type
    pub permission: PermissionType,
    /// Granted at
    pub granted_at: u64,
    /// Origin (if applicable)
    pub origin: Option<String>,
}

impl PermissionGrant {
    #[allow(dead_code)]
    pub fn new(extension_id: String, permission: PermissionType, origin: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            extension_id,
            permission,
            granted_at: now,
            origin,
        }
    }
}

/// Permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Extension ID
    pub extension_id: String,
    /// Permission type
    pub permission: PermissionType,
    /// Origin (if applicable)
    pub origin: Option<String>,
    /// Requested at
    pub requested_at: u64,
    /// Status
    pub status: PermissionRequestStatus,
}

/// Permission request status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionRequestStatus {
    Pending,
    Granted,
    Denied,
}

impl PermissionRequest {
    pub fn new(extension_id: String, permission: PermissionType, origin: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            extension_id,
            permission,
            origin,
            requested_at: now,
            status: PermissionRequestStatus::Pending,
        }
    }
}

/// Extension permissions manager
pub struct ExtensionPermissionsManager {
    granted_permissions: Arc<Mutex<HashMap<String, HashSet<PermissionType>>>>,
    permission_requests: Arc<Mutex<Vec<PermissionRequest>>>,
    host_permissions: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    storage_path: PathBuf,
}

impl ExtensionPermissionsManager {
    /// Create a new extension permissions manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            granted_permissions: Arc::new(Mutex::new(HashMap::new())),
            permission_requests: Arc::new(Mutex::new(Vec::new())),
            host_permissions: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Request permission
    pub fn request_permission(&self, extension_id: String, permission: PermissionType, origin: Option<String>) -> Result<PermissionRequest, Box<dyn std::error::Error>> {
        let request = PermissionRequest::new(extension_id.clone(), permission, origin);
        
        let mut requests = self.permission_requests.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        requests.push(request.clone());
        self.save_to_disk()?;
        
        Ok(request)
    }
    
    /// Grant permission
    pub fn grant_permission(&self, extension_id: String, permission: PermissionType) -> Result<(), Box<dyn std::error::Error>> {
        let mut granted = self.granted_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        granted.entry(extension_id.clone())
            .or_insert_with(HashSet::new)
            .insert(permission.clone());
        
        // Update request status
        let mut requests = self.permission_requests.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for request in requests.iter_mut() {
            if request.extension_id == extension_id && request.permission == permission {
                request.status = PermissionRequestStatus::Granted;
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Deny permission
    pub fn deny_permission(&self, extension_id: String, permission: PermissionType) -> Result<(), Box<dyn std::error::Error>> {
        let mut granted = self.granted_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(permissions) = granted.get_mut(&extension_id) {
            permissions.remove(&permission);
        }
        
        // Update request status
        let mut requests = self.permission_requests.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for request in requests.iter_mut() {
            if request.extension_id == extension_id && request.permission == permission {
                request.status = PermissionRequestStatus::Denied;
            }
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Check if extension has permission
    pub fn has_permission(&self, extension_id: &str, permission: PermissionType) -> bool {
        let granted = self.granted_permissions.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(permissions) = granted.get(extension_id) {
            permissions.contains(&permission)
        } else {
            false
        }
    }
    
    /// Get all permissions for extension
    pub fn get_permissions(&self, extension_id: &str) -> HashSet<PermissionType> {
        let granted = self.granted_permissions.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        granted.get(extension_id).cloned().unwrap_or_default()
    }
    
    /// Revoke all permissions for extension
    pub fn revoke_all_permissions(&self, extension_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut granted = self.granted_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        granted.remove(extension_id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add host permission
    pub fn add_host_permission(&self, extension_id: String, host: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut host_perms = self.host_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        host_perms.entry(extension_id)
            .or_insert_with(HashSet::new)
            .insert(host);
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove host permission
    pub fn remove_host_permission(&self, extension_id: String, host: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut host_perms = self.host_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(hosts) = host_perms.get_mut(&extension_id) {
            hosts.remove(&host);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Check host permission
    pub fn has_host_permission(&self, extension_id: &str, host: &str) -> bool {
        let host_perms = self.host_permissions.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(hosts) = host_perms.get(extension_id) {
            hosts.contains(host) || hosts.iter().any(|h| host.contains(h) || h.contains(host))
        } else {
            false
        }
    }
    
    /// Get host permissions for extension
    pub fn get_host_permissions(&self, extension_id: &str) -> HashSet<String> {
        let host_perms = self.host_permissions.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        host_perms.get(extension_id).cloned().unwrap_or_default()
    }
    
    /// Get pending permission requests
    pub fn get_pending_requests(&self) -> Vec<PermissionRequest> {
        let requests = self.permission_requests.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        requests.iter()
            .filter(|r| r.status == PermissionRequestStatus::Pending)
            .cloned()
            .collect()
    }
    
    /// Get all permission requests
    pub fn get_all_requests(&self) -> Vec<PermissionRequest> {
        let requests = self.permission_requests.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        requests.clone()
    }
    
    /// Get all extensions with permissions
    pub fn get_all_extensions(&self) -> Vec<String> {
        let granted = self.granted_permissions.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        granted.keys().cloned().collect()
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("extension_permissions.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let data: (HashMap<String, HashSet<PermissionType>>, HashMap<String, HashSet<String>>) = serde_json::from_str(&content)?;
            if let Ok(mut gp) = self.granted_permissions.lock() {
                *gp = data.0;
            }
            if let Ok(mut hp) = self.host_permissions.lock() {
                *hp = data.1;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("extension_permissions.json");
        
        let granted = self.granted_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let hosts = self.host_permissions.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&(&*granted, &*hosts))?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Request permission
#[tauri::command]
pub fn request_permission(
    extension_id: String,
    permission: String,
    origin: Option<String>,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<PermissionRequest, String> {
    let perm = PermissionType::from_str(&permission);
    manager.request_permission(extension_id, perm, origin)
        .map_err(|e| format!("Failed to request permission: {}", e))
}

/// Grant permission
#[tauri::command]
pub fn grant_permission(
    extension_id: String,
    permission: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<(), String> {
    let perm = PermissionType::from_str(&permission);
    manager.grant_permission(extension_id, perm)
        .map_err(|e| format!("Failed to grant permission: {}", e))
}

/// Deny permission
#[tauri::command]
pub fn deny_permission(
    extension_id: String,
    permission: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<(), String> {
    let perm = PermissionType::from_str(&permission);
    manager.deny_permission(extension_id, perm)
        .map_err(|e| format!("Failed to deny permission: {}", e))
}

/// Check if extension has permission
#[tauri::command]
pub fn check_permission(
    extension_id: String,
    permission: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<bool, String> {
    let perm = PermissionType::from_str(&permission);
    Ok(manager.has_permission(&extension_id, perm))
}

/// Get all permissions for extension
#[tauri::command]
pub fn get_extension_permissions(
    extension_id: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<Vec<String>, String> {
    let perms = manager.get_permissions(&extension_id);
    Ok(perms.iter().map(|p| p.as_str().to_string()).collect())
}

/// Revoke all permissions for extension
#[tauri::command]
pub fn revoke_all_permissions(
    extension_id: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<(), String> {
    manager.revoke_all_permissions(&extension_id)
        .map_err(|e| format!("Failed to revoke permissions: {}", e))
}

/// Add host permission
#[tauri::command]
pub fn add_host_permission(
    extension_id: String,
    host: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<(), String> {
    manager.add_host_permission(extension_id, host)
        .map_err(|e| format!("Failed to add host permission: {}", e))
}

/// Remove host permission
#[tauri::command]
pub fn remove_host_permission(
    extension_id: String,
    host: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<(), String> {
    manager.remove_host_permission(extension_id, host)
        .map_err(|e| format!("Failed to remove host permission: {}", e))
}

/// Check host permission
#[tauri::command]
pub fn check_host_permission(
    extension_id: String,
    host: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<bool, String> {
    Ok(manager.has_host_permission(&extension_id, &host))
}

/// Get host permissions for extension
#[tauri::command]
pub fn get_host_permissions(
    extension_id: String,
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_host_permissions(&extension_id).into_iter().collect())
}

/// Get pending permission requests
#[tauri::command]
pub fn get_pending_permission_requests(
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<Vec<PermissionRequest>, String> {
    Ok(manager.get_pending_requests())
}

/// Get all permission requests
#[tauri::command]
pub fn get_all_permission_requests(
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<Vec<PermissionRequest>, String> {
    Ok(manager.get_all_requests())
}

/// Get all extensions with permissions
#[tauri::command]
pub fn get_all_extensions_with_permissions(
    manager: State<'_, Arc<ExtensionPermissionsManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_all_extensions())
}
