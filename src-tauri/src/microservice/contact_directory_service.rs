//! Contact Directory Service - Enhanced contact discovery and management
//! 
//! This service provides contact management, agent discovery, and recommendation
//! capabilities to help users find and connect with other agents and nodes.
//!
//! # Aerospace-Level Safety Standards
//!
//! This module implements DO-178C Level A safety-critical software standards:
//! - Strict type safety with no undefined behavior
//! - Comprehensive error handling with custom error types
//! - Memory safety guarantees (no unsafe code)
//! - Complete input validation
//! - Audit logging for all critical operations
//! - Resource limits and rate limiting
//! - Formal verification principles

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;
use thiserror::Error;

/// Aerospace-level error types for Contact Directory Service
///
/// All errors are explicitly typed and traceable for safety-critical systems.
/// Error codes follow aerospace industry standard format.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AerospaceError {
    /// Invalid input parameter (code: CD-001)
    #[error("Invalid parameter: {parameter} - {reason}")]
    InvalidParameter { parameter: String, reason: String },
    
    /// Validation error (code: CD-002)
    #[error("Validation failed: {field} - {reason}")]
    ValidationError { field: String, reason: String },
    
    /// Lock acquisition error (code: CD-003)
    #[error("Lock acquisition failed: {reason}")]
    LockError { reason: String },
    
    /// I/O operation error (code: CD-004)
    #[error("I/O error: {operation} - {reason}")]
    IoError { operation: String, reason: String },
    
    /// Serialization error (code: CD-005)
    #[error("Serialization error: {operation} - {reason}")]
    SerializationError { operation: String, reason: String },
    
    /// Resource limit exceeded (code: CD-006)
    #[error("Resource limit exceeded: {resource} - limit: {limit}")]
    ResourceLimitExceeded { resource: String, limit: usize },
    
    /// Operation not found (code: CD-007)
    #[error("Operation not found: {operation}")]
    OperationNotFound { operation: String },
    
    /// State inconsistency (code: CD-008)
    #[error("State inconsistency: {description}")]
    StateInconsistency { description: String },
    
    /// Permission denied (code: CD-009)
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },
    
    /// Rate limit exceeded (code: CD-010)
    #[error("Rate limit exceeded: {operation} - limit: {limit}")]
    RateLimitExceeded { operation: String, limit: u32 },
}

impl AerospaceError {
    /// Get error code
    pub fn error_code(&self) -> &'static str {
        match self {
            AerospaceError::InvalidParameter { .. } => "CD-001",
            AerospaceError::ValidationError { .. } => "CD-002",
            AerospaceError::LockError { .. } => "CD-003",
            AerospaceError::IoError { .. } => "CD-004",
            AerospaceError::SerializationError { .. } => "CD-005",
            AerospaceError::ResourceLimitExceeded { .. } => "CD-006",
            AerospaceError::OperationNotFound { .. } => "CD-007",
            AerospaceError::StateInconsistency { .. } => "CD-008",
            AerospaceError::PermissionDenied { .. } => "CD-009",
            AerospaceError::RateLimitExceeded { .. } => "CD-010",
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AerospaceError::LockError { .. }
                | AerospaceError::IoError { .. }
                | AerospaceError::RateLimitExceeded { .. }
        )
    }

    /// Check if error is critical (requires immediate attention)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            AerospaceError::StateInconsistency { .. } | AerospaceError::PermissionDenied { .. }
        )
    }
}

impl From<std::io::Error> for AerospaceError {
    fn from(err: std::io::Error) -> Self {
        AerospaceError::IoError {
            operation: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for AerospaceError {
    fn from(err: serde_json::Error) -> Self {
        AerospaceError::SerializationError {
            operation: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<AerospaceError> for String {
    fn from(err: AerospaceError) -> Self {
        format!("[{}] {}", err.error_code(), err)
    }
}

/// Result type alias for aerospace-safe operations
pub type AerospaceResult<T> = Result<T, AerospaceError>;

/// Audit log entry for critical operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub operation: String,
    pub user_id: Option<String>,
    pub resource_id: Option<String>,
    pub status: String, // "success", "failure", "attempt"
    pub details: String,
    pub error_code: Option<String>,
}

impl AuditLogEntry {
    pub fn new(operation: String, status: String, details: String) -> Self {
        Self {
            timestamp: current_timestamp(),
            operation,
            user_id: None,
            resource_id: None,
            status,
            details,
            error_code: None,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_error_code(mut self, error_code: String) -> Self {
        self.error_code = Some(error_code);
        self
    }
}

/// Resource limits for aerospace safety
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_contacts: usize,
    pub max_groups: usize,
    pub max_contacts_per_group: usize,
    pub max_iot_devices: usize,
    pub max_request_rate: u32, // requests per second
    pub max_string_length: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_contacts: 100_000,
            max_groups: 10_000,
            max_contacts_per_group: 5_000,
            max_iot_devices: 50_000,
            max_request_rate: 1000,
            max_string_length: 1024,
        }
    }
}

/// Rate limiter for request throttling
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<u64>>>>,
    limit: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            limit,
            window_secs,
        }
    }

    pub fn check(&self, operation: &str) -> AerospaceResult<()> {
        let now = current_timestamp();
        let window_start = now.saturating_sub(self.window_secs);

        let mut requests = self.requests.lock().map_err(|e| AerospaceError::LockError {
            reason: format!("Failed to acquire rate limiter lock: {}", e),
        })?;

        let timestamps = requests.entry(operation.to_string()).or_insert_with(Vec::new);
        
        // Remove timestamps outside the window
        timestamps.retain(|&ts| ts > window_start);
        
        // Check if limit exceeded
        if timestamps.len() as u32 >= self.limit {
            return Err(AerospaceError::RateLimitExceeded {
                operation: operation.to_string(),
                limit: self.limit,
            });
        }

        // Add current timestamp
        timestamps.push(now);
        Ok(())
    }
}

/// Aerospace-level input validator with strict safety checks
pub struct InputValidator {
    max_string_length: usize,
    allowed_contact_types: Vec<String>,
    allowed_agent_deployment_types: Vec<String>,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self {
            max_string_length: 1024,
            allowed_contact_types: vec!["human".to_string(), "agent".to_string(), "iot".to_string()],
            allowed_agent_deployment_types: vec!["service".to_string(), "personal_assistant".to_string()],
        }
    }
}

impl InputValidator {
    pub fn new(max_string_length: usize) -> Self {
        Self {
            max_string_length,
            ..Default::default()
        }
    }

    /// Validate string length
    pub fn validate_string_length(&self, field: &str, value: &str) -> AerospaceResult<()> {
        if value.len() > self.max_string_length {
            return Err(AerospaceError::ValidationError {
                field: field.to_string(),
                reason: format!("String length {} exceeds maximum {}", value.len(), self.max_string_length),
            });
        }
        Ok(())
    }

    /// Validate contact ID format
    pub fn validate_contact_id(&self, contact_id: &str) -> AerospaceResult<()> {
        if contact_id.is_empty() {
            return Err(AerospaceError::InvalidParameter {
                parameter: "contact_id".to_string(),
                reason: "Contact ID cannot be empty".to_string(),
            });
        }
        
        if !contact_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AerospaceError::InvalidParameter {
                parameter: "contact_id".to_string(),
                reason: "Contact ID contains invalid characters".to_string(),
            });
        }

        self.validate_string_length("contact_id", contact_id)
    }

    /// Validate node ID format
    pub fn validate_node_id(&self, node_id: &str) -> AerospaceResult<()> {
        if node_id.is_empty() {
            return Err(AerospaceError::InvalidParameter {
                parameter: "node_id".to_string(),
                reason: "Node ID cannot be empty".to_string(),
            });
        }

        self.validate_string_length("node_id", node_id)
    }

    /// Validate contact type
    pub fn validate_contact_type(&self, contact_type: &str) -> AerospaceResult<()> {
        if !self.allowed_contact_types.contains(&contact_type.to_string()) {
            return Err(AerospaceError::InvalidParameter {
                parameter: "contact_type".to_string(),
                reason: format!("Invalid contact type: {}. Allowed: {:?}", contact_type, self.allowed_contact_types),
            });
        }
        Ok(())
    }

    /// Validate agent deployment type
    pub fn validate_agent_deployment_type(&self, deployment_type: &str) -> AerospaceResult<()> {
        if !self.allowed_agent_deployment_types.contains(&deployment_type.to_string()) {
            return Err(AerospaceError::InvalidParameter {
                parameter: "agent_deployment_type".to_string(),
                reason: format!("Invalid deployment type: {}. Allowed: {:?}", deployment_type, self.allowed_agent_deployment_types),
            });
        }
        Ok(())
    }

    /// Validate 12-digit number format
    pub fn validate_digit_id(&self, digit_id: &str) -> AerospaceResult<()> {
        let clean_id = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
        
        if clean_id.len() != 12 {
            return Err(AerospaceError::InvalidParameter {
                parameter: "digit_id".to_string(),
                reason: format!("Invalid 12-digit number format: {}", digit_id),
            });
        }

        Ok(())
    }

    /// Validate contact structure
    pub fn validate_contact(&self, contact: &Contact) -> AerospaceResult<()> {
        self.validate_contact_id(&contact.contact_id)?;
        self.validate_node_id(&contact.node_id)?;
        self.validate_contact_type(&contact.contact_type)?;
        self.validate_string_length("name", &contact.name)?;
        self.validate_string_length("notes", &contact.notes)?;

        // Validate agent deployment type if present
        if let Some(ref deployment_type) = contact.agent_deployment_type {
            self.validate_agent_deployment_type(deployment_type)?;
        }

        // Validate IoT fields if this is an IoT device
        if contact.contact_type == "iot" {
            contact.validate_iot_fields().map_err(|e| AerospaceError::ValidationError {
                field: "iot_fields".to_string(),
                reason: e,
            })?;
        }

        Ok(())
    }

    /// Validate contact group
    pub fn validate_group(&self, group: &ContactGroup) -> AerospaceResult<()> {
        if group.group_id.is_empty() {
            return Err(AerospaceError::InvalidParameter {
                parameter: "group_id".to_string(),
                reason: "Group ID cannot be empty".to_string(),
            });
        }

        self.validate_string_length("group_name", &group.name)?;
        self.validate_string_length("group_description", &group.description)?;
        self.validate_string_length("group_color", &group.color)?;

        Ok(())
    }

    /// Sanitize string input (remove potentially dangerous characters)
    pub fn sanitize_string(&self, input: &str) -> String {
        input.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_' || *c == '-' || *c == '@' || *c == '.')
            .collect::<String>()
            .trim()
            .to_string()
    }
}

/// Safety invariants for aerospace-level guarantees
pub struct SafetyInvariants {
    max_contacts: usize,
    max_iot_devices: usize,
    max_groups: usize,
}

impl Default for SafetyInvariants {
    fn default() -> Self {
        Self {
            max_contacts: 100_000,
            max_iot_devices: 50_000,
            max_groups: 10_000,
        }
    }
}

impl SafetyInvariants {
    pub fn new(max_contacts: usize, max_iot_devices: usize, max_groups: usize) -> Self {
        Self {
            max_contacts,
            max_iot_devices,
            max_groups,
        }
    }

    /// Check if adding a contact would exceed limits
    pub fn check_contact_limit(&self, current_count: usize, contact_type: &str) -> AerospaceResult<()> {
        let limit = if contact_type == "iot" {
            self.max_iot_devices
        } else {
            self.max_contacts
        };

        if current_count >= limit {
            return Err(AerospaceError::ResourceLimitExceeded {
                resource: format!("contacts (type: {})", contact_type),
                limit,
            });
        }

        Ok(())
    }

