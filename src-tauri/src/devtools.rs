//! Basic DevTools Integration for Exodus Browser
//! 
//! This module provides basic developer tools integration.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;

/// DevTools panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DevToolsPanel {
    Elements,
    Console,
    Sources,
    Network,
    Performance,
    Memory,
    Application,
    Security,
    Audits,
}

impl DevToolsPanel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "elements" => DevToolsPanel::Elements,
            "console" => DevToolsPanel::Console,
            "sources" => DevToolsPanel::Sources,
            "network" => DevToolsPanel::Network,
            "performance" => DevToolsPanel::Performance,
            "memory" => DevToolsPanel::Memory,
            "application" => DevToolsPanel::Application,
            "security" => DevToolsPanel::Security,
            "audits" => DevToolsPanel::Audits,
            _ => DevToolsPanel::Console,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            DevToolsPanel::Elements => "elements",
            DevToolsPanel::Console => "console",
            DevToolsPanel::Sources => "sources",
            DevToolsPanel::Network => "network",
            DevToolsPanel::Performance => "performance",
            DevToolsPanel::Memory => "memory",
            DevToolsPanel::Application => "application",
            DevToolsPanel::Security => "security",
            DevToolsPanel::Audits => "audits",
        }
    }
}

/// Console log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLogEntry {
    /// Log level
    pub level: String,
    /// Message
    pub message: String,
    /// Source URL
    pub source: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
    /// Timestamp
    pub timestamp: u64,
}

impl ConsoleLogEntry {
    pub fn new(level: String, message: String, source: String, line: u32, column: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            level,
            message,
            source,
            line,
            column,
            timestamp: now,
        }
    }
}

/// Network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Request ID
    pub id: String,
    /// URL
    pub url: String,
    /// Method
    pub method: String,
    /// Status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Request headers
    pub request_headers: HashMap<String, String>,
    /// Response headers
    pub response_headers: HashMap<String, String>,
    /// Request body
    pub request_body: Option<String>,
    /// Response body
    pub response_body: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Duration in milliseconds
    pub duration: u64,
}

impl NetworkRequest {
    pub fn new(url: String, method: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            method,
            status: 0,
            status_text: String::new(),
            request_headers: HashMap::new(),
            response_headers: HashMap::new(),
            request_body: None,
            response_body: None,
            timestamp: now,
            duration: 0,
        }
    }
}

/// DevTools settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsSettings {
    /// Enable DevTools
    pub enabled: bool,
    /// Open automatically on error
    pub open_on_error: bool,
    /// Preserve console logs
    pub preserve_logs: bool,
    /// Disable JavaScript
    pub disable_javascript: bool,
    /// Disable cache
    pub disable_cache: bool,
    /// Emulate device
    pub emulate_device: bool,
    /// Device type
    pub device_type: String,
}

impl Default for DevToolsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            open_on_error: false,
            preserve_logs: true,
            disable_javascript: false,
            disable_cache: false,
            emulate_device: false,
            device_type: "desktop".to_string(),
        }
    }
}

/// DevTools manager
pub struct DevToolsManager {
    settings: Arc<Mutex<DevToolsSettings>>,
    console_logs: Arc<Mutex<Vec<ConsoleLogEntry>>>,
    network_requests: Arc<Mutex<Vec<NetworkRequest>>>,
    open_panels: Arc<Mutex<HashSet<DevToolsPanel>>>,
    storage_path: PathBuf,
}

