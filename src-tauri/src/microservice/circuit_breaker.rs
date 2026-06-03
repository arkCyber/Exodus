//! Circuit Breaker Pattern for Fault Tolerance
//!
//! Prevents cascading failures by stopping requests to failing services

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting recovery
    pub timeout: Duration,
    /// Number of successful calls needed to close circuit
    pub success_threshold: u32,
    /// Time window for counting failures
    pub window_duration: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
            window_duration: Duration::from_secs(30),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<u64>,
    pub last_state_change: u64,
    pub total_calls: u64,
    pub rejected_calls: u64,
}

/// Internal state of circuit breaker
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
    total_calls: u64,
    rejected_calls: u64,
}

/// Circuit Breaker for protecting service calls
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default config
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a circuit breaker with custom config
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                last_state_change: Instant::now(),
                total_calls: 0,
                rejected_calls: 0,
            })),
        }
    }

    /// Check if a call should be allowed
    pub async fn allow_request(&self) -> bool {
        let mut state = self.state.write().await;
        state.total_calls += 1;

        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if state.last_state_change.elapsed() >= self.config.timeout {
                    // Transition to half-open
                    state.state = CircuitState::HalfOpen;
                    state.success_count = 0;
                    state.failure_count = 0;
                    state.last_state_change = Instant::now();
                    true
                } else {
                    state.rejected_calls += 1;
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests to test service
                true
            }
        }
    }

    /// Record a successful call
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
                state.last_failure_time = None;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    // Close the circuit
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_state_change = Instant::now();
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                state.state = CircuitState::Closed;
                state.failure_count = 0;
                state.last_state_change = Instant::now();
            }
        }
    }

    /// Record a failed call
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::Closed => {
                // Check if we should open the circuit
                if state.failure_count >= self.config.failure_threshold {
                    state.state = CircuitState::Open;
                    state.last_state_change = Instant::now();
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                state.state = CircuitState::Open;
                state.success_count = 0;
                state.last_state_change = Instant::now();
            }
            CircuitState::Open => {
                // Already open, just update timestamp
                state.last_state_change = Instant::now();
            }
        }
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await;
        CircuitBreakerStats {
            state: state.state,
            failure_count: state.failure_count,
            success_count: state.success_count,
            last_failure_time: state.last_failure_time.map(|t| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs()
                    - t.elapsed().as_secs()
            }),
            last_state_change: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs()
                - state.last_state_change.elapsed().as_secs(),
            total_calls: state.total_calls,
            rejected_calls: state.rejected_calls,
        }
    }

    /// Reset the circuit breaker
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
        state.last_state_change = Instant::now();
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.state
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(1),
            success_threshold: 2,
            window_duration: Duration::from_secs(10),
        };
        let cb = CircuitBreaker::with_config(config);

        // Should be closed initially
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert!(cb.allow_request().await);

        // Record failures
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        // Should be open now
        assert_eq!(cb.get_state().await, CircuitState::Open);
        assert!(!cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            success_threshold: 2,
            window_duration: Duration::from_secs(10),
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        assert!(cb.allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Record successes to close
        cb.record_success().await;
        cb.record_success().await;

        // Should be closed now
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}