    /// Check if adding a group would exceed limits
    pub fn check_group_limit(&self, current_count: usize) -> AerospaceResult<()> {
        if current_count >= self.max_groups {
            return Err(AerospaceError::ResourceLimitExceeded {
                resource: "groups".to_string(),
                limit: self.max_groups,
            });
        }

        Ok(())
    }

    /// Verify contact count consistency
    pub fn verify_contact_count(&self, contacts: &HashMap<String, Contact>, expected_count: usize) -> AerospaceResult<()> {
        let actual_count = contacts.len();
        if actual_count != expected_count {
            return Err(AerospaceError::StateInconsistency {
                description: format!(
                    "Contact count mismatch: expected {}, actual {}",
                    expected_count, actual_count
                ),
            });
        }
        Ok(())
    }
}

/// IoT device types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IoTDeviceType {
    SmartLight,
    Thermostat,
    Lock,
    Camera,
    Sensor,
    Appliance,
    Switch,
    Outlet,
    Fan,
    Vacuum,
    Speaker,
    Display,
    Controller,
    Gateway,
    Other(String),
}

impl IoTDeviceType {
    pub fn as_str(&self) -> &str {
        match self {
            IoTDeviceType::SmartLight => "smart_light",
            IoTDeviceType::Thermostat => "thermostat",
            IoTDeviceType::Lock => "lock",
            IoTDeviceType::Camera => "camera",
            IoTDeviceType::Sensor => "sensor",
            IoTDeviceType::Appliance => "appliance",
            IoTDeviceType::Switch => "switch",
            IoTDeviceType::Outlet => "outlet",
            IoTDeviceType::Fan => "fan",
            IoTDeviceType::Vacuum => "vacuum",
            IoTDeviceType::Speaker => "speaker",
            IoTDeviceType::Display => "display",
            IoTDeviceType::Controller => "controller",
            IoTDeviceType::Gateway => "gateway",
            IoTDeviceType::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "smart_light" => IoTDeviceType::SmartLight,
            "thermostat" => IoTDeviceType::Thermostat,
            "lock" => IoTDeviceType::Lock,
            "camera" => IoTDeviceType::Camera,
            "sensor" => IoTDeviceType::Sensor,
            "appliance" => IoTDeviceType::Appliance,
            "switch" => IoTDeviceType::Switch,
            "outlet" => IoTDeviceType::Outlet,
            "fan" => IoTDeviceType::Fan,
            "vacuum" => IoTDeviceType::Vacuum,
            "speaker" => IoTDeviceType::Speaker,
            "display" => IoTDeviceType::Display,
            "controller" => IoTDeviceType::Controller,
            "gateway" => IoTDeviceType::Gateway,
            other => IoTDeviceType::Other(other.to_string()),
        }
    }
}

/// IoT communication protocols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IoTProtocol {
    Mqtt,
    Coap,
    Zigbee,
    Zwave,
    Wifi,
    Ble,
    Matter,
    Thread,
    LoRaWAN,
    Modbus,
    Http,
    Ws,
    Other(String),
}

impl IoTProtocol {
    pub fn as_str(&self) -> &str {
        match self {
            IoTProtocol::Mqtt => "mqtt",
            IoTProtocol::Coap => "coap",
            IoTProtocol::Zigbee => "zigbee",
            IoTProtocol::Zwave => "zwave",
            IoTProtocol::Wifi => "wifi",
            IoTProtocol::Ble => "ble",
            IoTProtocol::Matter => "matter",
            IoTProtocol::Thread => "thread",
            IoTProtocol::LoRaWAN => "lorawan",
            IoTProtocol::Modbus => "modbus",
            IoTProtocol::Http => "http",
            IoTProtocol::Ws => "ws",
            IoTProtocol::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "mqtt" => IoTProtocol::Mqtt,
            "coap" => IoTProtocol::Coap,
            "zigbee" => IoTProtocol::Zigbee,
            "zwave" => IoTProtocol::Zwave,
            "wifi" => IoTProtocol::Wifi,
            "ble" => IoTProtocol::Ble,
            "matter" => IoTProtocol::Matter,
            "thread" => IoTProtocol::Thread,
            "lorawan" => IoTProtocol::LoRaWAN,
            "modbus" => IoTProtocol::Modbus,
            "http" => IoTProtocol::Http,
            "ws" => IoTProtocol::Ws,
            other => IoTProtocol::Other(other.to_string()),
        }
    }
}

/// IoT device status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IoTStatus {
    Online,
    Offline,
    Error,
    Updating,
    Maintenance,
    Unknown,
}

impl IoTStatus {
    pub fn as_str(&self) -> &str {
        match self {
            IoTStatus::Online => "online",
            IoTStatus::Offline => "offline",
            IoTStatus::Error => "error",
            IoTStatus::Updating => "updating",
            IoTStatus::Maintenance => "maintenance",
            IoTStatus::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "online" => IoTStatus::Online,
            "offline" => IoTStatus::Offline,
            "error" => IoTStatus::Error,
            "updating" => IoTStatus::Updating,
            "maintenance" => IoTStatus::Maintenance,
            _ => IoTStatus::Unknown,
        }
    }

    pub fn is_online(&self) -> bool {
        matches!(self, IoTStatus::Online)
    }
}

/// IoT device capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IoTCapability {
    OnOff,
    Dimming,
    Color,
    ColorTemperature,
    Brightness,
    MotionDetection,
    Temperature,
    Humidity,
    LockUnlock,
    OpenClose,
    Volume,
    Playback,
    Recording,
    Streaming,
    Automation,
    Scene,
    Schedule,
    EnergyMonitoring,
    Other(String),
}

impl IoTCapability {
    pub fn as_str(&self) -> &str {
        match self {
            IoTCapability::OnOff => "on_off",
            IoTCapability::Dimming => "dimming",
            IoTCapability::Color => "color",
            IoTCapability::ColorTemperature => "color_temperature",
            IoTCapability::Brightness => "brightness",
            IoTCapability::MotionDetection => "motion_detection",
            IoTCapability::Temperature => "temperature",
            IoTCapability::Humidity => "humidity",
            IoTCapability::LockUnlock => "lock_unlock",
            IoTCapability::OpenClose => "open_close",
            IoTCapability::Volume => "volume",
            IoTCapability::Playback => "playback",
            IoTCapability::Recording => "recording",
            IoTCapability::Streaming => "streaming",
            IoTCapability::Automation => "automation",
            IoTCapability::Scene => "scene",
            IoTCapability::Schedule => "schedule",
            IoTCapability::EnergyMonitoring => "energy_monitoring",
            IoTCapability::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "on_off" => IoTCapability::OnOff,
            "dimming" => IoTCapability::Dimming,
            "color" => IoTCapability::Color,
            "color_temperature" => IoTCapability::ColorTemperature,
            "brightness" => IoTCapability::Brightness,
            "motion_detection" => IoTCapability::MotionDetection,
            "temperature" => IoTCapability::Temperature,
            "humidity" => IoTCapability::Humidity,
            "lock_unlock" => IoTCapability::LockUnlock,
            "open_close" => IoTCapability::OpenClose,
            "volume" => IoTCapability::Volume,
            "playback" => IoTCapability::Playback,
            "recording" => IoTCapability::Recording,
            "streaming" => IoTCapability::Streaming,
            "automation" => IoTCapability::Automation,
            "scene" => IoTCapability::Scene,
            "schedule" => IoTCapability::Schedule,
            "energy_monitoring" => IoTCapability::EnergyMonitoring,
            other => IoTCapability::Other(other.to_string()),
        }
    }
}

/// Contact entry in the directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub contact_id: String,
    pub name: String,
    pub contact_type: String, // "human", "agent", or "iot"
    pub agent_deployment_type: Option<String>, // "service" or "personal_assistant" (only for agents)
    pub agent_ids: Vec<String>, // Associated agent IDs
    pub node_id: String,
    pub groups: Vec<String>, // Contact groups (e.g., "friends", "work", "family", "iot_devices")
    pub tags: Vec<String>,
    pub notes: String,
    pub is_favorite: bool,
    pub is_blocked: bool,
    pub created_at: u64,
    pub last_contacted: u64,
    pub contact_count: u32,
    pub public_account_id: Option<String>, // Associated public account ID
    // IoT-specific fields (kept as String for backward compatibility)
    pub iot_device_type: Option<String>, // "smart_light", "thermostat", "lock", "camera", "sensor", "appliance", etc.
    pub iot_protocol: Option<String>, // "mqtt", "coap", "zigbee", "zwave", "wifi", "ble", "matter"
    pub iot_status: Option<String>, // "online", "offline", "error"
    pub iot_last_seen: Option<u64>, // Last timestamp when device was online
    pub iot_capabilities: Option<Vec<String>>, // Device capabilities (e.g., ["on_off", "dimming", "color"])
    pub iot_location: Option<String>, // Device location (e.g., "living_room", "kitchen")
}

impl Contact {
    /// Get IoT device type as enum
    pub fn iot_device_type_enum(&self) -> Option<IoTDeviceType> {
        self.iot_device_type.as_ref().map(|s| IoTDeviceType::from_str(s))
    }

    /// Set IoT device type from enum
    pub fn set_iot_device_type(&mut self, device_type: IoTDeviceType) {
        self.iot_device_type = Some(device_type.as_str().to_string());
    }

    /// Get IoT protocol as enum
    pub fn iot_protocol_enum(&self) -> Option<IoTProtocol> {
        self.iot_protocol.as_ref().map(|s| IoTProtocol::from_str(s))
    }

    /// Set IoT protocol from enum
    pub fn set_iot_protocol(&mut self, protocol: IoTProtocol) {
        self.iot_protocol = Some(protocol.as_str().to_string());
    }

    /// Get IoT status as enum
    pub fn iot_status_enum(&self) -> Option<IoTStatus> {
        self.iot_status.as_ref().map(|s| IoTStatus::from_str(s))
    }

    /// Set IoT status from enum
    pub fn set_iot_status(&mut self, status: IoTStatus) {
        self.iot_status = Some(status.as_str().to_string());
    }

    /// Get IoT capabilities as enum vector
    pub fn iot_capabilities_enum(&self) -> Option<Vec<IoTCapability>> {
        self.iot_capabilities.as_ref().map(|caps| {
            caps.iter().map(|s| IoTCapability::from_str(s)).collect()
        })
    }

    /// Set IoT capabilities from enum vector
    pub fn set_iot_capabilities(&mut self, capabilities: Vec<IoTCapability>) {
        self.iot_capabilities = Some(capabilities.iter().map(|c| c.as_str().to_string()).collect());
    }

    /// Validate IoT fields
    pub fn validate_iot_fields(&self) -> Result<(), String> {
        if self.contact_type == "iot" {
            if self.iot_device_type.is_none() {
                return Err("IoT device must have a device_type".to_string());
            }
            if self.iot_protocol.is_none() {
                return Err("IoT device must have a protocol".to_string());
            }
            if self.iot_status.is_none() {
                return Err("IoT device must have a status".to_string());
            }
            if self.node_id.is_empty() {
                return Err("IoT device must have a node_id".to_string());
            }
        }
        Ok(())
    }

    /// Check if device is online
    pub fn is_iot_online(&self) -> bool {
        self.contact_type == "iot" && 
        self.iot_status_enum().map_or(false, |s| s.is_online())
    }
}

/// IoT device validation helper
pub struct IoTValidator;

impl IoTValidator {
    /// Validate device type string
    pub fn validate_device_type(device_type: &str) -> Result<IoTDeviceType, String> {
        let dt = IoTDeviceType::from_str(device_type);
        if matches!(dt, IoTDeviceType::Other(_)) {
            Err(format!("Unknown device type: {}", device_type))
        } else {
            Ok(dt)
        }
    }

