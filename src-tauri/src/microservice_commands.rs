//! Tauri commands for microservice management

use crate::microservice::{ServiceRegistry, ServiceSupervisor, ServiceStatus, ServiceInfo, supervisor::ServiceHandle, log_collector::{LogCollector, LogEntry}, resource_monitor::ResourceMonitor};
use std::sync::Arc;

/// Register a service in the registry
#[tauri::command]
pub async fn microservice_register(
    name: String,
    socket_path: String,
    pid: u32,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<(), String> {
    let service = ServiceInfo::new(name.clone(), socket_path, pid);
    registry.register(service)
        .map_err(|e| format!("Failed to register service: {}", e))?;
    
    registry.update_status(&name, ServiceStatus::Running)
        .map_err(|e| format!("Failed to update status: {}", e))?;
    
    Ok(())
}

/// Unregister a service
#[tauri::command]
pub async fn microservice_unregister(
    name: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<bool, String> {
    let removed = registry.unregister(&name)
        .map_err(|e| format!("Failed to unregister service: {}", e))?;
    
    Ok(removed.is_some())
}

/// List all registered services
#[tauri::command]
pub async fn microservice_list(
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let services = registry.list()
        .map_err(|e| format!("Failed to list services: {}", e))?;
    
    let json = serde_json::to_string(&services)
        .map_err(|e| format!("Failed to serialize services: {}", e))?;
    
    Ok(json)
}

/// Get service status
#[tauri::command]
pub async fn microservice_status(
    name: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    let service = registry.get(&name)
        .map_err(|e| format!("Failed to get service: {}", e))?
        .ok_or_else(|| format!("Service not found: {}", name))?;
    
    let status = format!(
        "{{\"name\":\"{}\",\"status\":\"{:?}\",\"socket\":\"{}\",\"pid\":{},\"heartbeat\":{},\"version\":\"{}\"}}",
        service.name, service.status, service.socket_path, service.pid, service.last_heartbeat, service.version
    );
    
    Ok(status)
}

/// Update service heartbeat
#[tauri::command]
pub async fn microservice_heartbeat(
    name: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<(), String> {
    registry.update_heartbeat(&name)
        .map_err(|e| format!("Failed to update heartbeat: {}", e))?;
    
    Ok(())
}

/// Health check for all services
#[tauri::command]
pub async fn microservice_health_check_all(
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<Vec<(String, bool)>, String> {
    let services = registry.list()
        .map_err(|e| format!("Failed to list services: {}", e))?;
    
    let mut results = Vec::new();
    for service in services {
        let healthy = registry.check_health(&service.name, 30)
            .unwrap_or(false);
        results.push((service.name, healthy));
    }
    
    Ok(results)
}

/// Start a microservice
#[tauri::command]
pub async fn microservice_start(
    name: String,
    binary_path: String,
    socket_path: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
    supervisor: tauri::State<'_, Arc<ServiceSupervisor>>,
) -> Result<u32, String> {
    // Check if service already exists
    if let Ok(Some(_)) = registry.get(&name) {
        return Err(format!("Service '{}' is already registered", name));
    }
    
    // Verify binary path exists
    if !std::path::Path::new(&binary_path).exists() {
        return Err(format!("Binary path does not exist: {}", binary_path));
    }
    
    // Remove existing socket if it exists
    if std::path::Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)
            .map_err(|e| format!("Failed to remove existing socket: {}", e))?;
    }
    
    // Create socket directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create socket directory: {}", e))?;
    }
    
    // Start the microservice process
    let mut cmd = std::process::Command::new(&binary_path);
    cmd.arg("--socket-path")
       .arg(&socket_path)
       .arg("--service-name")
       .arg(&name);
    
    // Set up stdout/stderr redirection for logging
    #[cfg(unix)]
    {
        use std::process::Stdio;
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
    }
    
    // Spawn the process
    let child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn microservice process: {}", e))?;
    
    let pid = child.id();
    
    // Register the service with the registry
    let service = ServiceInfo::new(name.clone(), socket_path, pid);
    registry.register(service)
        .map_err(|e| format!("Failed to register service: {}", e))?;
    
    // Update status to starting
    registry.update_status(&name, crate::microservice::ServiceStatus::Starting)
        .map_err(|e| format!("Failed to update status: {}", e))?;
    
    // Add to supervisor for health monitoring
    let handle = ServiceHandle::new(name.clone(), pid)
        .with_max_restarts(5)
        .with_restart_delay(std::time::Duration::from_secs(3));
    supervisor.register_service(handle)
        .await
        .map_err(|e| format!("Failed to add service to supervisor: {}", e))?;
    
    // Wait a moment for the service to start
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // Check if process is still alive
    #[cfg(unix)]
    {
        use std::process::Command;
        let result = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                registry.update_status(&name, crate::microservice::ServiceStatus::Running)
                    .map_err(|e| format!("Failed to update status: {}", e))?;
            } else {
                registry.update_status(&name, crate::microservice::ServiceStatus::Error)
                    .map_err(|e| format!("Failed to update status: {}", e))?;
                return Err(format!("Service '{}' failed to start", name));
            }
        } else {
            registry.update_status(&name, crate::microservice::ServiceStatus::Error)
                .map_err(|e| format!("Failed to update status: {}", e))?;
            return Err(format!("Service '{}' failed to start", name));
        }
    }
    
    #[cfg(windows)]
    {
        use std::process::Command;
        let result = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                registry.update_status(&name, crate::microservice::ServiceStatus::Running)
                    .map_err(|e| format!("Failed to update status: {}", e))?;
            } else {
                registry.update_status(&name, crate::microservice::ServiceStatus::Error)
                    .map_err(|e| format!("Failed to update status: {}", e))?;
                return Err(format!("Service '{}' failed to start", name));
            }
        } else {
            registry.update_status(&name, crate::microservice::ServiceStatus::Error)
                .map_err(|e| format!("Failed to update status: {}", e))?;
            return Err(format!("Service '{}' failed to start", name));
        }
    }
    
    Ok(pid)
}

