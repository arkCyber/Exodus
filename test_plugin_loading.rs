//! Standalone test for plugin loading
//! Run with: rustc --edition 2021 -L target/release/deps test_plugin_loading.rs -o test_plugin_loading --extern libloading=target/release/deps/liblibloading-*.rlib --extern serde=target/release/deps/libserde-*.rlib --extern serde_json=target/release/deps/libserde_json-*.rlib

use std::path::PathBuf;
use std::ffi::CStr;

fn main() {
    let plugin_path = PathBuf::from("/tmp/exodus_test_plugins/native/libexodus_example_plugin.dylib");
    
    if !plugin_path.exists() {
        eprintln!("Plugin file not found at: {:?}", plugin_path);
        eprintln!("Run: cp examples/native-plugin/target/release/libexodus_example_plugin.dylib /tmp/exodus_test_plugins/native/");
        std::process::exit(1);
    }

    println!("Testing plugin loading: {:?}", plugin_path);

    unsafe {
        // Load the library
        let library = libloading::Library::new(&plugin_path).expect("Failed to load library");
        println!("✓ Library loaded successfully");
        
        // Check version symbol
        type PluginVersionFn = unsafe extern "C" fn() -> *const std::ffi::c_char;
        let version_fn: libloading::Symbol<PluginVersionFn> = library.get(b"exodus_plugin_version")
            .expect("Failed to find version symbol");
        
        let version_ptr = version_fn();
        let version_cstr = CStr::from_ptr(version_ptr);
        let version_str = version_cstr.to_str().unwrap();
        
        println!("✓ Plugin API version: {}", version_str);
        assert_eq!(version_str, "1.0.0");
        
        // Check init symbol
        type PluginInitFn = unsafe extern "C" fn() -> *mut std::ffi::c_void;
        let init_fn: libloading::Symbol<PluginInitFn> = library.get(b"exodus_plugin_init")
            .expect("Failed to find init symbol");
        
        let plugin_ptr = init_fn();
        println!("✓ Plugin initialized: {:?}", plugin_ptr);
        
        if !plugin_ptr.is_null() {
            println!("✓ Plugin instance created successfully");
        } else {
            eprintln!("✗ Plugin initialization returned null");
            std::process::exit(1);
        }
        
        // Check deinit symbol
        type PluginDeinitFn = unsafe extern "C" fn(plugin: *mut std::ffi::c_void);
        let deinit_fn: libloading::Symbol<PluginDeinitFn> = library.get(b"exodus_plugin_deinit")
            .expect("Failed to find deinit symbol");
        
        deinit_fn(plugin_ptr);
        println!("✓ Plugin deinitialized successfully");
    }
    
    println!("\n✅ All plugin loading tests passed!");
}