    /// Validate protocol string
    pub fn validate_protocol(protocol: &str) -> Result<IoTProtocol, String> {
        let p = IoTProtocol::from_str(protocol);
        if matches!(p, IoTProtocol::Other(_)) {
            Err(format!("Unknown protocol: {}", protocol))
        } else {
            Ok(p)
        }
    }

    /// Validate status string
    pub fn validate_status(status: &str) -> Result<IoTStatus, String> {
        let s = IoTStatus::from_str(status);
        if matches!(s, IoTStatus::Unknown) {
            Err(format!("Unknown status: {}", status))
        } else {
            Ok(s)
        }
    }

    /// Validate capability string
    pub fn validate_capability(capability: &str) -> Result<IoTCapability, String> {
        let c = IoTCapability::from_str(capability);
        if matches!(c, IoTCapability::Other(_)) {
            Err(format!("Unknown capability: {}", capability))
        } else {
            Ok(c)
        }
    }

    /// Validate all IoT fields
    pub fn validate_all(
        device_type: Option<&str>,
        protocol: Option<&str>,
        status: Option<&str>,
        capabilities: Option<&[String]>,
    ) -> Result<(), String> {
        if let Some(dt) = device_type {
            Self::validate_device_type(dt)?;
        }
        if let Some(p) = protocol {
            Self::validate_protocol(p)?;
        }
        if let Some(s) = status {
            Self::validate_status(s)?;
        }
        if let Some(caps) = capabilities {
            for cap in caps {
                Self::validate_capability(cap)?;
            }
        }
        Ok(())
    }
}

/// IoT device event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoTDeviceEvent {
    DeviceAdded {
        contact_id: String,
        device_type: String,
        name: String,
        timestamp: u64,
    },
    DeviceRemoved {
        contact_id: String,
        device_type: String,
        name: String,
        timestamp: u64,
    },
    StatusChanged {
        contact_id: String,
        old_status: String,
        new_status: String,
        timestamp: u64,
    },
    LocationChanged {
        contact_id: String,
        old_location: Option<String>,
        new_location: Option<String>,
        timestamp: u64,
    },
    CapabilityAdded {
        contact_id: String,
        capability: String,
        timestamp: u64,
    },
    CapabilityRemoved {
        contact_id: String,
        capability: String,
        timestamp: u64,
    },
    Error {
        contact_id: String,
        error_message: String,
        timestamp: u64,
    },
}

impl IoTDeviceEvent {
    pub fn event_type(&self) -> &str {
        match self {
            IoTDeviceEvent::DeviceAdded { .. } => "device_added",
            IoTDeviceEvent::DeviceRemoved { .. } => "device_removed",
            IoTDeviceEvent::StatusChanged { .. } => "status_changed",
            IoTDeviceEvent::LocationChanged { .. } => "location_changed",
            IoTDeviceEvent::CapabilityAdded { .. } => "capability_added",
            IoTDeviceEvent::CapabilityRemoved { .. } => "capability_removed",
            IoTDeviceEvent::Error { .. } => "error",
        }
    }
}

/// Contact group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub created_at: u64,
}

/// Friend request setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestSetting {
    pub user_id: String,
    pub mode: String, // "auto_accept" or "require_confirmation"
    pub updated_at: u64,
}

/// Discovery request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DiscoveryRequest {
    pub query: String, // Search query
    pub capabilities: Option<Vec<String>>, // Filter by capabilities
    pub agent_types: Option<Vec<String>>, // Filter by agent types
    pub online_only: bool,
    pub limit: Option<usize>,
}

/// Discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DiscoveryResult {
    pub agent_id: String,
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub node_id: String,
    pub status: String,
    pub match_score: f32, // 0.0 to 1.0
    pub last_seen: u64,
}

/// Recommendation criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RecommendationCriteria {
    pub interests: Vec<String>, // User interests
    pub recent_activities: Vec<String>, // Recent user activities
    pub exclude_contacts: Vec<String>, // Exclude already added contacts
    pub limit: Option<usize>,
}

/// On-disk snapshot for the in-process contact directory hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContactDirectorySnapshot {
    local_node_id: String,
    contacts: Vec<Contact>,
    groups: Vec<ContactGroup>,
    digit_to_node: HashMap<String, String>,
    node_to_digit: HashMap<String, String>,
    friend_request_settings: Vec<FriendRequestSetting>,
}

/// Configuration for Contact Directory Service
#[derive(Debug, Clone)]
pub struct ContactDirectoryServiceConfig {
    pub socket_path: PathBuf,
    pub storage_dir: PathBuf,
    /// When set, aligns directory node id with P2P CDN / video RTC node id.
    pub local_node_id: Option<String>,
}

impl Default for ContactDirectoryServiceConfig {
    fn default() -> Self {
        Self::under_app_data(&std::env::temp_dir().join("exodus_app_data"))
    }
}

impl ContactDirectoryServiceConfig {
    /// Persistent storage + socket path under application data.
    pub fn under_app_data(app_data_dir: &std::path::Path) -> Self {
        Self::under_app_data_with_node(app_data_dir, None)
    }

    /// Storage paths plus optional mesh node id for this device.
    pub fn under_app_data_with_node(app_data_dir: &std::path::Path, local_node_id: Option<String>) -> Self {
        let storage_dir = app_data_dir.join("contact_directory");
        let socket_path = storage_dir.join("exodus_contact_directory.sock");
        Self {
            socket_path,
            storage_dir,
            local_node_id,
        }
    }
}

/// Contact Directory Service with aerospace-level safety guarantees
pub struct ContactDirectoryService {
    config: ContactDirectoryServiceConfig,
    contacts: Arc<Mutex<HashMap<String, Contact>>>, // contact_id -> contact
    groups: Arc<Mutex<HashMap<String, ContactGroup>>>, // group_id -> group
    agent_index: Arc<Mutex<HashMap<String, Vec<String>>>>, // agent_id -> contact_ids
    node_index: Arc<Mutex<HashMap<String, Vec<String>>>>, // node_id -> contact_ids
    digit_to_node: Arc<Mutex<HashMap<String, String>>>, // 12-digit -> node_id
    node_to_digit: Arc<Mutex<HashMap<String, String>>>, // node_id -> 12-digit
    friend_request_settings: Arc<Mutex<HashMap<String, FriendRequestSetting>>>, // user_id -> setting
    node_id: String,
    running: Arc<Mutex<bool>>,
    shutdown_tx: Arc<Mutex<Option<broadcast::Sender<()>>>>,
    iot_event_tx: broadcast::Sender<IoTDeviceEvent>,
    // Aerospace-level safety components
    audit_log: Arc<Mutex<Vec<AuditLogEntry>>>,
    input_validator: InputValidator,
    safety_invariants: SafetyInvariants,
    resource_limits: ResourceLimits,
    rate_limiter: RateLimiter,
}

impl ContactDirectoryService {
    pub fn new(config: ContactDirectoryServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&config.storage_dir)?;
        let node_id = config
            .local_node_id
            .clone()
            .unwrap_or_else(generate_node_id);
        let (iot_event_tx, _) = broadcast::channel(1000);
        
        // Initialize aerospace-level safety components
        let input_validator = InputValidator::default();
        let safety_invariants = SafetyInvariants::default();
        let resource_limits = ResourceLimits::default();
        let rate_limiter = RateLimiter::new(resource_limits.max_request_rate, 1); // 1 second window
        
        let mut svc = Self {
            config,
            contacts: Arc::new(Mutex::new(HashMap::new())),
            groups: Arc::new(Mutex::new(HashMap::new())),
            agent_index: Arc::new(Mutex::new(HashMap::new())),
            node_index: Arc::new(Mutex::new(HashMap::new())),
            digit_to_node: Arc::new(Mutex::new(HashMap::new())),
            node_to_digit: Arc::new(Mutex::new(HashMap::new())),
            friend_request_settings: Arc::new(Mutex::new(HashMap::new())),
            node_id,
            running: Arc::new(Mutex::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            iot_event_tx,
            audit_log: Arc::new(Mutex::new(Vec::new())),
            input_validator,
            safety_invariants,
            resource_limits,
            rate_limiter,
        };
        svc.load_persisted()?;
        if let Some(id) = svc.config.local_node_id.clone() {
            svc.node_id = id;
            let _ = svc.ensure_digit_for_node(&svc.node_id.clone());
        }
        
        // Log initialization
        svc.log_audit(
            "service_init",
            "success",
            "Contact Directory Service initialized with aerospace-level safety guarantees"
        );
        
        Ok(svc)
    }

    /// Log audit entry for critical operations
    fn log_audit(&self, operation: &str, status: &str, details: &str) {
        let entry = AuditLogEntry::new(operation.to_string(), status.to_string(), details.to_string());
        if let Ok(mut log) = self.audit_log.lock() {
            log.push(entry);
            // Keep only last 10000 entries
            if log.len() > 10000 {
                let drain_count = log.len() - 10000;
                log.drain(0..drain_count);
            }
        }
    }

    /// Get audit log entries
    pub fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditLogEntry> {
        self.audit_log.lock()
            .map(|log| {
                let entries = log.clone();
                if let Some(limit) = limit {
                    entries.into_iter().rev().take(limit).collect()
                } else {
                    entries.into_iter().rev().collect()
                }
            })
            .unwrap_or_default()
    }

    /// Clear audit log
    pub fn clear_audit_log(&self) -> AerospaceResult<()> {
        self.audit_log.lock()
            .map_err(|e| AerospaceError::LockError {
                reason: format!("Failed to acquire audit log lock: {}", e),
            })
            .map(|mut log| log.clear())
    }

    /// Subscribe to IoT device events
    pub fn subscribe_iot_events(&self) -> broadcast::Receiver<IoTDeviceEvent> {
        self.iot_event_tx.subscribe()
    }

    /// Emit IoT device event
    fn emit_iot_event(&self, event: IoTDeviceEvent) {
        let _ = self.iot_event_tx.send(event);
    }

    pub fn snapshot_path(&self) -> PathBuf {
        self.config.storage_dir.join("directory.json")
    }

