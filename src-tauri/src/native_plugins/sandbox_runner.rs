//! Sandboxed plugin runner - runs in a separate process with seccomp filtering
//! 
//! This is the entry point for sandboxed plugin processes. It receives commands
//! via Unix Domain Socket and applies seccomp filters for security.

use std::path::PathBuf;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use libloading::{Library, Symbol};
use super::{sandbox::{PluginMessage, PluginResponse, apply_seccomp_filter}, PluginError, ExodusPlugin, PluginContext, PLUGIN_API_VERSION};
use std::collections::HashMap;

/// Main entry point for sandboxed plugin process
pub async fn run_sandboxed_plugin(plugin_path: PathBuf) -> Result<(), PluginError> {
    // Get configuration from environment
    let socket_path = std::env::var("EXODUS_PLUGIN_SOCKET")
        .map_err(|_| PluginError::ExecutionError("EXODUS_PLUGIN_SOCKET not set".to_string()))?;
    
    let _plugin_id = std::env::var("EXODUS_PLUGIN_ID")
        .map_err(|_| PluginError::ExecutionError("EXODUS_PLUGIN_ID not set".to_string()))?;
    
    let enable_seccomp = std::env::var("EXODUS_PLUGIN_SECCOMP").is_ok();
    
    // Apply seccomp filter if enabled
    if enable_seccomp {
        // For now, allow filesystem and network for compatibility
        // In production, these should be based on plugin permissions
        apply_seccomp_filter(true, true)?;
    }
    
    // Load the plugin
    let library = unsafe { Library::new(&plugin_path) }
        .map_err(|e| PluginError::ExecutionError(format!("Failed to load plugin: {}", e)))?;
    
    // Get plugin version
    type PluginVersionFn = unsafe extern "C" fn() -> *const std::ffi::c_char;
    let version_fn: Symbol<PluginVersionFn> = unsafe { library.get(b"exodus_plugin_version") }
        .map_err(|_| PluginError::ExecutionError("Plugin version symbol not found".to_string()))?;
    
    let version_ptr = unsafe { version_fn() };
    let version_cstr = unsafe { std::ffi::CStr::from_ptr(version_ptr) };
    let version_str = version_cstr.to_str()
        .map_err(|_| PluginError::ExecutionError("Invalid version string".to_string()))?;
    
    if version_str != PLUGIN_API_VERSION {
        return Err(PluginError::ExecutionError(format!("Unsupported API version: {}", version_str)));
    }
    
    // Get plugin init function
    type PluginInitFn = unsafe extern "C" fn() -> *mut dyn ExodusPlugin;
    let init_fn: Symbol<PluginInitFn> = unsafe { library.get(b"exodus_plugin_init") }
        .map_err(|_| PluginError::ExecutionError("Plugin init symbol not found".to_string()))?;
    
    let plugin_ptr = unsafe { init_fn() };
    
    if plugin_ptr.is_null() {
        return Err(PluginError::ExecutionError("Plugin initialization returned null".to_string()));
    }
    
    // Get plugin metadata
    let metadata = unsafe { (*plugin_ptr).metadata() }.clone();
    
    // Initialize plugin context
    let data_dir = std::env::temp_dir().join("exodus_plugins").join(&metadata.id);
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| PluginError::ExecutionError(format!("Failed to create data directory: {}", e)))?;
    
    let context = PluginContext {
        plugin_id: metadata.id.clone(),
        data_dir,
        config: HashMap::new(),
    };
    
    // Initialize plugin
    unsafe {
        (*plugin_ptr).initialize(context)
            .map_err(|e| PluginError::ExecutionError(format!("Plugin initialization failed: {}", e)))?;
    }
    
    // Create Unix Domain Socket listener
    let listener = UnixListener::bind(&socket_path)
        .map_err(|e| PluginError::ExecutionError(format!("Failed to bind socket: {}", e)))?;
    
    // Accept connections and handle commands
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                // Read message length
                let mut len_buf: [u8; 4] = [0u8; 4];
                if let Err(e) = stream.read_exact(&mut len_buf).await {
                    eprintln!("Failed to read message length: {}", e);
                    continue;
                }
                
                let len = u32::from_be_bytes(len_buf) as usize;
                let mut message_buf = vec![0u8; len];
                
                if let Err(e) = stream.read_exact(&mut message_buf).await {
                    eprintln!("Failed to read message: {}", e);
                    continue;
                }
                
                // Deserialize message
                let message: PluginMessage = match serde_json::from_slice(&message_buf) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Failed to deserialize message: {}", e);
                        continue;
                    }
                };
                
                // Execute command
                let result = unsafe {
                    (*plugin_ptr).handle_command(&message.command, message.params.clone())
                };
                
                // Send response
                let response = PluginResponse {
                    id: message.id,
                    result: result.as_ref().ok().cloned(),
                    error: result.as_ref().err().map(|e| e.to_string()),
                };
                
                let response_json = match serde_json::to_vec(&response) {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("Failed to serialize response: {}", e);
                        continue;
                    }
                };
                
                let response_len = response_json.len() as u32;
                
                if let Err(e) = stream.write_all(&response_len.to_be_bytes()).await {
                    eprintln!("Failed to write response length: {}", e);
                    continue;
                }
                
                if let Err(e) = stream.write_all(&response_json).await {
                    eprintln!("Failed to write response: {}", e);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                break;
            }
        }
    }
    
    // Cleanup
    unsafe {
        let _ = (*plugin_ptr).cleanup();
    }
    
    // Remove socket
    if std::path::Path::new(&socket_path).exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    
    Ok(())
}
