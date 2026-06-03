//! Microservice Bus for IPC Communication

use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::registry::{ServiceRegistry, ServiceStatus};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

/// Microservice Bus
pub struct MicroserviceBus {
    #[allow(dead_code)]
    registry: Arc<ServiceRegistry>,
    #[allow(dead_code)]
    timeout: Duration,
}

impl MicroserviceBus {
    /// Create a new microservice bus
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            timeout: Duration::from_secs(5),
        }
    }

    /// Set request timeout
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Call a service synchronously
    #[allow(dead_code)]
    pub fn call_service(&self, service_name: &str, request: JsonRpcRequest) -> Result<JsonRpcResponse, BusError> {
        // Get service info
        let service = self.registry.get(service_name)?
            .ok_or_else(|| BusError::ServiceNotFound(service_name.to_string()))?;

        // Check service status
        if !service.status.is_running() {
            return Err(BusError::ServiceNotReady(service_name.to_string()));
        }

        // Connect to service socket
        let mut stream = UnixStream::connect(&service.socket_path)
            .map_err(|e| BusError::ConnectionError(format!("Failed to connect to {}: {}", service_name, e)))?;

        // Set timeout
        stream.set_read_timeout(Some(self.timeout))
            .map_err(|e| BusError::ConnectionError(format!("Failed to set timeout: {}", e)))?;

        // Send request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| BusError::SerializationError(e.to_string()))?;
        
        writeln!(stream, "{}", request_json)
            .map_err(|e| BusError::ConnectionError(format!("Failed to send request: {}", e)))?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_str = String::new();
        reader.read_line(&mut response_str)
            .map_err(|e| BusError::ConnectionError(format!("Failed to read response: {}", e)))?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_str.trim())
            .map_err(|e| BusError::SerializationError(format!("Failed to parse response: {}", e)))?;

        Ok(response)
    }

    /// Call a service asynchronously
    #[allow(dead_code)]
    pub async fn call_service_async(&self, service_name: &str, request: JsonRpcRequest) -> Result<JsonRpcResponse, BusError> {
        let registry = Arc::clone(&self.registry);
        let service_name = service_name.to_string();
        let timeout = self.timeout;

        tokio::task::spawn_blocking(move || {
            let service = registry.get(&service_name)?
                .ok_or_else(|| BusError::ServiceNotFound(service_name.clone()))?;

            if !service.status.is_running() {
                return Err(BusError::ServiceNotReady(service_name));
            }

            let mut stream = UnixStream::connect(&service.socket_path)
                .map_err(|e| BusError::ConnectionError(format!("Failed to connect: {}", e)))?;

            stream.set_read_timeout(Some(timeout))
                .map_err(|e| BusError::ConnectionError(format!("Failed to set timeout: {}", e)))?;

            let request_json = serde_json::to_string(&request)
                .map_err(|e| BusError::SerializationError(e.to_string()))?;
            
            writeln!(stream, "{}", request_json)
                .map_err(|e| BusError::ConnectionError(format!("Failed to send: {}", e)))?;

            let mut reader = BufReader::new(stream);
            let mut response_str = String::new();
            reader.read_line(&mut response_str)
                .map_err(|e| BusError::ConnectionError(format!("Failed to read: {}", e)))?;

            let response: JsonRpcResponse = serde_json::from_str(&response_str.trim())
                .map_err(|e| BusError::SerializationError(format!("Failed to parse: {}", e)))?;

            Ok(response)
        })
        .await
        .map_err(|e| BusError::TaskError(e.to_string()))?
    }

    /// Broadcast a request to all running services
    #[allow(dead_code)]
    pub async fn broadcast(&self, request: JsonRpcRequest) -> Vec<Result<JsonRpcResponse, BusError>> {
        let services = match self.registry.list() {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let mut handles = vec![];
        for service in services {
            if service.status.is_running() {
                let _registry = Arc::clone(&self.registry);
                let service_name = service.name.clone();
                let request = request.clone();
                let timeout = self.timeout;

                let handle = tokio::task::spawn_blocking(move || {
                    let mut stream = UnixStream::connect(&service.socket_path)?;
                    stream.set_read_timeout(Some(timeout))?;

                    let request_json = serde_json::to_string(&request)?;
                    writeln!(stream, "{}", request_json)?;

                    let mut reader = BufReader::new(stream);
                    let mut response_str = String::new();
                    reader.read_line(&mut response_str)?;

                    let response: JsonRpcResponse = serde_json::from_str(&response_str.trim())?;
                    Ok(response)
                });

                handles.push((service_name, handle));
            }
        }

        let mut results = vec![];
        for (_service_name, handle) in handles {
            let result: Result<JsonRpcResponse, BusError> = handle.await
                .map_err(|e: tokio::task::JoinError| BusError::TaskError(e.to_string()))
                .and_then(|r: Result<JsonRpcResponse, std::io::Error>| r.map_err(|e: std::io::Error| BusError::ConnectionError(e.to_string())));
            results.push(result);
        }

        results
    }

    /// Health check for a service
    #[allow(dead_code)]
    pub async fn health_check(&self, service_name: &str) -> Result<ServiceStatus, BusError> {
        let request = JsonRpcRequest::new(
            "health.check",
            serde_json::json!({}),
            serde_json::json!(1),
        );

        match self.call_service_async(service_name, request).await {
            Ok(response) => {
                if response.error.is_none() {
                    Ok(ServiceStatus::Running)
                } else {
                    Ok(ServiceStatus::Error)
                }
            }
            Err(_) => Ok(ServiceStatus::Stopped),
        }
    }

    /// Health check for all services
    #[allow(dead_code)]
    pub async fn health_check_all(&self) -> Vec<(String, ServiceStatus)> {
        let services = match self.registry.list() {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let mut results = vec![];
        for service in services {
            let status = match self.health_check(&service.name).await {
                Ok(s) => s,
                Err(_) => ServiceStatus::Error,
            };
            results.push((service.name, status));
        }

        results
    }
}

/// Bus error
#[derive(Debug, Clone)]
pub enum BusError {
    ServiceNotFound(String),
    ServiceNotReady(String),
    ConnectionError(String),
    SerializationError(String),
    TaskError(String),
    RegistryError(String),
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            Self::ServiceNotReady(name) => write!(f, "Service not ready: {}", name),
            Self::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::TaskError(msg) => write!(f, "Task error: {}", msg),
            Self::RegistryError(msg) => write!(f, "Registry error: {}", msg),
        }
    }
}