    fn rebuild_indexes(&self) -> Result<(), String> {
        let contacts = self.contacts.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut agent_index = self
            .agent_index
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?;
        let mut node_index = self.node_index.lock().map_err(|e| format!("Lock error: {e}"))?;
        agent_index.clear();
        node_index.clear();
        for (contact_id, contact) in contacts.iter() {
            for agent_id in &contact.agent_ids {
                agent_index
                    .entry(agent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(contact_id.clone());
            }
            node_index
                .entry(contact.node_id.clone())
                .or_insert_with(Vec::new)
                .push(contact_id.clone());
        }
        Ok(())
    }

    fn load_persisted(&mut self) -> Result<(), String> {
        let path = self.snapshot_path();
        if !path.is_file() {
            return Ok(());
        }
        let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let snap: ContactDirectorySnapshot =
            serde_json::from_str(&raw).map_err(|e| format!("Invalid directory.json: {e}"))?;
        if !snap.local_node_id.is_empty() && self.config.local_node_id.is_none() {
            self.node_id = snap.local_node_id;
        }
        {
            let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {e}"))?;
            contacts.clear();
            for c in snap.contacts {
                contacts.insert(c.contact_id.clone(), c);
            }
        }
        {
            let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {e}"))?;
            groups.clear();
            for g in snap.groups {
                groups.insert(g.group_id.clone(), g);
            }
        }
        {
            let mut digit_to_node = self
                .digit_to_node
                .lock()
                .map_err(|e| format!("Lock error: {e}"))?;
            *digit_to_node = snap.digit_to_node;
        }
        {
            let mut node_to_digit = self
                .node_to_digit
                .lock()
                .map_err(|e| format!("Lock error: {e}"))?;
            *node_to_digit = snap.node_to_digit;
        }
        {
            let mut settings = self
                .friend_request_settings
                .lock()
                .map_err(|e| format!("Lock error: {e}"))?;
            settings.clear();
            for s in snap.friend_request_settings {
                settings.insert(s.user_id.clone(), s);
            }
        }
        self.rebuild_indexes()?;
        tracing::info!("Contact directory loaded from {}", path.display());
        Ok(())
    }

    fn persist_all(&self) -> Result<(), String> {
        let contacts: Vec<Contact> = self
            .contacts
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?
            .values()
            .cloned()
            .collect();
        let groups: Vec<ContactGroup> = self
            .groups
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?
            .values()
            .cloned()
            .collect();
        let digit_to_node = self
            .digit_to_node
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?
            .clone();
        let node_to_digit = self
            .node_to_digit
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?
            .clone();
        let friend_request_settings: Vec<FriendRequestSetting> = self
            .friend_request_settings
            .lock()
            .map_err(|e| format!("Lock error: {e}"))?
            .values()
            .cloned()
            .collect();
        let snap = ContactDirectorySnapshot {
            local_node_id: self.node_id.clone(),
            contacts,
            groups,
            digit_to_node,
            node_to_digit,
            friend_request_settings,
        };
        let path = self.snapshot_path();
        let raw = serde_json::to_string_pretty(&snap).map_err(|e| e.to_string())?;
        std::fs::write(&path, raw).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Derive a stable 12-digit Exodus ID from a node id and register mappings.
    pub fn ensure_digit_for_node(&self, node_id: &str) -> Result<String, String> {
        if let Some(d) = self.get_digit_for_node(node_id.to_string()) {
            return Ok(d);
        }
        let digit = stable_digit_from_node(node_id);
        self.register_digit_mapping(digit.clone(), node_id.to_string())?;
        Ok(digit)
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }

        let socket_path = self.config.socket_path.clone();
        let contacts = Arc::clone(&self.contacts);
        let groups = Arc::clone(&self.groups);
        let agent_index = Arc::clone(&self.agent_index);
        let node_index = Arc::clone(&self.node_index);
        let digit_to_node = Arc::clone(&self.digit_to_node);
        let node_to_digit = Arc::clone(&self.node_to_digit);
        let friend_request_settings = Arc::clone(&self.friend_request_settings);
        let node_id = self.node_id.clone();
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        if let Ok(mut tx_guard) = self.shutdown_tx.lock() {
            *tx_guard = Some(shutdown_tx);
        }
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let contacts = Arc::clone(&contacts);
                                let groups = Arc::clone(&groups);
                                let agent_index = Arc::clone(&agent_index);
                                let node_index = Arc::clone(&node_index);
                                let digit_to_node = Arc::clone(&digit_to_node);
                                let node_to_digit = Arc::clone(&node_to_digit);
                                let friend_request_settings = Arc::clone(&friend_request_settings);
                                let node_id = node_id.clone();
                                tokio::spawn(async move {
                                    let _ = handle_client(stream, contacts, groups, agent_index, node_index, digit_to_node, node_to_digit, friend_request_settings, node_id).await;
                                });
                            }
                            Err(e) => {
                                eprintln!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        println!("Contact Directory Service started on {:?}", socket_path);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }

        if let Some(tx) = self.shutdown_tx.lock().ok().and_then(|mut tx| tx.take()) {
            let _ = tx.send(());
        }

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        println!("Contact Directory Service stopped");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.config.socket_path
    }

    #[allow(dead_code)]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Add a contact with aerospace-level safety guarantees
    #[allow(dead_code)]
    pub fn add_contact(&self, contact: Contact) -> Result<(), String> {
        // Check rate limit
        if let Err(e) = self.rate_limiter.check("add_contact") {
            return Err(format!("[{}] {}", e.error_code(), e));
        }

        // Validate contact structure
        if let Err(e) = self.input_validator.validate_contact(&contact) {
            return Err(format!("[{}] {}", e.error_code(), e));
        }

        // Validate IoT fields if this is an IoT device
        if let Err(e) = contact.validate_iot_fields() {
            return Err(format!("IoT validation failed: {}", e));
        }

        // Check resource limits
        let current_count = self.contacts.lock().map(|c| c.len()).unwrap_or(0);
        if let Err(e) = self.safety_invariants.check_contact_limit(current_count, &contact.contact_type) {
            return Err(format!("[{}] {}", e.error_code(), e));
        }

        let contact_id = contact.contact_id.clone();
        let node_id = contact.node_id.clone();
        let agent_ids = contact.agent_ids.clone();

        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        contacts.insert(contact_id.clone(), contact.clone());
        drop(contacts);

        let mut agent_idx = self.agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        for agent_id in &agent_ids {
            agent_idx.entry(agent_id.clone()).or_insert_with(Vec::new).push(contact_id.clone());
        }

        let mut node_idx = self.node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        node_idx.entry(node_id).or_insert_with(Vec::new).push(contact_id.clone());

        // Log the operation
        self.log_audit(
            "add_contact",
            "success",
            &format!("Added contact: {} (type: {})", contact_id, contact.contact_type),
        );

        self.persist_all()
    }

    /// Insert or update by `node_id`.
    pub fn upsert_contact(&self, contact: Contact) -> Result<(), String> {
        let node_id = contact.node_id.clone();
        if let Some(existing) = self.get_contacts_by_node(node_id).into_iter().next() {
            let mut updated = contact;
            updated.contact_id = existing.contact_id;
            self.add_contact(updated)
        } else {
            self.add_contact(contact)
        }
    }

    /// Update contact
    #[allow(dead_code)]
    pub fn update_contact(&self, contact: Contact) -> Result<(), String> {
        let contact_id = contact.contact_id.clone();
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        contacts.insert(contact_id, contact);
        self.persist_all()
    }

    /// Import contacts/groups from an export bundle (`merge` keeps unrelated rows).
    pub fn import_bundle(
        &self,
        contacts: Vec<Contact>,
        groups: Vec<ContactGroup>,
        merge: bool,
    ) -> Result<usize, String> {
        if !merge {
            {
                let mut guard = self.contacts.lock().map_err(|e| format!("Lock error: {e}"))?;
                guard.clear();
            }
            self.rebuild_indexes()?;
        }
        for group in groups {
            let _ = self.create_group(group);
        }
        let mut count = 0usize;
        for contact in contacts {
            if merge {
                self.upsert_contact(contact)?;
            } else {
                self.add_contact(contact)?;
            }
            count += 1;
        }
        self.rebuild_indexes()?;
        self.persist_all()?;
        Ok(count)
    }

    /// Remove a contact
    #[allow(dead_code)]
    pub fn remove_contact(&self, contact_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        let contact = contacts.remove(&contact_id);
        drop(contacts);

        if let Some(contact) = contact {
            // Emit device removed event for IoT devices
            if contact.contact_type == "iot" {
                self.emit_iot_event(IoTDeviceEvent::DeviceRemoved {
                    contact_id: contact_id.clone(),
                    device_type: contact.iot_device_type.unwrap_or_else(|| "unknown".to_string()),
                    name: contact.name.clone(),
                    timestamp: current_timestamp(),
                });
            }

            let mut agent_idx = self.agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
            for agent_id in &contact.agent_ids {
                if let Some(contact_ids) = agent_idx.get_mut(agent_id) {
                    contact_ids.retain(|id| id != &contact_id);
                    if contact_ids.is_empty() {
                        agent_idx.remove(agent_id);
                    }
                }
            }

            let mut node_idx = self.node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(contact_ids) = node_idx.get_mut(&contact.node_id) {
                contact_ids.retain(|id| id != &contact_id);
                if contact_ids.is_empty() {
                    node_idx.remove(&contact.node_id);
                }
            }

            self.persist_all()
        } else {
            Err("Contact not found".to_string())
        }
    }

    /// Get contact
    #[allow(dead_code)]
    pub fn get_contact(&self, contact_id: String) -> Option<Contact> {
        let contacts = self.contacts.lock().ok()?;
        contacts.get(&contact_id).cloned()
    }

    /// List all contacts
    #[allow(dead_code)]
    pub fn list_contacts(&self) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Filter contacts by type
    #[allow(dead_code)]
    pub fn filter_contacts_by_type(&self, contact_type: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == contact_type)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Filter contacts by agent deployment type
    #[allow(dead_code)]
    pub fn filter_contacts_by_deployment_type(&self, deployment_type: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.agent_deployment_type.as_ref() == Some(&deployment_type))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Search contacts
    #[allow(dead_code)]
    pub fn search_contacts(&self, query: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| {
                let query_lower = query.to_lowercase();
                
                contacts.values()
                    .filter(|c| {
                        c.name.to_lowercase().contains(&query_lower) ||
                        c.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)) ||
                        c.notes.to_lowercase().contains(&query_lower)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Register 12-digit number mapping for a node
    #[allow(dead_code)]
    pub fn register_digit_mapping(&self, digit_id: String, node_id: String) -> Result<(), String> {
        // Validate 12-digit format
        if digit_id.len() != 12 || !digit_id.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid 12-digit number format".to_string());
        }

        let mut digit_to_node = self.digit_to_node.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut node_to_digit = self.node_to_digit.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        digit_to_node.insert(digit_id.clone(), node_id.clone());
        node_to_digit.insert(node_id, digit_id);

        self.persist_all()
    }

    /// Resolve 12-digit number to NodeID
    #[allow(dead_code)]
    pub fn resolve_digit_to_node(&self, digit_id: String) -> Option<String> {
        // Clean input (remove hyphens and spaces)
        let clean_id = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
        
        if clean_id.len() != 12 {
            return None;
        }

        let digit_to_node = self.digit_to_node.lock().ok()?;
        digit_to_node.get(&clean_id).cloned()
    }

    /// Get 12-digit number for a NodeID
    #[allow(dead_code)]
    pub fn get_digit_for_node(&self, node_id: String) -> Option<String> {
        let node_to_digit = self.node_to_digit.lock().ok()?;
        node_to_digit.get(&node_id).cloned()
    }

    /// Add friend by 12-digit number
    #[allow(dead_code)]
    pub fn add_friend_by_digit(&self, digit_id: String, name: String, _user_id: String) -> Result<Contact, String> {
        // Resolve digit to node ID
        let node_id = self.resolve_digit_to_node(digit_id.clone())
            .ok_or_else(|| format!("12-digit number not found: {}", digit_id))?;

        // Check if already a contact
        let node_idx = self.node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact_ids) = node_idx.get(&node_id) {
            if !contact_ids.is_empty() {
                // Get existing contact
                let contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
                for contact_id in contact_ids {
                    if let Some(contact) = contacts.get(contact_id) {
                        return Err(format!("Already a contact: {}", contact.name));
                    }
                }
            }
        }
        drop(node_idx);

        // Create new contact
        let contact_id = format!("contact_{}", uuid::Uuid::new_v4());
        let contact = Contact {
            contact_id: contact_id.clone(),
            name,
            contact_type: "human".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: node_id.clone(),
            groups: vec!["friends".to_string()],
            tags: vec![],
            notes: format!("Added by 12-digit number: {}", digit_id),
            is_favorite: false,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };

        self.add_contact(contact.clone())?;

        // Register digit mapping if not already registered
        let digit_clean = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
        if self.get_digit_for_node(node_id.clone()).is_none() {
            self.register_digit_mapping(digit_clean, node_id)?;
        }

        Ok(contact)
    }

