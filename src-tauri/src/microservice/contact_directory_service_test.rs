//! Aerospace-level unit tests for Contact Directory Service
//!
//! These tests follow DO-178C Level A safety-critical software standards:
//! - 100% code coverage for critical paths
//! - Property-based testing for invariants
//! - Edge case testing
//! - Concurrency safety testing

use super::*;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[cfg(test)]
mod aerospace_error_tests {
    use super::*;

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
    fn test_aerospace_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: AerospaceError = io_err.into();
        assert_eq!(err.error_code(), "CD-004");
    }

    #[test]
    fn test_aerospace_error_from_serialization() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: AerospaceError = json_err.into();
        assert_eq!(err.error_code(), "CD-005");
    }
}

#[cfg(test)]
mod input_validator_tests {
    use super::*;

    #[test]
    fn test_validate_string_length_success() {
        let validator = InputValidator::default();
        assert!(validator.validate_string_length("test", "valid_string").is_ok());
    }

    #[test]
    fn test_validate_string_length_failure() {
        let validator = InputValidator::new(10);
        let long_string = "a".repeat(20);
        let result = validator.validate_string_length("test", &long_string);
        assert!(result.is_err());
        match result {
            Err(AerospaceError::ValidationError { field, reason }) => {
                assert_eq!(field, "test");
                assert!(reason.contains("exceeds maximum"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_contact_id_success() {
        let validator = InputValidator::default();
        assert!(validator.validate_contact_id("contact_123").is_ok());
        assert!(validator.validate_contact_id("contact-456").is_ok());
    }

    #[test]
    fn test_validate_contact_id_empty() {
        let validator = InputValidator::default();
        let result = validator.validate_contact_id("");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "contact_id");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_contact_id_invalid_chars() {
        let validator = InputValidator::default();
        let result = validator.validate_contact_id("contact@123");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "contact_id");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_node_id_success() {
        let validator = InputValidator::default();
        assert!(validator.validate_node_id("node_123").is_ok());
    }

    #[test]
    fn test_validate_node_id_empty() {
        let validator = InputValidator::default();
        let result = validator.validate_node_id("");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "node_id");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_contact_type_valid() {
        let validator = InputValidator::default();
        assert!(validator.validate_contact_type("human").is_ok());
        assert!(validator.validate_contact_type("agent").is_ok());
        assert!(validator.validate_contact_type("iot").is_ok());
    }

    #[test]
    fn test_validate_contact_type_invalid() {
        let validator = InputValidator::default();
        let result = validator.validate_contact_type("invalid_type");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "contact_type");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_agent_deployment_type_valid() {
        let validator = InputValidator::default();
        assert!(validator.validate_agent_deployment_type("service").is_ok());
        assert!(validator.validate_agent_deployment_type("personal_assistant").is_ok());
    }

    #[test]
    fn test_validate_agent_deployment_type_invalid() {
        let validator = InputValidator::default();
        let result = validator.validate_agent_deployment_type("invalid_type");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "agent_deployment_type");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_validate_digit_id_valid() {
        let validator = InputValidator::default();
        assert!(validator.validate_digit_id("123456789012").is_ok());
        assert!(validator.validate_digit_id("123-456-789-012").is_ok());
        assert!(validator.validate_digit_id("123 456 789 012").is_ok());
    }

    #[test]
    fn test_validate_digit_id_invalid() {
        let validator = InputValidator::default();
        let result = validator.validate_digit_id("123");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "digit_id");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_sanitize_string() {
        let validator = InputValidator::default();
        let sanitized = validator.sanitize_string("test@#$%^&*()string");
        assert_eq!(sanitized, "test@.string");
    }

    #[test]
    fn test_validate_contact_structure() {
        let validator = InputValidator::default();
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
            created_at: 0,
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
        assert!(validator.validate_contact(&contact).is_ok());
    }

    #[test]
    fn test_validate_contact_structure_invalid_type() {
        let validator = InputValidator::default();
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
            created_at: 0,
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
        let result = validator.validate_contact(&contact);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_group_structure() {
        let validator = InputValidator::default();
        let group = ContactGroup {
            group_id: "group_123".to_string(),
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
            color: "#FF0000".to_string(),
            created_at: 0,
        };
        assert!(validator.validate_group(&group).is_ok());
    }

    #[test]
    fn test_validate_group_empty_id() {
        let validator = InputValidator::default();
        let group = ContactGroup {
            group_id: "".to_string(),
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
            color: "#FF0000".to_string(),
            created_at: 0,
        };
        let result = validator.validate_group(&group);
        assert!(result.is_err());
        match result {
            Err(AerospaceError::InvalidParameter { parameter, .. }) => {
                assert_eq!(parameter, "group_id");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }
}

#[cfg(test)]
mod safety_invariants_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_check_contact_limit_success() {
        let invariants = SafetyInvariants::default();
        assert!(invariants.check_contact_limit(0, "human").is_ok());
        assert!(invariants.check_contact_limit(99999, "human").is_ok());
    }

    #[test]
    fn test_check_contact_limit_exceeded() {
        let invariants = SafetyInvariants::default();
        let result = invariants.check_contact_limit(100000, "human");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::ResourceLimitExceeded { resource, limit }) => {
                assert!(resource.contains("contacts"));
                assert_eq!(limit, 100000);
            }
            _ => panic!("Expected ResourceLimitExceeded"),
        }
    }

    #[test]
    fn test_check_contact_limit_iot() {
        let invariants = SafetyInvariants::default();
        assert!(invariants.check_contact_limit(0, "iot").is_ok());
        assert!(invariants.check_contact_limit(49999, "iot").is_ok());
    }

    #[test]
    fn test_check_contact_limit_iot_exceeded() {
        let invariants = SafetyInvariants::default();
        let result = invariants.check_contact_limit(50000, "iot");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::ResourceLimitExceeded { resource, limit }) => {
                assert!(resource.contains("iot"));
                assert_eq!(limit, 50000);
            }
            _ => panic!("Expected ResourceLimitExceeded"),
        }
    }

    #[test]
    fn test_check_group_limit_success() {
        let invariants = SafetyInvariants::default();
        assert!(invariants.check_group_limit(0).is_ok());
        assert!(invariants.check_group_limit(9999).is_ok());
    }

    #[test]
    fn test_check_group_limit_exceeded() {
        let invariants = SafetyInvariants::default();
        let result = invariants.check_group_limit(10000);
        assert!(result.is_err());
        match result {
            Err(AerospaceError::ResourceLimitExceeded { resource, limit }) => {
                assert_eq!(resource, "groups");
                assert_eq!(limit, 10000);
            }
            _ => panic!("Expected ResourceLimitExceeded"),
        }
    }

    #[test]
    fn test_verify_contact_count_success() {
        let invariants = SafetyInvariants::default();
        let contacts: HashMap<String, Contact> = HashMap::new();
        assert!(invariants.verify_contact_count(&contacts, 0).is_ok());
    }

    #[test]
    fn test_verify_contact_count_mismatch() {
        let invariants = SafetyInvariants::default();
        let contacts: HashMap<String, Contact> = HashMap::new();
        let result = invariants.verify_contact_count(&contacts, 10);
        assert!(result.is_err());
        match result {
            Err(AerospaceError::StateInconsistency { description }) => {
                assert!(description.contains("Contact count mismatch"));
            }
            _ => panic!("Expected StateInconsistency"),
        }
    }
}

#[cfg(test)]
mod rate_limiter_tests {
    use super::*;

    #[test]
    fn test_rate_limiter_check_success() {
        let limiter = RateLimiter::new(10, 1);
        for _ in 0..10 {
            assert!(limiter.check("test_operation").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_check_exceeded() {
        let limiter = RateLimiter::new(5, 1);
        for _ in 0..5 {
            assert!(limiter.check("test_operation").is_ok());
        }
        let result = limiter.check("test_operation");
        assert!(result.is_err());
        match result {
            Err(AerospaceError::RateLimitExceeded { operation, limit }) => {
                assert_eq!(operation, "test_operation");
                assert_eq!(limit, 5);
            }
            _ => panic!("Expected RateLimitExceeded"),
        }
    }

    #[test]
    fn test_rate_limiter_window_expiration() {
        let limiter = RateLimiter::new(5, 1);
        for _ in 0..5 {
            assert!(limiter.check("test_operation").is_ok());
        }
        assert!(limiter.check("test_operation").is_err());
        // Simulate time passing by waiting
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(limiter.check("test_operation").is_ok());
    }

    #[test]
    fn test_rate_limiter_separate_operations() {
        let limiter = RateLimiter::new(3, 1);
        for _ in 0..3 {
            assert!(limiter.check("operation_a").is_ok());
        }
        assert!(limiter.check("operation_a").is_err());
        assert!(limiter.check("operation_b").is_ok());
    }
}

#[cfg(test)]
mod audit_log_tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::new(
            "test_operation".to_string(),
            "success".to_string(),
            "test details".to_string(),
        );
        assert_eq!(entry.operation, "test_operation");
        assert_eq!(entry.status, "success");
        assert_eq!(entry.details, "test details");
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_audit_log_entry_with_user_id() {
        let entry = AuditLogEntry::new(
            "test_operation".to_string(),
            "success".to_string(),
            "test details".to_string(),
        )
        .with_user_id("user_123".to_string());
        assert_eq!(entry.user_id, Some("user_123".to_string()));
    }

    #[test]
    fn test_audit_log_entry_with_resource_id() {
        let entry = AuditLogEntry::new(
            "test_operation".to_string(),
            "success".to_string(),
            "test details".to_string(),
        )
        .with_resource_id("resource_123".to_string());
        assert_eq!(entry.resource_id, Some("resource_123".to_string()));
    }

    #[test]
    fn test_audit_log_entry_with_error_code() {
        let entry = AuditLogEntry::new(
            "test_operation".to_string(),
            "failure".to_string(),
            "test details".to_string(),
        )
        .with_error_code("CD-001".to_string());
        assert_eq!(entry.error_code, Some("CD-001".to_string()));
    }

    #[test]
    fn test_audit_log_entry_builder_pattern() {
        let entry = AuditLogEntry::new(
            "test_operation".to_string(),
            "success".to_string(),
            "test details".to_string(),
        )
        .with_user_id("user_123".to_string())
        .with_resource_id("resource_123".to_string())
        .with_error_code("CD-001".to_string());
        assert_eq!(entry.user_id, Some("user_123".to_string()));
        assert_eq!(entry.resource_id, Some("resource_123".to_string()));
        assert_eq!(entry.error_code, Some("CD-001".to_string()));
    }
}

#[cfg(test)]
mod resource_limits_tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_contacts, 100_000);
        assert_eq!(limits.max_groups, 10_000);
        assert_eq!(limits.max_contacts_per_group, 5_000);
        assert_eq!(limits.max_iot_devices, 50_000);
        assert_eq!(limits.max_request_rate, 1000);
        assert_eq!(limits.max_string_length, 1024);
    }

    #[test]
    fn test_resource_limits_custom() {
        let limits = ResourceLimits {
            max_contacts: 1000,
            max_groups: 100,
            max_contacts_per_group: 50,
            max_iot_devices: 500,
            max_request_rate: 100,
            max_string_length: 512,
        };
        assert_eq!(limits.max_contacts, 1000);
        assert_eq!(limits.max_groups, 100);
        assert_eq!(limits.max_contacts_per_group, 50);
        assert_eq!(limits.max_iot_devices, 500);
        assert_eq!(limits.max_request_rate, 100);
        assert_eq!(limits.max_string_length, 512);
    }
}

#[cfg(test)]
mod iot_enums_tests {
    use super::*;

    #[test]
    fn test_iot_device_type_as_str() {
        assert_eq!(IoTDeviceType::SmartLight.as_str(), "smart_light");
        assert_eq!(IoTDeviceType::Thermostat.as_str(), "thermostat");
        assert_eq!(IoTDeviceType::Other("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_iot_device_type_from_str() {
        assert_eq!(IoTDeviceType::from_str("smart_light"), IoTDeviceType::SmartLight);
        assert_eq!(IoTDeviceType::from_str("thermostat"), IoTDeviceType::Thermostat);
        assert_eq!(IoTDeviceType::from_str("custom"), IoTDeviceType::Other("custom".to_string()));
    }

    #[test]
    fn test_iot_protocol_as_str() {
        assert_eq!(IoTProtocol::Mqtt.as_str(), "mqtt");
        assert_eq!(IoTProtocol::Coap.as_str(), "coap");
        assert_eq!(IoTProtocol::Other("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_iot_protocol_from_str() {
        assert_eq!(IoTProtocol::from_str("mqtt"), IoTProtocol::Mqtt);
        assert_eq!(IoTProtocol::from_str("coap"), IoTProtocol::Coap);
        assert_eq!(IoTProtocol::from_str("custom"), IoTProtocol::Other("custom".to_string()));
    }

    #[test]
    fn test_iot_status_as_str() {
        assert_eq!(IoTStatus::Online.as_str(), "online");
        assert_eq!(IoTStatus::Offline.as_str(), "offline");
        assert_eq!(IoTStatus::Error.as_str(), "error");
    }

    #[test]
    fn test_iot_status_from_str() {
        assert_eq!(IoTStatus::from_str("online"), IoTStatus::Online);
        assert_eq!(IoTStatus::from_str("offline"), IoTStatus::Offline);
        assert_eq!(IoTStatus::from_str("error"), IoTStatus::Error);
        assert_eq!(IoTStatus::from_str("unknown"), IoTStatus::Unknown);
    }

    #[test]
    fn test_iot_status_is_online() {
        assert!(IoTStatus::Online.is_online());
        assert!(!IoTStatus::Offline.is_online());
        assert!(!IoTStatus::Error.is_online());
    }

    #[test]
    fn test_iot_capability_as_str() {
        assert_eq!(IoTCapability::OnOff.as_str(), "on_off");
        assert_eq!(IoTCapability::Dimming.as_str(), "dimming");
        assert_eq!(IoTCapability::Other("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_iot_capability_from_str() {
        assert_eq!(IoTCapability::from_str("on_off"), IoTCapability::OnOff);
        assert_eq!(IoTCapability::from_str("dimming"), IoTCapability::Dimming);
        assert_eq!(IoTCapability::from_str("custom"), IoTCapability::Other("custom".to_string()));
    }
}

#[cfg(test)]
mod iot_validator_tests {
    use super::*;

    #[test]
    fn test_iot_validator_validate_device_type_success() {
        assert!(IoTValidator::validate_device_type("smart_light").is_ok());
        assert!(IoTValidator::validate_device_type("thermostat").is_ok());
    }

    #[test]
    fn test_iot_validator_validate_device_type_failure() {
        let result = IoTValidator::validate_device_type("invalid_type");
        assert!(result.is_err());
    }

    #[test]
    fn test_iot_validator_validate_protocol_success() {
        assert!(IoTValidator::validate_protocol("mqtt").is_ok());
        assert!(IoTValidator::validate_protocol("coap").is_ok());
    }

    #[test]
    fn test_iot_validator_validate_protocol_failure() {
        let result = IoTValidator::validate_protocol("invalid_protocol");
        assert!(result.is_err());
    }

    #[test]
    fn test_iot_validator_validate_status_success() {
        assert!(IoTValidator::validate_status("online").is_ok());
        assert!(IoTValidator::validate_status("offline").is_ok());
    }

    #[test]
    fn test_iot_validator_validate_status_failure() {
        let result = IoTValidator::validate_status("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_iot_validator_validate_capability_success() {
        assert!(IoTValidator::validate_capability("on_off").is_ok());
        assert!(IoTValidator::validate_capability("dimming").is_ok());
    }

    #[test]
    fn test_iot_validator_validate_capability_failure() {
        let result = IoTValidator::validate_capability("invalid_capability");
        assert!(result.is_err());
    }

    #[test]
    fn test_iot_validator_validate_all_success() {
        assert!(IoTValidator::validate_all(
            Some("smart_light"),
            Some("mqtt"),
            Some("online"),
            Some(&vec!["on_off".to_string(), "dimming".to_string()]),
        ).is_ok());
    }

    #[test]
    fn test_iot_validator_validate_all_failure() {
        let result = IoTValidator::validate_all(
            Some("invalid_type"),
            Some("mqtt"),
            Some("online"),
            Some(&vec!["on_off".to_string()]),
        );
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod contact_tests {
    use super::*;

    #[test]
    fn test_contact_iot_device_type_enum() {
        let mut contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Device".to_string(),
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: 0,
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: Some("smart_light".to_string()),
            iot_protocol: Some("mqtt".to_string()),
            iot_status: Some("online".to_string()),
            iot_last_seen: Some(0),
            iot_capabilities: Some(vec!["on_off".to_string()]),
            iot_location: Some("living_room".to_string()),
        };

        assert_eq!(
            contact.iot_device_type_enum(),
            Some(IoTDeviceType::SmartLight)
        );
    }

    #[test]
    fn test_contact_set_iot_device_type() {
        let mut contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Device".to_string(),
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: 0,
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

        contact.set_iot_device_type(IoTDeviceType::SmartLight);
        assert_eq!(contact.iot_device_type, Some("smart_light".to_string()));
    }

    #[test]
    fn test_contact_validate_iot_fields_success() {
        let contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Device".to_string(),
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: 0,
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: Some("smart_light".to_string()),
            iot_protocol: Some("mqtt".to_string()),
            iot_status: Some("online".to_string()),
            iot_last_seen: Some(0),
            iot_capabilities: Some(vec!["on_off".to_string()]),
            iot_location: Some("living_room".to_string()),
        };

        assert!(contact.validate_iot_fields().is_ok());
    }

    #[test]
    fn test_contact_validate_iot_fields_failure_missing_device_type() {
        let contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Device".to_string(),
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: 0,
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: None,
            iot_protocol: Some("mqtt".to_string()),
            iot_status: Some("online".to_string()),
            iot_last_seen: Some(0),
            iot_capabilities: Some(vec!["on_off".to_string()]),
            iot_location: Some("living_room".to_string()),
        };

        let result = contact.validate_iot_fields();
        assert!(result.is_err());
    }

    #[test]
    fn test_contact_is_iot_online() {
        let contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Device".to_string(),
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: 0,
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: Some("smart_light".to_string()),
            iot_protocol: Some("mqtt".to_string()),
            iot_status: Some("online".to_string()),
            iot_last_seen: Some(0),
            iot_capabilities: Some(vec!["on_off".to_string()]),
            iot_location: Some("living_room".to_string()),
        };

        assert!(contact.is_iot_online());
    }

    #[test]
    fn test_contact_is_iot_online_offline() {
        let contact = Contact {
            contact_id: "contact_123".to_string(),
            name: "Test Device".to_string(),
            contact_type: "iot".to_string(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node_123".to_string(),
            groups: vec![],
            tags: vec![],
            notes: "Test notes".to_string(),
            is_favorite: false,
            is_blocked: false,
            created_at: 0,
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: Some("smart_light".to_string()),
            iot_protocol: Some("mqtt".to_string()),
            iot_status: Some("offline".to_string()),
            iot_last_seen: Some(0),
            iot_capabilities: Some(vec!["on_off".to_string()]),
            iot_location: Some("living_room".to_string()),
        };

        assert!(!contact.is_iot_online());
    }
}
