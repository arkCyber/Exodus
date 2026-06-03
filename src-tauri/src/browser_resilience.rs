//! Browser Resilience and Fault Tolerance
//!
//! Aerospace-grade fault tolerance for browser core operations including:
//! - Navigation retry with exponential backoff
//! - WebView operation circuit breaking
//! - Tab recovery mechanisms
//! - Graceful degradation for critical failures

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::microservice::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::microservice::resilience::{RetryConfig, retry_with_backoff, HealthStatus, HealthCheck};

/// Browser operation types for circuit breaking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserOperation {
    /// Page navigation
    Navigate,
    /// JavaScript evaluation
    Eval,
    /// Content capture
    Capture,
    /// Screenshot
    Screenshot,
    /// Tab creation
    CreateTab,
    /// Tab close
    CloseTab,
}

/// Browser resilience configuration
#[derive(Debug, Clone)]
pub struct BrowserResilienceConfig {
    /// Retry configuration for navigation
    pub navigation_retry: RetryConfig,
    /// Retry configuration for eval
    pub eval_retry: RetryConfig,
    /// Retry configuration for capture
    pub capture_retry: RetryConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Maximum number of failed tabs before tab recovery
    pub max_failed_tabs: usize,
    /// Tab recovery timeout in seconds
    pub tab_recovery_timeout: u64,
}

impl Default for BrowserResilienceConfig {
    fn default() -> Self {
        Self {
            navigation_retry: RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
            },
            eval_retry: RetryConfig {
                max_attempts: 2,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(2),
                backoff_multiplier: 1.5,
                jitter_factor: 0.2,
            },
            capture_retry: RetryConfig {
                max_attempts: 2,
                initial_delay: Duration::from_millis(50),
                max_delay: Duration::from_secs(1),
                backoff_multiplier: 1.2,
                jitter_factor: 0.3,
            },
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                timeout: Duration::from_secs(30),
                success_threshold: 2,
                window_duration: Duration::from_secs(60),
            },
            max_failed_tabs: 10,
            tab_recovery_timeout: 300, // 5 minutes
        }
    }
}

/// Browser resilience manager
pub struct BrowserResilienceManager {
    config: BrowserResilienceConfig,
    circuit_breakers: Arc<RwLock<std::collections::HashMap<BrowserOperation, Arc<CircuitBreaker>>>>,
    failed_tabs: Arc<RwLock<std::collections::HashMap<String, TabFailureInfo>>>,
    health_checks: Arc<RwLock<Vec<HealthCheck>>>,
}

/// Information about a failed tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabFailureInfo {
    pub tab_label: String,
    pub failure_count: u32,
    pub last_failure: u64,
    pub last_operation: BrowserOperation,
    pub is_recoverable: bool,
}

impl BrowserResilienceManager {
    pub fn new() -> Self {
        Self::with_config(BrowserResilienceConfig::default())
    }

    pub fn with_config(config: BrowserResilienceConfig) -> Self {
        let mut circuit_breakers = std::collections::HashMap::new();
        
        // Create circuit breakers for each operation type
        for op in [
            BrowserOperation::Navigate,
            BrowserOperation::Eval,
            BrowserOperation::Capture,
            BrowserOperation::Screenshot,
            BrowserOperation::CreateTab,
            BrowserOperation::CloseTab,
        ] {
            circuit_breakers.insert(
                op,
                Arc::new(CircuitBreaker::with_config(config.circuit_breaker.clone())),
            );
        }

        Self {
            config,
            circuit_breakers: Arc::new(RwLock::new(circuit_breakers)),
            failed_tabs: Arc::new(RwLock::new(std::collections::HashMap::new())),
            health_checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute navigation with retry and circuit breaking
    pub async fn navigate_with_resilience<F, Fut, T, E>(
        &self,
        operation: F,
        tab_label: &str,
    ) -> Result<T, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let cb = self.get_circuit_breaker(BrowserOperation::Navigate).await;
        
        // Check circuit breaker
        if !cb.allow_request().await {
            self.record_tab_failure(tab_label, BrowserOperation::Navigate, false).await;
            return Err("Navigation circuit breaker is open".to_string());
        }

        let result = retry_with_backoff(operation, self.config.navigation_retry.clone()).await;

        match &result {
            Ok(_) => {
                cb.record_success().await;
                self.clear_tab_failure(tab_label).await;
            }
            Err(_) => {
                cb.record_failure().await;
                self.record_tab_failure(tab_label, BrowserOperation::Navigate, true).await;
            }
        }

        result.map_err(|e| e.to_string())
    }

    /// Execute eval with retry and circuit breaking
    pub async fn eval_with_resilience<F, Fut, T, E>(
        &self,
        operation: F,
        _tab_label: &str,
    ) -> Result<T, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let cb = self.get_circuit_breaker(BrowserOperation::Eval).await;
        
        if !cb.allow_request().await {
            return Err("Eval circuit breaker is open".to_string());
        }

        let result = retry_with_backoff(operation, self.config.eval_retry.clone()).await;

        match &result {
            Ok(_) => cb.record_success().await,
            Err(_) => cb.record_failure().await,
        }

        result.map_err(|e| e.to_string())
    }