/// Stop a microservice
#[tauri::command]
pub async fn microservice_stop(
    name: String,
    force: bool,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
    supervisor: tauri::State<'_, Arc<ServiceSupervisor>>,
) -> Result<bool, String> {
    // Check if service exists
    let service_info = registry.get(&name)
        .map_err(|e| format!("Failed to get service: {}", e))?
        .ok_or_else(|| format!("Service not found: {}", name))?;
    
    // Update status to stopping
    registry.update_status(&name, crate::microservice::ServiceStatus::Stopping)
        .map_err(|e| format!("Failed to update status: {}", e))?;
    
    // Stop supervision first
    let _ = supervisor.stop_service(&name).await;
    
    // Kill the process
    let pid = service_info.pid;
    if pid > 0 {
        #[cfg(unix)]
        {
            use std::process::Command;
            let signal = if force { "-9" } else { "-TERM" };
            let result = Command::new("kill")
                .arg(signal)
                .arg(pid.to_string())
                .output();
            
            match result {
                Ok(output) => {
                    if !output.status.success() {
                        tracing::warn!("Kill command failed for PID {}: {:?}", pid, output);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute kill command: {}", e);
                }
            }
            
            // Wait for graceful shutdown if not forced
            if !force {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                
                // Check if process is still alive
                let check_result = Command::new("ps")
                    .arg("-p")
                    .arg(pid.to_string())
                    .output();
                
                if let Ok(output) = check_result {
                    if output.status.success() {
                        // Force kill if graceful shutdown failed
                        let _ = Command::new("kill")
                            .arg("-9")
                            .arg(pid.to_string())
                            .output();
                        tracing::warn!("Force killed service {} after graceful shutdown timeout", name);
                    }
                }
            }
        }
        
        #[cfg(windows)]
        {
            use std::process::Command;
            let args = if force {
                vec!["/PID", &pid.to_string(), "/F"]
            } else {
                vec!["/PID", &pid.to_string()]
            };
            
            let result = Command::new("taskkill")
                .args(&args)
                .output();
            
            match result {
                Ok(output) => {
                    if !output.status.success() {
                        tracing::warn!("Taskkill command failed for PID {}: {:?}", pid, output);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute taskkill command: {}", e);
                }
            }
            
            // Wait for graceful shutdown if not forced
            if !force {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                
                // Check if process is still alive
                let check_result = Command::new("tasklist")
                    .arg("/FI")
                    .arg(format!("PID eq {}", pid))
                    .output();
                
                if let Ok(output) = check_result {
                    if output.status.success() {
                        // Force kill if graceful shutdown failed
                        let _ = Command::new("taskkill")
                            .args(["/PID", &pid.to_string(), "/F"])
                            .output();
                        tracing::warn!("Force killed service {} after graceful shutdown timeout", name);
                    }
                }
            }
        }
    }
    
    // Remove socket file if it exists
    if std::path::Path::new(&service_info.socket_path).exists() {
        std::fs::remove_file(&service_info.socket_path)
            .map_err(|e| {
                tracing::warn!("Failed to remove socket file: {}", e);
                format!("Failed to remove socket file: {}", e)
            })?;
    }
    
    // Update status to stopped
    registry.update_status(&name, crate::microservice::ServiceStatus::Stopped)
        .map_err(|e| format!("Failed to update status: {}", e))?;
    
    // Unregister from registry
    let removed = registry.unregister(&name)
        .map_err(|e| format!("Failed to unregister service: {}", e))?;
    
    // Unregister from supervisor
    let _ = supervisor.unregister_service(&name).await;
    
    Ok(removed.is_some())
}

/// Get microservice socket directory
#[tauri::command]
pub async fn microservice_socket_dir(
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    Ok(registry.socket_dir().to_string_lossy().to_string())
}

/// Get logs for a microservice
#[tauri::command]
pub async fn microservice_get_logs(
    service: String,
    limit: Option<usize>,
    log_collector: tauri::State<'_, Arc<LogCollector>>,
) -> Result<String, String> {
    let logs = log_collector.get_logs(&service, limit);
    serde_json::to_string(&logs)
        .map_err(|e| format!("Failed to serialize logs: {}", e))
}

/// Add a log entry
#[tauri::command]
pub async fn microservice_add_log(
    entry: LogEntry,
    log_collector: tauri::State<'_, Arc<LogCollector>>,
) -> Result<(), String> {
    log_collector.add_log(entry);
    Ok(())
}

/// Save logs to file
#[tauri::command]
pub async fn microservice_save_logs(
    service: String,
    log_collector: tauri::State<'_, Arc<LogCollector>>,
) -> Result<String, String> {
    let log_file = log_collector.save_logs_to_file(&service)
        .map_err(|e| format!("Failed to save logs: {}", e))?;
    Ok(log_file.to_string_lossy().to_string())
}

/// Clear logs for a service
#[tauri::command]
pub async fn microservice_clear_logs(
    service: String,
    log_collector: tauri::State<'_, Arc<LogCollector>>,
) -> Result<(), String> {
    log_collector.clear_logs(&service);
    Ok(())
}

/// Clear all logs
#[tauri::command]
pub async fn microservice_clear_all_logs(
    log_collector: tauri::State<'_, Arc<LogCollector>>,
) -> Result<(), String> {
    log_collector.clear_all_logs();
    Ok(())
}

/// Get all logs
#[tauri::command]
pub async fn microservice_get_all_logs(
    log_collector: tauri::State<'_, Arc<LogCollector>>,
) -> Result<String, String> {
    let logs = log_collector.get_all_logs();
    serde_json::to_string(&logs)
        .map_err(|e| format!("Failed to serialize logs: {}", e))
}

/// Collect resource metrics for a service
#[tauri::command]
pub async fn microservice_collect_metrics(
    service: String,
    pid: u32,
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<String, String> {
    let metrics = resource_monitor.collect_metrics(service, pid)
        .map_err(|e| format!("Failed to collect metrics: {}", e))?;
    serde_json::to_string(&metrics)
        .map_err(|e| format!("Failed to serialize metrics: {}", e))
}

/// Get resource metrics for a service
#[tauri::command]
pub async fn microservice_get_metrics(
    service: String,
    limit: Option<usize>,
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<String, String> {
    let metrics = resource_monitor.get_metrics(&service, limit);
    serde_json::to_string(&metrics)
        .map_err(|e| format!("Failed to serialize metrics: {}", e))
}

/// Get latest resource metrics for a service
#[tauri::command]
pub async fn microservice_get_latest_metrics(
    service: String,
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<String, String> {
    let metrics = resource_monitor.get_latest_metrics(&service)
        .ok_or_else(|| format!("No metrics found for service: {}", service))?;
    serde_json::to_string(&metrics)
        .map_err(|e| format!("Failed to serialize metrics: {}", e))
}

/// Get all resource metrics
#[tauri::command]
pub async fn microservice_get_all_metrics(
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<String, String> {
    let metrics = resource_monitor.get_all_metrics();
    serde_json::to_string(&metrics)
        .map_err(|e| format!("Failed to serialize metrics: {}", e))
}

/// Get average resource usage for a service
#[tauri::command]
pub async fn microservice_get_average_usage(
    service: String,
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<String, String> {
    let usage = resource_monitor.get_average_usage(&service)
        .ok_or_else(|| format!("No metrics found for service: {}", service))?;
    serde_json::to_string(&usage)
        .map_err(|e| format!("Failed to serialize usage: {}", e))
}

/// Clear resource metrics for a service
#[tauri::command]
pub async fn microservice_clear_metrics(
    service: String,
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<(), String> {
    resource_monitor.clear_metrics(&service);
    Ok(())
}

/// Clear all resource metrics
#[tauri::command]
pub async fn microservice_clear_all_metrics(
    resource_monitor: tauri::State<'_, Arc<ResourceMonitor>>,
) -> Result<(), String> {
    resource_monitor.clear_all_metrics();
    Ok(())
}

/// Perform health check for a service
#[tauri::command]
pub async fn microservice_health_check(
    name: String,
    supervisor: tauri::State<'_, Arc<ServiceSupervisor>>,
) -> Result<String, String> {
    let result = supervisor.health_check(&name).await
        .map_err(|e| format!("Health check failed: {}", e))?;
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize health check result: {}", e))
}

/// Get health history for a service
#[tauri::command]
pub async fn microservice_get_health_history(
    name: String,
    limit: Option<usize>,
    supervisor: tauri::State<'_, Arc<ServiceSupervisor>>,
) -> Result<String, String> {
    let history = supervisor.get_health_history(&name, limit).await;
    serde_json::to_string(&history)
        .map_err(|e| format!("Failed to serialize health history: {}", e))
}

/// Get health statistics for a service
#[tauri::command]
pub async fn microservice_get_health_stats(
    name: String,
    supervisor: tauri::State<'_, Arc<ServiceSupervisor>>,
) -> Result<String, String> {
    let stats = supervisor.get_health_stats(&name).await
        .ok_or_else(|| format!("No health stats found for service: {}", name))?;
    serde_json::to_string(&stats)
        .map_err(|e| format!("Failed to serialize health stats: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microservice::{log_collector::LogEntry, resource_monitor::ResourceMetrics, supervisor::HealthCheckResult};
    
    #[test]
    fn test_command_signatures() {
        // Verify command signatures are valid
        // This is a compile-time check that the commands exist
        let _ = microservice_register;
        let _ = microservice_unregister;
        let _ = microservice_list;
        let _ = microservice_status;
        let _ = microservice_heartbeat;
        let _ = microservice_health_check_all;
        let _ = microservice_start;
        let _ = microservice_stop;
        let _ = microservice_socket_dir;
        let _ = microservice_get_logs;
        let _ = microservice_add_log;
        let _ = microservice_save_logs;
        let _ = microservice_clear_logs;
        let _ = microservice_clear_all_logs;
        let _ = microservice_get_all_logs;
        let _ = microservice_collect_metrics;
        let _ = microservice_get_metrics;
        let _ = microservice_get_latest_metrics;
        let _ = microservice_get_all_metrics;
        let _ = microservice_get_average_usage;
        let _ = microservice_clear_metrics;
        let _ = microservice_clear_all_metrics;
        let _ = microservice_health_check;
        let _ = microservice_get_health_history;
        let _ = microservice_get_health_stats;
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            timestamp: 1234567890,
            level: "INFO".to_string(),
            message: "Test message".to_string(),
            service: "test-service".to_string(),
        };
        
        let json = serde_json::to_string(&entry).expect("Failed to serialize LogEntry");
        let deserialized: LogEntry = serde_json::from_str(&json).expect("Failed to deserialize LogEntry");
        
        assert_eq!(deserialized.timestamp, entry.timestamp);
        assert_eq!(deserialized.level, entry.level);
        assert_eq!(deserialized.message, entry.message);
        assert_eq!(deserialized.service, entry.service);
    }

    #[test]
    fn test_resource_metrics_serialization() {
        let metrics = ResourceMetrics {
            service: "test-service".to_string(),
            pid: 1234,
            cpu_percent: 10.5,
            memory_mb: 100.0,
            memory_percent: 1.0,
            thread_count: 5,
            file_descriptors: 10,
            timestamp: 1234567890,
        };
        
        let json = serde_json::to_string(&metrics).expect("Failed to serialize ResourceMetrics");
        let deserialized: ResourceMetrics = serde_json::from_str(&json).expect("Failed to deserialize ResourceMetrics");
        
        assert_eq!(deserialized.service, metrics.service);
        assert_eq!(deserialized.pid, metrics.pid);
        assert_eq!(deserialized.cpu_percent, metrics.cpu_percent);
        assert_eq!(deserialized.memory_mb, metrics.memory_mb);
    }

    #[test]
    fn test_health_check_result_serialization() {
        let result = HealthCheckResult {
            timestamp: 1234567890,
            healthy: true,
            response_time_ms: 100,
            error: None,
        };
        
        let json = serde_json::to_string(&result).expect("Failed to serialize HealthCheckResult");
        let deserialized: HealthCheckResult = serde_json::from_str(&json).expect("Failed to deserialize HealthCheckResult");
        
        assert_eq!(deserialized.timestamp, result.timestamp);
        assert_eq!(deserialized.healthy, result.healthy);
        assert_eq!(deserialized.response_time_ms, result.response_time_ms);
    }
}
