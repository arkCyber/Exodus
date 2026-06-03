//! Microservice Bridge - Generic JSON-RPC 2.0 invoker for frontend
//!
//! This module provides a unified interface for the frontend to invoke
//! any microservice via JSON-RPC 2.0 over Unix Domain Sockets.

use crate::microservice::{ServiceRegistry, ServiceInfo};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, error, info};

/// JSON-RPC 2.0 Request
#[derive(Debug, serde::Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: Option<Value>,
    id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, serde::Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// Generic microservice invoker
pub struct MicroserviceBridge {
    registry: Arc<ServiceRegistry>,
}

impl MicroserviceBridge {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Invoke a microservice method via JSON-RPC 2.0
    pub async fn invoke(
        &self,
        service_name: &str,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, String> {
        debug!("Invoking microservice: {}::{}", service_name, method);

        // Get service info from registry
        let service_info = self
            .registry
            .get(service_name)
            .map_err(|e| format!("Failed to get service info: {}", e))?
            .ok_or_else(|| format!("Service '{}' not found in registry", service_name))?;

        let socket_path = PathBuf::from(&service_info.socket_path);

        // Check if service is healthy
        if !service_info.is_healthy(30) {
            return Err(format!("Service '{}' is not healthy", service_name));
        }

        // Build JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: chrono::Utc::now().timestamp_millis() as u64,
        };

        let request_str = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        debug!("Sending request to {}: {}", socket_path.display(), request_str);

        // Connect to Unix Domain Socket
        let mut socket = UnixStream::connect(&socket_path)
            .await
            .map_err(|e| format!("Failed to connect to service socket '{}': {}", socket_path.display(), e))?;

        // Send request
        socket
            .write_all((request_str + "\n").as_bytes())
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        // Read response
        let mut response_buf = Vec::new();
        let mut temp_buf = [0u8; 8192];

        loop {
            match socket.read(&mut temp_buf).await {
                Ok(0) => break,
                Ok(n) => response_buf.extend_from_slice(&temp_buf[..n]),
                Err(e) => return Err(format!("Failed to read response: {}", e)),
            }
        }

        let response_str = String::from_utf8(response_buf)
            .map_err(|e| format!("Response is not valid UTF-8: {}", e))?;

        debug!("Received response: {}", response_str);

        // Parse JSON-RPC response
        let response: JsonRpcResponse = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse JSON-RPC response: {}", e))?;

        // Check for RPC error
        if let Some(error) = response.error {
            return Err(format!(
                "RPC Error (code {}): {}",
                error.code, error.message
            ));
        }

        // Return result
        response
            .result
            .ok_or_else(|| "No result in response".to_string())
    }

    /// Check if a service is available
    pub async fn is_service_available(&self, service_name: &str) -> bool {
        match self.registry.get(service_name) {
            Ok(Some(info)) => info.is_healthy(30),
            _ => false,
        }
    }

    /// List all registered services
    pub async fn list_services(&self) -> Result<Vec<String>, String> {
        let services = self.registry.list()
            .map_err(|e| format!("Failed to list services: {}", e))?;
        
        Ok(services.into_iter().map(|s| s.name).collect())
    }
}

