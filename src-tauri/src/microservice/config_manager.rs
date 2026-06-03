//! Configuration Management for Dynamic Updates
//!
//! Manage service configurations with hot-reload capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub service_name: Option<String>,
    pub environment: String,
    pub version: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
}

impl ConfigEntry {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            key: key.into(),
            value: value.into(),
            service_name: None,
            environment: "default".to_string(),
            version: 1,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn for_service(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = env.into();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub service_name: Option<String>,
    pub timestamp: u64,
}

/// Configuration manager
pub struct ConfigManager {
    configs: Arc<RwLock<HashMap<String, ConfigEntry>>>,
    change_listeners: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<ConfigChangeEvent>>>>,
    history: Arc<RwLock<Vec<ConfigChangeEvent>>>,
    max_history: usize,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            change_listeners: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        }
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Set a configuration value
    pub async fn set(&self, mut entry: ConfigEntry) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        
        let key = entry.key.clone();
        let old_value = configs.get(&key).map(|e| e.value.clone());
        
        // Update version and timestamp
        if let Some(existing) = configs.get(&key) {
            entry.version = existing.version + 1;
        }
        entry.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        configs.insert(key.clone(), entry.clone());
        
        // Notify listeners
        let event = ConfigChangeEvent {
            key: key.clone(),
            old_value,
            new_value: entry.value.clone(),
            service_name: entry.service_name.clone(),
            timestamp: entry.updated_at,
        };
        
        self.notify_listeners(event.clone()).await;
        
        // Add to history
        let mut history = self.history.write().await;
        history.push(event);
        if history.len() > self.max_history {
            history.remove(0);
        }
        
        Ok(())
    }

    /// Get a configuration value
    pub async fn get(&self, key: &str) -> Option<ConfigEntry> {
        let configs = self.configs.read().await;
        configs.get(key).cloned()
    }

    /// Get configuration value by service
    pub async fn get_for_service(&self, service_name: &str, key: &str) -> Option<ConfigEntry> {
        let configs = self.configs.read().await;
        
        // Try service-specific config first
        let service_key = format!("{}:{}", service_name, key);
        if let Some(entry) = configs.get(&service_key) {
            return Some(entry.clone());
        }
        
        // Fall back to global config
        configs.get(key).cloned()
    }

    /// Get all configurations for a service
    pub async fn get_all_for_service(&self, service_name: &str) -> Vec<ConfigEntry> {
        let configs = self.configs.read().await;
        
        configs
            .values()
            .filter(|entry| {
                entry.service_name.as_ref().map(|s| s == service_name).unwrap_or(false)
                    || entry.service_name.is_none()
            })
            .cloned()
            .collect()
    }

    /// Delete a configuration
    pub async fn delete(&self, key: &str) -> Result<bool, String> {
        let mut configs = self.configs.write().await;
        
        if let Some(entry) = configs.remove(key) {
            // Notify listeners
            let event = ConfigChangeEvent {
                key: key.to_string(),
                old_value: Some(entry.value),
                new_value: String::new(),
                service_name: entry.service_name,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs(),
            };
            
            self.notify_listeners(event.clone()).await;
            
            // Add to history
            let mut history = self.history.write().await;
            history.push(event);
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List all configuration keys
    pub async fn list_keys(&self) -> Vec<String> {
        let configs = self.configs.read().await;
        configs.keys().cloned().collect()
    }

    /// List all configurations
    pub async fn list_all(&self) -> Vec<ConfigEntry> {
        let configs = self.configs.read().await;
        configs.values().cloned().collect()
    }

    /// Subscribe to configuration changes
    pub async fn subscribe(&self) -> tokio::sync::mpsc::UnboundedReceiver<ConfigChangeEvent> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut listeners = self.change_listeners.write().await;
        listeners.push(tx);
        rx
    }

    /// Notify all listeners of a configuration change
    async fn notify_listeners(&self, event: ConfigChangeEvent) {
        let mut listeners = self.change_listeners.write().await;
        
        // Remove closed channels
        listeners.retain(|tx| !tx.is_closed());
        
        // Send to all active listeners
        for tx in listeners.iter() {
            let _ = tx.send(event.clone());
        }
    }

    /// Get configuration change history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<ConfigChangeEvent> {
        let history = self.history.read().await;
        
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.clone()
        }
    }

    /// Get configuration statistics
    pub async fn get_stats(&self) -> ConfigStats {
        let configs = self.configs.read().await;
        let history = self.history.read().await;
        
        let total_configs = configs.len();
        let mut services = std::collections::HashSet::new();
        let mut environments = std::collections::HashSet::new();
        
        for entry in configs.values() {
            if let Some(service) = &entry.service_name {
                services.insert(service.clone());
            }
            environments.insert(entry.environment.clone());
        }
        
        ConfigStats {
            total_configs,
            total_services: services.len(),
            total_environments: environments.len(),
            total_changes: history.len(),
        }
    }

    /// Export all configurations
    pub async fn export(&self) -> Result<String, String> {
        let configs = self.configs.read().await;
        serde_json::to_string_pretty(&*configs)
            .map_err(|e| format!("Failed to export configs: {}", e))
    }

    /// Import configurations
    pub async fn import(&self, json: &str) -> Result<usize, String> {
        let entries: HashMap<String, ConfigEntry> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse configs: {}", e))?;
        
        let count = entries.len();
        
        for (_, entry) in entries {
            self.set(entry).await?;
        }
        
        Ok(count)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigStats {
    pub total_configs: usize,
    pub total_services: usize,
    pub total_environments: usize,
    pub total_changes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_set_and_get() {
        let manager = ConfigManager::new();
        
        let entry = ConfigEntry::new("test.key", "test.value");
        manager.set(entry).await.expect("Failed to set config");
        
        let retrieved = manager.get("test.key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Expected config").value, "test.value");
    }

    #[tokio::test]
    async fn test_service_specific_config() {
        let manager = ConfigManager::new();
        
        let entry = ConfigEntry::new("service-a:port", "8080")
            .for_service("service-a");
        manager.set(entry).await.expect("Failed to set config");
        
        let retrieved = manager.get_for_service("service-a", "port").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "8080");
    }

    #[tokio::test]
    async fn test_config_change_notification() {
        let manager = ConfigManager::new();
        let mut rx = manager.subscribe().await;
        
        let entry = ConfigEntry::new("test.key", "test.value");
        manager.set(entry).await.expect("Failed to set config");
        
        let event = rx.try_recv();
        assert!(event.is_ok());
        
        let event = event.expect("Expected event");
        assert_eq!(event.key, "test.key");
        assert_eq!(event.new_value, "test.value");
    }
}