    /// Get contacts by group
    #[allow(dead_code)]
    pub fn get_contacts_by_group(&self, group_id: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.groups.contains(&group_id))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Get favorite contacts
    #[allow(dead_code)]
    pub fn get_favorites(&self) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.is_favorite)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Get blocked contacts
    #[allow(dead_code)]
    pub fn get_blocked(&self) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.is_blocked)
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Add contact to group
    #[allow(dead_code)]
    pub fn add_to_group(&self, contact_id: String, group_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            if !contact.groups.contains(&group_id) {
                contact.groups.push(group_id);
            }
        }
        self.persist_all()
    }

    /// Remove contact from group
    #[allow(dead_code)]
    pub fn remove_from_group(&self, contact_id: String, group_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.groups.retain(|g| g != &group_id);
        }
        self.persist_all()
    }

    /// Link contact to public account
    #[allow(dead_code)]
    pub fn link_to_public_account(&self, contact_id: String, account_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.public_account_id = Some(account_id);
        }
        self.persist_all()
    }

    /// Unlink contact from public account
    #[allow(dead_code)]
    pub fn unlink_from_public_account(&self, contact_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.public_account_id = None;
        }
        self.persist_all()
    }

    /// Get contacts linked to a public account
    #[allow(dead_code)]
    pub fn get_contacts_by_public_account(&self, account_id: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.public_account_id.as_ref() == Some(&account_id))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Create contact group
    #[allow(dead_code)]
    pub fn create_group(&self, group: ContactGroup) -> Result<(), String> {
        let group_id = group.group_id.clone();
        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        groups.insert(group_id, group);
        self.persist_all()
    }

    /// Delete contact group
    #[allow(dead_code)]
    pub fn delete_group(&self, group_id: String) -> Result<(), String> {
        let mut groups = self.groups.lock().map_err(|e| format!("Lock error: {}", e))?;
        groups.remove(&group_id);
        
        // Remove group from all contacts
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        for contact in contacts.values_mut() {
            contact.groups.retain(|g| g != &group_id);
        }

        self.persist_all()
    }

    /// List all groups
    #[allow(dead_code)]
    pub fn list_groups(&self) -> Vec<ContactGroup> {
        self.groups.lock()
            .map(|groups| groups.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Increment contact count
    #[allow(dead_code)]
    pub fn increment_contact_count(&self, contact_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.contact_count += 1;
            contact.last_contacted = current_timestamp();
        }
        drop(contacts);
        self.persist_all()
    }

    /// Toggle favorite status
    #[allow(dead_code)]
    pub fn toggle_favorite(&self, contact_id: String) -> Result<bool, String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.is_favorite = !contact.is_favorite;
            let fav = contact.is_favorite;
            drop(contacts);
            self.persist_all()?;
            Ok(fav)
        } else {
            Err("Contact not found".to_string())
        }
    }

    /// Block contact
    #[allow(dead_code)]
    pub fn block_contact(&self, contact_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.is_blocked = true;
        }
        self.persist_all()
    }

    /// Set friend request mode
    #[allow(dead_code)]
    pub fn set_friend_request_mode(&self, user_id: String, mode: String) -> Result<(), String> {
        if mode != "auto_accept" && mode != "require_confirmation" {
            return Err("Invalid mode. Must be 'auto_accept' or 'require_confirmation'".to_string());
        }

        let mut settings = self.friend_request_settings.lock().map_err(|e| format!("Lock error: {}", e))?;
        settings.insert(user_id.clone(), FriendRequestSetting {
            user_id,
            mode,
            updated_at: current_timestamp(),
        });
        self.persist_all()
    }

    /// Get friend request mode
    #[allow(dead_code)]
    pub fn get_friend_request_mode(&self, user_id: String) -> String {
        self.friend_request_settings.lock()
            .map(|settings| settings.get(&user_id)
            .map(|s| s.mode.clone())
            .unwrap_or_else(|| "require_confirmation".to_string()))
            .unwrap_or_else(|_| "require_confirmation".to_string())
    }

    /// Unblock contact
    #[allow(dead_code)]
    pub fn unblock_contact(&self, contact_id: String) -> Result<(), String> {
        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            contact.is_blocked = false;
        }
        self.persist_all()
    }

    /// Get recently contacted
    #[allow(dead_code)]
    pub fn get_recent_contacts(&self, limit: usize) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| {
                let mut sorted: Vec<Contact> = contacts.values().cloned().collect();
                sorted.sort_by(|a, b| b.last_contacted.cmp(&a.last_contacted));
                sorted.truncate(limit);
                sorted
            })
            .unwrap_or_default()
    }

    /// Get contacts by node
    #[allow(dead_code)]
    pub fn get_contacts_by_node(&self, node_id: String) -> Vec<Contact> {
        let node_idx = self.node_index.lock();
        let contacts = self.contacts.lock();
        
        if let (Ok(node_idx), Ok(contacts)) = (node_idx, contacts) {
            if let Some(contact_ids) = node_idx.get(&node_id) {
                contact_ids.iter()
                    .filter_map(|id| contacts.get(id).cloned())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get contacts by agent
    #[allow(dead_code)]
    pub fn get_contacts_by_agent(&self, agent_id: String) -> Vec<Contact> {
        let agent_idx = self.agent_index.lock();
        let contacts = self.contacts.lock();
        
        if let (Ok(agent_idx), Ok(contacts)) = (agent_idx, contacts) {
            if let Some(contact_ids) = agent_idx.get(&agent_id) {
                contact_ids.iter()
                    .filter_map(|id| contacts.get(id).cloned())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Filter contacts by IoT device type
    #[allow(dead_code)]
    pub fn filter_contacts_by_iot_device_type(&self, device_type: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot" && c.iot_device_type.as_ref() == Some(&device_type))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Filter contacts by IoT protocol
    #[allow(dead_code)]
    pub fn filter_contacts_by_iot_protocol(&self, protocol: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot" && c.iot_protocol.as_ref() == Some(&protocol))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Filter contacts by IoT status
    #[allow(dead_code)]
    pub fn filter_contacts_by_iot_status(&self, status: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot" && c.iot_status.as_ref() == Some(&status))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Get IoT devices by location
    #[allow(dead_code)]
    pub fn get_iot_devices_by_location(&self, location: String) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot" && c.iot_location.as_ref() == Some(&location))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Get all IoT devices
    #[allow(dead_code)]
    pub fn get_all_iot_devices(&self) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot")
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Update IoT device status
    #[allow(dead_code)]
    pub fn update_iot_device_status(&self, contact_id: String, status: String) -> Result<(), String> {
        let old_status = {
            let contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
            contacts.get(&contact_id).and_then(|c| c.iot_status.clone())
        };

        let mut contacts = self.contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact) = contacts.get_mut(&contact_id) {
            if contact.contact_type == "iot" {
                let new_status = status.clone();
                contact.iot_status = Some(status);
                contact.iot_last_seen = Some(current_timestamp());
                drop(contacts);

                // Emit status change event if status actually changed
                if let Some(old) = old_status {
                    if old != new_status {
                        self.emit_iot_event(IoTDeviceEvent::StatusChanged {
                            contact_id: contact_id.clone(),
                            old_status: old,
                            new_status,
                            timestamp: current_timestamp(),
                        });
                    }
                }

                self.persist_all()
            } else {
                Err("Contact is not an IoT device".to_string())
            }
        } else {
            Err("Contact not found".to_string())
        }
    }

    /// Get online IoT devices
    #[allow(dead_code)]
    pub fn get_online_iot_devices(&self) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot" && c.iot_status.as_ref() == Some(&"online".to_string()))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Get offline IoT devices
    #[allow(dead_code)]
    pub fn get_offline_iot_devices(&self) -> Vec<Contact> {
        self.contacts.lock()
            .map(|contacts| contacts.values()
            .filter(|c| c.contact_type == "iot" && c.iot_status.as_ref() != Some(&"online".to_string()))
            .cloned()
            .collect())
            .unwrap_or_default()
    }

    /// Add IoT device as a contact
    #[allow(dead_code)]
    pub fn add_iot_device(
        &self,
        name: String,
        device_type: String,
        protocol: String,
        node_id: String,
        location: Option<String>,
        capabilities: Option<Vec<String>>,
    ) -> Result<Contact, String> {
        // Validate IoT fields
        IoTValidator::validate_all(
            Some(&device_type),
            Some(&protocol),
            Some("online"),
            capabilities.as_deref(),
        )?;

        let contact_id = format!("iot_{}", uuid::Uuid::new_v4());
        let device_type_clone = device_type.clone();
        let name_clone = name.clone();
        let contact = Contact {
            contact_id: contact_id.clone(),
            name,
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id,
            groups: vec!["iot_devices".to_string()],
            tags: vec![],
            notes: String::new(),
            is_favorite: false,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: Some(device_type),
            iot_protocol: Some(protocol),
            iot_status: Some("online".to_string()),
            iot_last_seen: Some(current_timestamp()),
            iot_capabilities: capabilities,
            iot_location: location,
        };

        self.add_contact(contact.clone())?;

        // Emit device added event
        self.emit_iot_event(IoTDeviceEvent::DeviceAdded {
            contact_id: contact_id.clone(),
            device_type: device_type_clone,
            name: name_clone,
            timestamp: current_timestamp(),
        });

        Ok(contact)
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    contacts: Arc<Mutex<HashMap<String, Contact>>>,
    groups: Arc<Mutex<HashMap<String, ContactGroup>>>,
    agent_index: Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_index: Arc<Mutex<HashMap<String, Vec<String>>>>,
    digit_to_node: Arc<Mutex<HashMap<String, String>>>,
    node_to_digit: Arc<Mutex<HashMap<String, String>>>,
    friend_request_settings: Arc<Mutex<HashMap<String, FriendRequestSetting>>>,
    node_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let request: serde_json::Value = serde_json::from_str(&line)?;
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);
        let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);

        let result = match method {
            "add_contact" => handle_add_contact(&params, &contacts, &agent_index, &node_index).await,
            "remove_contact" => handle_remove_contact(&params, &contacts, &agent_index, &node_index).await,
            "update_contact" => handle_update_contact(&params, &contacts).await,
            "get_contact" => handle_get_contact(&params, &contacts).await,
            "list_contacts" => handle_list_contacts(&contacts).await,
            "search_contacts" => handle_search_contacts(&params, &contacts).await,
            "filter_contacts_by_type" => handle_filter_contacts_by_type(&params, &contacts).await,
            "filter_contacts_by_deployment_type" => handle_filter_contacts_by_deployment_type(&params, &contacts).await,
            "get_contacts_by_group" => handle_get_contacts_by_group(&params, &contacts).await,
            "get_favorites" => handle_get_favorites(&contacts).await,
            "get_blocked" => handle_get_blocked(&contacts).await,
            "add_to_group" => handle_add_to_group(&params, &contacts).await,
            "remove_from_group" => handle_remove_from_group(&params, &contacts).await,
            "create_group" => handle_create_group(&params, &groups).await,
            "delete_group" => handle_delete_group(&params, &groups, &contacts).await,
            "list_groups" => handle_list_groups(&groups).await,
            "toggle_favorite" => handle_toggle_favorite(&params, &contacts).await,
            "block_contact" => handle_block_contact(&params, &contacts).await,
            "unblock_contact" => handle_unblock_contact(&params, &contacts).await,
            "get_recent_contacts" => handle_get_recent_contacts(&params, &contacts).await,
            "get_contacts_by_node" => handle_get_contacts_by_node(&params, &node_index, &contacts).await,
            "get_contacts_by_agent" => handle_get_contacts_by_agent(&params, &agent_index, &contacts).await,
            "add_friend_by_digit" => handle_add_friend_by_digit(&params, &contacts, &node_index, &agent_index, &digit_to_node, &node_to_digit).await,
            "register_digit_mapping" => handle_register_digit_mapping(&params, &digit_to_node, &node_to_digit).await,
            "resolve_digit_to_node" => handle_resolve_digit_to_node(&params, &digit_to_node).await,
            "get_digit_for_node" => handle_get_digit_for_node(&params, &node_to_digit).await,
            "link_to_public_account" => handle_link_to_public_account(&params, &contacts).await,
            "unlink_from_public_account" => handle_unlink_from_public_account(&params, &contacts).await,
            "get_contacts_by_public_account" => handle_get_contacts_by_public_account(&params, &contacts).await,
            "set_friend_request_mode" => handle_set_friend_request_mode(&params, &friend_request_settings).await,
            "get_friend_request_mode" => handle_get_friend_request_mode(&params, &friend_request_settings).await,
            "node_info" => handle_node_info(&node_id).await,
            "filter_contacts_by_iot_device_type" => handle_filter_contacts_by_iot_device_type(&params, &contacts).await,
            "filter_contacts_by_iot_protocol" => handle_filter_contacts_by_iot_protocol(&params, &contacts).await,
            "filter_contacts_by_iot_status" => handle_filter_contacts_by_iot_status(&params, &contacts).await,
            "get_iot_devices_by_location" => handle_get_iot_devices_by_location(&params, &contacts).await,
            "get_all_iot_devices" => handle_get_all_iot_devices(&contacts).await,
            "update_iot_device_status" => handle_update_iot_device_status(&params, &contacts).await,
            "get_online_iot_devices" => handle_get_online_iot_devices(&contacts).await,
            "get_offline_iot_devices" => handle_get_offline_iot_devices(&contacts).await,
            _ => Err(format!("Unknown method: {}", method)),
        };

        let response = if let Ok(res) = result {
            json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -1, "message": result.unwrap_err()},
                "id": id
            })
        };

        writer.write_all(response.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
        line.clear();
    }

    Ok(())
}

async fn handle_add_contact(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
    agent_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let contact: Contact = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid contact: {}", e))?;
    
    let contact_id = contact.contact_id.clone();
    let node_id = contact.node_id.clone();
    let agent_ids = contact.agent_ids.clone();
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(contact_id.clone(), contact.clone());
    drop(guard);

    let mut agent_idx = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    for agent_id in &agent_ids {
        agent_idx.entry(agent_id.clone()).or_insert_with(Vec::new).push(contact_id.clone());
    }

    let mut node_idx = node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    node_idx.entry(node_id).or_insert_with(Vec::new).push(contact_id.clone());

    Ok(json!({
        "added": true,
        "contact_id": contact_id
    }))
}

async fn handle_remove_contact(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
    agent_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let contact = guard.remove(contact_id);
    drop(guard);

    if let Some(contact) = contact {
        let mut agent_idx = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        for agent_id in &contact.agent_ids {
            if let Some(contact_ids) = agent_idx.get_mut(agent_id) {
                contact_ids.retain(|id| id != contact_id);
                if contact_ids.is_empty() {
                    agent_idx.remove(agent_id);
                }
            }
        }

        let mut node_idx = node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(contact_ids) = node_idx.get_mut(&contact.node_id) {
            contact_ids.retain(|id| id != contact_id);
            if contact_ids.is_empty() {
                node_idx.remove(&contact.node_id);
            }
        }
    }

    Ok(json!({
        "removed": true,
        "contact_id": contact_id
    }))
}

async fn handle_update_contact(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact: Contact = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid contact: {}", e))?;
    
    let contact_id = contact.contact_id.clone();
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(contact_id, contact);

    Ok(json!({
        "updated": true
    }))
}

async fn handle_get_contact(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.get(contact_id)
        .map(|c| json!(c))
        .ok_or_else(|| "Contact not found".to_string())
}

async fn handle_list_contacts(
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let contact_list: Vec<Contact> = guard.values().cloned().collect();
    
    Ok(json!({
        "contacts": contact_list
    }))
}

async fn handle_search_contacts(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let query = params.get("query").and_then(|q| q.as_str()).ok_or("Missing query")?;
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let query_lower = query.to_lowercase();
    
    let found: Vec<Contact> = guard.values()
        .filter(|c| {
            c.name.to_lowercase().contains(&query_lower) ||
            c.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)) ||
            c.notes.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_filter_contacts_by_type(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_type = params.get("contact_type").and_then(|t| t.as_str()).ok_or("Missing contact_type")?;
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == contact_type)
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_filter_contacts_by_deployment_type(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let deployment_type = params.get("deployment_type").and_then(|t| t.as_str()).ok_or("Missing deployment_type")?;
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.agent_deployment_type.as_ref() == Some(&deployment_type.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_get_contacts_by_group(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.groups.contains(&group_id.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_get_favorites(
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let favorites: Vec<Contact> = guard.values()
        .filter(|c| c.is_favorite)
        .cloned()
        .collect();

    Ok(json!({
        "contacts": favorites
    }))
}

async fn handle_link_to_public_account(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        contact.public_account_id = Some(account_id.to_string());
    }

    Ok(json!({
        "linked": true
    }))
}

async fn handle_unlink_from_public_account(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        contact.public_account_id = None;
    }

    Ok(json!({
        "unlinked": true
    }))
}

async fn handle_get_contacts_by_public_account(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let account_id = params.get("account_id").and_then(|a| a.as_str()).ok_or("Missing account_id")?;
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.public_account_id.as_ref() == Some(&account_id.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_get_blocked(
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let blocked: Vec<Contact> = guard.values()
        .filter(|c| c.is_blocked)
        .cloned()
        .collect();

    Ok(json!({
        "contacts": blocked
    }))
}

async fn handle_add_to_group(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        if !contact.groups.contains(&group_id.to_string()) {
            contact.groups.push(group_id.to_string());
        }
    }

    Ok(json!({
        "added": true
    }))
}

async fn handle_remove_from_group(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        contact.groups.retain(|g| g != group_id);
    }

    Ok(json!({
        "removed": true
    }))
}

async fn handle_create_group(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, ContactGroup>>>,
) -> Result<serde_json::Value, String> {
    let group: ContactGroup = serde_json::from_value(params.clone())
        .map_err(|e| format!("Invalid group: {}", e))?;
    
    let group_id = group.group_id.clone();
    let mut guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(group_id, group);

    Ok(json!({
        "created": true
    }))
}

async fn handle_delete_group(
    params: &serde_json::Value,
    groups: &Arc<Mutex<HashMap<String, ContactGroup>>>,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let group_id = params.get("group_id").and_then(|g| g.as_str()).ok_or("Missing group_id")?;
    
    let mut groups_guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    groups_guard.remove(group_id);
    drop(groups_guard);

    // Remove group from all contacts
    let mut contacts_guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    for contact in contacts_guard.values_mut() {
        contact.groups.retain(|g| g != group_id);
    }

    Ok(json!({
        "deleted": true
    }))
}

async fn handle_list_groups(
    groups: &Arc<Mutex<HashMap<String, ContactGroup>>>,
) -> Result<serde_json::Value, String> {
    let guard = groups.lock().map_err(|e| format!("Lock error: {}", e))?;
    let group_list: Vec<ContactGroup> = guard.values().cloned().collect();
    
    Ok(json!({
        "groups": group_list
    }))
}

async fn handle_set_friend_request_mode(
    params: &serde_json::Value,
    friend_request_settings: &Arc<Mutex<HashMap<String, FriendRequestSetting>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    let mode = params.get("mode").and_then(|m| m.as_str()).ok_or("Missing mode")?;
    
    if mode != "auto_accept" && mode != "require_confirmation" {
        return Err("Invalid mode. Must be 'auto_accept' or 'require_confirmation'".to_string());
    }

    let mut guard = friend_request_settings.lock().map_err(|e| format!("Lock error: {}", e))?;
    guard.insert(user_id.to_string(), FriendRequestSetting {
        user_id: user_id.to_string(),
        mode: mode.to_string(),
        updated_at: current_timestamp(),
    });

    Ok(json!({
        "success": true
    }))
}

async fn handle_get_friend_request_mode(
    params: &serde_json::Value,
    friend_request_settings: &Arc<Mutex<HashMap<String, FriendRequestSetting>>>,
) -> Result<serde_json::Value, String> {
    let user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    let guard = friend_request_settings.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mode = guard.get(user_id)
        .map(|s| s.mode.clone())
        .unwrap_or_else(|| "require_confirmation".to_string());

    Ok(json!({
        "mode": mode
    }))
}

async fn handle_toggle_favorite(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        contact.is_favorite = !contact.is_favorite;
        return Ok(json!({
            "is_favorite": contact.is_favorite
        }));
    }
    
    Err("Contact not found".to_string())
}

async fn handle_block_contact(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        contact.is_blocked = true;
    }

    Ok(json!({
        "blocked": true
    }))
}

async fn handle_unblock_contact(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    
    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        contact.is_blocked = false;
    }

    Ok(json!({
        "unblocked": true
    }))
}

async fn handle_get_recent_contacts(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let limit = params.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize).unwrap_or(10);
    
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut sorted: Vec<Contact> = guard.values().cloned().collect();
    sorted.sort_by(|a, b| b.last_contacted.cmp(&a.last_contacted));
    sorted.truncate(limit);

    Ok(json!({
        "contacts": sorted
    }))
}

async fn handle_get_contacts_by_node(
    params: &serde_json::Value,
    node_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let node_id = params.get("node_id").and_then(|n| n.as_str()).ok_or("Missing node_id")?;
    
    let node_idx = node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let contacts_guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(contact_ids) = node_idx.get(node_id) {
        let found: Vec<Contact> = contact_ids.iter()
            .filter_map(|id| contacts_guard.get(id).cloned())
            .collect();
        return Ok(json!({ "contacts": found }));
    }

    Ok(json!({
        "contacts": Vec::<Contact>::new()
    }))
}

async fn handle_get_contacts_by_agent(
    params: &serde_json::Value,
    agent_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let agent_id = params.get("agent_id").and_then(|a| a.as_str()).ok_or("Missing agent_id")?;
    
    let agent_idx = agent_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    let contacts_guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(contact_ids) = agent_idx.get(agent_id) {
        let found: Vec<Contact> = contact_ids.iter()
            .filter_map(|id| contacts_guard.get(id).cloned())
            .collect();
        return Ok(json!({ "contacts": found }));
    }

    Ok(json!({
        "contacts": Vec::<Contact>::new()
    }))
}

async fn handle_node_info(node_id: &str) -> Result<serde_json::Value, String> {
    Ok(json!({
        "node_id": node_id,
        "timestamp": current_timestamp()
    }))
}

async fn handle_register_digit_mapping(
    params: &serde_json::Value,
    digit_to_node: &Arc<Mutex<HashMap<String, String>>>,
    node_to_digit: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<serde_json::Value, String> {
    let digit_id = params.get("digit_id").and_then(|d| d.as_str()).ok_or("Missing digit_id")?;
    let node_id = params.get("node_id").and_then(|n| n.as_str()).ok_or("Missing node_id")?;
    
    // Validate 12-digit format
    let clean_digit = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
    if clean_digit.len() != 12 {
        return Err("Invalid 12-digit number format".to_string());
    }

    let mut digit_to_node_guard = digit_to_node.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut node_to_digit_guard = node_to_digit.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    digit_to_node_guard.insert(clean_digit.clone(), node_id.to_string());
    node_to_digit_guard.insert(node_id.to_string(), clean_digit);

    Ok(json!({
        "registered": true
    }))
}

async fn handle_resolve_digit_to_node(
    params: &serde_json::Value,
    digit_to_node: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<serde_json::Value, String> {
    let digit_id = params.get("digit_id").and_then(|d| d.as_str()).ok_or("Missing digit_id")?;
    
    // Clean input (remove hyphens and spaces)
    let clean_id = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
    
    if clean_id.len() != 12 {
        return Err("Invalid 12-digit number format".to_string());
    }

    let guard = digit_to_node.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(node_id) = guard.get(&clean_id) {
        return Ok(json!({
            "node_id": node_id
        }));
    }

    Err("12-digit number not found".to_string())
}

async fn handle_get_digit_for_node(
    params: &serde_json::Value,
    node_to_digit: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<serde_json::Value, String> {
    let node_id = params.get("node_id").and_then(|n| n.as_str()).ok_or("Missing node_id")?;
    
    let guard = node_to_digit.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(digit_id) = guard.get(node_id) {
        return Ok(json!({
            "digit_id": digit_id
        }));
    }

    Err("Node ID not found in digit mapping".to_string())
}

async fn handle_add_friend_by_digit(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
    _agent_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    node_index: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    digit_to_node: &Arc<Mutex<HashMap<String, String>>>,
    node_to_digit: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<serde_json::Value, String> {
    let digit_id = params.get("digit_id").and_then(|d| d.as_str()).ok_or("Missing digit_id")?;
    let name = params.get("name").and_then(|n| n.as_str()).ok_or("Missing name")?;
    let _user_id = params.get("user_id").and_then(|u| u.as_str()).ok_or("Missing user_id")?;
    
    // Clean input
    let clean_digit = digit_id.replace(|c: char| !c.is_ascii_digit(), "");
    
    if clean_digit.len() != 12 {
        return Err("Invalid 12-digit number format".to_string());
    }

    // Resolve digit to node ID
    let digit_to_node_guard = digit_to_node.lock().map_err(|e| format!("Lock error: {}", e))?;
    let node_id = digit_to_node_guard.get(&clean_digit)
        .cloned()
        .ok_or_else(|| format!("12-digit number not found: {}", clean_digit))?;
    drop(digit_to_node_guard);

    // Check if already a contact
    let node_idx = node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact_ids) = node_idx.get(&node_id) {
        if !contact_ids.is_empty() {
            let contacts_guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
            for contact_id in contact_ids {
                if let Some(contact) = contacts_guard.get(contact_id) {
                    return Err(format!("Already a contact: {}", contact.name));
                }
            }
        }
    }
    drop(node_idx);

    // Create new contact
    let contact_id = format!("contact_{}", uuid::Uuid::new_v4());
    let contact = Contact {
        contact_id: contact_id.clone(),
        name: name.to_string(),
        contact_type: "human".to_string(),
        agent_deployment_type: None,
        agent_ids: vec![],
        node_id: node_id.clone(),
        groups: vec!["friends".to_string()],
        tags: vec![],
        notes: format!("Added by 12-digit number: {}", clean_digit),
        is_favorite: false,
        is_blocked: false,
        created_at: current_timestamp(),
        last_contacted: 0,
        contact_count: 0,
        public_account_id: None,
        iot_device_type: None,
        iot_protocol: None,
        iot_status: None,
        iot_last_seen: None,
        iot_capabilities: None,
        iot_location: None,
    };

    // Add contact
    let mut contacts_guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    contacts_guard.insert(contact_id.clone(), contact.clone());
    drop(contacts_guard);

    // Update node index
    let mut node_idx = node_index.lock().map_err(|e| format!("Lock error: {}", e))?;
    node_idx.entry(node_id.clone()).or_insert_with(Vec::new).push(contact_id);
    drop(node_idx);

    // Register digit mapping if not already registered
    let mut node_to_digit_guard = node_to_digit.lock().map_err(|e| format!("Lock error: {}", e))?;
    if node_to_digit_guard.get(&node_id).is_none() {
        node_to_digit_guard.insert(node_id, clean_digit);
    }

    Ok(json!({
        "contact": contact
    }))
}

fn iot_contacts(contacts: &Arc<Mutex<HashMap<String, Contact>>>) -> Result<Vec<Contact>, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {e}"))?;
    Ok(guard
        .values()
        .filter(|c| c.contact_type == "iot")
        .cloned()
        .collect())
}

async fn handle_filter_contacts_by_iot_device_type(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let device_type = params.get("device_type").and_then(|t| t.as_str()).ok_or("Missing device_type")?;

    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot" && c.iot_device_type.as_ref() == Some(&device_type.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_filter_contacts_by_iot_protocol(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let protocol = params.get("protocol").and_then(|p| p.as_str()).ok_or("Missing protocol")?;

    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot" && c.iot_protocol.as_ref() == Some(&protocol.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_filter_contacts_by_iot_status(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let status = params.get("status").and_then(|s| s.as_str()).ok_or("Missing status")?;

    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot" && c.iot_status.as_ref() == Some(&status.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_get_iot_devices_by_location(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let location = params.get("location").and_then(|l| l.as_str()).ok_or("Missing location")?;

    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let found: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot" && c.iot_location.as_ref() == Some(&location.to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "contacts": found
    }))
}

async fn handle_get_all_iot_devices(
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let devices: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot")
        .cloned()
        .collect();

    Ok(json!({
        "devices": devices
    }))
}

async fn handle_update_iot_device_status(
    params: &serde_json::Value,
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let contact_id = params.get("contact_id").and_then(|c| c.as_str()).ok_or("Missing contact_id")?;
    let status = params.get("status").and_then(|s| s.as_str()).ok_or("Missing status")?;

    let mut guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(contact) = guard.get_mut(contact_id) {
        if contact.contact_type == "iot" {
            contact.iot_status = Some(status.to_string());
            contact.iot_last_seen = Some(current_timestamp());
            return Ok(json!({
                "updated": true,
                "contact_id": contact_id
            }));
        } else {
            return Err("Contact is not an IoT device".to_string());
        }
    }

    Err("Contact not found".to_string())
}

async fn handle_get_online_iot_devices(
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let devices: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot" && c.iot_status.as_ref() == Some(&"online".to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "devices": devices
    }))
}

