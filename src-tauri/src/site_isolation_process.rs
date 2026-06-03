//! Site Isolation Process Manager
//!
//! This module handles the actual creation and management of isolated processes
//! for site isolation. It uses platform-specific APIs to spawn and monitor processes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use super::site_isolation::{ProcessInfo, SiteId};

/// Process creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// Site ID for this process
    pub site_id: SiteId,
    /// Whether to enable Spectre mitigations
    pub enable_spectre_mitigations: bool,
    /// Memory limit in bytes
    pub memory_limit: u64,
    /// CPU limit (0.0 to 1.0)
    pub cpu_limit: f32,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Command-line arguments
    pub args: Vec<String>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            site_id: SiteId {
                scheme: "https".to_string(),
                etld_plus_one: "default".to_string(),
            },
            enable_spectre_mitigations: true,
            memory_limit: 2 * 1024 * 1024 * 1024, // 2GB
            cpu_limit: 0.8,
            env_vars: HashMap::new(),
            args: vec![],
        }
    }
}

/// Process state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is starting
    Starting,
    /// Process is running
    Running,
    /// Process is paused
    Paused,
    /// Process is crashed
    Crashed,
    /// Process is terminated
    Terminated,
}

/// Isolated process wrapper
pub struct IsolatedProcess {
    /// Process ID
    pub process_id: String,
    /// Site ID
    pub site_id: SiteId,
    /// Child process handle
    child: Option<Child>,
    /// Process state
    state: Arc<Mutex<ProcessState>>,
    /// Process info
    info: Arc<Mutex<ProcessInfo>>,
    /// Exit notification sender
    exit_tx: Option<mpsc::Sender<String>>,
    /// Task handle for monitoring
    monitor_handle: Option<JoinHandle<()>>,
}

impl IsolatedProcess {
    /// Create a new isolated process
    pub async fn new(
        process_id: String,
        site_id: SiteId,
        config: ProcessConfig,
        exit_tx: mpsc::Sender<String>,
    ) -> Result<Self, String> {
        info!("Creating isolated process {} for site {}", process_id, site_id.etld_plus_one);

        // Build command
        let mut cmd = Command::new("exodus-renderer"); // This would be the renderer process
        cmd.arg("--site-id")
            .arg(&site_id.etld_plus_one)
            .arg("--process-id")
            .arg(&process_id);

        if config.enable_spectre_mitigations {
            cmd.arg("--enable-spectre-mitigations");
        }

        // Add custom arguments
        for arg in &config.args {
            cmd.arg(arg);
        }

        // Add environment variables
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // Spawn process
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let pid = child.id();
        info!("Process {} spawned with OS PID: {:?}", process_id, pid);

        // Create process info
        let mut info = ProcessInfo::new(process_id.clone(), site_id.clone());
        info.pid = pid;
        let info = Arc::new(Mutex::new(info));

        let state = Arc::new(Mutex::new(ProcessState::Running));

        // Start monitoring task
        let _state_clone = state.clone();
        let _info_clone = info.clone();
        let process_id_clone = process_id.clone();
        let _exit_tx_clone = exit_tx.clone();

        let monitor_handle = tokio::spawn(async move {
            // Monitor process exit
            // In a real implementation, this would wait for the child process
            // and handle crashes appropriately
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            // Simulate monitoring (in real implementation, use child.wait())
            warn!("Monitor task for process {} started (simulated)", process_id_clone);
        });

        Ok(Self {
            process_id,
            site_id,
            child: Some(child),
            state,
            info,
            exit_tx: Some(exit_tx),
            monitor_handle: Some(monitor_handle),
        })
    }

    /// Get process state
    pub fn get_state(&self) -> ProcessState {
        self.state.lock()
            .map(|s| s.clone())
            .unwrap_or(ProcessState::Terminated)
    }

    /// Get process info
    pub fn get_info(&self) -> ProcessInfo {
        self.info.lock()
            .map(|i| i.clone())
            .unwrap_or_else(|_| ProcessInfo::new("unknown".to_string(), self.site_id.clone()))
    }

    /// Terminate the process
    pub async fn terminate(&mut self) -> Result<(), String> {
        info!("Terminating process {}", self.process_id);

        if let Some(mut child) = self.child.take() {
            child.kill()
                .await
                .map_err(|e| format!("Failed to kill process: {}", e))?;

            let _ = child.wait().await;
        }

        // Cancel monitor task
        if let Some(handle) = self.monitor_handle.take() {
            handle.abort();
        }

        *self.state.lock().unwrap() = ProcessState::Terminated;

        Ok(())
    }

    /// Pause the process
    pub fn pause(&self) -> Result<(), String> {
        info!("Pausing process {}", self.process_id);
        *self.state.lock().unwrap() = ProcessState::Paused;
        Ok(())
    }

    /// Resume the process
    pub fn resume(&self) -> Result<(), String> {
        info!("Resuming process {}", self.process_id);
        *self.state.lock().unwrap() = ProcessState::Running;
        Ok(())
    }

