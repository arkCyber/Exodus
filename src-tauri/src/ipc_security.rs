//! Exodus Browser — IPC Security
//!
//! Provides aerospace-grade security for inter-process communication (IPC)
//! including message validation, sanitization, and access control.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// IPC message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IpcMessageType {
    Command,
    Event,
    Response,
    Error,
}

/// IPC message priority
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IpcPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// IPC security level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecurityLevel {
    Untrusted,
    Low,
    Medium,
    High,
    Critical,
}

/// IPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcMessage {
    pub message_id: String,
    pub message_type: IpcMessageType,
    pub source: String,
    pub destination: String,
    pub command: String,
    pub payload: serde_json::Value,
    pub priority: IpcPriority,
    pub security_level: SecurityLevel,
    pub timestamp: u64,
    pub ttl: Option<u64>, // Time to live in seconds
}

/// IPC security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicy {
    pub allowed_commands: HashSet<String>,
    pub blocked_commands: HashSet<String>,
    pub allowed_sources: HashSet<String>,
    pub blocked_sources: HashSet<String>,
    pub max_message_size: usize,
    pub rate_limit_per_second: u32,
    pub require_encryption: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allowed_commands: HashSet::new(),
            blocked_commands: HashSet::new(),
            allowed_sources: HashSet::new(),
            blocked_sources: HashSet::new(),
            max_message_size: 10 * 1024 * 1024, // 10MB
            rate_limit_per_second: 100,
            require_encryption: false,
        }
    }
}

/// IPC security error
#[derive(Debug)]
pub enum IpcSecurityError {
    MessageTooLarge,
    CommandNotAllowed,
    SourceNotAllowed,
    RateLimitExceeded,
    MessageExpired,
    InvalidPayload,
    SanitizationFailed(String),
}

impl std::fmt::Display for IpcSecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcSecurityError::MessageTooLarge => write!(f, "Message exceeds size limit"),
            IpcSecurityError::CommandNotAllowed => write!(f, "Command not allowed"),
            IpcSecurityError::SourceNotAllowed => write!(f, "Source not allowed"),
            IpcSecurityError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            IpcSecurityError::MessageExpired => write!(f, "Message has expired"),
            IpcSecurityError::InvalidPayload => write!(f, "Invalid payload"),
            IpcSecurityError::SanitizationFailed(msg) => write!(f, "Sanitization failed: {}", msg),
        }
    }
}

impl std::error::Error for IpcSecurityError {}

/// Rate limiter
#[derive(Debug, Clone)]
struct RateLimiter {
    timestamps: Arc<Mutex<Vec<u64>>>,
    limit: u32,
}

impl RateLimiter {
    fn new(limit: u32) -> Self {
        Self {
            timestamps: Arc::new(Mutex::new(Vec::new())),
            limit,
        }
    }

    fn check(&self) -> Result<(), IpcSecurityError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut timestamps = self.timestamps.lock()
            .map_err(|_| IpcSecurityError::InternalError("Failed to acquire timestamps lock".to_string()))?;
        
        // Remove timestamps older than 1 second
        timestamps.retain(|&t| now - t < 1);

        if timestamps.len() as u32 >= self.limit {
            return Err(IpcSecurityError::RateLimitExceeded);
        }

        timestamps.push(now);
        Ok(())
    }
}

/// IPC security manager
pub struct IpcSecurityManager {
    policy: Arc<Mutex<SecurityPolicy>>,
    rate_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    message_history: Arc<Mutex<Vec<IpcMessage>>>,
    max_history_size: usize,
}

