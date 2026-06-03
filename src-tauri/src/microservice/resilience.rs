//! Unified Resilience and Fault Tolerance Patterns
//!
//! This module provides centralized fault tolerance mechanisms including:
//! - Retry with exponential backoff
//! - Circuit breaker integration
//! - Timeout management
//! - Fallback mechanisms
//! - Health checking

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use rand::Rng;

use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Random jitter to prevent thundering herd
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Retry policy for different scenarios
#[derive(Debug, Clone, Copy)]
pub enum RetryPolicy {
    /// No retry
    None,
    /// Quick retry for transient failures
    Transient,
    /// Aggressive retry for network issues
    Aggressive,
    /// Conservative retry for critical operations
    Conservative,
}

impl RetryPolicy {
    pub fn to_config(&self) -> RetryConfig {
        match self {
            RetryPolicy::None => RetryConfig {
                max_attempts: 1,
                ..Default::default()
            },
            RetryPolicy::Transient => RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(50),
                max_delay: Duration::from_secs(1),
                backoff_multiplier: 1.5,
                jitter_factor: 0.2,
            },
            RetryPolicy::Aggressive => RetryConfig {
                max_attempts: 5,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
            },
            RetryPolicy::Conservative => RetryConfig {
                max_attempts: 2,
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(2),
                backoff_multiplier: 1.2,
                jitter_factor: 0.3,
            },
        }
    }
}

/// Execute operation with retry and exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    let mut delay = config.initial_delay;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                
                // Don't retry on last attempt
                if attempt < config.max_attempts - 1 {
                    // Add jitter to prevent thundering herd
                    let jitter = if config.jitter_factor > 0.0 {
                        let mut rng = rand::thread_rng();
                        let jitter_amount = delay.as_millis() as f64 * config.jitter_factor;
                        Duration::from_millis(rng.gen_range(0..=jitter_amount as u64))
                    } else {
                        Duration::from_millis(0)
                    };
                    
                    tokio::time::sleep(delay + jitter).await;
                    
                    // Exponential backoff with cap
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * config.backoff_multiplier)
                            .min(config.max_delay.as_millis() as f64) as u64
                    );
                }
            }
        }
    }

    Err(last_error.expect("Expected error after all retry attempts failed"))
}

/// Resilient HTTP client with circuit breaker and retry
#[derive(Clone)]
pub struct ResilientHttpClient {
    client: reqwest::Client,
    circuit_breaker: Arc<CircuitBreaker>,
    retry_config: RetryConfig,
}

impl ResilientHttpClient {
    pub fn new() -> Self {
        Self::with_retry_config(RetryConfig::default())
    }

    pub fn with_retry_config(retry_config: RetryConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let circuit_breaker_config = CircuitBreakerConfig {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
            window_duration: Duration::from_secs(30),
        };

        Self {
            client,
            circuit_breaker: Arc::new(CircuitBreaker::with_config(circuit_breaker_config)),
            retry_config,
        }
    }

    pub fn with_circuit_breaker_config(
        retry_config: RetryConfig,
        cb_config: CircuitBreakerConfig,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            circuit_breaker: Arc::new(CircuitBreaker::with_config(cb_config)),
            retry_config,
        }
    }

    /// Execute HTTP GET with resilience patterns
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, String> {
        // Check circuit breaker
        if !self.circuit_breaker.allow_request().await {
            return Err("Circuit breaker is open".to_string());
        }

        let url = url.to_string();
        let client = self.client.clone();
        let cb = self.circuit_breaker.clone();

        let result = retry_with_backoff(
            || async {
                let response = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| format!("HTTP request failed: {e}"))?;

                if response.status().is_success() {
                    Ok(response)
                } else {
                    Err(format!("HTTP error: {}", response.status()))
                }
            },
            self.retry_config.clone(),
        ).await;

        match &result {
            Ok(_) => cb.record_success().await,
            Err(_) => cb.record_failure().await,
        }

        result
    }

    /// Execute HTTP POST with resilience patterns
    pub async fn post(&self, url: &str, body: serde_json::Value) -> Result<reqwest::Response, String> {
        // Check circuit breaker
        if !self.circuit_breaker.allow_request().await {
            return Err("Circuit breaker is open".to_string());
        }

        let url = url.to_string();
        let client = self.client.clone();
        let cb = self.circuit_breaker.clone();

        let result = retry_with_backoff(
            || async {
                let response = client
                    .post(&url)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("HTTP request failed: {e}"))?;

                if response.status().is_success() {
                    Ok(response)
                } else {
                    Err(format!("HTTP error: {}", response.status()))
                }
            },
            self.retry_config.clone(),
        ).await;

        match &result {
            Ok(_) => cb.record_success().await,
            Err(_) => cb.record_failure().await,
        }

        result
    }

    /// Get circuit breaker stats
    pub async fn get_circuit_breaker_stats(&self) -> super::circuit_breaker::CircuitBreakerStats {
        self.circuit_breaker.get_stats().await
    }

    /// Reset circuit breaker
    pub async fn reset_circuit_breaker(&self) {
        self.circuit_breaker.reset().await;
    }
}

