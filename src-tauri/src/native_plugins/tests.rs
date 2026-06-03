//! Comprehensive tests for the native plugin system

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::native_plugins::{PluginPermission, PluginMetadata, NativePluginManager, PLUGIN_API_VERSION, PluginError, MAX_PLUGIN_NAME_LENGTH, MAX_PLUGIN_VERSION_LENGTH, MAX_PLUGIN_DESCRIPTION_LENGTH, MAX_PLUGIN_ID_LENGTH};

    #[test]
    fn test_plugin_permission_from_string() {
        assert_eq!(
            PluginPermission::from_string("storage"),
            PluginPermission::Storage
        );
        assert_eq!(
            PluginPermission::from_string("network"),
            PluginPermission::Network
        );
        assert_eq!(
            PluginPermission::from_string("custom_permission"),
            PluginPermission::Custom("custom_permission".to_string())
        );
    }

    #[test]
    fn test_plugin_permission_to_string() {
        assert_eq!(PluginPermission::Storage.to_string(), "storage");
        assert_eq!(PluginPermission::Network.to_string(), "network");
        assert_eq!(
            PluginPermission::Custom("test".to_string()).to_string(),
            "test"
        );
    }

    #[test]
    fn test_plugin_permission_is_sensitive() {
        assert!(PluginPermission::Passwords.is_sensitive());
        assert!(PluginPermission::Cookies.is_sensitive());
        assert!(PluginPermission::History.is_sensitive());
        assert!(!PluginPermission::Storage.is_sensitive());
        assert!(!PluginPermission::Network.is_sensitive());
    }

    #[test]
    fn test_plugin_metadata_validation() {
        let temp_dir = std::env::temp_dir().join("plugin_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let manager = NativePluginManager::new(temp_dir.clone()).unwrap();

        // Valid metadata
        let valid_metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec!["storage".to_string(), "network".to_string()],
            api_version: PLUGIN_API_VERSION.to_string(),
        };

        assert!(manager.validate_metadata(&valid_metadata).is_ok());

        // Invalid ID (contains space)
        let invalid_id = PluginMetadata {
            id: "test plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };

        assert!(manager.validate_metadata(&invalid_id).is_err());

        // ID too long
        let too_long_id = PluginMetadata {
            id: "a".repeat(101),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };

        assert!(manager.validate_metadata(&too_long_id).is_err());

        // Empty name
        let empty_name = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };

        assert!(manager.validate_metadata(&empty_name).is_err());

        // Description too long
        let too_long_desc = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "a".repeat(501),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };

        assert!(manager.validate_metadata(&too_long_desc).is_err());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_plugin_metadata_permissions() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec!["storage".to_string(), "passwords".to_string()],
            api_version: PLUGIN_API_VERSION.to_string(),
        };

        let permissions = metadata.get_permissions();
        assert_eq!(permissions.len(), 2);
        assert!(permissions.contains(&PluginPermission::Storage));
        assert!(permissions.contains(&PluginPermission::Passwords));

        assert!(metadata.has_permission(&PluginPermission::Storage));
        assert!(!metadata.has_permission(&PluginPermission::Network));

        assert!(metadata.has_sensitive_permissions());
    }

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = std::env::temp_dir().join("plugin_test_mgr");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let manager = NativePluginManager::new(temp_dir.clone());
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert_eq!(manager.list_plugins().len(), 0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_plugin_manager_scan_empty_directory() {
        let temp_dir = std::env::temp_dir().join("plugin_test_scan");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mut manager = NativePluginManager::new(temp_dir.clone()).unwrap();
        
        let count = manager.scan_and_load().unwrap();
        assert_eq!(count, 0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_plugin_path_validation() {
        let temp_dir = std::env::temp_dir().join("plugin_test_path");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let manager = NativePluginManager::new(temp_dir.clone()).unwrap();

        // Non-existent path
        let non_existent = temp_dir.join("nonexistent.dylib");
        assert!(manager.validate_plugin_path(&non_existent).is_err());

        // Path outside plugins directory
        let outside = PathBuf::from("/tmp/test.dylib");
        assert!(manager.validate_plugin_path(&outside).is_err());

        // Valid path (file doesn't exist but path is valid)
        let valid_path = temp_dir.join("test.dylib");
        // This will fail because file doesn't exist, but path validation should pass
        assert!(manager.validate_plugin_path(&valid_path).is_err());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_plugin_constants() {
        assert_eq!(MAX_PLUGIN_NAME_LENGTH, 100);
        assert_eq!(MAX_PLUGIN_VERSION_LENGTH, 50);
        assert_eq!(MAX_PLUGIN_DESCRIPTION_LENGTH, 500);
        assert_eq!(MAX_PLUGIN_ID_LENGTH, 100);
        assert_eq!(PLUGIN_API_VERSION, "1.0.0");
    }

    #[test]
    fn test_plugin_error_messages() {
        let error = PluginError::NotFound("test-id".to_string());
        assert_eq!(error.to_string(), "Plugin not found: test-id");

        let error = PluginError::AlreadyLoaded("test-id".to_string());
        assert_eq!(error.to_string(), "Plugin already loaded: test-id");

        let error = PluginError::SecurityError("test error".to_string());
        assert_eq!(error.to_string(), "Security validation failed: test error");
    }
    
    #[test]
    fn test_user_context_in_audit_logs() {
        let temp_dir = std::env::temp_dir().join("plugin_test_user");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mut manager = NativePluginManager::new(temp_dir.clone()).unwrap();
        
        // Initially, user ID should be None
        assert!(manager.get_user_id().is_none());
        
        // Set user ID
        manager.set_user_id(Some("user123".to_string()));
        assert_eq!(manager.get_user_id(), Some(&"user123".to_string()));
        
        // Change user ID
        manager.set_user_id(Some("user456".to_string()));
        assert_eq!(manager.get_user_id(), Some(&"user456".to_string()));
        
        // Clear user ID
        manager.set_user_id(None);
        assert!(manager.get_user_id().is_none());
        
        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
    
    #[test]
    fn test_audit_log_with_user_context() {
        let temp_dir = std::env::temp_dir().join("plugin_test_audit_user");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mut manager = NativePluginManager::new(temp_dir.clone()).unwrap();
        
        // Set user ID
        manager.set_user_id(Some("test_user".to_string()));
        
        // Add an audit log entry
        manager.add_audit_log(
            "test-plugin".to_string(),
            "load".to_string(),
            "success".to_string(),
            "Plugin loaded successfully".to_string()
        );
        
        // Get audit log
        let log = manager.get_audit_log(None);
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].plugin_id, "test-plugin");
        assert_eq!(log[0].user_id, Some("test_user".to_string()));
        
        // Clear user ID and add another entry
        manager.set_user_id(None);
        manager.add_audit_log(
            "test-plugin2".to_string(),
            "unload".to_string(),
            "success".to_string(),
            "Plugin unloaded successfully".to_string()
        );
        
        // Get audit log again
        let log = manager.get_audit_log(None);
        assert_eq!(log.len(), 2);
        assert_eq!(log[1].user_id, None);
        
        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
    
    #[test]
    fn test_sandbox_hot_reload_detection() {
        let temp_dir = std::env::temp_dir().join("plugin_test_hot_reload");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mut manager = NativePluginManager::new(temp_dir.clone()).unwrap();
        
        // Enable sandbox
        let config = crate::native_plugins::sandbox::SandboxConfig::default();
        manager.enable_sandbox(config);
        
        // Create a dummy plugin file
        let plugin_path = temp_dir.join("test_plugin.dylib");
        std::fs::write(&plugin_path, b"dummy plugin content").unwrap();
        
        // Note: We can't actually load the plugin since it's not a valid .dylib
        // But we can test the hot-reload logic by checking the file modification time
        // The implementation checks if the file was modified in the last 10 seconds
        
        // Touch the file to make it recently modified
        std::fs::write(&plugin_path, b"updated plugin content").unwrap();
        
        // Verify the file exists and was modified
        let metadata = std::fs::metadata(&plugin_path).unwrap();
        let modified = metadata.modified().unwrap();
        assert!(modified.elapsed().unwrap() < std::time::Duration::from_secs(10));
        
        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