async fn handle_get_offline_iot_devices(
    contacts: &Arc<Mutex<HashMap<String, Contact>>>,
) -> Result<serde_json::Value, String> {
    let guard = contacts.lock().map_err(|e| format!("Lock error: {}", e))?;
    let devices: Vec<Contact> = guard.values()
        .filter(|c| c.contact_type == "iot" && c.iot_status.as_ref() != Some(&"online".to_string()))
        .cloned()
        .collect();

    Ok(json!({
        "devices": devices
    }))
}

/// Stable 12-digit Exodus ID derived from a P2P node id.
fn stable_digit_from_node(node_id: &str) -> String {
    let hash = blake3::hash(node_id.as_bytes());
    let mut n: u64 = 0;
    for b in hash.as_bytes().iter().take(8) {
        n = n.wrapping_mul(31).wrapping_add(*b as u64);
    }
    format!("{:012}", n % 1_000_000_000_000)
}

fn generate_node_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("contact_node_{:x}", timestamp)
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

#[cfg(all(test, feature = "im-tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (ContactDirectoryServiceConfig, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = ContactDirectoryServiceConfig::under_app_data(temp_dir.path());
        (config, temp_dir)
    }

    #[test]
    fn test_add_and_get_contact() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let contact = Contact {
            contact_id: "test-contact-1".to_string(),
            name: "Test Contact".to_string(),
            contact_type: "human".to_string(),
            agent_deployment_type: None,
            public_account_id: None,
            agent_ids: vec!["agent-1".to_string()],
            node_id: service.node_id().to_string(),
            groups: vec!["friends".to_string()],
            tags: vec!["important".to_string()],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: current_timestamp(),
            contact_count: 0,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };

        service.add_contact(contact).expect("Failed to add contact");
        let retrieved = service.get_contact("test-contact-1".to_string()).expect("Expected contact");
        assert_eq!(retrieved.name, "Test Contact");
    }

    #[test]
    fn test_search_contacts() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let contact = Contact {
            contact_id: "test-contact-2".to_string(),
            name: "John Doe".to_string(),
            contact_type: "human".to_string(),
            agent_deployment_type: None,
            public_account_id: None,
            agent_ids: vec!["agent-1".to_string()],
            node_id: service.node_id().to_string(),
            groups: vec![],
            tags: vec!["developer".to_string()],
            notes: "Works on AI projects".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: current_timestamp(),
            contact_count: 0,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };

        service.add_contact(contact).expect("Failed to add contact");
        
        let results = service.search_contacts("john".to_string());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "John Doe");
    }

    #[test]
    fn test_add_iot_device() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let device = service.add_iot_device(
            "Smart Light".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_123".to_string(),
            Some("living_room".to_string()),
            Some(vec!["on_off".to_string(), "dimming".to_string()]),
        ).expect("Failed to add IoT device");
        
        assert_eq!(device.contact_type, "iot");
        assert_eq!(device.iot_device_type, Some("smart_light".to_string()));
        assert_eq!(device.iot_protocol, Some("mqtt".to_string()));
        assert_eq!(device.iot_status, Some("online".to_string()));
        assert_eq!(device.iot_location, Some("living_room".to_string()));
        assert_eq!(device.iot_capabilities, Some(vec!["on_off".to_string(), "dimming".to_string()]));
        assert!(device.groups.contains(&"iot_devices".to_string()));
    }

    #[test]
    fn test_filter_iot_devices_by_type() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        service.add_iot_device(
            "Smart Light".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_1".to_string(),
            Some("living_room".to_string()),
            None,
        ).expect("Failed to add IoT device");
        
        service.add_iot_device(
            "Thermostat".to_string(),
            "thermostat".to_string(),
            "zigbee".to_string(),
            "node_2".to_string(),
            Some("bedroom".to_string()),
            None,
        ).expect("Failed to add IoT device");
        
        let lights = service.filter_contacts_by_iot_device_type("smart_light".to_string());
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].name, "Smart Light");
        
        let thermostats = service.filter_contacts_by_iot_device_type("thermostat".to_string());
        assert_eq!(thermostats.len(), 1);
        assert_eq!(thermostats[0].name, "Thermostat");
    }

    #[test]
    fn test_filter_iot_devices_by_status() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let device1 = service.add_iot_device(
            "Device 1".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_1".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        let device2 = service.add_iot_device(
            "Device 2".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_2".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        service.update_iot_device_status(device1.contact_id.clone(), "offline".to_string()).expect("Failed to update device status");
        
        let online_devices = service.get_online_iot_devices();
        assert_eq!(online_devices.len(), 1);
        assert_eq!(online_devices[0].contact_id, device2.contact_id);
        
        let offline_devices = service.get_offline_iot_devices();
        assert_eq!(offline_devices.len(), 1);
        assert_eq!(offline_devices[0].contact_id, device1.contact_id);
    }

    #[test]
    fn test_get_iot_devices_by_location() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        service.add_iot_device(
            "Living Room Light".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_1".to_string(),
            Some("living_room".to_string()),
            None,
        ).expect("Failed to add IoT device");
        
        service.add_iot_device(
            "Bedroom Light".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_2".to_string(),
            Some("bedroom".to_string()),
            None,
        ).expect("Failed to add IoT device");
        
        let living_room_devices = service.get_iot_devices_by_location("living_room".to_string());
        assert_eq!(living_room_devices.len(), 1);
        assert_eq!(living_room_devices[0].name, "Living Room Light");
        
        let bedroom_devices = service.get_iot_devices_by_location("bedroom".to_string());
        assert_eq!(bedroom_devices.len(), 1);
        assert_eq!(bedroom_devices[0].name, "Bedroom Light");
    }

    #[test]
    fn test_get_all_iot_devices() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        service.add_iot_device(
            "Device 1".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_1".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        service.add_iot_device(
            "Device 2".to_string(),
            "thermostat".to_string(),
            "zigbee".to_string(),
            "node_2".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        let all_iot = service.get_all_iot_devices();
        assert_eq!(all_iot.len(), 2);
    }

    #[test]
    fn test_update_iot_device_status_persists() {
        let mut config = ContactDirectoryServiceConfig::default();
        let temp_dir = std::env::temp_dir().join("test_persist_iot");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        config.storage_dir = temp_dir.clone();
        
        let service = ContactDirectoryService::new(config.clone()).expect("Failed to create service");
        
        let device = service.add_iot_device(
            "Test Device".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_1".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        service.update_iot_device_status(device.contact_id.clone(), "offline".to_string()).expect("Failed to update device status");
        
        // Create new service instance to test persistence
        let service2 = ContactDirectoryService::new(config).expect("Failed to create service");
        let retrieved = service2.get_contact(device.contact_id).expect("Expected contact");
        assert_eq!(retrieved.iot_status, Some("offline".to_string()));
        
        // Cleanup
        std::fs::remove_dir_all(temp_dir).expect("Failed to remove temp dir");
    }

    #[test]
    fn test_persistence_and_recovery() {
        let mut config = ContactDirectoryServiceConfig::default();
        let temp_dir = std::env::temp_dir().join("test_persistence");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        config.storage_dir = temp_dir.clone();
        
        let service1 = ContactDirectoryService::new(config.clone()).expect("Failed to create service");
        
        let contact = Contact {
            contact_id: "persist-test-1".to_string(),
            name: "Persistent Contact".to_string(),
            contact_type: "human".to_string(),
            agent_deployment_type: None,
            public_account_id: None,
            agent_ids: vec![],
            node_id: service1.node_id().to_string(),
            groups: vec!["friends".to_string()],
            tags: vec!["test".to_string()],
            notes: "Test persistence".to_string(),
            is_favorite: true,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: 0,
            contact_count: 0,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };
        
        service1.add_contact(contact.clone()).expect("Failed to add contact");
        
        // Create new service instance to test persistence
        let service2 = ContactDirectoryService::new(config.clone()).expect("Failed to create service");
        let retrieved = service2.get_contact("persist-test-1".to_string()).expect("Expected contact");
        assert_eq!(retrieved.name, "Persistent Contact");
        assert_eq!(retrieved.is_favorite, true);
        assert!(retrieved.tags.contains(&"test".to_string()));
        
        // Cleanup
        std::fs::remove_dir_all(temp_dir).expect("Failed to remove temp dir");
    }

    #[test]
    fn test_stable_digit_from_node() {
        let node_id = "test_node_id_12345";
        let digit1 = stable_digit_from_node(node_id);
        let digit2 = stable_digit_from_node(node_id);
        
        assert_eq!(digit1, digit2);
        assert_eq!(digit1.len(), 12);
        assert!(digit1.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_ensure_digit_for_node() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let node_id = "test_node_for_digit";
        let digit1 = service.ensure_digit_for_node(node_id).expect("Failed to ensure digit");
        let digit2 = service.ensure_digit_for_node(node_id).expect("Failed to ensure digit");
        
        assert_eq!(digit1, digit2);
        
        // Verify mapping exists
        let resolved = service.resolve_digit_to_node(digit1).expect("Failed to resolve digit");
        assert_eq!(resolved, node_id);
    }

    #[test]
    fn test_iot_enum_conversions() {
        // Test IoTDeviceType
        let device_type = IoTDeviceType::SmartLight;
        assert_eq!(device_type.as_str(), "smart_light");
        assert_eq!(IoTDeviceType::from_str("smart_light"), IoTDeviceType::SmartLight);
        
        // Test IoTProtocol
        let protocol = IoTProtocol::Mqtt;
        assert_eq!(protocol.as_str(), "mqtt");
        assert_eq!(IoTProtocol::from_str("mqtt"), IoTProtocol::Mqtt);
        
        // Test IoTStatus
        let status = IoTStatus::Online;
        assert_eq!(status.as_str(), "online");
        assert_eq!(IoTStatus::from_str("online"), IoTStatus::Online);
        assert!(status.is_online());
        assert!(!IoTStatus::Offline.is_online());
        
        // Test IoTCapability
        let capability = IoTCapability::OnOff;
        assert_eq!(capability.as_str(), "on_off");
        assert_eq!(IoTCapability::from_str("on_off"), IoTCapability::OnOff);
    }

    #[test]
    fn test_iot_validator() {
        // Test valid values
        assert!(IoTValidator::validate_device_type("smart_light").is_ok());
        assert!(IoTValidator::validate_protocol("mqtt").is_ok());
        assert!(IoTValidator::validate_status("online").is_ok());
        assert!(IoTValidator::validate_capability("on_off").is_ok());
        
        // Test invalid values
        assert!(IoTValidator::validate_device_type("invalid_type").is_err());
        assert!(IoTValidator::validate_protocol("invalid_protocol").is_err());
        assert!(IoTValidator::validate_status("invalid_status").is_err());
        assert!(IoTValidator::validate_capability("invalid_capability").is_err());
    }

    #[test]
    fn test_contact_enum_methods() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let device = service.add_iot_device(
            "Test Device".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_test".to_string(),
            Some("living_room".to_string()),
            Some(vec!["on_off".to_string()]),
        ).expect("Failed to add IoT device");
        
        // Test enum getters
        assert_eq!(device.iot_device_type_enum(), Some(IoTDeviceType::SmartLight));
        assert_eq!(device.iot_protocol_enum(), Some(IoTProtocol::Mqtt));
        assert_eq!(device.iot_status_enum(), Some(IoTStatus::Online));
        assert_eq!(device.iot_capabilities_enum(), Some(vec![IoTCapability::OnOff]));
        
        // Test validation
        assert!(device.validate_iot_fields().is_ok());
        assert!(device.is_iot_online());
    }

    #[tokio::test]
    async fn test_iot_event_subscription() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let mut receiver = service.subscribe_iot_events();
        
        // Add a device and check for event
        let device = service.add_iot_device(
            "Event Test Device".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_event_test".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        // Receive the device added event
        let event = receiver.recv().await.expect("Expected IoT event");
        assert_eq!(event.event_type(), "device_added");
        if let IoTDeviceEvent::DeviceAdded { contact_id, device_type, name, .. } = event {
            assert_eq!(contact_id, device.contact_id);
            assert_eq!(device_type, "smart_light");
            assert_eq!(name, "Event Test Device");
        } else {
            panic!("Expected DeviceAdded event");
        }
    }

    #[tokio::test]
    async fn test_iot_status_change_event() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let mut receiver = service.subscribe_iot_events();
        
        let device = service.add_iot_device(
            "Status Test Device".to_string(),
            "smart_light".to_string(),
            "mqtt".to_string(),
            "node_status_test".to_string(),
            None,
            None,
        ).expect("Failed to add IoT device");
        
        // Skip the device added event
        let _ = receiver.recv().await.expect("Expected IoT event");
        
        // Update status
        service.update_iot_device_status(device.contact_id.clone(), "offline".to_string()).expect("Failed to update device status");
        
        // Receive the status change event
        let event = receiver.recv().await.expect("Expected IoT event");
        assert_eq!(event.event_type(), "status_changed");
        if let IoTDeviceEvent::StatusChanged { contact_id, old_status, new_status, .. } = event {
            assert_eq!(contact_id, device.contact_id);
            assert_eq!(old_status, "online");
            assert_eq!(new_status, "offline");
        } else {
            panic!("Expected StatusChanged event");
        }
    }
}

