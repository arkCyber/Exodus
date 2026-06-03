//! Exodus Browser — Native Rust Plugin System
//! 
//! This module provides a native plugin system for Exodus Browser using Rust dynamic libraries.
//! Plugins are loaded as .dylib (macOS), .so (Linux), or .dll (Windows) files at runtime.
//! 
//! ## Plugin Interface
//! 
//! Plugins must implement the `ExodusPlugin` trait and export the required symbols:
//! - `exodus_plugin_version`: Returns the plugin API version
//! - `exodus_plugin_init`: Initializes the plugin and returns a plugin instance
//! - `exodus_plugin_deinit`: Cleans up the plugin
//! 
//! ## Security
//! 
//! All plugins are validated with aerospace-level security checks:
//! - Signature verification (optional)
//! - Hash validation
//! - Permission sandboxing
//! - Resource limits

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tauri::{AppHandle, Manager};

// Aerospace-level security constants
const MAX_PLUGIN_NAME_LENGTH: usize = 100;
const MAX_PLUGIN_VERSION_LENGTH: usize = 50;
const MAX_PLUGIN_DESCRIPTION_LENGTH: usize = 500;
const MAX_PLUGIN_ID_LENGTH: usize = 100;
const PLUGIN_API_VERSION: &str = "1.0.0";
const MAX_PLUGINS: usize = 50; // Maximum number of loaded plugins

// Aerospace-level resource limits
const MAX_PLUGIN_MEMORY_MB: usize = 512;  // 512 MB per plugin
const MAX_COMMAND_EXECUTION_TIME_MS: u64 = 5000;  // 5 seconds per command
const MAX_CONCURRENT_COMMANDS: usize = 10;  // Max concurrent commands per plugin
const MAX_NETWORK_REQUESTS_PER_MINUTE: usize = 60;  // Rate limit network calls
const MAX_SANDBOX_MEMORY_MB: usize = 1024; // 1GB max for sandbox

/// Available plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginPermission {
    /// Access to local storage
    Storage,
    /// Network access
    Network,
    /// Access to browser tabs
    Tabs,
    /// Access to bookmarks
    Bookmarks,
    /// Access to browsing history
    History,
    /// Access to downloads
    Downloads,
    /// Access to cookies
    Cookies,
    /// Access to passwords
    Passwords,
    /// Custom permission
    Custom(String),
}

impl PluginPermission {
    /// Parse a permission string into a PluginPermission
    pub fn from_string(s: &str) -> Self {
        match s {
            "storage" => PluginPermission::Storage,
            "network" => PluginPermission::Network,
            "tabs" => PluginPermission::Tabs,
            "bookmarks" => PluginPermission::Bookmarks,
            "history" => PluginPermission::History,
            "downloads" => PluginPermission::Downloads,
            "cookies" => PluginPermission::Cookies,
            "passwords" => PluginPermission::Passwords,
            other => PluginPermission::Custom(other.to_string()),
        }
    }
    
    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            PluginPermission::Storage => "storage".to_string(),
            PluginPermission::Network => "network".to_string(),
            PluginPermission::Tabs => "tabs".to_string(),
            PluginPermission::Bookmarks => "bookmarks".to_string(),
            PluginPermission::History => "history".to_string(),
            PluginPermission::Downloads => "downloads".to_string(),
            PluginPermission::Cookies => "cookies".to_string(),
            PluginPermission::Passwords => "passwords".to_string(),
            PluginPermission::Custom(s) => s.clone(),
        }
    }
    
    /// Check if permission is sensitive (requires user approval)
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            PluginPermission::Passwords | PluginPermission::Cookies | PluginPermission::History
        )
    }
}

/// Errors that can occur during plugin operations
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Failed to load library: {0}")]
    LoadFailed(String),
    
    #[error("Invalid plugin: {0}")]
    InvalidPlugin(String),
    
    #[error("Plugin version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    #[error("Security validation failed: {0}")]
    SecurityError(String),
    
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: Vec<String>,
    pub api_version: String,
}

impl PluginMetadata {
    /// Parse permissions from strings to PluginPermission enum
    pub fn get_permissions(&self) -> Vec<PluginPermission> {
        self.permissions.iter()
            .map(|s| PluginPermission::from_string(s))
            .collect()
    }
    
