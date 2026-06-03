//! Simple standalone test for plugin symbol verification
//! Run with: rustc test_plugin_simple.rs --extern libloading=target/release/deps/liblibloading-*.rlib -o test_plugin_simple

use std::path::PathBuf;
use std::ffi::CStr;

fn main() {
    let plugin_path = PathBuf::from("/tmp/exodus_test_plugins/native/libexodus_example_plugin.dylib");
    
    if !plugin_path.exists() {
        eprintln!("Plugin file not found at: {:?}", plugin_path);
        eprintln!("Run: cp examples/native-plugin/target/release/libexodus_example_plugin.dylib /tmp/exodus_test_plugins/native/");
        std::process::exit(1);
    }

    println!("Testing plugin symbols: {:?}", plugin_path);

    unsafe {
        let library = libloading::Library::new(&plugin_path).expect("Failed to load library");
        println!("✓ Library loaded successfully");
        
        // Test version symbol
        type PluginVersionFn = unsafe extern "C" fn() -> *const std::ffi::c_char;
        let version_fn: libloading::Symbol<PluginVersionFn> = library.get(b"exodus_plugin_version")
            .expect("Failed to find version symbol");
        
        let version_ptr = version_fn();
        let version_cstr = CStr::from_ptr(version_ptr);
        let version_str = version_cstr.to_str().unwrap();
        
        println!("✓ Plugin API version: {}", version_str);
        assert_eq!(version_str, "1.0.0");
        
        // Test init symbol exists
        type PluginInitFn = unsafe extern "C" fn() -> *mut std::ffi::c_void;
        let _init_fn: libloading::Symbol<PluginInitFn> = library.get(b"exodus_plugin_init")
            .expect("Failed to find init symbol");
        println!("✓ Init symbol found");
        
        // Test deinit symbol exists
        type PluginDeinitFn = unsafe extern "C" fn(plugin: *mut std::ffi::c_void);
        let _deinit_fn: libloading::Symbol<PluginDeinitFn> = library.get(b"exodus_plugin_deinit")
            .expect("Failed to find deinit symbol");
        println!("✓ Deinit symbol found");
    }
    
    println!("\n✅ All plugin symbol tests passed!");
}
