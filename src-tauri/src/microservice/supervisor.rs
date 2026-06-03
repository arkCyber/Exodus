//! Service Supervisor for Monitoring and Auto-recovery

use serde::{Deserialize, Serialize};
use super::registry::{ServiceRegistry, ServiceStatus};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Service handle for supervision
#[derive(Debug, Clone)]
pub struct ServiceHandle {
    pub name: String,
    #[allow(dead_code)]
    pub pid: u32,
    #[allow(dead_code)]
    pub restart_count: u32,
    pub max_restarts: u32,
    pub restart_delay: Duration,
}

impl ServiceHandle {
    pub fn new(name: impl Into<String>, pid: u32) -> Self {
        Self {
            name: name.into(),
            pid,
            restart_count: 0,
            max_restarts: 3,
            restart_delay: Duration::from_secs(2),
        }
    }

    pub fn with_max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

    pub fn with_restart_delay(mut self, delay: Duration) -> Self {
        self.restart_delay = delay;
        self
    }

    #[allow(dead_code)]
    pub fn can_restart(&self) -> bool {
        self.restart_count < self.max_restarts
    }

    #[allow(dead_code)]
    pub fn increment_restart(&mut self) {
        self.restart_count += 1;
    }
}

/// Service Supervisor
pub struct ServiceSupervisor {
    registry: Arc<ServiceRegistry>,
    handles: Arc<Mutex<HashMap<String, ServiceHandle>>>,
    #[allow(dead_code)]
    health_check_interval: Duration,
    health_history: Arc<Mutex<HashMap<String, Vec<HealthCheckResult>>>>,
    max_health_history: usize,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub timestamp: u64,
    pub healthy: bool,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

/// Health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStats {
    pub service: String,
    pub total_checks: usize,
    pub healthy_checks: usize,
    pub unhealthy_checks: usize,
    pub health_percentage: f64,
    pub avg_response_time_ms: f64,
}

