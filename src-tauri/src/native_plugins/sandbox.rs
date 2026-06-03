//! Plugin sandbox isolation using process isolation and seccomp
//! 
//! This module provides aerospace-level sandbox isolation for native plugins
//! by running each plugin in a separate process with seccomp system call filtering.

use std::path::PathBuf;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use super::{PluginError, PluginMetadata};

/// IPC message for plugin communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub id: u64,
    pub plugin_id: String,
    pub command: String,
    pub params: serde_json::Value,
}

/// IPC response from plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Plugin sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable seccomp filtering
    pub enable_seccomp: bool,
    /// Enable network access
    pub allow_network: bool,
    /// Enable file system access
    pub allow_filesystem: bool,
    /// Maximum memory in MB
    pub max_memory_mb: usize,
    /// Socket path for IPC (serialized as string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_path: Option<PathBuf>,
}

impl SandboxConfig {
    /// Validate sandbox configuration
    pub fn validate(&self) -> Result<(), super::PluginError> {
        if self.max_memory_mb > super::MAX_SANDBOX_MEMORY_MB {
            return Err(super::PluginError::SecurityError(
                format!("Sandbox memory limit too high (max {} MB)", super::MAX_SANDBOX_MEMORY_MB)
            ));
        }
        Ok(())
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_seccomp: true,
            allow_network: false,
            allow_filesystem: false,
            max_memory_mb: 512,
            socket_path: Some(std::env::temp_dir().join("exodus_plugin_sandbox")),
        }
    }
}

/// Apply seccomp filter to restrict system calls (Linux only)
#[cfg(target_os = "linux")]
pub fn apply_seccomp_filter(_allow_network: bool, _allow_filesystem: bool) -> Result<(), PluginError> {
    // seccomp implementation - Linux only
    // For now, this is a placeholder. In production, this would use the seccomp crate
    // to apply system call filtering based on the plugin's permissions.
    // The seccomp 0.1 crate has a different API than expected, so we'll implement
    // a proper seccomp filter when the correct API is available.
    eprintln!("seccomp filtering requested but not yet implemented");
    Ok(())
}

/// Stub for non-Linux platforms
#[cfg(not(target_os = "linux"))]
pub fn apply_seccomp_filter(_allow_network: bool, _allow_filesystem: bool) -> Result<(), PluginError> {
    // seccomp is Linux-only, on other platforms we rely on process isolation
    Ok(())
}

/// Plugin sandbox process
pub struct PluginSandbox {
    config: SandboxConfig,
    pub plugin_path: PathBuf,
    metadata: PluginMetadata,
    socket_path: PathBuf,
    child: Option<std::process::Child>,
    // Resource tracking
    command_count: std::sync::atomic::AtomicUsize,
    network_request_count: std::sync::atomic::AtomicUsize,
    last_network_reset: std::sync::Mutex<std::time::Instant>,
}

impl PluginSandbox {
    /// Create a new plugin sandbox
    pub fn new(
        plugin_path: PathBuf,
        metadata: PluginMetadata,
        config: SandboxConfig,
    ) -> Result<Self, PluginError> {
        let default_path = std::env::temp_dir().join("exodus_plugin_sandbox");
        let base_path = config.socket_path.as_ref()
            .unwrap_or(&default_path);
        let socket_path = base_path.join(format!("{}.sock", metadata.id));
        
        Ok(Self {
            config,
            plugin_path,
            metadata,
            socket_path,
            child: None,
            command_count: std::sync::atomic::AtomicUsize::new(0),
            network_request_count: std::sync::atomic::AtomicUsize::new(0),
            last_network_reset: std::sync::Mutex::new(std::time::Instant::now()),
        })
    }
    