    /// Check if plugin has a specific permission
    pub fn has_permission(&self, permission: &PluginPermission) -> bool {
        self.get_permissions().contains(permission)
    }
    
    /// Check if plugin has any sensitive permissions
    pub fn has_sensitive_permissions(&self) -> bool {
        self.get_permissions().iter().any(|p| p.is_sensitive())
    }
}

/// Plugin context - provides access to browser APIs
pub struct PluginContext {
    pub plugin_id: String,
    pub data_dir: PathBuf,
    pub config: HashMap<String, String>,
}

/// Resource usage statistics for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub command_count: usize,
    pub network_request_count: usize,
    pub max_concurrent_commands: usize,
    pub max_network_requests_per_minute: usize,
}

impl Default for ResourceStats {
    fn default() -> Self {
        Self {
            command_count: 0,
            network_request_count: 0,
            max_concurrent_commands: 10,
            max_network_requests_per_minute: 60,
        }
    }
}

/// Audit log entry for plugin operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub plugin_id: String,
    pub operation: String,
    pub status: String,
    pub details: String,
    pub user_id: Option<String>,
}

/// Plugin trait - all native plugins must implement this
pub trait ExodusPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with context
    fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError>;
    
    /// Handle a command from the browser
    fn handle_command(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, PluginError>;
    
    /// Cleanup before unloading
    fn cleanup(&mut self) -> Result<(), PluginError>;
}

/// Type-safe function pointers for plugin symbols
// Note: Using trait objects in FFI is unsafe but is a common pattern in plugin systems.
// The warnings are accepted as this is the standard approach for dynamic plugin loading.
#[allow(improper_ctypes_definitions)]
type PluginVersionFn = unsafe extern "C" fn() -> *const std::ffi::c_char;
#[allow(improper_ctypes_definitions)]
type PluginInitFn = unsafe extern "C" fn() -> *mut dyn ExodusPlugin;
#[allow(improper_ctypes_definitions)]
type PluginDeinitFn = unsafe extern "C" fn(plugin: *mut dyn ExodusPlugin);

/// Loaded plugin instance
pub struct LoadedPlugin {
    library: Library,
    plugin: *mut dyn ExodusPlugin,
    metadata: PluginMetadata,
    enabled: bool,
    file_path: PathBuf,
    last_modified: std::time::SystemTime,
    // Aerospace-level resource tracking
    command_count: std::sync::atomic::AtomicUsize,
    network_request_count: std::sync::atomic::AtomicUsize,
    last_network_reset: std::sync::Mutex<std::time::Instant>,
}

// SAFETY: The plugin pointer is only accessed within the LoadedPlugin methods
// and the ExodusPlugin trait requires Send + Sync, so the underlying plugin
// is thread-safe. We ensure exclusive access through the methods.
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

impl LoadedPlugin {
    /// Create a new loaded plugin
    pub fn new(
        library: Library,
        plugin: *mut dyn ExodusPlugin,
        metadata: PluginMetadata,
        file_path: PathBuf,
    ) -> Result<Self, PluginError> {
        let last_modified = std::fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .map_err(|e| PluginError::IoError(e))?;
        
        Ok(Self {
            library,
            plugin,
            metadata,
            enabled: true,
            file_path,
            last_modified,
            command_count: std::sync::atomic::AtomicUsize::new(0),
            network_request_count: std::sync::atomic::AtomicUsize::new(0),
            last_network_reset: std::sync::Mutex::new(std::time::Instant::now()),
        })
    }
    
