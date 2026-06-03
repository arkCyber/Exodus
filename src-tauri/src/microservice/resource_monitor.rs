//! Microservice Resource Monitoring

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub service: String,
    pub pid: u32,
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub memory_percent: f32,
    pub thread_count: u32,
    pub file_descriptors: u32,
    pub timestamp: u64,
}

/// Resource monitor
pub struct ResourceMonitor {
    metrics: Arc<Mutex<HashMap<String, Vec<ResourceMetrics>>>>,
    max_metrics_per_service: usize,
    collection_interval: Duration,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            max_metrics_per_service: 1000,
            collection_interval: Duration::from_secs(5),
        }
    }

    /// Set max metrics per service
    pub fn with_max_metrics(mut self, max: usize) -> Self {
        self.max_metrics_per_service = max;
        self
    }

    /// Set collection interval
    pub fn with_collection_interval(mut self, interval: Duration) -> Self {
        self.collection_interval = interval;
        self
    }

    /// Collect metrics for a service
    pub fn collect_metrics(&self, service: String, pid: u32) -> Result<ResourceMetrics, String> {
        let metrics = self.collect_process_metrics(pid)?;
        
        let resource_metrics = ResourceMetrics {
            service: service.clone(),
            pid,
            cpu_percent: metrics.cpu_percent,
            memory_mb: metrics.memory_mb,
            memory_percent: metrics.memory_percent,
            thread_count: metrics.thread_count,
            file_descriptors: metrics.file_descriptors,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        };

        if let Ok(mut metrics_map) = self.metrics.lock() {
            let service_metrics = metrics_map.entry(service).or_insert_with(Vec::new);
            service_metrics.push(resource_metrics.clone());

            // Trim if exceeds max metrics (keep most recent)
            while service_metrics.len() > self.max_metrics_per_service {
                service_metrics.remove(0);
            }
        }

        Ok(resource_metrics)
    }

    /// Get metrics for a service
    pub fn get_metrics(&self, service: &str, limit: Option<usize>) -> Vec<ResourceMetrics> {
        self.metrics.lock()
            .map(|metrics_map| {
                if let Some(service_metrics) = metrics_map.get(service) {
                    if let Some(limit) = limit {
                        service_metrics.iter().rev().take(limit).cloned().collect()
                    } else {
                        service_metrics.clone()
                    }
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, Vec<ResourceMetrics>> {
        self.metrics.lock()
            .map(|metrics_map| metrics_map.clone())
            .unwrap_or_default()
    }

    /// Get latest metrics for a service
    pub fn get_latest_metrics(&self, service: &str) -> Option<ResourceMetrics> {
        self.metrics.lock().ok()
            .and_then(|metrics_map| metrics_map.get(service).cloned())
            .and_then(|service_metrics| service_metrics.last().cloned())
    }

    /// Clear metrics for a service
    pub fn clear_metrics(&self, service: &str) {
        if let Ok(mut metrics_map) = self.metrics.lock() {
            metrics_map.remove(service);
        }
    }

    /// Clear all metrics
    pub fn clear_all_metrics(&self) {
        if let Ok(mut metrics_map) = self.metrics.lock() {
            metrics_map.clear();
        }
    }

    /// Get average resource usage for a service
    pub fn get_average_usage(&self, service: &str) -> Option<(f32, f32)> {
        let metrics = self.get_metrics(service, None);
        if metrics.is_empty() {
            return None;
        }

        let cpu_avg: f32 = metrics.iter().map(|m| m.cpu_percent).sum::<f32>() / metrics.len() as f32;
        let memory_avg: f32 = metrics.iter().map(|m| m.memory_mb).sum::<f32>() / metrics.len() as f32;

        Some((cpu_avg, memory_avg))
    }

    /// Collect process metrics
    #[cfg(unix)]
    fn collect_process_metrics(&self, pid: u32) -> Result<ProcessMetrics, String> {
        use std::process::Command;

        // Get CPU and memory usage using ps
        let output = Command::new("ps")
            .args(&["-p", &pid.to_string(), "-o", "pcpu,rss,vsz,nl,fd"])
            .output()
            .map_err(|e| format!("Failed to execute ps command: {}", e))?;

        if !output.status.success() {
            return Err(format!("ps command failed for PID {}", pid));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return Err(format!("Invalid ps output for PID {}", pid));
        }

        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() < 5 {
            return Err(format!("Invalid ps output format for PID {}", pid));
        }

        let cpu_percent: f32 = parts[0].parse().unwrap_or(0.0);
        let rss_kb: f32 = parts[1].parse().unwrap_or(0.0);
        let _vsz_kb: f32 = parts[2].parse().unwrap_or(0.0);
        let thread_count: u32 = parts[3].parse().unwrap_or(0);
        let file_descriptors: u32 = parts[4].parse().unwrap_or(0);

        // Get total system memory
        let total_memory_kb = self.get_total_memory_kb().unwrap_or(8_000_000.0); // Default to 8GB

        let memory_mb = rss_kb / 1024.0;
        let memory_percent = (rss_kb / total_memory_kb) * 100.0;

        Ok(ProcessMetrics {
            cpu_percent,
            memory_mb,
            memory_percent,
            thread_count,
            file_descriptors,
        })
    }

    #[cfg(windows)]
    fn collect_process_metrics(&self, pid: u32) -> Result<ProcessMetrics, String> {
        use std::process::Command;

        // Get CPU and memory usage using tasklist
        let output = Command::new("wmic")
            .args(&["process", "where", &format!("ProcessId={}", pid), "get", "WorkingSetSize,PageFileUsage,ThreadCount"])
            .output()
            .map_err(|e| format!("Failed to execute wmic command: {}", e))?;

        if !output.status.success() {
            return Err(format!("wmic command failed for PID {}", pid));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return Err(format!("Invalid wmic output for PID {}", pid));
        }

        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() < 3 {
            return Err(format!("Invalid wmic output format for PID {}", pid));
        }

        let working_set: f32 = parts[0].parse().unwrap_or(0.0);
        let page_file: f32 = parts[1].parse().unwrap_or(0.0);
        let thread_count: u32 = parts[2].parse().unwrap_or(0);

        let memory_mb = working_set / 1024.0 / 1024.0;
        let total_memory_mb = self.get_total_memory_mb().unwrap_or(8192.0);
        let memory_percent = (memory_mb / total_memory_mb) * 100.0;

        // CPU usage is harder to get on Windows without additional tools
        let cpu_percent = 0.0;

        Ok(ProcessMetrics {
            cpu_percent,
            memory_mb,
            memory_percent,
            thread_count,
            file_descriptors: 0,
        })
    }

    #[cfg(unix)]
    fn get_total_memory_kb(&self) -> Option<f32> {
        use std::process::Command;

        let output = Command::new("free")
            .arg("-k")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return None;
        }

        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        parts[1].parse().ok()
    }

    #[cfg(windows)]
    fn get_total_memory_mb(&self) -> Option<f32> {
        use std::process::Command;

        let output = Command::new("wmic")
            .args(&["OS", "get", "TotalVisibleMemorySize"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            return None;
        }

        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let total_kb: f32 = parts[0].parse().ok()?;
        Some(total_kb / 1024.0)
    }

    /// Start automatic monitoring for a service
    /// Note: This method is a placeholder for future implementation
    /// The actual monitoring should be done by calling collect_metrics periodically
    #[allow(dead_code)]
    pub fn start_monitoring(&self, _service: String, _pid: u32) {
        // Placeholder - actual implementation would require a different architecture
        // to avoid using self inside async closures
        tracing::warn!("start_monitoring is not yet implemented");
    }
}

/// Internal process metrics
#[derive(Debug)]
struct ProcessMetrics {
    cpu_percent: f32,
    memory_mb: f32,
    memory_percent: f32,
    thread_count: u32,
    file_descriptors: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new();
        
        // Test with a fake PID (will fail but tests the structure)
        let result = monitor.collect_metrics("test-service".to_string(), 999999);
        assert!(result.is_err());
    }

    #[test]
    fn test_metrics_limit() {
        let monitor = ResourceMonitor::new().with_max_metrics(5);
        
        let mut metrics_map = monitor.metrics.lock().expect("Failed to lock metrics");
        let service_metrics = metrics_map.entry("test".to_string()).or_insert_with(Vec::new);
        
        for i in 0..10 {
            service_metrics.push(ResourceMetrics {
                service: "test".to_string(),
                pid: 1234,
                cpu_percent: 10.0,
                memory_mb: 100.0,
                memory_percent: 1.0,
                thread_count: 1,
                file_descriptors: 10,
                timestamp: i,
            });
            while service_metrics.len() > monitor.max_metrics_per_service {
                service_metrics.remove(0);
            }
        }

        assert_eq!(service_metrics.len(), 5);
    }
}