impl ServiceSupervisor {
    /// Create a new service supervisor
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            handles: Arc::new(Mutex::new(HashMap::new())),
            health_check_interval: Duration::from_secs(10),
            health_history: Arc::new(Mutex::new(HashMap::new())),
            max_health_history: 100,
        }
    }

    /// Set health check interval
    #[allow(dead_code)]
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set max health history
    #[allow(dead_code)]
    pub fn with_max_health_history(mut self, max: usize) -> Self {
        self.max_health_history = max;
        self
    }

    /// Register a service for supervision
    pub async fn register_service(&self, handle: ServiceHandle) -> Result<(), SupervisorError> {
        let mut handles = self.handles.lock().await;
        handles.insert(handle.name.clone(), handle);
        Ok(())
    }

    /// Unregister a service from supervision
    pub async fn unregister_service(&self, name: &str) -> Result<(), SupervisorError> {
        let mut handles = self.handles.lock().await;
        handles.remove(name);
        
        // Clear health history
        let mut history = self.health_history.lock().await;
        history.remove(name);
        
        Ok(())
    }

    /// Perform a health check for a service
    pub async fn health_check(&self, name: &str) -> Result<HealthCheckResult, SupervisorError> {
        let start = std::time::Instant::now();
        
        let service_info = self.registry.get(name)
            .map_err(|e| SupervisorError::LockError(e.to_string()))?
            .ok_or_else(|| SupervisorError::ServiceNotFound(name.to_string()))?;
        
        let healthy = service_info.is_healthy(30);
        let response_time = start.elapsed().as_millis() as u64;
        
        let result = HealthCheckResult {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            healthy,
            response_time_ms: response_time,
            error: if healthy { None } else { Some("Service unhealthy".to_string()) },
        };
        
        // Record health check result
        let mut history = self.health_history.lock().await;
        let service_history = history.entry(name.to_string()).or_insert_with(Vec::new);
        service_history.push(result.clone());
        
        // Trim history if exceeds max
        if service_history.len() > self.max_health_history {
            service_history.remove(0);
        }
        
        Ok(result)
    }

    /// Get health history for a service
    pub async fn get_health_history(&self, name: &str, limit: Option<usize>) -> Vec<HealthCheckResult> {
        let history = self.health_history.lock().await;
        if let Some(service_history) = history.get(name) {
            if let Some(limit) = limit {
                service_history.iter().rev().take(limit).cloned().collect()
            } else {
                service_history.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// Get health statistics for a service
    pub async fn get_health_stats(&self, name: &str) -> Option<HealthStats> {
        let history = self.health_history.lock().await;
        let service_history = history.get(name)?;
        
        if service_history.is_empty() {
            return None;
        }
        
        let total_checks = service_history.len();
        let healthy_checks = service_history.iter().filter(|r| r.healthy).count();
        let avg_response_time: f64 = service_history.iter()
            .map(|r| r.response_time_ms as f64)
            .sum::<f64>() / total_checks as f64;
        
        Some(HealthStats {
            service: name.to_string(),
            total_checks,
            healthy_checks,
            unhealthy_checks: total_checks - healthy_checks,
            health_percentage: (healthy_checks as f64 / total_checks as f64) * 100.0,
            avg_response_time_ms: avg_response_time,
        })
    }

    /// Start supervision loop
    #[allow(dead_code)]
    pub async fn start_supervision(&self) -> Result<(), SupervisorError> {
        let registry = Arc::clone(&self.registry);
        let handles = Arc::clone(&self.handles);
        let health_history = Arc::clone(&self.health_history);
        let max_health_history = self.max_health_history;
        let interval = self.health_check_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;

                let handle_names: Vec<String> = {
                    let handles_guard = handles.lock().await;
                    handles_guard.keys().cloned().collect()
                };

                for name in handle_names {
                    // Perform health check
                    let start = std::time::Instant::now();
                    let service_info = match registry.get(&name) {
                        Ok(Some(info)) => info,
                        Ok(None) => continue,
                        Err(_) => continue,
                    };
                    
                    let healthy = service_info.is_healthy(30);
                    let response_time = start.elapsed().as_millis() as u64;
                    
                    let result = HealthCheckResult {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs(),
                        healthy,
                        response_time_ms: response_time,
                        error: if healthy { None } else { Some("Service unhealthy".to_string()) },
                    };
                    
                    // Record health check result
                    {
                        let mut history = health_history.lock().await;
                        let service_history = history.entry(name.clone()).or_insert_with(Vec::new);
                        service_history.push(result.clone());
                        
                        // Trim history if exceeds max
                        if service_history.len() > max_health_history {
                            service_history.remove(0);
                        }
                    }
                    
                    if !result.healthy {
                        let restart_count = {
                            let mut handles_guard = handles.lock().await;
                            if let Some(handle) = handles_guard.get_mut(&name) {
                                if handle.can_restart() {
                                    handle.increment_restart();
                                    handle.restart_count
                                } else {
                                    tracing::error!("Service {} has exceeded max restart attempts", name);
                                    let _ = registry.update_status(&name, ServiceStatus::Error);
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        };
                        
                        // Update status to stopping
                        let _ = registry.update_status(&name, ServiceStatus::Stopping);
                        
                        // Get the service info to find the PID
                        if let Ok(Some(service_info)) = registry.get(&name) {
                            // Kill the existing process if it has a PID
                            let pid = service_info.pid;
                            if pid > 0 {
                                tracing::info!("Killing process {} for service {}", pid, name);
                                #[cfg(unix)]
                                {
                                    use std::process::Command;
                                    let _ = Command::new("kill")
                                        .arg("-TERM")
                                        .arg(pid.to_string())
                                        .output();
                                }
                                #[cfg(windows)]
                                {
                                    use std::process::Command;
                                    let _ = Command::new("taskkill")
                                        .arg("/PID")
                                        .arg(pid.to_string())
                                        .arg("/F")
                                        .output();
                                }
                                
                                // Wait for graceful shutdown
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                            
                            // Mark as stopped
                            let _ = registry.update_status(&name, ServiceStatus::Stopped);
                        }
                        
                        tracing::warn!("Service {} is unhealthy, attempting restart (attempt {})", name, restart_count);
                    }
                }
            }
        });

        Ok(())
    }

    /// Restart a service
    #[allow(dead_code)]
    pub async fn restart_service(&self, name: &str) -> Result<(), SupervisorError> {
        tracing::info!("Restarting service: {}", name);
        
        // Update status to stopping
        let _ = self.registry.update_status(name, ServiceStatus::Stopping);
        
        // Get the service info to find the PID
        let service_info = self.registry.get(name)
            .map_err(|e| SupervisorError::LockError(e.to_string()))?
            .ok_or_else(|| SupervisorError::ServiceNotFound(name.to_string()))?;
        
        // Kill the existing process if it has a PID
        let pid = service_info.pid;
        if pid > 0 {
            tracing::info!("Killing process {} for service {}", pid, name);
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();
            }
            #[cfg(windows)]
            {
                use std::process::Command;
                let _ = Command::new("taskkill")
                    .arg("/PID")
                    .arg(pid.to_string())
                    .arg("/F")
                    .output();
            }
            
            // Wait for graceful shutdown
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        
        // For now, just mark as stopped and ready to restart
        // In a real implementation, we would start a new process here
        let _ = self.registry.update_status(name, ServiceStatus::Stopped);
        
        Ok(())
    }

    /// Stop supervision for a service
    pub async fn stop_service(&self, name: &str) -> Result<(), SupervisorError> {
        tracing::info!("Stopping service: {}", name);
        
        let _ = self.registry.update_status(name, ServiceStatus::Stopping);
        let _ = self.registry.update_status(name, ServiceStatus::Stopped);
        
        Ok(())
    }

    /// Get supervision status for a service
    #[allow(dead_code)]
    pub async fn get_service_status(&self, name: &str) -> Result<Option<SupervisionStatus>, SupervisorError> {
        let handles = self.handles.lock().await;
        let handle = handles.get(name).cloned();
        
        let service_info = match self.registry.get(name) {
            Ok(Some(info)) => Some(info.status),
            Ok(None) => None,
            Err(_) => None,
        };
        
        Ok(handle.map(|h| SupervisionStatus {
            name: h.name,
            pid: h.pid,
            restart_count: h.restart_count,
            max_restarts: h.max_restarts,
            service_status: service_info,
        }))
    }
}

/// Supervision status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SupervisionStatus {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub pid: u32,
    #[allow(dead_code)]
    pub restart_count: u32,
    #[allow(dead_code)]
    pub max_restarts: u32,
    #[allow(dead_code)]
    pub service_status: Option<ServiceStatus>,
}

/// Supervisor error
#[derive(Debug, Clone)]
pub enum SupervisorError {
    ServiceNotFound(String),
    #[allow(dead_code)]
    RestartFailed(String),
    LockError(String),
}

impl std::fmt::Display for SupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            Self::RestartFailed(msg) => write!(f, "Restart failed: {}", msg),
            Self::LockError(msg) => write!(f, "Lock error: {}", msg),
        }
    }
}

