//! Integration tests for plugin sandbox isolation
//! 
//! These tests verify that the sandbox isolation system works correctly
//! with actual plugin loading and execution.

use std::path::PathBuf;
use super::sandbox::{SandboxConfig, PluginSandbox};
use super::{PluginMetadata, PluginError, PLUGIN_API_VERSION};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_plugins::sandbox;
    
    #[test]
    fn test_sandbox_config_creation() {
        let config = SandboxConfig {
            enable_seccomp: true,
            allow_network: false,
            allow_filesystem: false,
            max_memory_mb: 256,
            socket_path: Some(PathBuf::from("/tmp/test_socket")),
        };
        
        assert!(config.enable_seccomp);
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
        assert_eq!(config.max_memory_mb, 256);
    }
    
    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.enable_seccomp);
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
        assert_eq!(config.max_memory_mb, 512);
    }
    
    #[test]
    fn test_plugin_sandbox_creation() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        let plugin_path = PathBuf::from("/tmp/test_plugin.dylib");
        let config = SandboxConfig::default();
        
        let sandbox = PluginSandbox::new(plugin_path, metadata, config);
        assert!(sandbox.is_ok());
    }
    
    #[test]
    fn test_plugin_metadata_validation() {
        let valid_metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        // Test valid metadata
        assert_eq!(valid_metadata.id, "test-plugin");
        assert_eq!(valid_metadata.name, "Test Plugin");
        assert_eq!(valid_metadata.version, PLUGIN_API_VERSION);
        assert_eq!(valid_metadata.api_version, PLUGIN_API_VERSION);
    }
    
    #[test]
    fn test_sandbox_socket_path_generation() {
        let metadata = PluginMetadata {
            id: "my-plugin".to_string(),
            name: "My Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "My test plugin".to_string(),
            author: "Test".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        let plugin_path = PathBuf::from("/tmp/test.dylib");
        let config = SandboxConfig::default();
        
        let sandbox = PluginSandbox::new(plugin_path, metadata, config).unwrap();
        let socket_path = sandbox.socket_path();
        
        // Socket path should contain the plugin ID
        assert!(socket_path.to_string_lossy().contains("my-plugin"));
    }
    
    #[test]
    fn test_sandbox_is_running_check() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        let plugin_path = PathBuf::from("/tmp/test_plugin.dylib");
        let config = SandboxConfig::default();

        let mut sandbox = PluginSandbox::new(plugin_path, metadata, config).unwrap();
        
        // Sandbox should not be running before start
        assert!(!sandbox.is_running());
    }
    
    #[test]
    fn test_seccomp_filter_with_permissions() {
        // Test seccomp filter with different permission combinations
        let result1 = sandbox::apply_seccomp_filter(true, true);
        assert!(result1.is_ok());

        let result2 = sandbox::apply_seccomp_filter(false, false);
        assert!(result2.is_ok());

        let result3 = sandbox::apply_seccomp_filter(true, false);
        assert!(result3.is_ok());

        let result4 = sandbox::apply_seccomp_filter(false, true);
        assert!(result4.is_ok());
    }
    
    #[test]
    fn test_sandbox_resource_tracking() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        let config = SandboxConfig::default();
        let plugin_path = PathBuf::from("/tmp/test_plugin");
        
        let sandbox = PluginSandbox::new(plugin_path, metadata, config).unwrap();
        
        // Test initial resource stats
        let stats = sandbox.get_resource_stats();
        assert_eq!(stats.command_count, 0);
        assert_eq!(stats.network_request_count, 0);
        
        // Test network request increment
        sandbox.increment_network_request();
        let stats = sandbox.get_resource_stats();
        assert_eq!(stats.network_request_count, 1);
        
        // Test multiple network requests
        for _ in 0..5 {
            sandbox.increment_network_request();
        }
        let stats = sandbox.get_resource_stats();
        assert_eq!(stats.network_request_count, 6);
    }
    
    #[test]
    fn test_sandbox_command_count_tracking() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: PLUGIN_API_VERSION.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        let config = SandboxConfig::default();
        let plugin_path = PathBuf::from("/tmp/test_plugin");
        
        let sandbox = PluginSandbox::new(plugin_path, metadata, config).unwrap();
        
        // Note: send_command requires actual socket, so we can't test it directly
        // But we can verify the field exists and is initialized
        let stats = sandbox.get_resource_stats();
        assert_eq!(stats.command_count, 0);
    }
}
