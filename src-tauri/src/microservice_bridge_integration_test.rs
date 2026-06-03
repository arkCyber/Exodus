//! Integration tests for microservice bridge
//!
//! These tests verify the end-to-end flow from Tauri commands
//! to microservice invocation via Unix Domain Sockets.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microservice::{ServiceRegistry, ServiceInfo, ServiceStatus};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::net::UnixListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_microservice_bridge_invoke() {
        // Create a temporary socket path
        let socket_path = PathBuf::from("/tmp/test-microservice-bridge.sock");
        
        // Clean up any existing socket
        let _ = std::fs::remove_file(&socket_path);

        // Create a simple echo server
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");
        let socket_path_clone = socket_path.clone();

        // Spawn the echo server
        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                if let Ok(n) = socket.read(&mut buf).await {
                    // Echo back the request as a JSON-RPC response
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let response = format!(
                        r#"{{"jsonrpc":"2.0","result":{{"echo":"{}"}},"id":1}}"#,
                        request.trim()
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                }
            }
        });

        // Give the server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create service registry
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        
        // Register the test service
        let service_info = ServiceInfo::new("test-service", &socket_path_clone, std::process::id())
            .with_status(ServiceStatus::Running);
        registry.register(service_info).expect("Failed to register service");

        // Create bridge and invoke
        let bridge = MicroserviceBridge::new(registry);
        let result = bridge
            .invoke("test-service", "echo", Some(serde_json::json!({"message": "hello"})))
            .await;

        assert!(result.is_ok(), "Bridge invocation failed: {:?}", result);

        // Clean up
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
        
        let service_info = ServiceInfo::new("unhealthy-service", "/tmp/test.sock", 1234)
            .with_status(ServiceStatus::Stopped);
        registry.register(service_info).expect("Failed to register service");

        let bridge = MicroserviceBridge::new(registry);
        let result = bridge.invoke("unhealthy-service", "test", None).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not healthy"));
    }

    #[tokio::test]
    async fn test_microservice_bridge_list_services() {
        let registry = Arc::new(ServiceRegistry::new(PathBuf::from("/tmp")));
        
        let service1 = ServiceInfo::new("service1", "/tmp/service1.sock", 1001);
        let service2 = ServiceInfo::new("service2", "/tmp/service2.sock", 1002);
        
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
        
        let service_info = ServiceInfo::new("available-service", "/tmp/test.sock", 1234)
            .with_status(ServiceStatus::Running);
        registry.register(service_info).expect("Failed to register service");

        let bridge = MicroserviceBridge::new(registry);
        
        assert!(bridge.is_service_available("available-service").await);
        assert!(!bridge.is_service_available("nonexistent-service").await);
    }
}