impl DevToolsManager {
    /// Create a new DevTools manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(DevToolsSettings::default())),
            console_logs: Arc::new(Mutex::new(Vec::new())),
            network_requests: Arc::new(Mutex::new(Vec::new())),
            open_panels: Arc::new(Mutex::new(HashSet::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Add console log
    pub fn add_console_log(&self, entry: ConsoleLogEntry) {
        if let Ok(mut logs) = self.console_logs.lock() {
            logs.push(entry);
            
            // Keep only last 1000 logs
            let len = logs.len();
            if len > 1000 {
                logs.drain(0..len - 1000);
            }
        }
    }
    
    /// Get console logs
    pub fn get_console_logs(&self) -> Vec<ConsoleLogEntry> {
        self.console_logs.lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }
    
    /// Clear console logs
    pub fn clear_console_logs(&self) {
        if let Ok(mut logs) = self.console_logs.lock() {
            logs.clear();
        }
    }
    
    /// Add network request
    pub fn add_network_request(&self, request: NetworkRequest) {
        if let Ok(mut requests) = self.network_requests.lock() {
            requests.push(request);
            
            // Keep only last 500 requests
            let len = requests.len();
            if len > 500 {
                requests.drain(0..len - 500);
            }
        }
    }
    
    /// Get network requests
    pub fn get_network_requests(&self) -> Vec<NetworkRequest> {
        self.network_requests.lock()
            .map(|requests| requests.clone())
            .unwrap_or_default()
    }
    
    /// Clear network requests
    pub fn clear_network_requests(&self) {
        if let Ok(mut requests) = self.network_requests.lock() {
            requests.clear();
        }
    }
    
    /// Open panel
    pub fn open_panel(&self, panel: DevToolsPanel) {
        if let Ok(mut panels) = self.open_panels.lock() {
            panels.insert(panel);
        }
    }
    
    /// Close panel
    pub fn close_panel(&self, panel: DevToolsPanel) {
        if let Ok(mut panels) = self.open_panels.lock() {
            panels.remove(&panel);
        }
    }
    
    /// Get open panels
    pub fn get_open_panels(&self) -> Vec<DevToolsPanel> {
        self.open_panels.lock()
            .map(|panels| panels.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get DevTools settings
    pub fn get_settings(&self) -> DevToolsSettings {
        self.settings.lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }
    
    /// Update DevTools settings
    pub fn update_settings(&self, settings: DevToolsSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let logs = self.console_logs.lock()
            .map(|logs| logs.len() as u64)
            .unwrap_or(0);
        let requests = self.network_requests.lock()
            .map(|requests| requests.len() as u64)
            .unwrap_or(0);
        let panels = self.open_panels.lock()
            .map(|panels| panels.len() as u64)
            .unwrap_or(0);
        
        let mut stats = HashMap::new();
        stats.insert("console_logs".to_string(), logs);
        stats.insert("network_requests".to_string(), requests);
        stats.insert("open_panels".to_string(), panels);
        
        stats
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("devtools_settings.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: DevToolsSettings = serde_json::from_str(&content)?;
            *self.settings.lock().map_err(|e| format!("Lock error: {}", e))? = settings;
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("devtools_settings.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add console log
#[tauri::command]
pub fn add_console_log(
    level: String,
    message: String,
    source: String,
    line: u32,
    column: u32,
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    let entry = ConsoleLogEntry::new(level, message, source, line, column);
    manager.add_console_log(entry);
    Ok(())
}

/// Get console logs
#[tauri::command]
pub fn get_console_logs(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<Vec<ConsoleLogEntry>, String> {
    Ok(manager.get_console_logs())
}

/// Clear console logs
#[tauri::command]
pub fn clear_console_logs(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    manager.clear_console_logs();
    Ok(())
}

/// Add network request
#[tauri::command]
pub fn add_network_request(
    url: String,
    method: String,
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    let request = NetworkRequest::new(url, method);
    manager.add_network_request(request);
    Ok(())
}

/// Get network requests
#[tauri::command]
pub fn get_network_requests(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<Vec<NetworkRequest>, String> {
    Ok(manager.get_network_requests())
}

/// Clear network requests
#[tauri::command]
pub fn clear_network_requests(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    manager.clear_network_requests();
    Ok(())
}

/// Open DevTools panel
#[tauri::command]
pub fn open_devtools_panel(
    panel: String,
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    let p = DevToolsPanel::from_str(&panel);
    manager.open_panel(p);
    Ok(())
}

/// Close DevTools panel
#[tauri::command]
pub fn close_devtools_panel(
    panel: String,
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    let p = DevToolsPanel::from_str(&panel);
    manager.close_panel(p);
    Ok(())
}

/// Get open DevTools panels
#[tauri::command]
pub fn get_open_devtools_panels(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<Vec<String>, String> {
    let panels = manager.get_open_panels();
    Ok(panels.iter().map(|p| p.as_str().to_string()).collect())
}

/// Get DevTools settings
#[tauri::command]
pub fn get_devtools_settings(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<DevToolsSettings, String> {
    Ok(manager.get_settings())
}

/// Update DevTools settings
#[tauri::command]
pub fn update_devtools_settings(
    settings: DevToolsSettings,
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get DevTools statistics
#[tauri::command]
pub fn get_devtools_stats(
    manager: State<'_, Arc<DevToolsManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_stats())
}
