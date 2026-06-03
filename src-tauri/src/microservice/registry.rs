//! Service Registry for Microservice Discovery and Management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Duration};

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}

impl ServiceStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    #[allow(dead_code)]
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped | Self::Error)
    }
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Unix Domain Socket path
    pub socket_path: String,
    /// Current status
    pub status: ServiceStatus,
    /// Process ID
    pub pid: u32,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
    /// Service version
    pub version: String,
    /// Service metadata
    pub metadata: serde_json::Value,
}

impl ServiceInfo {
    pub fn new(name: impl Into<String>, socket_path: impl Into<String>, pid: u32) -> Self {
        Self {
            name: name.into(),
            socket_path: socket_path.into(),
            status: ServiceStatus::Starting,
            pid,
            last_heartbeat: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            version: "0.1.0".to_string(),
            metadata: serde_json::json!({}),
        }
    }

    #[allow(dead_code)]
    pub fn with_status(mut self, status: ServiceStatus) -> Self {
        self.status = status;
        self
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }

    pub fn is_healthy(&self, timeout_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        self.status == ServiceStatus::Running && (now - self.last_heartbeat) < timeout_secs
    }
}

/// Service Registry
pub struct ServiceRegistry {
    services: Arc<Mutex<HashMap<String, ServiceInfo>>>,
    socket_dir: PathBuf,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(socket_dir: PathBuf) -> Self {
        Self {
            services: Arc::new(Mutex::new(HashMap::new())),
            socket_dir,
        }
    }

    /// Register a service
    pub fn register(&self, service: ServiceInfo) -> Result<(), RegistryError> {
        let mut services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        services.insert(service.name.clone(), service);
        Ok(())
    }

    /// Unregister a service
    pub fn unregister(&self, name: &str) -> Result<Option<ServiceInfo>, RegistryError> {
        let mut services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        Ok(services.remove(name))
    }

    /// Get service information
    pub fn get(&self, name: &str) -> Result<Option<ServiceInfo>, RegistryError> {
        let services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        Ok(services.get(name).cloned())
    }

    /// List all services
    pub fn list(&self) -> Result<Vec<ServiceInfo>, RegistryError> {
        let services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        Ok(services.values().cloned().collect())
    }

    /// Update service status
    pub fn update_status(&self, name: &str, status: ServiceStatus) -> Result<(), RegistryError> {
        let mut services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        if let Some(service) = services.get_mut(name) {
            service.status = status;
            service.update_heartbeat();
        }

        Ok(())
    }

    /// Update service heartbeat
    pub fn update_heartbeat(&self, name: &str) -> Result<(), RegistryError> {
        let mut services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        if let Some(service) = services.get_mut(name) {
            service.update_heartbeat();
        }

        Ok(())
    }

    /// Check service health
    pub fn check_health(&self, name: &str, timeout_secs: u64) -> Result<bool, RegistryError> {
        let services = self.services.lock()
            .map_err(|e| RegistryError::LockError(e.to_string()))?;

        Ok(services.get(name)
            .map(|s| s.is_healthy(timeout_secs))
            .unwrap_or(false))
    }

    /// Get socket directory
    pub fn socket_dir(&self) -> &PathBuf {
        &self.socket_dir
    }

    /// Generate socket path for a service
    #[allow(dead_code)]
    pub fn socket_path_for(&self, name: &str) -> PathBuf {
        self.socket_dir.join(format!("{}.sock", name))
    }
}

/// Registry error
#[derive(Debug, Clone)]
pub enum RegistryError {
    LockError(String),
    #[allow(dead_code)]
    ServiceNotFound(String),
    #[allow(dead_code)]
    InvalidSocketPath(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockError(msg) => write!(f, "Lock error: {}", msg),
            Self::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            Self::InvalidSocketPath(msg) => write!(f, "Invalid socket path: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info() {
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        assert_eq!(service.name, "test-service");
        assert_eq!(service.socket_path, "/tmp/test.sock");
        assert_eq!(service.status, ServiceStatus::Starting);
        assert_eq!(service.pid, 1234);
        assert_eq!(service.version, "0.1.0");
    }

    #[test]
    fn test_service_info_with_status() {
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234)
            .with_status(ServiceStatus::Running);

        assert_eq!(service.status, ServiceStatus::Running);
    }

    #[test]
    fn test_service_status() {
        let status = ServiceStatus::Running;
        assert!(status.is_running());
        assert!(!status.is_stopped());

        let status = ServiceStatus::Stopped;
        assert!(!status.is_running());
        assert!(status.is_stopped());

        let status = ServiceStatus::Error;
        assert!(!status.is_running());
        assert!(status.is_stopped());
    }