    /// Execute capture with retry and circuit breaking
    pub async fn capture_with_resilience<F, Fut, T, E>(
        &self,
        operation: F,
        _tab_label: &str,
    ) -> Result<T, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let cb = self.get_circuit_breaker(BrowserOperation::Capture).await;
        
        if !cb.allow_request().await {
            return Err("Capture circuit breaker is open".to_string());
        }

        let result = retry_with_backoff(operation, self.config.capture_retry.clone()).await;

        match &result {
            Ok(_) => cb.record_success().await,
            Err(_) => cb.record_failure().await,
        }

        result.map_err(|e| e.to_string())
    }

    /// Record a tab failure
    async fn record_tab_failure(&self, tab_label: &str, operation: BrowserOperation, recoverable: bool) {
        let mut failed_tabs = self.failed_tabs.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = failed_tabs.entry(tab_label.to_string()).or_insert(TabFailureInfo {
            tab_label: tab_label.to_string(),
            failure_count: 0,
            last_failure: 0,
            last_operation: operation,
            is_recoverable: true,
        });

        info.failure_count += 1;
        info.last_failure = now;
        info.last_operation = operation;
        info.is_recoverable = recoverable;

        tracing::warn!(
            "Tab {} failed: operation={:?}, count={}, recoverable={}",
            tab_label, operation, info.failure_count, recoverable
        );
    }

    /// Clear a tab failure
    async fn clear_tab_failure(&self, tab_label: &str) {
        let mut failed_tabs = self.failed_tabs.write().await;
        failed_tabs.remove(tab_label);
    }

    /// Get circuit breaker for an operation
    async fn get_circuit_breaker(&self, operation: BrowserOperation) -> Arc<CircuitBreaker> {
        let breakers = self.circuit_breakers.read().await;
        breakers.get(&operation).cloned().unwrap_or_else(|| {
            Arc::new(CircuitBreaker::with_config(self.config.circuit_breaker.clone()))
        })
    }

    /// Get circuit breaker state for an operation
    pub async fn get_circuit_breaker_state(&self, operation: BrowserOperation) -> CircuitState {
        let cb = self.get_circuit_breaker(operation).await;
        cb.get_state().await
    }

    /// Get all failed tabs
    pub async fn get_failed_tabs(&self) -> Vec<TabFailureInfo> {
        let failed_tabs = self.failed_tabs.read().await;
        failed_tabs.values().cloned().collect()
    }

    /// Attempt to recover a failed tab
    pub async fn recover_tab(&self, tab_label: &str) -> Result<(), String> {
        let mut failed_tabs = self.failed_tabs.write().await;
        
        if let Some(info) = failed_tabs.get(tab_label) {
            if !info.is_recoverable {
                return Err("Tab is not recoverable".to_string());
            }
            
            // Reset circuit breakers for this tab's operations
            let cb = self.get_circuit_breaker(info.last_operation).await;
            cb.reset().await;
            
            failed_tabs.remove(tab_label);
            tracing::info!("Tab {} recovered", tab_label);
            Ok(())
        } else {
            Err("Tab not in failed state".to_string())
        }
    }

    /// Record a health check
    pub async fn record_health_check(&self, check: HealthCheck) {
        let mut checks = self.health_checks.write().await;
        // Update existing check or add new one
        if let Some(existing) = checks.iter_mut().find(|c| c.service_name == check.service_name) {
            *existing = check;
        } else {
            checks.push(check);
        }
    }
    
    /// Create a health check with current timestamp
    pub fn create_health_check(service_name: &str, status: HealthStatus, message: &str) -> HealthCheck {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        HealthCheck {
            service_name: service_name.to_string(),
            status,
            message: message.to_string(),
            last_check: now,
        }
    }

    /// Get overall browser health
    pub async fn get_browser_health(&self) -> HealthStatus {
        let checks = self.health_checks.read().await;
        if checks.is_empty() {
            return HealthStatus::Healthy;
        }

        let unhealthy_count = checks.iter().filter(|c| c.status == HealthStatus::Unhealthy).count();
        let degraded_count = checks.iter().filter(|c| c.status == HealthStatus::Degraded).count();

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Clean up stale failure records
    pub async fn cleanup_stale_failures(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut failed_tabs = self.failed_tabs.write().await;
        let before_count = failed_tabs.len();
        
        failed_tabs.retain(|_, info| {
            now - info.last_failure < self.config.tab_recovery_timeout
        });

        let after_count = failed_tabs.len();
        if before_count > after_count {
            tracing::info!("Cleaned up {} stale tab failures", before_count - after_count);
        }
    }

    /// Reset all circuit breakers
    pub async fn reset_all_circuit_breakers(&self) {
        let breakers = self.circuit_breakers.read().await;
        for cb in breakers.values() {
            cb.reset().await;
        }
        tracing::info!("All circuit breakers reset");
    }
}