impl Default for ResilientHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub service_name: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_check: Instant,
}

/// Health monitor for tracking service health
pub struct HealthMonitor {
    checks: Arc<RwLock<Vec<HealthCheck>>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a health check result
    pub async fn record_check(&self, check: HealthCheck) {
        let mut checks = self.checks.write().await;
        // Update existing check or add new one
        if let Some(existing) = checks.iter_mut().find(|c| c.service_name == check.service_name) {
            *existing = check;
        } else {
            checks.push(check);
        }
    }

    /// Get all health checks
    pub async fn get_all_checks(&self) -> Vec<HealthCheck> {
        self.checks.read().await.clone()
    }

    /// Get health status for a specific service
    pub async fn get_service_health(&self, service_name: &str) -> Option<HealthCheck> {
        self.checks
            .read()
            .await
            .iter()
            .find(|c| c.service_name == service_name)
            .cloned()
    }

    /// Get overall system health
    pub async fn get_system_health(&self) -> HealthStatus {
        let checks = self.checks.read().await;
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
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback mechanism for service degradation
pub struct Fallback<T> {
    primary: Option<T>,
    fallback: Option<T>,
    fallback_triggered: Arc<RwLock<bool>>,
}

impl<T> Fallback<T> {
    pub fn new() -> Self {
        Self {
            primary: None,
            fallback: None,
            fallback_triggered: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_primary(primary: T) -> Self {
        Self {
            primary: Some(primary),
            fallback: None,
            fallback_triggered: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_fallback(primary: T, fallback: T) -> Self {
        Self {
            primary: Some(primary),
            fallback: Some(fallback),
            fallback_triggered: Arc::new(RwLock::new(false)),
        }
    }

    pub fn set_primary(&mut self, primary: T) {
        self.primary = Some(primary);
        // Note: fallback_triggered reset should be done in async context
    }

    pub async fn reset_fallback_trigger(&self) {
        *self.fallback_triggered.write().await = false;
    }

    pub fn set_fallback(&mut self, fallback: T) {
        self.fallback = Some(fallback);
    }

    pub async fn is_fallback_active(&self) -> bool {
        *self.fallback_triggered.read().await
    }

    /// Execute operation with fallback
    pub async fn execute<F, Fut, R>(&self, operation: F) -> Result<R, String>
    where
        F: Fn(&T) -> Fut,
        Fut: std::future::Future<Output = Result<R, String>>,
    {
        if let Some(primary) = &self.primary {
            match operation(primary).await {
                Ok(result) => {
                    // Reset fallback trigger on success
                    *self.fallback_triggered.write().await = false;
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Primary operation failed: {}, trying fallback", e);
                    *self.fallback_triggered.write().await = true;
                }
            }
        }

        if let Some(fallback) = &self.fallback {
            operation(fallback).await
        } else {
            Err("No primary or fallback available".to_string())
        }
    }

    /// Execute operation with custom fallback function
    pub async fn execute_with_custom_fallback<P, FB, Fut, R>(
        &self,
        primary_op: P,
        fallback_op: FB,
    ) -> Result<R, String>
    where
        P: FnOnce() -> Fut,
        FB: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<R, String>>,
    {
        if self.primary.is_some() {
            match primary_op().await {
                Ok(result) => {
                    *self.fallback_triggered.write().await = false;
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Primary operation failed: {}, trying fallback", e);
                    *self.fallback_triggered.write().await = true;
                }
            }
        }

        fallback_op().await
    }
}

impl<T> Default for Fallback<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Service degradation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationLevel {
    Full,
    Degraded,
    Minimal,
    Offline,
}

/// Service degradation manager
pub struct DegradationManager {
    current_level: Arc<RwLock<DegradationLevel>>,
    thresholds: DegradationThresholds,
}

/// Thresholds for triggering degradation
#[derive(Debug, Clone)]
pub struct DegradationThresholds {
    pub error_rate_threshold: f64,
    pub latency_threshold_ms: u64,
    pub circuit_breaker_threshold: u32,
}

impl Default for DegradationThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.5, // 50% error rate
            latency_threshold_ms: 5000, // 5 seconds
            circuit_breaker_threshold: 5, // 5 failures
        }
    }
}

impl DegradationManager {
    pub fn new() -> Self {
        Self {
            current_level: Arc::new(RwLock::new(DegradationLevel::Full)),
            thresholds: DegradationThresholds::default(),
        }
    }

    pub fn with_thresholds(thresholds: DegradationThresholds) -> Self {
        Self {
            current_level: Arc::new(RwLock::new(DegradationLevel::Full)),
            thresholds,
        }
    }

    pub async fn get_current_level(&self) -> DegradationLevel {
        *self.current_level.read().await
    }

    pub async fn set_level(&self, level: DegradationLevel) {
        let old_level = *self.current_level.read().await;
        if old_level != level {
            tracing::info!("Service degradation level changed: {:?} -> {:?}", old_level, level);
            *self.current_level.write().await = level;
        }
    }

    pub async fn should_degrade(&self, error_rate: f64, latency_ms: u64, circuit_failures: u32) -> bool {
        error_rate > self.thresholds.error_rate_threshold
            || latency_ms > self.thresholds.latency_threshold_ms
            || circuit_failures > self.thresholds.circuit_breaker_threshold
    }

    pub async fn evaluate_and_adjust(
        &self,
        error_rate: f64,
        latency_ms: u64,
        circuit_failures: u32,
    ) {
        if self.should_degrade(error_rate, latency_ms, circuit_failures).await {
            let current = *self.current_level.read().await;
            let new_level = match current {
                DegradationLevel::Full => DegradationLevel::Degraded,
                DegradationLevel::Degraded => DegradationLevel::Minimal,
                DegradationLevel::Minimal => DegradationLevel::Offline,
                DegradationLevel::Offline => DegradationLevel::Offline,
            };
            self.set_level(new_level).await;
        } else {
            // Recover gradually
            let current = *self.current_level.read().await;
            let new_level = match current {
                DegradationLevel::Offline => DegradationLevel::Minimal,
                DegradationLevel::Minimal => DegradationLevel::Degraded,
                DegradationLevel::Degraded => DegradationLevel::Full,
                DegradationLevel::Full => DegradationLevel::Full,
            };
            self.set_level(new_level).await;
        }
    }
}

impl Default for DegradationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();
        
        let result = retry_with_backoff(
            || async {
                let count = attempts_clone.fetch_add(1, Ordering::SeqCst);
                if count < 1 {
                    Err("transient error")
                } else {
                    Ok("success")
                }
            },
            RetryConfig::default(),
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.expect("Expected successful result"), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_exhausted() {
        let config = RetryConfig {
            max_attempts: 3,
            ..Default::default()
        };

        let result = retry_with_backoff(
            || async { Err::<(), &str>("persistent error") },
            config,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        
        let check = HealthCheck {
            service_name: "test-service".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            last_check: Instant::now(),
        };
        
        monitor.record_check(check.clone()).await;
        
        let retrieved = monitor.get_service_health("test-service").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Expected retrieved health status").status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_monitor_system_health() {
        let monitor = HealthMonitor::new();
        
        // Add healthy check
        monitor.record_check(HealthCheck {
            service_name: "service-a".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            last_check: Instant::now(),
        }).await;
        
        assert_eq!(monitor.get_system_health().await, HealthStatus::Healthy);
        
        // Add degraded check
        monitor.record_check(HealthCheck {
            service_name: "service-b".to_string(),
            status: HealthStatus::Degraded,
            message: "Slow".to_string(),
            last_check: Instant::now(),
        }).await;
        
        assert_eq!(monitor.get_system_health().await, HealthStatus::Degraded);
        
        // Add unhealthy check
        monitor.record_check(HealthCheck {
            service_name: "service-c".to_string(),
            status: HealthStatus::Unhealthy,
            message: "Failed".to_string(),
            last_check: Instant::now(),
        }).await;
        
        assert_eq!(monitor.get_system_health().await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_fallback_triggers() {
        let fallback = Fallback::with_fallback("primary".to_string(), "fallback".to_string());
        
        let result = fallback.execute(|_: &String| async move { Err::<String, String>("primary failed".to_string()) }).await;
        assert!(result.is_err()); // Fallback also fails
        assert!(fallback.is_fallback_active().await);
    }

    #[tokio::test]
    async fn test_degradation_manager() {
        let manager = DegradationManager::new();
        
        assert_eq!(manager.get_current_level().await, DegradationLevel::Full);
        
        // Trigger degradation
        manager.set_level(DegradationLevel::Degraded).await;
        assert_eq!(manager.get_current_level().await, DegradationLevel::Degraded);
        
        // Test should_degrade
        assert!(manager.should_degrade(0.6, 6000, 6).await);
        assert!(!manager.should_degrade(0.3, 2000, 2).await);
    }

    #[tokio::test]
    async fn test_degradation_manager_auto_adjust() {
        let manager = DegradationManager::new();
        
        // Should degrade with high error rate
        manager.evaluate_and_adjust(0.8, 6000, 10).await;
        assert_eq!(manager.get_current_level().await, DegradationLevel::Degraded);
        
        // Should recover with low error rate
        manager.evaluate_and_adjust(0.1, 1000, 1).await;
        assert_eq!(manager.get_current_level().await, DegradationLevel::Full);
    }

    #[tokio::test]
    async fn test_retry_policy_configs() {
        let transient = RetryPolicy::Transient.to_config();
        assert_eq!(transient.max_attempts, 3);
        assert_eq!(transient.initial_delay, Duration::from_millis(50));
        
        let aggressive = RetryPolicy::Aggressive.to_config();
        assert_eq!(aggressive.max_attempts, 5);
        assert_eq!(aggressive.max_delay, Duration::from_secs(10));
        
        let conservative = RetryPolicy::Conservative.to_config();
        assert_eq!(conservative.max_attempts, 2);
    }
}