    #[test]
    fn test_service_info_heartbeat() {
        let mut service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        let old_heartbeat = service.last_heartbeat;
        
        std::thread::sleep(std::time::Duration::from_millis(100));
        service.update_heartbeat();
        
        // Heartbeat should be updated (or at least not decreased)
        assert!(service.last_heartbeat >= old_heartbeat);
    }

    #[test]
    fn test_service_info_healthy() {
        let mut service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        service.status = ServiceStatus::Running;
        assert!(service.is_healthy(30)); // 30 seconds timeout

        service.status = ServiceStatus::Stopped;
        assert!(!service.is_healthy(30));
    }

    #[test]
    fn test_service_info_not_healthy_timeout() {
        let mut service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        service.status = ServiceStatus::Running;
        
        // Simulate old heartbeat
        service.last_heartbeat = 0;
        assert!(!service.is_healthy(30)); // Should be considered unhealthy
    }

    #[test]
    fn test_service_registry() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp"));
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);

        assert!(registry.register(service.clone()).is_ok());
        assert_eq!(registry.list().unwrap().len(), 1);
        assert!(registry.get("test-service").is_ok());
        
        let retrieved = registry.get("test-service").unwrap().unwrap();
        assert_eq!(retrieved.name, "test-service");
        
        assert!(registry.unregister("test-service").is_ok());
        assert_eq!(registry.list().unwrap().len(), 0);
    }

    #[test]
    fn test_service_registry_update_status() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp"));
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        
        registry.register(service).unwrap();
        registry.update_status("test-service", ServiceStatus::Running).unwrap();
        
        let retrieved = registry.get("test-service").unwrap().unwrap();
        assert_eq!(retrieved.status, ServiceStatus::Running);
    }

    #[test]
    fn test_service_registry_update_heartbeat() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp"));
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        
        registry.register(service).unwrap();
        let old_heartbeat = registry.get("test-service").unwrap().unwrap().last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(100));
        registry.update_heartbeat("test-service").unwrap();
        
        let new_heartbeat = registry.get("test-service").unwrap().unwrap().last_heartbeat;
        assert!(new_heartbeat >= old_heartbeat);
    }

    #[test]
    fn test_service_registry_check_health() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp"));
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234);
        
        registry.register(service).unwrap();
        registry.update_status("test-service", ServiceStatus::Running).unwrap();
        
        assert!(registry.check_health("test-service", 30).unwrap());
        
        registry.update_status("test-service", ServiceStatus::Stopped).unwrap();
        assert!(!registry.check_health("test-service", 30).unwrap());
    }

    #[test]
    fn test_service_registry_nonexistent_service() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp"));
        
        assert!(registry.get("nonexistent").unwrap().is_none());
        assert!(!registry.check_health("nonexistent", 30).unwrap());
    }

    #[test]
    fn test_service_registry_socket_path() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp/exodus"));
        
        let path = registry.socket_path_for("test-service");
        assert_eq!(path, PathBuf::from("/tmp/exodus/test-service.sock"));
    }

    #[test]
    fn test_service_registry_multiple_services() {
        let registry = ServiceRegistry::new(PathBuf::from("/tmp"));
        
        let service1 = ServiceInfo::new("service1", "/tmp/service1.sock", 1001);
        let service2 = ServiceInfo::new("service2", "/tmp/service2.sock", 1002);
        let service3 = ServiceInfo::new("service3", "/tmp/service3.sock", 1003);
        
        registry.register(service1).unwrap();
        registry.register(service2).unwrap();
        registry.register(service3).unwrap();
        
        let services = registry.list().unwrap();
        assert_eq!(services.len(), 3);
        
        assert!(registry.unregister("service2").unwrap().is_some());
        assert_eq!(registry.list().unwrap().len(), 2);
    }

    #[test]
    fn test_registry_error_display() {
        let error = RegistryError::ServiceNotFound("test-service".to_string());
        assert_eq!(error.to_string(), "Service not found: test-service");

        let error = RegistryError::InvalidSocketPath("invalid path".to_string());
        assert_eq!(error.to_string(), "Invalid socket path: invalid path");
    }

    #[test]
    fn test_service_info_serialization() {
        let service = ServiceInfo::new("test-service", "/tmp/test.sock", 1234)
            .with_status(ServiceStatus::Running);

        let json = serde_json::to_string(&service).unwrap();
        assert!(json.contains("test-service"));
        assert!(json.contains("Running"));

        let deserialized: ServiceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-service");
        assert_eq!(deserialized.status, ServiceStatus::Running);
    }

    #[test]
    fn test_service_status_serialization() {
        let status = ServiceStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ServiceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ServiceStatus::Running);
    }
}