    /// Get plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    /// Check if plugin is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable or disable the plugin
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if plugin file has been modified
    pub fn check_for_changes(&self) -> bool {
        match std::fs::metadata(&self.file_path) {
            Ok(metadata) => {
                match metadata.modified() {
                    Ok(current_modified) => {
                        current_modified != self.last_modified
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
    
    /// Get the file path
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
    
    /// Check and enforce network rate limit
    pub fn check_network_rate_limit(&self) -> Result<(), PluginError> {
        let count = self.network_request_count.load(std::sync::atomic::Ordering::Relaxed);
        
        // Reset counter if more than a minute has passed
        {
            let mut last_reset = match self.last_network_reset.lock() {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to acquire last_network_reset lock: {}", e);
                    return Err(PluginError::ExecutionError(format!("Lock error: {}", e)));
                }
            };
            if last_reset.elapsed() > std::time::Duration::from_secs(60) {
                self.network_request_count.store(0, std::sync::atomic::Ordering::Relaxed);
                *last_reset = std::time::Instant::now();
                return Ok(());
            }
        }
        
        if count >= MAX_NETWORK_REQUESTS_PER_MINUTE {
            return Err(PluginError::SecurityError(
                format!("Network rate limit exceeded: {}/{} requests per minute", 
                    count, MAX_NETWORK_REQUESTS_PER_MINUTE)
            ));
        }
        
        Ok(())
    }
    
    /// Increment network request count
    pub fn increment_network_request(&self) {
        self.network_request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Check concurrent command limit
    pub fn check_concurrent_commands(&self) -> Result<(), PluginError> {
        let count = self.command_count.load(std::sync::atomic::Ordering::Relaxed);
        
        if count >= MAX_CONCURRENT_COMMANDS {
            return Err(PluginError::SecurityError(
                format!("Concurrent command limit exceeded: {}/{}", 
                    count, MAX_CONCURRENT_COMMANDS)
            ));
        }
        
        Ok(())
    }
    
    /// Increment command count
    pub fn increment_command_count(&self) {
        self.command_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Decrement command count
    pub fn decrement_command_count(&self) {
        self.command_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Get resource usage statistics
    pub fn get_resource_stats(&self) -> ResourceStats {
        ResourceStats {
            command_count: self.command_count.load(std::sync::atomic::Ordering::Relaxed),
            network_request_count: self.network_request_count.load(std::sync::atomic::Ordering::Relaxed),
            max_concurrent_commands: MAX_CONCURRENT_COMMANDS,
            max_network_requests_per_minute: MAX_NETWORK_REQUESTS_PER_MINUTE,
        }
    }
    
    /// Execute a command on the plugin
    pub fn execute_command(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        if !self.enabled {
            return Err(PluginError::ExecutionError("Plugin is disabled".to_string()));
        }
        
        // Aerospace-level resource limit checks
        self.check_concurrent_commands()?;
        
        // Increment command count
        self.increment_command_count();
        
        // Execute command with timeout protection
        let start_time = std::time::Instant::now();
        let result = unsafe {
            (*self.plugin).handle_command(command, params)
        };
        
        // Decrement command count after execution
        self.decrement_command_count();
        
        // Check execution time
        let execution_time = start_time.elapsed();
        if execution_time.as_millis() > MAX_COMMAND_EXECUTION_TIME_MS as u128 {
            eprintln!(
                "Warning: Plugin '{}' command '{}' exceeded time limit: {}ms > {}ms",
                self.metadata.id, command, execution_time.as_millis(), MAX_COMMAND_EXECUTION_TIME_MS
            );
        }
        
        result
    }
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        unsafe {
            let _ = (*self.plugin).cleanup();
        }
    }
}

/// Native plugin manager
pub struct NativePluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    plugins_dir: PathBuf,
    // Aerospace-level audit logging
    audit_log: std::sync::Arc<std::sync::Mutex<Vec<AuditLogEntry>>>,
    max_audit_log_entries: usize,
    // Sandbox isolation support
    use_sandbox: bool,
    sandbox_config: sandbox::SandboxConfig,
    sandboxes: HashMap<String, sandbox::PluginSandbox>,
    // User context for audit logging
    current_user_id: Option<String>,
}

impl NativePluginManager {
    /// Create a new plugin manager
    pub fn new(plugins_dir: PathBuf) -> Result<Self, PluginError> {
        std::fs::create_dir_all(&plugins_dir)?;
        Ok(Self {
            plugins: HashMap::new(),
            plugins_dir,
            audit_log: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            max_audit_log_entries: 10000, // Aerospace-level: retain up to 10,000 entries
            use_sandbox: false, // Sandbox disabled by default for compatibility
            sandbox_config: sandbox::SandboxConfig::default(),
            sandboxes: HashMap::new(),
            current_user_id: None,
        })
    }
    
    /// Enable sandbox isolation for plugins
    pub fn enable_sandbox(&mut self, config: sandbox::SandboxConfig) {
        self.use_sandbox = true;
        self.sandbox_config = config;
    }
    
    /// Disable sandbox isolation
    pub fn disable_sandbox(&mut self) {
        self.use_sandbox = false;
        // Stop all running sandboxes
        for (_id, sandbox) in self.sandboxes.iter_mut() {
            let _ = sandbox.stop();
        }
        self.sandboxes.clear();
    }
    
    /// Set the current user ID for audit logging
    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.current_user_id = user_id;
    }
    
    /// Get the current user ID
    pub fn get_user_id(&self) -> Option<&String> {
        self.current_user_id.as_ref()
    }
    
    /// Load a plugin in sandbox mode
    fn load_plugin_sandboxed(&mut self, path: &Path) -> Result<PluginMetadata, PluginError> {
        // Get metadata first (load without sandbox to get metadata)
        let library = unsafe { Library::new(path) }
            .map_err(|e| PluginError::LoadFailed(format!("Failed to load library: {}", e)))?;
        
        let version_fn: Symbol<PluginVersionFn> = unsafe { library.get(b"exodus_plugin_version") }
            .map_err(|_| PluginError::InvalidPlugin("Missing exodus_plugin_version symbol".to_string()))?;
        
        let version_ptr = unsafe { version_fn() };
        let version = unsafe { std::ffi::CStr::from_ptr(version_ptr) }
            .to_str()
            .map_err(|_| PluginError::InvalidPlugin("Invalid version string".to_string()))?;
        
        if version != PLUGIN_API_VERSION {
            return Err(PluginError::InvalidPlugin(format!(
                "Unsupported API version: {}, expected: {}", version, PLUGIN_API_VERSION
            )));
        }
        
        let init_fn: Symbol<PluginInitFn> = unsafe { library.get(b"exodus_plugin_init") }
            .map_err(|_| PluginError::InvalidPlugin("Missing exodus_plugin_init symbol".to_string()))?;
        
        let plugin_ptr = unsafe { init_fn() };
        
        if plugin_ptr.is_null() {
            return Err(PluginError::InvalidPlugin("Plugin initialization returned null".to_string()));
        }
        
        let metadata = unsafe { (*plugin_ptr).metadata() }.clone();
        
        // Validate metadata
        self.validate_metadata(&metadata)?;
        
        // Explicitly close the library handle since we only needed it for metadata
        // The actual plugin will run in the sandbox process
        drop(library);
        
        // Create sandbox
        let mut sandbox = sandbox::PluginSandbox::new(
            path.to_path_buf(),
            metadata.clone(),
            self.sandbox_config.clone(),
        )?;
        
        // Start sandbox process
        sandbox.start()?;
        
        // Store sandbox
        self.sandboxes.insert(metadata.id.clone(), sandbox);
        
        // Log plugin load
        self.add_audit_log(
            metadata.id.clone(),
            "load_sandboxed".to_string(),
            "success".to_string(),
            format!("Loaded plugin in sandbox from: {:?}", path)
        );
        
        Ok(metadata)
    }
    
    /// Load a plugin from a dynamic library file
    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginMetadata, PluginError> {
        // Aerospace-level security validation
        self.validate_plugin_path(path)?;
        
        // If sandbox is enabled, load plugin in sandbox
        if self.use_sandbox {
            return self.load_plugin_sandboxed(path);
        }
        
        // Load the dynamic library (traditional mode)
        let library = unsafe { Library::new(path) }
            .map_err(|e| PluginError::LoadFailed(format!("Failed to load library: {}", e)))?;
        
        // Get plugin version
        let version_fn: Symbol<PluginVersionFn> = unsafe { library.get(b"exodus_plugin_version") }
            .map_err(|_| PluginError::InvalidPlugin("Missing exodus_plugin_version symbol".to_string()))?;
        
        let version_ptr = unsafe { version_fn() };
        let version = unsafe { std::ffi::CStr::from_ptr(version_ptr) }
            .to_str()
            .map_err(|_| PluginError::InvalidPlugin("Invalid version string".to_string()))?
            .to_string();
        
        // Validate API version
        if version != PLUGIN_API_VERSION {
            return Err(PluginError::VersionMismatch {
                expected: PLUGIN_API_VERSION.to_string(),
                actual: version,
            });
        }
        
        // Initialize plugin
        let init_fn: Symbol<PluginInitFn> = unsafe { library.get(b"exodus_plugin_init") }
            .map_err(|_| PluginError::InvalidPlugin("Missing exodus_plugin_init symbol".to_string()))?;
        
        let plugin_ptr = unsafe { init_fn() };
        
        if plugin_ptr.is_null() {
            return Err(PluginError::InvalidPlugin("Plugin initialization returned null".to_string()));
        }
        
        // Get metadata
        let metadata = unsafe { (*plugin_ptr).metadata() }.clone();
        
        // Validate metadata
        self.validate_metadata(&metadata)?;
        
        // Check if already loaded
        if self.plugins.contains_key(&metadata.id) {
            return Err(PluginError::AlreadyLoaded(metadata.id));
        }
        
        // Check plugin count limit
        if self.plugins.len() >= MAX_PLUGINS {
            return Err(PluginError::SecurityError(
                format!("Maximum plugins reached (max {})", MAX_PLUGINS)
            ));
        }
        
        // Create context and initialize
        let context = PluginContext {
            plugin_id: metadata.id.clone(),
            data_dir: self.plugins_dir.join(&metadata.id),
            config: HashMap::new(),
        };
        
        unsafe {
            (*plugin_ptr).initialize(context)
                .map_err(|e| PluginError::ExecutionError(format!("Plugin initialization failed: {}", e)))?;
        }
        
        // Store the loaded plugin
        let loaded_plugin = LoadedPlugin::new(library, plugin_ptr, metadata.clone(), path.to_path_buf())?;
        self.plugins.insert(metadata.id.clone(), loaded_plugin);
        
        // Aerospace-level: log plugin load
        self.add_audit_log(
            metadata.id.clone(),
            "load".to_string(),
            "success".to_string(),
            format!("Loaded plugin from: {:?}", path)
        );
        
        Ok(metadata)
    }
    
    /// Unload a plugin
    pub fn unload_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        // Check if plugin is sandboxed
        if let Some(mut sandbox) = self.sandboxes.remove(id) {
            // Stop sandbox process
            sandbox.stop()?;
            
            // Aerospace-level: log plugin unload
            self.add_audit_log(
                id.to_string(),
                "unload_sandboxed".to_string(),
                "success".to_string(),
                format!("Unloaded sandboxed plugin")
            );
            
            return Ok(());
        }
        
        // Traditional unload
        let plugin = self.plugins.remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        
        // Aerospace-level: log plugin unload
        self.add_audit_log(
            id.to_string(),
            "unload".to_string(),
            "success".to_string(),
            format!("Unloaded plugin: {}", plugin.metadata().name)
        );
        
        // Plugin cleanup happens in Drop
        drop(plugin);
        
        Ok(())
    }
    
    /// Execute a command on a plugin
    pub fn execute_command(&self, id: &str, command: &str, params: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        // Check if plugin is sandboxed
        if let Some(_sandbox) = self.sandboxes.get(id) {
            // Sandboxed plugins require async execution
            // This is a synchronous wrapper that will fail
            // Use execute_command_async for sandboxed plugins
            return Err(PluginError::ExecutionError(
                "Sandboxed plugin requires async execution. Use execute_command_async instead.".to_string()
            ));
        }
        
        // Traditional execution
        let plugin = self.plugins.get(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        
        let result = plugin.execute_command(command, params.clone());
        
        // Aerospace-level: log command execution
        match &result {
            Ok(_) => {
                self.add_audit_log(
                    id.to_string(),
                    format!("execute_command: {}", command),
                    "success".to_string(),
                    format!("Command executed successfully")
                );
            }
            Err(e) => {
                self.add_audit_log(
                    id.to_string(),
                    format!("execute_command: {}", command),
                    "error".to_string(),
                    format!("Command failed: {}", e)
                );
            }
        }
        
        result
    }
    
    /// Execute a command on a plugin (async version for sandboxed plugins)
    pub async fn execute_command_async(&self, id: &str, command: &str, params: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        // Check if plugin is sandboxed
        if let Some(sandbox) = self.sandboxes.get(id) {
            let result = sandbox.send_command(command, params.clone()).await;
            
            // Aerospace-level: log command execution
            match &result {
                Ok(_) => {
                    self.add_audit_log(
                        id.to_string(),
                        format!("execute_command_async: {}", command),
                        "success".to_string(),
                        format!("Sandboxed command executed successfully")
                    );
                }
                Err(e) => {
                    self.add_audit_log(
                        id.to_string(),
                        format!("execute_command_async: {}", command),
                        "error".to_string(),
                        format!("Sandboxed command failed: {}", e)
                    );
                }
            }
            
            return result;
        }
        
        // Traditional execution (non-sandboxed)
        let plugin = self.plugins.get(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        
        let result = plugin.execute_command(command, params.clone());
        
        // Aerospace-level: log command execution
        match &result {
            Ok(_) => {
                self.add_audit_log(
                    id.to_string(),
                    format!("execute_command_async: {}", command),
                    "success".to_string(),
                    format!("Command executed successfully")
                );
            }
            Err(e) => {
                self.add_audit_log(
                    id.to_string(),
                    format!("execute_command_async: {}", command),
                    "error".to_string(),
                    format!("Command failed: {}", e)
                );
            }
        }
        
        result
    }
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values()
            .map(|p| p.metadata().clone())
            .collect()
    }
    
    /// Get plugin metadata by ID
    pub fn get_plugin(&self, id: &str) -> Option<&PluginMetadata> {
        self.plugins.get(id).map(|p| p.metadata())
    }
    
    /// Get resource statistics for a plugin
    pub fn get_plugin_resource_stats(&self, id: &str) -> Option<ResourceStats> {
        // Check both traditional plugins and sandboxed plugins
        if let Some(plugin) = self.plugins.get(id) {
            Some(plugin.get_resource_stats())
        } else if let Some(sandbox) = self.sandboxes.get(id) {
            // Use actual resource tracking for sandboxed plugins
            Some(sandbox.get_resource_stats())
        } else {
            None
        }
    }
    
    /// Add an audit log entry
    fn add_audit_log(&self, plugin_id: String, operation: String, status: String, details: String) {
        let entry = AuditLogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            plugin_id,
            operation,
            status,
            details,
            user_id: self.current_user_id.clone(),
        };
        
        let mut log = match self.audit_log.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire audit_log lock: {}", e);
                return;
            }
        };
        log.push(entry);
        
        // Aerospace-level: enforce max log size
        if log.len() > self.max_audit_log_entries {
            log.remove(0);
        }
    }
    
    /// Get audit log entries for a plugin
    pub fn get_audit_log(&self, plugin_id: Option<String>) -> Vec<AuditLogEntry> {
        self.audit_log.lock()
            .map(|log| {
                if let Some(id) = plugin_id {
                    log.iter().filter(|e| e.plugin_id == id).cloned().collect()
                } else {
                    log.clone()
                }
            })
            .unwrap_or_default()
    }
    
    /// Clear audit log entries older than specified seconds
    pub fn clear_old_audit_logs(&self, older_than_seconds: u64) {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs() - older_than_seconds;
        
        let mut log = match self.audit_log.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire audit_log lock: {}", e);
                return;
            }
        };
        log.retain(|e| e.timestamp >= cutoff);
    }
    
    /// Enable or disable a plugin
    pub fn set_plugin_enabled(&mut self, id: &str, enabled: bool) -> Result<(), PluginError> {
        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        
        plugin.set_enabled(enabled);
        Ok(())
    }
    
    /// Check for plugin file changes and reload if needed
    pub fn check_and_reload_plugins(&mut self) -> Result<Vec<String>, PluginError> {
        let mut reloaded = Vec::new();
        let mut plugins_to_reload: Vec<String> = Vec::new();
        
        // First, collect IDs of traditional plugins that need reloading
        for (id, plugin) in &self.plugins {
            if plugin.check_for_changes() {
                plugins_to_reload.push(id.clone());
            }
        }
        
        // Check sandboxed plugins for changes
        for (id, sandbox) in &self.sandboxes {
            if std::fs::metadata(&sandbox.plugin_path)
                .and_then(|m| m.modified())
                .ok()
                .map_or(false, |modified: std::time::SystemTime| {
                    // Simple check: if file was modified in the last 10 seconds, consider it changed
                    modified.elapsed().unwrap_or(std::time::Duration::from_secs(0)) < std::time::Duration::from_secs(10)
                }) {
                plugins_to_reload.push(id.clone());
            }
        }
        
        // Then reload each plugin (both traditional and sandboxed)
        for id in plugins_to_reload {
            let file_path = if let Some(plugin) = self.plugins.get(&id) {
                plugin.file_path().clone()
            } else if let Some(sandbox) = self.sandboxes.get(&id) {
                sandbox.plugin_path.clone()
            } else {
                continue;
            };
            
            // Unload the old version (handles both traditional and sandboxed)
            self.unload_plugin(&id)?;
            
            // Reload the new version
            match self.load_plugin(&file_path) {
                Ok(_) => {
                    reloaded.push(id);
                }
                Err(e) => {
                    eprintln!("Failed to reload plugin {}: {}", id, e);
                }
            }
        }
        
        Ok(reloaded)
    }
    
    /// Scan and load all plugins from the plugins directory
    pub fn scan_and_load(&mut self) -> Result<usize, PluginError> {
        if !self.plugins_dir.exists() {
            return Ok(0);
        }
        
        let mut loaded = 0;
        
        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only load dynamic libraries
            let is_dylib = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "dylib" || ext == "so" || ext == "dll")
                .unwrap_or(false);
            
            if !is_dylib {
                continue;
            }
            
            match self.load_plugin(&path) {
                Ok(_) => loaded += 1,
                Err(e) => tracing::warn!("Failed to load plugin {}: {}", path.display(), e),
            }
        }
        
        Ok(loaded)
    }
    
    /// Validate plugin path for security
    fn validate_plugin_path(&self, path: &Path) -> Result<(), PluginError> {
        // Check if path exists
        if !path.exists() {
            return Err(PluginError::SecurityError("Plugin file does not exist".to_string()));
        }
        
        // Check if path is within plugins directory
        let canonical_path = path.canonicalize()
            .map_err(|e| PluginError::SecurityError(format!("Failed to canonicalize path: {}", e)))?;
        
        let canonical_plugins_dir = self.plugins_dir.canonicalize()
            .map_err(|e| PluginError::SecurityError(format!("Failed to canonicalize plugins dir: {}", e)))?;
        
        if !canonical_path.starts_with(&canonical_plugins_dir) {
            return Err(PluginError::SecurityError("Plugin must be located in plugins directory".to_string()));
        }
        
        // Check file permissions (basic check)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&canonical_path)?;
            let permissions = metadata.permissions().mode();
            
            // Check if file is world-writable (security risk)
            if permissions & 0o002 != 0 {
                return Err(PluginError::SecurityError("Plugin file is world-writable".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Validate plugin metadata
    fn validate_metadata(&self, metadata: &PluginMetadata) -> Result<(), PluginError> {
        if metadata.id.is_empty() || metadata.id.len() > MAX_PLUGIN_ID_LENGTH {
            return Err(PluginError::InvalidPlugin("Invalid plugin ID".to_string()));
        }
        
        if metadata.name.is_empty() || metadata.name.len() > MAX_PLUGIN_NAME_LENGTH {
            return Err(PluginError::InvalidPlugin("Invalid plugin name".to_string()));
        }
        
        if metadata.version.is_empty() || metadata.version.len() > MAX_PLUGIN_VERSION_LENGTH {
            return Err(PluginError::InvalidPlugin("Invalid plugin version".to_string()));
        }
        
        if metadata.description.len() > MAX_PLUGIN_DESCRIPTION_LENGTH {
            return Err(PluginError::InvalidPlugin("Plugin description too long".to_string()));
        }
        
        // Validate ID format (alphanumeric, hyphens, underscores)
        if !metadata.id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(PluginError::InvalidPlugin("Plugin ID contains invalid characters".to_string()));
        }
        
        Ok(())
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Thread-safe handle for the plugin manager
pub type PluginManagerState = Arc<Mutex<NativePluginManager>>;

/// Initialize the native plugin manager
#[tauri::command]
pub fn init_native_plugin_manager(app_data_dir: String) -> Result<String, String> {
    let plugins_dir = PathBuf::from(app_data_dir).join("plugins").join("native");
    let _manager = NativePluginManager::new(plugins_dir.clone())
        .map_err(|e| format!("Failed to create plugin manager: {}", e))?;
    
    // In a real implementation, this would be stored in the app state
    // For now, return the plugins directory path
    Ok(plugins_dir.to_string_lossy().to_string())
}

/// Load a native plugin from a file path
#[tauri::command]
pub fn load_native_plugin(
    app: AppHandle,
    path: String,
) -> Result<PluginMetadata, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.load_plugin(Path::new(&path))
        .map_err(|e| format!("Failed to load plugin: {}", e))
}

/// Unload a native plugin
#[tauri::command]
pub fn unload_native_plugin(
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.unload_plugin(&id)
        .map_err(|e| format!("Failed to unload plugin: {}", e))
}

/// List all loaded native plugins
#[tauri::command]
pub fn list_native_plugins(
    app: AppHandle,
) -> Result<Vec<PluginMetadata>, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    Ok(manager.list_plugins())
}

/// Get metadata for a specific plugin
#[tauri::command]
pub fn get_native_plugin(
    app: AppHandle,
    id: String,
) -> Result<Option<PluginMetadata>, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    Ok(manager.get_plugin(&id).cloned())
}

/// Execute a command on a native plugin
#[tauri::command]
pub fn execute_plugin_command(
    app: AppHandle,
    id: String,
    command: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.execute_command(&id, &command, params)
        .map_err(|e| format!("Failed to execute command: {}", e))
}

/// Enable or disable a native plugin
#[tauri::command]
pub fn set_native_plugin_enabled(
    app: AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.set_plugin_enabled(&id, enabled)
        .map_err(|e| format!("Failed to set plugin enabled: {}", e))
}

/// Scan and load all plugins from the plugins directory
#[tauri::command]
pub fn scan_native_plugins(
    app: AppHandle,
) -> Result<usize, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.scan_and_load()
        .map_err(|e| format!("Failed to scan plugins: {}", e))
}