impl IpcSecurityManager {
    /// Create a new IPC security manager
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy: Arc::new(Mutex::new(policy)),
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            message_history: Arc::new(Mutex::new(Vec::new())),
            max_history_size: 10000,
        }
    }

    /// Validate an IPC message
    pub fn validate_message(&self, message: &IpcMessage) -> Result<(), IpcSecurityError> {
        let policy = self.policy.lock()
            .map_err(|_| IpcSecurityError::InternalError("Failed to acquire policy lock".to_string()))?;

        // Check message size
        let message_size = serde_json::to_vec(message)
            .map_err(|_| IpcSecurityError::InvalidPayload)?
            .len();
        if message_size > policy.max_message_size {
            return Err(IpcSecurityError::MessageTooLarge);
        }

        // Check if command is blocked
        if policy.blocked_commands.contains(&message.command) {
            return Err(IpcSecurityError::CommandNotAllowed);
        }

        // Check if command is allowed (if allowed_commands is not empty)
        if !policy.allowed_commands.is_empty() && !policy.allowed_commands.contains(&message.command) {
            return Err(IpcSecurityError::CommandNotAllowed);
        }

        // Check if source is blocked
        if policy.blocked_sources.contains(&message.source) {
            return Err(IpcSecurityError::SourceNotAllowed);
        }

        // Check if source is allowed (if allowed_sources is not empty)
        if !policy.allowed_sources.is_empty() && !policy.allowed_sources.contains(&message.source) {
            return Err(IpcSecurityError::SourceNotAllowed);
        }

        // Check message expiration
        if let Some(ttl) = message.ttl {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now > message.timestamp + ttl {
                return Err(IpcSecurityError::MessageExpired);
            }
        }

        // Check rate limit
        let mut rate_limiters = self.rate_limiters.lock()
            .map_err(|_| IpcSecurityError::InternalError("Failed to acquire rate limiters lock".to_string()))?;
        let rate_limiter = rate_limiters
            .entry(message.source.clone())
            .or_insert_with(|| RateLimiter::new(policy.rate_limit_per_second));
        rate_limiter.check()?;

        Ok(())
    }

    /// Sanitize an IPC message payload
    pub fn sanitize_payload(&self, payload: &serde_json::Value) -> Result<serde_json::Value, IpcSecurityError> {
        // Remove potentially dangerous fields
        let mut sanitized = payload.clone();

        if let Some(obj) = sanitized.as_object_mut() {
            // Remove common dangerous keys
            obj.remove("__proto__");
            obj.remove("constructor");
            obj.remove("prototype");
            
            // Sanitize string values
            for value in obj.values_mut() {
                self.sanitize_value(value)?;
            }
        }

        Ok(sanitized)
    }

    /// Sanitize a JSON value recursively
    fn sanitize_value(&self, value: &mut serde_json::Value) -> Result<(), IpcSecurityError> {
        match value {
            serde_json::Value::String(s) => {
                // Remove null bytes and control characters
                *s = s.chars()
                    .filter(|c| *c as u32 >= 32 || *c == '\n' || *c == '\r' || *c == '\t')
                    .collect();
                
                // Limit string length
                if s.len() > 100000 {
                    *s = s.chars().take(100000).collect();
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.sanitize_value(item)?;
                }
            }
            serde_json::Value::Object(obj) => {
                for value in obj.values_mut() {
                    self.sanitize_value(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Process an IPC message (validate and sanitize)
    pub fn process_message(&self, mut message: IpcMessage) -> Result<IpcMessage, IpcSecurityError> {
        // Validate message
        self.validate_message(&message)?;

        // Sanitize payload
        message.payload = self.sanitize_payload(&message.payload)?;

        // Add to history
        self.add_to_history(message.clone());

        Ok(message)
    }

    /// Add message to history
    fn add_to_history(&self, message: IpcMessage) {
        let mut history = self.message_history.lock()
            .expect("Failed to acquire message history lock");
        history.push(message);

        // Trim history if it exceeds max size
        if history.len() > self.max_history_size {
            history.truncate(self.max_history_size);
        }
    }

    /// Get message history
    pub fn get_message_history(&self, limit: usize) -> Vec<IpcMessage> {
        let history = self.message_history.lock()
            .expect("Failed to acquire message history lock");
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get security policy
    pub fn get_policy(&self) -> SecurityPolicy {
        self.policy.lock()
            .expect("Failed to acquire policy lock")
            .clone()
    }

    /// Set security policy
    pub fn set_policy(&self, policy: SecurityPolicy) {
        let mut p = self.policy.lock()
            .expect("Failed to acquire policy lock");
        *p = policy;
    }

    /// Add allowed command
    pub fn add_allowed_command(&self, command: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.allowed_commands.insert(command);
    }

    /// Remove allowed command
    pub fn remove_allowed_command(&self, command: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.allowed_commands.remove(&command);
    }

    /// Add blocked command
    pub fn add_blocked_command(&self, command: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.blocked_commands.insert(command);
    }

    /// Remove blocked command
    pub fn remove_blocked_command(&self, command: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.blocked_commands.remove(&command);
    }

    /// Add allowed source
    pub fn add_allowed_source(&self, source: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.allowed_sources.insert(source);
    }

    /// Remove allowed source
    pub fn remove_allowed_source(&self, source: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.allowed_sources.remove(&source);
    }

    /// Add blocked source
    pub fn add_blocked_source(&self, source: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.blocked_sources.insert(source);
    }

    /// Remove blocked source
    pub fn remove_blocked_source(&self, source: String) {
        let mut policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        policy.blocked_sources.remove(&source);
    }

    /// Clear message history
    pub fn clear_history(&self) {
        let mut history = self.message_history.lock()
            .expect("Failed to acquire message history lock");
        history.clear();
    }

    /// Get statistics
    pub fn get_stats(&self) -> IpcSecurityStats {
        let history = self.message_history.lock()
            .expect("Failed to acquire message history lock");
        let policy = self.policy.lock()
            .expect("Failed to acquire policy lock");
        
        IpcSecurityStats {
            total_messages: history.len(),
            allowed_commands: policy.allowed_commands.len(),
            blocked_commands: policy.blocked_commands.len(),
            allowed_sources: policy.allowed_sources.len(),
            blocked_sources: policy.blocked_sources.len(),
            max_message_size: policy.max_message_size,
            rate_limit: policy.rate_limit_per_second,
        }
    }
}

/// IPC security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcSecurityStats {
    pub total_messages: usize,
    pub allowed_commands: usize,
    pub blocked_commands: usize,
    pub allowed_sources: usize,
    pub blocked_sources: usize,
    pub max_message_size: usize,
    pub rate_limit: u32,
}

impl Default for IpcSecurityManager {
    fn default() -> Self {
        Self::new(SecurityPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_security_manager_creation() {
        let manager = IpcSecurityManager::new(SecurityPolicy::default());
        assert!(manager.get_stats().total_messages == 0);
    }

    #[test]
    fn test_message_validation() {
        let manager = IpcSecurityManager::new(SecurityPolicy::default());
        
        let message = IpcMessage {
            message_id: "test-1".to_string(),
            message_type: IpcMessageType::Command,
            source: "test-source".to_string(),
            destination: "test-dest".to_string(),
            command: "test-command".to_string(),
            payload: serde_json::json!({"data": "test"}),
            priority: IpcPriority::Normal,
            security_level: SecurityLevel::Medium,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: Some(60),
        };

        assert!(manager.validate_message(&message).is_ok());
    }

    #[test]
    fn test_blocked_command() {
        let mut policy = SecurityPolicy::default();
        policy.blocked_commands.insert("blocked-cmd".to_string());
        let manager = IpcSecurityManager::new(policy);
        
        let message = IpcMessage {
            message_id: "test-1".to_string(),
            message_type: IpcMessageType::Command,
            source: "test-source".to_string(),
            destination: "test-dest".to_string(),
            command: "blocked-cmd".to_string(),
            payload: serde_json::json!({"data": "test"}),
            priority: IpcPriority::Normal,
            security_level: SecurityLevel::Medium,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: Some(60),
        };

        assert!(manager.validate_message(&message).is_err());
    }

    #[test]
    fn test_payload_sanitization() {
        let manager = IpcSecurityManager::new(SecurityPolicy::default());
        
        let payload = serde_json::json!({
            "__proto__": "dangerous",
            "constructor": "dangerous",
            "safe": "data"
        });

        let sanitized = manager.sanitize_payload(&payload).unwrap();
        
        assert!(sanitized.get("__proto__").is_none());
        assert!(sanitized.get("constructor").is_none());
        assert_eq!(sanitized.get("safe"), Some(&serde_json::json!("data")));
    }

    #[test]
    fn test_rate_limiting() {
        let mut policy = SecurityPolicy::default();
        policy.rate_limit_per_second = 5;
        let manager = IpcSecurityManager::new(policy);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for _ in 0..5 {
            let message = IpcMessage {
                message_id: "test-1".to_string(),
                message_type: IpcMessageType::Command,
                source: "test-source".to_string(),
                destination: "test-dest".to_string(),
                command: "test-command".to_string(),
                payload: serde_json::json!({"data": "test"}),
                priority: IpcPriority::Normal,
                security_level: SecurityLevel::Medium,
                timestamp: now,
                ttl: Some(60),
            };
            assert!(manager.validate_message(&message).is_ok());
        }

        // 6th message should fail
        let message = IpcMessage {
            message_id: "test-2".to_string(),
            message_type: IpcMessageType::Command,
            source: "test-source".to_string(),
            destination: "test-dest".to_string(),
            command: "test-command".to_string(),
            payload: serde_json::json!({"data": "test"}),
            priority: IpcPriority::Normal,
            security_level: SecurityLevel::Medium,
            timestamp: now,
            ttl: Some(60),
        };
        assert!(manager.validate_message(&message).is_err());
    }
}