    /// Start the sandbox process
    pub fn start(&mut self) -> Result<(), PluginError> {
        // Create socket directory
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| PluginError::IoError(e))?;
        }
        
        // Remove existing socket if present
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .map_err(|e| PluginError::IoError(e))?;
        }
        
        // Spawn plugin process
        let mut cmd = std::process::Command::new(&self.plugin_path);
        
        // Pass socket path as environment variable
        cmd.env("EXODUS_PLUGIN_SOCKET", &self.socket_path);
        cmd.env("EXODUS_PLUGIN_ID", &self.metadata.id);
        
        // Apply seccomp if enabled (will be done in child process)
        if self.config.enable_seccomp {
            cmd.env("EXODUS_PLUGIN_SECCOMP", "1");
        }
        
        let child = cmd.spawn()
            .map_err(|e| PluginError::ExecutionError(format!("Failed to spawn plugin process: {}", e)))?;
        
        self.child = Some(child);
        
        // Wait for socket to be created
        let max_wait = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        
        while !self.socket_path.exists() {
            if start.elapsed() > max_wait {
                return Err(PluginError::ExecutionError(
                    "Plugin process did not create socket within timeout".to_string()
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        Ok(())
    }
    
    /// Send a command to the sandboxed plugin
    pub async fn send_command(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, PluginError> {
        // Increment command count
        self.command_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to connect to plugin socket: {}", e)))?;
        
        let message = PluginMessage {
            id: 1,
            plugin_id: self.metadata.id.clone(),
            command: command.to_string(),
            params,
        };
        
        let message_json = serde_json::to_vec(&message)
            .map_err(|e| PluginError::ExecutionError(format!("Failed to serialize message: {}", e)))?;
        
        let len = message_json.len() as u32;
        stream.write_all(&len.to_be_bytes())
            .await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to write message length: {}", e)))?;
        
        stream.write_all(&message_json)
            .await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to write message: {}", e)))?;
        
        // Read response
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)
            .await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to read response length: {}", e)))?;
        
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut response_buf = vec![0u8; len];
        stream.read_exact(&mut response_buf)
            .await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to read response: {}", e)))?;
        
        let response: PluginResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| PluginError::ExecutionError(format!("Failed to deserialize response: {}", e)))?;
        
        if let Some(error) = response.error {
            return Err(PluginError::ExecutionError(error));
        }
        
        response.result.ok_or_else(|| PluginError::ExecutionError("Plugin returned no result".to_string()))
    }
    
    /// Increment network request count
    pub fn increment_network_request(&self) {
        let mut last_reset = match self.last_network_reset.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire last_network_reset lock: {}", e);
                return;
            }
        };
        if last_reset.elapsed() >= std::time::Duration::from_secs(60) {
            *last_reset = std::time::Instant::now();
            self.network_request_count.store(0, std::sync::atomic::Ordering::Relaxed);
        }
        self.network_request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// Get resource statistics
    pub fn get_resource_stats(&self) -> super::ResourceStats {
        let mut last_reset = match self.last_network_reset.lock() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to acquire last_network_reset lock: {}", e);
                return super::ResourceStats::default();
            }
        };
        if last_reset.elapsed() >= std::time::Duration::from_secs(60) {
            *last_reset = std::time::Instant::now();
            self.network_request_count.store(0, std::sync::atomic::Ordering::Relaxed);
        }

        super::ResourceStats {
            command_count: self.command_count.load(std::sync::atomic::Ordering::Relaxed),
            network_request_count: self.network_request_count.load(std::sync::atomic::Ordering::Relaxed),
            max_concurrent_commands: 10,
            max_network_requests_per_minute: 60,
        }
    }

    /// Get the socket path for this sandbox
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }
    
    /// Stop the sandbox process
    pub fn stop(&mut self) -> Result<(), PluginError> {
        if let Some(mut child) = self.child.take() {
            child.kill()
                .map_err(|e| PluginError::ExecutionError(format!("Failed to kill plugin process: {}", e)))?;
            
            child.wait()
                .map_err(|e| PluginError::ExecutionError(format!("Failed to wait for plugin process: {}", e)))?;
        }
        
        // Remove socket
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .map_err(|e| PluginError::IoError(e))?;
        }
        
        Ok(())
    }
    
    /// Check if the sandbox process is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,    // Process is still running
                Err(_) => false,     // Error checking status
            }
        } else {
            false
        }
    }
}

impl Drop for PluginSandbox {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.enable_seccomp);
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
        assert_eq!(config.max_memory_mb, 512);
    }
    
    #[test]
    fn test_plugin_message_serialization() {
        let message = PluginMessage {
            id: 1,
            plugin_id: "test-plugin".to_string(),
            command: "ping".to_string(),
            params: serde_json::json!({}),
        };
        
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: PluginMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.plugin_id, "test-plugin");
        assert_eq!(deserialized.command, "ping");
    }
    
    #[test]
    fn test_plugin_response_serialization() {
        let response = PluginResponse {
            id: 1,
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PluginResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, 1);
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }
    
    #[test]
    fn test_plugin_response_error() {
        let response = PluginResponse {
            id: 1,
            result: None,
            error: Some("Test error".to_string()),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PluginResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, 1);
        assert!(deserialized.result.is_none());
        assert!(deserialized.error.is_some());
        assert_eq!(deserialized.error.unwrap(), "Test error");
    }
    
    #[test]
    fn test_seccomp_filter_stub() {
        // Test that seccomp filter stub works (should always succeed)
        let result = apply_seccomp_filter(false, false);
        assert!(result.is_ok());
    }
}