impl std::error::Error for SupervisorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_handle() {
        let handle = ServiceHandle::new("test-service", 1234);
        assert_eq!(handle.name, "test-service");
        assert_eq!(handle.pid, 1234);
        assert!(handle.can_restart());
        
        let mut handle = handle;
        handle.increment_restart();
        assert_eq!(handle.restart_count, 1);
    }

    #[test]
    fn test_service_handle_with_options() {
        let handle = ServiceHandle::new("test-service", 1234)
            .with_max_restarts(5)
            .with_restart_delay(Duration::from_secs(5));
        
        assert_eq!(handle.max_restarts, 5);
        assert_eq!(handle.restart_delay, Duration::from_secs(5));
    }

    #[test]
    fn test_service_handle_can_restart() {
        let handle = ServiceHandle::new("test-service", 1234)
            .with_max_restarts(3);
        
        assert!(handle.can_restart());
        
        let mut handle = handle;
        handle.increment_restart();
        handle.increment_restart();
        handle.increment_restart();
        
        assert!(!handle.can_restart());
    }

    #[test]
    fn test_supervisor_error_display() {
        let error = SupervisorError::ServiceNotFound("test-service".to_string());
        assert_eq!(error.to_string(), "Service not found: test-service");

        let error = SupervisorError::RestartFailed("failed to restart".to_string());
        assert_eq!(error.to_string(), "Restart failed: failed to restart");
    }
}