impl std::error::Error for BusError {}

impl From<super::registry::RegistryError> for BusError {
    fn from(err: super::registry::RegistryError) -> Self {
        Self::RegistryError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::registry::ServiceRegistry;
    use std::path::PathBuf;

    #[test]
    fn test_bus_error_display() {
        let error = BusError::ServiceNotFound("test-service".to_string());
        assert_eq!(error.to_string(), "Service not found: test-service");
    }

    #[test]
    fn test_bus_error_from_registry_error() {
        let registry_error = super::super::registry::RegistryError::ServiceNotFound("test".to_string());
        let bus_error: BusError = registry_error.into();
        assert!(matches!(bus_error, BusError::RegistryError(_)));
    }

    #[test]
    fn test_microservice_bus_creation() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        let bus = MicroserviceBus::new(registry);
        assert_eq!(bus.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_microservice_bus_with_timeout() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        let bus = MicroserviceBus::new(registry).with_timeout(Duration::from_secs(10));
        assert_eq!(bus.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_bus_error_variants() {
        let error = BusError::ServiceNotReady("test".to_string());
        assert!(matches!(error, BusError::ServiceNotReady(_)));

        let error = BusError::ConnectionError("connection failed".to_string());
        assert!(matches!(error, BusError::ConnectionError(_)));

        let error = BusError::SerializationError("parse error".to_string());
        assert!(matches!(error, BusError::SerializationError(_)));

        let error = BusError::TaskError("task failed".to_string());
        assert!(matches!(error, BusError::TaskError(_)));
    }
}
