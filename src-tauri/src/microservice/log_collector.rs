//! Microservice Log Collection

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
    pub service: String,
}

/// Log collector
pub struct LogCollector {
    log_dir: PathBuf,
    logs: Arc<Mutex<HashMap<String, Vec<LogEntry>>>>,
    max_entries_per_service: usize,
}

/// Log collector error
#[derive(Debug)]
#[allow(dead_code)]
pub enum LogCollectorError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    NotFound(String),
}

impl std::fmt::Display for LogCollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for LogCollectorError {}

impl LogCollector {
    /// Create a new log collector
    pub fn new(log_dir: PathBuf) -> Result<Self, std::io::Error> {
        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&log_dir)?;
        
        Ok(Self {
            log_dir,
            logs: Arc::new(Mutex::new(HashMap::new())),
            max_entries_per_service: 10000,
        })
    }

    /// Set max entries per service
    #[allow(dead_code)]
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries_per_service = max;
        self
    }

    /// Add a log entry with error handling
    pub fn add_log(&self, entry: LogEntry) {
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        let service_logs = logs.entry(entry.service.clone()).or_insert_with(Vec::new);
        
        service_logs.push(entry);
        
        // Trim if exceeds max entries
        if service_logs.len() > self.max_entries_per_service {
            service_logs.remove(0);
        }
    }

    /// Get logs for a service with error handling
    pub fn get_logs(&self, service: &str, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(service_logs) = logs.get(service) {
            if let Some(limit) = limit {
                service_logs.iter().rev().take(limit).cloned().collect()
            } else {
                service_logs.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// Get all logs with error handling
    pub fn get_all_logs(&self) -> HashMap<String, Vec<LogEntry>> {
        let logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        logs.clone()
    }

    /// Clear logs for a service with error handling
    pub fn clear_logs(&self, service: &str) {
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        logs.remove(service);
    }

    /// Clear all logs with error handling
    pub fn clear_all_logs(&self) {
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        logs.clear();
    }

    /// Save logs to file with retry mechanism
    pub fn save_logs_to_file(&self, service: &str) -> Result<PathBuf, std::io::Error> {
        let logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(service_logs) = logs.get(service) {
            let log_file = self.log_dir.join(format!("{}.log", service));
            
            // Retry mechanism for file operations
            let mut retries = 3;
            let mut last_error = None;
            
            while retries > 0 {
                match File::create(&log_file) {
                    Ok(mut file) => {
                        for entry in service_logs {
                            if writeln!(
                                file,
                                "[{}] [{}] [{}] {}",
                                entry.timestamp,
                                entry.level,
                                entry.service,
                                entry.message
                            ).is_err() {
                                let _ = last_error.insert(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Failed to write log entry",
                                ));
                                retries -= 1;
                                continue;
                            }
                        }
                        return Ok(log_file);
                    }
                    Err(e) => {
                        last_error = Some(e);
                        retries -= 1;
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
            
            Err(last_error.unwrap_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to save logs after retries",
                )
            }))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No logs found for service: {}", service),
            ))
        }
    }

    /// Load logs from file
    #[allow(dead_code)]
    pub fn load_logs_from_file(&self, service: &str) -> Result<(), std::io::Error> {
        let log_file = self.log_dir.join(format!("{}.log", service));
        
        if !log_file.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Log file not found for service: {}", service),
            ));
        }

        let file = File::open(&log_file)?;
        let reader = BufReader::new(file);
        let mut logs = self.logs.lock().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Lock error: {}", e)))?;
        let service_logs = logs.entry(service.to_string()).or_insert_with(Vec::new);

        for line in reader.lines() {
            if let Ok(log_line) = line {
                // Parse log line format: [timestamp] [level] [service] message
                if let Some(timestamp_end) = log_line.find(']') {
                    let rest = &log_line[timestamp_end + 1..];
                    if let Some(level_start) = rest.find('[') {
                        if let Some(level_end) = rest[level_start + 1..].find(']') {
                            let level_end = level_start + 1 + level_end;
                            let rest2 = &rest[level_end + 1..];
                            if let Some(service_start) = rest2.find('[') {
                                if let Some(service_end) = rest2[service_start + 1..].find(']') {
                                    let service_end = service_start + 1 + service_end;
                                    let timestamp_str = &log_line[1..timestamp_end];
                                    let level = &rest[level_start + 1..level_end];
                                    let service_name = &rest2[service_start + 1..service_end];
                                    let message = &rest2[service_end + 2..];

                                    if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                                        service_logs.push(LogEntry {
                                            timestamp,
                                            level: level.to_string(),
                                            message: message.to_string(),
                                            service: service_name.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get log directory
    #[allow(dead_code)]
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// Collect logs from a running process
    #[cfg(unix)]
    #[allow(dead_code)]
    pub fn collect_from_process(&self, pid: u32, service: &str) -> Result<(), std::io::Error> {
        use std::process::Command;
        
        // Try to get logs from journalctl if available
        let result = Command::new("journalctl")
            .arg("--no-pager")
            .arg(format!("_PID={}", pid))
            .output();
        
        if result.is_ok() {
            let output = result.expect("Failed to get journalctl output");
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    self.add_log(LogEntry {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs(),
                        level: "INFO".to_string(),
                        message: line.to_string(),
                        service: service.to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_collector() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let collector = LogCollector::new(temp_dir.path().to_path_buf()).expect("Failed to create collector");
        
        let entry = LogEntry {
            timestamp: 1234567890,
            level: "INFO".to_string(),
            message: "Test message".to_string(),
            service: "test-service".to_string(),
        };
        
        collector.add_log(entry.clone());
        
        let logs = collector.get_logs("test-service", None);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_log_limits() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let collector = LogCollector::new(temp_dir.path().to_path_buf())
            .expect("Failed to create collector")
            .with_max_entries(5);
        
        for i in 0..10 {
            collector.add_log(LogEntry {
                timestamp: i,
                level: "INFO".to_string(),
                message: format!("Message {}", i),
                service: "test-service".to_string(),
            });
        }
        
        let logs = collector.get_logs("test-service", None);
        assert_eq!(logs.len(), 5);
        assert_eq!(logs[0].message, "Message 5");
    }

    #[test]
    fn test_log_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let collector = LogCollector::new(temp_dir.path().to_path_buf()).expect("Failed to create collector");
        
        let entry = LogEntry {
            timestamp: 1234567890,
            level: "INFO".to_string(),
            message: "Test message".to_string(),
            service: "test-service".to_string(),
        };
        
        collector.add_log(entry);
        
        let log_file = collector.save_logs_to_file("test-service").expect("Failed to save logs");
        assert!(log_file.exists());
        
        // Clear logs and reload
        collector.clear_logs("test-service");
        assert_eq!(collector.get_logs("test-service", None).len(), 0);
        
        collector.load_logs_from_file("test-service").expect("Failed to load logs");
        assert_eq!(collector.get_logs("test-service", None).len(), 1);
    }
}
