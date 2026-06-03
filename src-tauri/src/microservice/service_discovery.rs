//! Service Discovery for Dynamic Microservice Registration
//!
//! Enables services to find and communicate with each other dynamically

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use std::time::Duration;
/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub metadata: HashMap<String, String>,
    pub health_check_url: Option<String>,
    pub registered_at: u64,
    pub last_heartbeat: u64,
    pub version: String,
}

impl ServiceEndpoint {
    pub fn new(service_name: impl Into<String>, host: impl Into<String>, port: u16) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            service_name: service_name.into(),
            host: host.into(),
            port,
            protocol: "http".to_string(),
            metadata: HashMap::new(),
            health_check_url: None,
            registered_at: now,
            last_heartbeat: now,
            version: "1.0.0".to_string(),
        }
    }

    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = protocol.into();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_health_check(mut self, url: impl Into<String>) -> Self {
        self.health_check_url = Some(url.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn get_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}

/// Service discovery registry
pub struct ServiceDiscovery {
    services: Arc<RwLock<HashMap<String, Vec<ServiceEndpoint>>>>,
    heartbeat_timeout: u64, // seconds
}

impl ServiceDiscovery {
    /// Create a new service discovery registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout: 30, // 30 seconds
        }
    }

    /// Set heartbeat timeout
    pub fn with_heartbeat_timeout(mut self, timeout_secs: u64) -> Self {
        self.heartbeat_timeout = timeout_secs;
        self
    }

    /// Register a service endpoint
    pub async fn register(&self, endpoint: ServiceEndpoint) -> Result<(), String> {
        let mut services = self.services.write().await;
        
        let endpoints = services
            .entry(endpoint.service_name.clone())
            .or_insert_with(Vec::new);

        // Check if endpoint already exists (same host:port)
        let existing_idx = endpoints.iter().position(|e| {
            e.host == endpoint.host && e.port == endpoint.port
        });

        if let Some(idx) = existing_idx {
            // Update existing endpoint
            endpoints[idx] = endpoint;
        } else {
            // Add new endpoint
            endpoints.push(endpoint);
        }

        Ok(())
    }

    /// Deregister a service endpoint
    pub async fn deregister(&self, service_name: &str, host: &str, port: u16) -> Result<bool, String> {
        let mut services = self.services.write().await;
        
        let removed = if let Some(endpoints) = services.get_mut(service_name) {
            let initial_len = endpoints.len();
            endpoints.retain(|e| !(e.host == host && e.port == port));
            let was_removed = endpoints.len() < initial_len;
            
            // Check if we should remove the service entry
            let should_remove = endpoints.is_empty();
            (was_removed, should_remove)
        } else {
            (false, false)
        };
        
        // Remove service entry if no endpoints left
        if removed.1 {
            services.remove(service_name);
        }
        
        Ok(removed.0)
    }

    /// Update heartbeat for a service endpoint
    pub async fn heartbeat(&self, service_name: &str, host: &str, port: u16) -> Result<(), String> {
        let mut services = self.services.write().await;
        
        if let Some(endpoints) = services.get_mut(service_name) {
            if let Some(endpoint) = endpoints.iter_mut().find(|e| e.host == host && e.port == port) {
                endpoint.last_heartbeat = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs();
                Ok(())
            } else {
                Err(format!("Endpoint {}:{} not found for service {}", host, port, service_name))
            }
        } else {
            Err(format!("Service {} not found", service_name))
        }
    }

    /// Get all endpoints for a service
    pub async fn discover(&self, service_name: &str) -> Result<Vec<ServiceEndpoint>, String> {
        let services = self.services.read().await;
        
        if let Some(endpoints) = services.get(service_name) {
            // Filter out stale endpoints
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            
            let active_endpoints: Vec<ServiceEndpoint> = endpoints
                .iter()
                .filter(|e| now - e.last_heartbeat < self.heartbeat_timeout)
                .cloned()
                .collect();
            
            if active_endpoints.is_empty() {
                Err(format!("No active endpoints found for service {}", service_name))
            } else {
                Ok(active_endpoints)
            }
        } else {
            Err(format!("Service {} not found", service_name))
        }
    }

    /// Get a single endpoint for a service (load balanced)
    pub async fn get_endpoint(&self, service_name: &str) -> Result<ServiceEndpoint, String> {
        let endpoints = self.discover(service_name).await?;
        
        // Simple round-robin: return first endpoint
        // In production, you'd implement proper load balancing
        endpoints.into_iter().next()
            .ok_or_else(|| format!("No endpoints available for service {}", service_name))
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Get all endpoints across all services
    pub async fn list_all_endpoints(&self) -> HashMap<String, Vec<ServiceEndpoint>> {
        let services = self.services.read().await;
        
        // Filter out stale endpoints
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        services
            .iter()
            .map(|(name, endpoints)| {
                let active: Vec<ServiceEndpoint> = endpoints
                    .iter()
                    .filter(|e| now - e.last_heartbeat < self.heartbeat_timeout)
                    .cloned()
                    .collect();
                (name.clone(), active)
            })
            .filter(|(_, endpoints)| !endpoints.is_empty())
            .collect()
    }

    /// Clean up stale endpoints
    pub async fn cleanup_stale_endpoints(&self) -> usize {
        let mut services = self.services.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let mut removed_count = 0;
        
        // Remove stale endpoints
        services.retain(|_, endpoints| {
            let initial_len = endpoints.len();
            endpoints.retain(|e| now - e.last_heartbeat < self.heartbeat_timeout);
            removed_count += initial_len - endpoints.len();
            !endpoints.is_empty()
        });
        
        removed_count
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> ServiceDiscoveryStats {
        let services = self.services.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let total_services = services.len();
        let mut total_endpoints = 0;
        let mut active_endpoints = 0;
        let mut stale_endpoints = 0;
        
        for endpoints in services.values() {
            total_endpoints += endpoints.len();
            for endpoint in endpoints {
                if now - endpoint.last_heartbeat < self.heartbeat_timeout {
                    active_endpoints += 1;
                } else {
                    stale_endpoints += 1;
                }
            }
        }
        
        ServiceDiscoveryStats {
            total_services,
            total_endpoints,
            active_endpoints,
            stale_endpoints,
        }
    }
}

impl Default for ServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Service discovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryStats {
    pub total_services: usize,
    pub total_endpoints: usize,
    pub active_endpoints: usize,
    pub stale_endpoints: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registration() {
        let discovery = ServiceDiscovery::new();
        
        let endpoint = ServiceEndpoint::new("test-service", "localhost", 8080);
        discovery.register(endpoint).await.expect("Failed to register endpoint");
        
        let endpoints = discovery.discover("test-service").await.expect("Failed to discover service");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].service_name, "test-service");
    }

    #[tokio::test]
    async fn test_service_deregistration() {
        let discovery = ServiceDiscovery::new();
        
        let endpoint = ServiceEndpoint::new("test-service", "localhost", 8080);
        discovery.register(endpoint).await.expect("Failed to register endpoint");
        
        let removed = discovery.deregister("test-service", "localhost", 8080).await.expect("Failed to deregister");
        assert!(removed);
        
        let result = discovery.discover("test-service").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let discovery = ServiceDiscovery::new();
        
        let endpoint = ServiceEndpoint::new("test-service", "localhost", 8080);
        discovery.register(endpoint).await.expect("Failed to register endpoint");
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        discovery.heartbeat("test-service", "localhost", 8080).await.expect("Failed to send heartbeat");
        
        let endpoints = discovery.discover("test-service").await.expect("Failed to discover service");
        assert_eq!(endpoints.len(), 1);
    }
}