/// Tauri command to invoke any microservice
#[tauri::command]
pub async fn invoke_microservice(
    service_name: String,
    request: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<String, String> {
    info!("Received microservice invocation request for: {}", service_name);

    // Parse JSON-RPC request
    let request_value: Value = serde_json::from_str(&request)
        .map_err(|e| format!("Failed to parse request JSON: {}", e))?;

    let method = request_value
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'method' in request")?
        .to_string();

    let params = request_value.get("params").cloned();

    // Create bridge and invoke
    let bridge = MicroserviceBridge::new(registry.inner().clone());
    let result = bridge.invoke(&service_name, &method, params).await?;

    // Return result as JSON string
    serde_json::to_string(&result).map_err(|e| format!("Failed to serialize result: {}", e))
}

/// Tauri command to check if a service is available
#[tauri::command]
pub async fn is_microservice_available(
    service_name: String,
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<bool, String> {
    let bridge = MicroserviceBridge::new(registry.inner().clone());
    Ok(bridge.is_service_available(&service_name).await)
}

/// Tauri command to list all available microservices
#[tauri::command]
pub async fn list_microservices(
    registry: tauri::State<'_, Arc<ServiceRegistry>>,
) -> Result<Vec<String>, String> {
    let bridge = MicroserviceBridge::new(registry.inner().clone());
    bridge.list_services().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: "test_method".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test_method\""));
    }

    #[test]
    fn test_jsonrpc_response_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "result": {"success": true},
            "id": 1
        }"#;

        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32601,
                "message": "Method not found"
            },
            "id": 1
        }"#;

        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::microservice::{ServiceRegistry, ServiceInfo, ServiceStatus};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::net::UnixListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_microservice_bridge_invoke() {
        let socket_path = PathBuf::from("/tmp/test-microservice-bridge.sock");
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
        let socket_path_clone = socket_path.clone();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                if let Ok(n) = socket.read(&mut buf).await {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let response = format!(
                        r#"{{"jsonrpc":"2.0","result":{{"echo":"{}"}},"id":1}}"#,
                        request.trim().replace("\"", "\\\"")
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        let mut service_info = ServiceInfo::new("test-service", socket_path_clone.to_string_lossy().to_string(), std::process::id())
            .with_status(ServiceStatus::Running);
        service_info.update_heartbeat(); // Ensure heartbeat is recent
        registry.register(service_info).expect("Failed to register service");

        let bridge = MicroserviceBridge::new(registry);
        let result = bridge
            .invoke("test-service", "echo", Some(serde_json::json!({"message": "hello"})))
            .await;

        if let Err(e) = &result {
            eprintln!("Test failed with error: {}", e);
        }
        assert!(result.is_ok(), "Bridge invocation failed: {:?}", result);
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn test_microservice_bridge_service_not_found() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        let bridge = MicroserviceBridge::new(registry);

        let result = bridge.invoke("nonexistent-service", "test", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in registry"));
    }

    #[tokio::test]
    async fn test_microservice_bridge_unhealthy_service() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        
        let mut service_info = ServiceInfo::new("unhealthy-service", "/tmp/test.sock", 1234)
            .with_status(ServiceStatus::Stopped);
        service_info.update_heartbeat();
        registry.register(service_info).expect("Failed to register service");

        let bridge = MicroserviceBridge::new(registry);
        let result = bridge.invoke("unhealthy-service", "test", None).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not healthy"));
    }

    #[tokio::test]
    async fn test_microservice_bridge_list_services() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        
        let mut service1 = ServiceInfo::new("service1", "/tmp/service1.sock", 1001);
        service1.update_heartbeat();
        let mut service2 = ServiceInfo::new("service2", "/tmp/service2.sock", 1002);
        service2.update_heartbeat();
        
        registry.register(service1).expect("Failed to register service1");
        registry.register(service2).expect("Failed to register service2");

        let bridge = MicroserviceBridge::new(registry);
        let services = bridge.list_services().await;
        
        assert!(services.is_ok());
        let service_names = services.unwrap();
        assert_eq!(service_names.len(), 2);
        assert!(service_names.contains(&"service1".to_string()));
        assert!(service_names.contains(&"service2".to_string()));
    }

    #[tokio::test]
    async fn test_microservice_bridge_is_service_available() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        
        let mut service_info = ServiceInfo::new("available-service", "/tmp/test.sock", 1234)
            .with_status(ServiceStatus::Running);
        service_info.update_heartbeat();
        registry.register(service_info).expect("Failed to register service");

        let bridge = MicroserviceBridge::new(registry);
        
        assert!(bridge.is_service_available("available-service").await);
        assert!(!bridge.is_service_available("nonexistent-service").await);
    }
}
