//! Integration test for loading actual plugin files

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;
    use crate::native_plugins::{PLUGIN_API_VERSION, NativePluginManager};

    #[test]
    #[ignore] // Run this manually with: cargo test --lib native_plugins::integration_tests::test_load_actual_plugin -- --ignored
    fn test_load_actual_plugin() {
        let plugin_path = PathBuf::from("/tmp/exodus_test_plugins/native/libexodus_example_plugin.dylib");
        
        if !plugin_path.exists() {
            println!("Plugin file not found at: {:?}", plugin_path);
            println!("Run: cp examples/native-plugin/target/release/libexodus_example_plugin.dylib /tmp/exodus_test_plugins/native/");
            return;
        }

        let temp_dir = std::env::temp_dir().join("plugin_integration_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let mut manager = NativePluginManager::new(temp_dir.clone()).unwrap();

        // Try to load the actual plugin
        match manager.load_plugin(&plugin_path) {
            Ok(metadata) => {
                println!("Successfully loaded plugin: {}", metadata.name);
                println!("Plugin ID: {}", metadata.id);
                println!("Plugin version: {}", metadata.version);
                println!("Plugin permissions: {:?}", metadata.permissions);
                
                // Verify metadata
                assert_eq!(metadata.id, "example-plugin");
                assert_eq!(metadata.name, "Example Plugin");
                assert_eq!(metadata.version, PLUGIN_API_VERSION);
                
                // Test command execution
                let result = manager.execute_command(&metadata.id, "ping", serde_json::json!({}));
                assert!(result.is_ok());
                
                let result = result.unwrap();
                println!("Ping result: {:?}", result);
                
                // Test increment command
                let result1 = manager.execute_command(&metadata.id, "increment", serde_json::json!({}));
                assert!(result1.is_ok());
                
                let result2 = manager.execute_command(&metadata.id, "get_counter", serde_json::json!({}));
                assert!(result2.is_ok());
                
                let counter = result2.unwrap();
                println!("Counter value: {:?}", counter);
                
                // Unload the plugin
                let unload_result = manager.unload_plugin(&metadata.id);
                assert!(unload_result.is_ok());
                
                println!("Integration test passed!");
            }
            Err(e) => {
                println!("Failed to load plugin: {}", e);
                panic!("Plugin loading failed: {}", e);
            }
        }

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    #[ignore]
    fn test_plugin_version_check() {
        use libloading::{Library, Symbol};
        
        let plugin_path = PathBuf::from("/tmp/exodus_test_plugins/native/libexodus_example_plugin.dylib");
        
        if !plugin_path.exists() {
            println!("Plugin file not found at: {:?}", plugin_path);
            return;
        }

        unsafe {
            let library = Library::new(&plugin_path).expect("Failed to load library");
            
            type PluginVersionFn = unsafe extern "C" fn() -> *const std::ffi::c_char;
            let version_fn: Symbol<PluginVersionFn> = library.get(b"exodus_plugin_version")
                .expect("Failed to find version symbol");
            
            let version_ptr = version_fn();
            let version_cstr = std::ffi::CStr::from_ptr(version_ptr);
            let version_str = version_cstr.to_str().unwrap();
            
            println!("Plugin API version: {}", version_str);
            assert_eq!(version_str, PLUGIN_API_VERSION);
        }
    }
}
