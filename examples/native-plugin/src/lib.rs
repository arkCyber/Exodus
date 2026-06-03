//! Example Exodus Browser Native Plugin
//! 
//! This is a sample plugin that demonstrates how to create a native Rust plugin
//! for Exodus Browser using the dynamic library interface.

use std::sync::{Arc, Mutex};
use exodus_plugin_sdk::{ExodusPlugin, PluginMetadata, PluginContext, export_plugin};
use serde_json::Value;

/// Example plugin implementation
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    counter: Arc<Mutex<u32>>,
}

impl ExamplePlugin {
    /// Create a new example plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "example-plugin".to_string(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A sample native plugin for Exodus Browser".to_string(),
                author: "Exodus Team".to_string(),
                permissions: vec!["storage".to_string(), "network".to_string()],
                api_version: exodus_plugin_sdk::PLUGIN_API_VERSION.to_string(),
            },
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

impl ExodusPlugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&mut self, context: PluginContext) -> Result<(), String> {
        println!("[ExamplePlugin] Initializing with context: {:?}", context.plugin_id);
        
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&context.data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        
        println!("[ExamplePlugin] Data directory: {:?}", context.data_dir);
        Ok(())
    }
    
    fn handle_command(&self, command: &str, params: Value) -> Result<Value, String> {
        match command {
            "ping" => {
                Ok(serde_json::json!({
                    "status": "pong",
                    "message": "Example plugin is working!"
                }))
            }
            "increment" => {
                let mut counter = self.counter.lock().unwrap();
                *counter += 1;
                Ok(serde_json::json!({
                    "counter": *counter
                }))
            }
            "get_counter" => {
                let counter = self.counter.lock().unwrap();
                Ok(serde_json::json!({
                    "counter": *counter
                }))
            }
            "echo" => {
                Ok(params)
            }
            _ => {
                Err(format!("Unknown command: {}", command))
            }
        }
    }
    
    fn cleanup(&mut self) -> Result<(), String> {
        println!("[ExamplePlugin] Cleaning up");
        Ok(())
    }
}

// Export the plugin using the SDK macro
export_plugin!(ExamplePlugin);
