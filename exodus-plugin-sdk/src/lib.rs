//! Exodus Browser Plugin SDK
//! 
//! This SDK provides shared types and traits for developing native plugins
//! for Exodus Browser. It ensures type safety and API compatibility between
//! the browser and plugins.

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Current plugin API version
pub const PLUGIN_API_VERSION: &str = "1.0.0";

/// Aerospace-level validation constants
pub const MAX_PLUGIN_NAME_LENGTH: usize = 100;
pub const MAX_PLUGIN_VERSION_LENGTH: usize = 50;
pub const MAX_PLUGIN_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_PLUGIN_ID_LENGTH: usize = 100;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: Vec<String>,
    pub api_version: String,
}

impl PluginMetadata {
    /// Validate plugin metadata
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.id.len() > MAX_PLUGIN_ID_LENGTH {
            return Err("Invalid plugin ID".to_string());
        }
        
        if self.name.is_empty() || self.name.len() > MAX_PLUGIN_NAME_LENGTH {
            return Err("Invalid plugin name".to_string());
        }
        
        if self.version.is_empty() || self.version.len() > MAX_PLUGIN_VERSION_LENGTH {
            return Err("Invalid plugin version".to_string());
        }
        
        if self.description.len() > MAX_PLUGIN_DESCRIPTION_LENGTH {
            return Err("Plugin description too long".to_string());
        }
        
        // Validate ID format (alphanumeric, hyphens, underscores)
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Plugin ID contains invalid characters".to_string());
        }
        
        // Validate API version
        if self.api_version != PLUGIN_API_VERSION {
            return Err(format!(
                "API version mismatch: expected {}, got {}",
                PLUGIN_API_VERSION, self.api_version
            ));
        }
        
        Ok(())
    }
}

/// Plugin context - provides access to browser APIs
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_id: String,
    pub data_dir: PathBuf,
    pub config: HashMap<String, String>,
}

/// Plugin trait - all native plugins must implement this
pub trait ExodusPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with context
    fn initialize(&mut self, context: PluginContext) -> Result<(), String>;
    
    /// Handle a command from the browser
    fn handle_command(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, String>;
    
    /// Cleanup before unloading
    fn cleanup(&mut self) -> Result<(), String>;
}

/// Helper macro to export plugin FFI symbols
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        static mut PLUGIN_INSTANCE: Option<$plugin_type> = None;

        #[no_mangle]
        pub extern "C" fn exodus_plugin_version() -> *const std::ffi::c_char {
            std::ffi::CString::new($crate::PLUGIN_API_VERSION)
                .unwrap()
                .into_raw()
        }

        #[no_mangle]
        pub extern "C" fn exodus_plugin_init() -> *mut dyn $crate::ExodusPlugin {
            let plugin = <$plugin_type>::new();
            
            unsafe {
                PLUGIN_INSTANCE = Some(plugin);
                PLUGIN_INSTANCE.as_mut().unwrap() as *mut dyn $crate::ExodusPlugin
            }
        }

        #[no_mangle]
        pub extern "C" fn exodus_plugin_deinit(_plugin: *mut dyn $crate::ExodusPlugin) {
            unsafe {
                if let Some(mut plugin) = PLUGIN_INSTANCE.take() {
                    let _ = plugin.cleanup();
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_validation() {
        let valid_metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        assert!(valid_metadata.validate().is_ok());
        
        let invalid_id = PluginMetadata {
            id: "test plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: PLUGIN_API_VERSION.to_string(),
        };
        
        assert!(invalid_id.validate().is_err());
    }
    
    #[test]
    fn test_api_version_mismatch() {
        let wrong_version = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            api_version: "0.0.1".to_string(),
        };
        
        assert!(wrong_version.validate().is_err());
    }
}