impl Default for BrowserResilienceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands for browser resilience
use tauri::{State, AppHandle};

/// Get circuit breaker state for an operation
#[tauri::command]
pub async fn get_circuit_breaker_state(
    operation: BrowserOperation,
    manager: State<'_, Arc<BrowserResilienceManager>>,
) -> Result<CircuitState, String> {
    Ok(manager.get_circuit_breaker_state(operation).await)
}

/// Get all failed tabs
#[tauri::command]
pub async fn get_failed_tabs(
    manager: State<'_, Arc<BrowserResilienceManager>>,
) -> Result<Vec<TabFailureInfo>, String> {
    Ok(manager.get_failed_tabs().await)
}

/// Recover a failed tab
#[tauri::command]
pub async fn recover_tab(
    tab_label: String,
    manager: State<'_, Arc<BrowserResilienceManager>>,
) -> Result<(), String> {
    manager.recover_tab(&tab_label).await
}

/// Get browser health status
#[tauri::command]
pub async fn get_browser_health(
    manager: State<'_, Arc<BrowserResilienceManager>>,
) -> Result<HealthStatus, String> {
    Ok(manager.get_browser_health().await)
}

/// Clean up stale failure records
#[tauri::command]
pub async fn cleanup_stale_failures(
    manager: State<'_, Arc<BrowserResilienceManager>>,
) -> Result<(), String> {
    manager.cleanup_stale_failures().await;
    Ok(())
}

/// Reset all circuit breakers
#[tauri::command]
pub async fn reset_all_circuit_breakers(
    manager: State<'_, Arc<BrowserResilienceManager>>,
) -> Result<(), String> {
    manager.reset_all_circuit_breakers().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_resilience_manager_creation() {
        let manager = BrowserResilienceManager::new();
        assert_eq!(manager.get_failed_tabs().await.len(), 0);
    }

    #[tokio::test]
    async fn test_navigation_with_resilience_success() {
        let manager = BrowserResilienceManager::new();
        
        let result = manager.navigate_with_resilience(
            || async { Ok::<(), String>(()) },
            "test-tab",
        ).await;

        assert!(result.is_ok());
        assert_eq!(manager.get_failed_tabs().await.len(), 0);
    }

    #[tokio::test]
    async fn test_navigation_with_resilience_failure() {
        let manager = BrowserResilienceManager::new();
        
        let result = manager.navigate_with_resilience(
            || async { Err::<(), String>("navigation failed".to_string()) },
            "test-tab",
        ).await;

        assert!(result.is_err());
        assert_eq!(manager.get_failed_tabs().await.len(), 1);
    }

    #[tokio::test]
    async fn test_tab_recovery() {
        let manager = BrowserResilienceManager::new();
        
        // Fail a tab
        let _ = manager.navigate_with_resilience(
            || async { Err::<(), String>("navigation failed".to_string()) },
            "test-tab",
        ).await;

        assert_eq!(manager.get_failed_tabs().await.len(), 1);

        // Recover the tab
        let result = manager.recover_tab("test-tab").await;
        assert!(result.is_ok());
        assert_eq!(manager.get_failed_tabs().await.len(), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_state() {
        let manager = BrowserResilienceManager::new();
        
        let state = manager.get_circuit_breaker_state(BrowserOperation::Navigate).await;
        assert_eq!(state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_health_check_recording() {
        let manager = BrowserResilienceManager::new();
        
        let check = manager.create_health_check("navigation", HealthStatus::Healthy, "OK");
        
        manager.record_health_check(check).await;
        assert_eq!(manager.get_browser_health().await, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_cleanup_stale_failures() {
        let manager = BrowserResilienceManager::with_config(BrowserResilienceConfig {
            tab_recovery_timeout: 1, // 1 second
            ..Default::default()
        });
        
        // Fail a tab
        let _ = manager.navigate_with_resilience(
            || async { Err::<(), String>("navigation failed".to_string()) },
            "test-tab",
        ).await;

        assert_eq!(manager.get_failed_tabs().await.len(), 1);

        // Wait for timeout
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup
        manager.cleanup_stale_failures().await;
        assert_eq!(manager.get_failed_tabs().await.len(), 0);
    }
}