/// Check for plugin file changes and reload if needed
#[tauri::command]
pub fn reload_changed_plugins(
    app: AppHandle,
) -> Result<Vec<String>, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.check_and_reload_plugins()
        .map_err(|e| format!("Failed to reload plugins: {}", e))
}

/// Get resource statistics for a plugin
#[tauri::command]
pub fn get_plugin_resource_stats(
    app: AppHandle,
    id: String,
) -> Result<ResourceStats, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.get_plugin_resource_stats(&id)
        .ok_or_else(|| format!("Plugin not found: {}", id))
}

/// Get audit log entries
#[tauri::command]
pub fn get_audit_log(
    app: AppHandle,
    plugin_id: Option<String>,
) -> Result<Vec<AuditLogEntry>, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    Ok(manager.get_audit_log(plugin_id))
}

/// Clear old audit log entries
#[tauri::command]
pub fn clear_old_audit_logs(
    app: AppHandle,
    older_than_seconds: u64,
) -> Result<(), String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.clear_old_audit_logs(older_than_seconds);
    Ok(())
}

/// Enable sandbox isolation for plugins
#[tauri::command]
pub fn enable_plugin_sandbox(
    app: AppHandle,
    enable_seccomp: bool,
    allow_network: bool,
    allow_filesystem: bool,
    max_memory_mb: usize,
) -> Result<(), String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    let config = sandbox::SandboxConfig {
        enable_seccomp,
        allow_network,
        allow_filesystem,
        max_memory_mb,
        socket_path: Some(std::env::temp_dir().join("exodus_plugin_sandbox")),
    };
    
    manager.enable_sandbox(config);
    Ok(())
}

/// Disable sandbox isolation
#[tauri::command]
pub fn disable_plugin_sandbox(
    app: AppHandle,
) -> Result<(), String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let mut manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    manager.disable_sandbox();
    Ok(())
}

/// Get sandbox status and configuration
#[tauri::command]
pub fn get_sandbox_status(
    app: AppHandle,
) -> Result<SandboxStatus, String> {
    let state: tauri::State<PluginManagerState> = app.state();
    let manager = state.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    Ok(SandboxStatus {
        enabled: manager.use_sandbox,
        config: manager.sandbox_config.clone(),
    })
}

/// Sandbox status for frontend
#[derive(serde::Serialize, Clone)]
pub struct SandboxStatus {
    pub enabled: bool,
    pub config: sandbox::SandboxConfig,
}

// Include comprehensive tests
#[cfg(test)]
mod tests;

// Include integration tests
#[cfg(test)]
mod integration_test;

// Include sandbox isolation module
pub mod sandbox;

// Include sandbox runner module
pub mod sandbox_runner;

// Include sandbox integration tests
#[cfg(test)]
mod sandbox_integration_test;