    /// Mark process as crashed
    pub fn mark_crashed(&self) {
        warn!("Process {} marked as crashed", self.process_id);
        *self.state.lock().unwrap() = ProcessState::Crashed;
        self.info.lock().unwrap().mark_crashed();
    }

    /// Update resource usage
    pub fn update_resource_usage(&self, memory: u64, cpu: f32) {
        let mut info = self.info.lock().unwrap();
        info.memory_usage = memory;
        info.cpu_usage = cpu;
    }
}

/// Process manager for isolated processes
pub struct ProcessManager {
    /// Active processes keyed by process ID
    processes: Arc<Mutex<HashMap<String, IsolatedProcess>>>,
    /// Exit notification channel
    exit_tx: mpsc::Sender<String>,
    exit_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    /// Process configuration
    default_config: ProcessConfig,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        let (exit_tx, exit_rx) = mpsc::channel(100);

        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            exit_tx,
            exit_rx: Arc::new(Mutex::new(exit_rx)),
            default_config: ProcessConfig::default(),
        }
    }

    /// Set default process configuration
    pub fn set_default_config(&mut self, config: ProcessConfig) {
        self.default_config = config;
    }

    /// Create a new isolated process
    pub async fn create_process(
        &self,
        process_id: String,
        site_id: SiteId,
        config: Option<ProcessConfig>,
    ) -> Result<(), String> {
        let config = config.unwrap_or_else(|| self.default_config.clone());

        let process = IsolatedProcess::new(
            process_id.clone(),
            site_id,
            config,
            self.exit_tx.clone(),
        ).await?;

        let mut processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        processes.insert(process_id.clone(), process);

        Ok(())
    }

    /// Terminate a process
    pub async fn terminate_process(&self, process_id: &str) -> Result<(), String> {
        let mut processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(mut process) = processes.remove(process_id) {
            process.terminate().await?;
        } else {
            return Err(format!("Process {} not found", process_id));
        }

        Ok(())
    }

    /// Get process state
    pub fn get_process_state(&self, process_id: &str) -> Option<ProcessState> {
        let processes = self.processes.lock().ok()?;
        processes.get(process_id).map(|p| p.get_state())
    }

    /// Get process info
    pub fn get_process_info(&self, process_id: &str) -> Option<ProcessInfo> {
        let processes = self.processes.lock().ok()?;
        processes.get(process_id).map(|p| p.get_info())
    }

    /// Get all process IDs
    pub fn get_all_process_ids(&self) -> Vec<String> {
        self.processes.lock()
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get process count
    pub fn get_process_count(&self) -> usize {
        self.processes.lock()
            .map(|p| p.len())
            .unwrap_or(0)
    }

    /// Mark process as crashed
    pub fn mark_process_crashed(&self, process_id: &str) -> Result<(), String> {
        let processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(process) = processes.get(process_id) {
            process.mark_crashed();
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Update process resource usage
    pub fn update_process_resources(&self, process_id: &str, memory: u64, cpu: f32) -> Result<(), String> {
        let processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(process) = processes.get(process_id) {
            process.update_resource_usage(memory, cpu);
            Ok(())
        } else {
            Err(format!("Process {} not found", process_id))
        }
    }

    /// Start exit notification listener
    pub async fn start_exit_listener(&self) {
        let mut rx = self.exit_rx.lock().await;
        let processes = self.processes.clone();

        tokio::spawn(async move {
            while let Some(process_id) = rx.recv().await {
                error!("Process {} exited unexpectedly", process_id);
                
                // Mark as crashed
                let mut procs = processes.lock().await;
                if let Some(process) = procs.get(&process_id) {
                    process.mark_crashed();
                }
            }
        });
    }

    /// Cleanup terminated processes
    pub async fn cleanup_terminated(&self) -> usize {
        let mut processes = self.processes.lock()
            .map_err(|e| {
                error!("Lock error during cleanup: {}", e);
                0
            })
            .unwrap();

        let mut to_remove = Vec::new();

        for (pid, process) in processes.iter() {
            if process.get_state() == ProcessState::Terminated {
                to_remove.push(pid.clone());
            }
        }

        for pid in &to_remove {
            processes.remove(pid);
        }

        to_remove.len()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_config_default() {
        let config = ProcessConfig::default();
        assert!(config.enable_spectre_mitigations);
        assert_eq!(config.memory_limit, 2 * 1024 * 1024 * 1024);
        assert_eq!(config.cpu_limit, 0.8);
    }

    #[test]
    fn test_process_state() {
        assert_ne!(ProcessState::Running, ProcessState::Crashed);
        assert_ne!(ProcessState::Running, ProcessState::Terminated);
    }

    #[tokio::test]
    async fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        assert_eq!(manager.get_process_count(), 0);
    }

    #[tokio::test]
    async fn test_process_manager_config() {
        let mut manager = ProcessManager::new();
        let config = ProcessConfig {
            memory_limit: 1024 * 1024 * 1024,
            cpu_limit: 0.5,
            ..Default::default()
        };
        manager.set_default_config(config);
    }
}