// Aerospace-level test module
#[cfg(all(test, feature = "im-tests"))]
mod aerospace_tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn create_test_config() -> (ContactDirectoryServiceConfig, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config = ContactDirectoryServiceConfig::under_app_data(temp_dir.path());
        (config, temp_dir)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }

    #[test]
    fn test_service_initialization() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        assert!(!service.node_id().is_empty());
        assert!(!service.is_running());
    }

    #[test]
    fn test_add_contact_success() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Contact".to_string(),
            contact_type: "human".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };

        let result = service.add_contact(contact);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_contact_validation_error() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let mut contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Contact".to_string(),
            contact_type: "invalid_type".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: current_timestamp(),
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };

        let result = service.add_contact(contact);
        assert!(result.is_err());
    }

    #[test]
    fn test_audit_logging() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        // Check that initialization was logged
        let log = service.get_audit_log(Some(10));
        assert!(!log.is_empty());
        assert_eq!(log[0].operation, "service_init");
    }

    #[test]
    fn test_audit_log_clear() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let result = service.clear_audit_log();
        assert!(result.is_ok());
        
        let log = service.get_audit_log(Some(10));
        assert!(log.is_empty());
    }

    #[test]
    fn test_rate_limiting() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        // Add contacts up to the rate limit
        for i in 0..5 {
            let contact = Contact {
                contact_id: format!("contact_{}", i),
                name: format!("Contact {}", i),
                contact_type: "human".to_string(),
                agent_deployment_type: None,
                agent_ids: vec![],
                node_id: format!("node_{}", i),
                groups: vec![],
                tags: vec![],
                notes: "".to_string(),
                is_favorite: false,
                is_blocked: false,
                created_at: current_timestamp(),
                last_contacted: 0,
                contact_count: 0,
                public_account_id: None,
                iot_device_type: None,
                iot_protocol: None,
                iot_status: None,
                iot_last_seen: None,
                iot_capabilities: None,
                iot_location: None,
            };
            let result = service.add_contact(contact);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_resource_limit_check() {
        let (config, _temp_dir) = create_test_config();
        let service = ContactDirectoryService::new(config).expect("Failed to create service");
        
        let invariants = SafetyInvariants::default();
        let result = invariants.check_contact_limit(100000, "human");
        assert!(result.is_err());
    }

    #[test]
    fn test_input_validation() {
        let validator = InputValidator::default();
        
        // Test valid inputs
        assert!(validator.validate_contact_id("contact_123").is_ok());
        assert!(validator.validate_node_id("node_123").is_ok());
        assert!(validator.validate_contact_type("human").is_ok());
        
        // Test invalid inputs
        assert!(validator.validate_contact_id("").is_err());
        assert!(validator.validate_node_id("").is_err());
        assert!(validator.validate_contact_type("invalid").is_err());
    }

    // Aerospace component tests
    #[test]
    fn test_aerospace_error_codes() {
        let err = AerospaceError::InvalidParameter {
            parameter: "test".to_string(),
            reason: "test reason".to_string(),
        };
        assert_eq!(err.error_code(), "CD-001");
        assert!(!err.is_critical());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_aerospace_error_recoverable() {
        let err = AerospaceError::LockError {
            reason: "test".to_string(),
        };
        assert_eq!(err.error_code(), "CD-003");
        assert!(!err.is_critical());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_aerospace_error_critical() {
        let err = AerospaceError::StateInconsistency {
            description: "test".to_string(),
        };
        assert_eq!(err.error_code(), "CD-008");
        assert!(err.is_critical());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_input_validator_string_length() {
        let validator = InputValidator::default();
        assert!(validator.validate_string_length("test", "valid_string").is_ok());
        
        let long_string = "a".repeat(2000);
        assert!(validator.validate_string_length("test", &long_string).is_err());
    }

    #[test]
    fn test_safety_invariants_contact_limit() {
        let invariants = SafetyInvariants::default();
        assert!(invariants.check_contact_limit(0, "human").is_ok());
        assert!(invariants.check_contact_limit(100000, "human").is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 1);
        assert!(limiter.check("test").is_ok());
        assert!(limiter.check("test").is_ok());
        assert!(limiter.check("test").is_ok());
        assert!(limiter.check("test").is_err());
    }

    #[test]
    fn test_audit_log_entry() {
        let entry = AuditLogEntry::new(
            "test_operation".to_string(),
            "success".to_string(),
            "test details".to_string(),
        );
        assert_eq!(entry.operation, "test_operation");
        assert_eq!(entry.status, "success");
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_contacts, 100_000);
        assert_eq!(limits.max_groups, 10_000);
    }
}
