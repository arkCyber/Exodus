//! Exodus Browser — Native Rust plugin system.
//!
//! Supports loading Rust plugins as dynamic libraries (.so/.dll/.dylib).

use libloading::{Library, Symbol};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::error::PluginError;

/// Native plugin trait that all plugins must implement.
#[allow(dead_code)]
pub trait NativePlugin: Send + Sync {
    /// Plugin name.
    fn name(&self) -> &str;
    
    /// Plugin version.
    fn version(&self) -> &str;
    
    /// Called when plugin is loaded.
    fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError>;
    
    /// Called when plugin is unloaded.
    fn on_unload(&mut self) -> Result<(), PluginError>;
    
    /// Handle message from frontend or other plugins.
    fn handle_message(&mut self, message: Value) -> Result<Value, PluginError>;
}

/// Plugin context provided to plugins on load.
#[allow(dead_code)]
pub struct PluginContext {
    pub browser_api: BrowserApi,
    pub storage: PluginStorage,
    pub permissions: PermissionSet,
}

/// Browser API exposed to native plugins.
#[allow(dead_code)]
pub struct BrowserApi {
    #[allow(dead_code)]
    pub create_tab: fn(url: &str) -> Result<String, PluginError>,
    #[allow(dead_code)]
    pub navigate: fn(tab_id: &str, url: &str) -> Result<(), PluginError>,
    #[allow(dead_code)]
    pub execute_script: fn(tab_id: &str, script: &str) -> Result<String, PluginError>,
    #[allow(dead_code)]
    pub get_tabs: fn() -> Result<Vec<TabInfo>, PluginError>,
}

/// Plugin storage interface.
#[allow(dead_code)]
pub struct PluginStorage {
    pub get: fn(key: &str) -> Result<Option<String>, PluginError>,
    pub set: fn(key: &str, value: &str) -> Result<(), PluginError>,
    pub remove: fn(key: &str) -> Result<(), PluginError>,
}

/// Permission set for a plugin.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PermissionSet {
    pub system_access: bool,
    pub network_access: bool,
    pub file_access: bool,
    pub browser_control: bool,
}

/// Tab information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct TabInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub active: bool,
}
#[allow(dead_code)]
pub struct LoadedNativePlugin {
#[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub permissions: PermissionSet,
    #[allow(dead_code)]
    library: Library,
    instance: Arc<Mutex<dyn NativePlugin>>,
}

impl LoadedNativePlugin {
    /// Create a new loaded plugin from a dynamic library.
    pub fn from_library(
        path: PathBuf,
        library: Library,
        instance: Arc<Mutex<dyn NativePlugin>>,
    ) -> Result<Self, PluginError> {
        let name = instance.lock().map_err(|e| PluginError::LoadError(e.to_string()))?.name().to_string();
        let version = instance.lock().map_err(|e| PluginError::LoadError(e.to_string()))?.version().to_string();
        let id = format!("native_{}", name.to_lowercase().replace(' ', "_"));
        
        Ok(Self {
            id,
            name,
            version,
            path,
            enabled: true,
            permissions: PermissionSet::default(),
            library,
            instance,
        })
    }
    
    /// Enable the plugin.
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable the plugin.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Handle a message to the plugin.
    pub fn handle_message(&self, message: Value) -> Result<Value, PluginError> {
        if !self.enabled {
            return Err(PluginError::Disabled);
        }
        let mut instance = self.instance.lock().map_err(|e| PluginError::LoadError(e.to_string()))?;
        instance.handle_message(message)
    }
}

/// Native plugin manager.
pub struct NativePluginManager {
    plugins: Vec<LoadedNativePlugin>,
}

impl NativePluginManager {
    /// Create a new native plugin manager.
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }
    
    /// Load a native plugin from a dynamic library path.
    pub fn load_plugin(&mut self, path: PathBuf) -> Result<String, PluginError> {
        unsafe {
            let library = Library::new(&path)
                .map_err(|e| PluginError::LoadError(format!("Failed to load library: {}", e)))?;
            
            // Try to load the plugin creation function
            let create_plugin: Symbol<fn() -> *mut dyn NativePlugin> = library
                .get(b"create_plugin")
                .map_err(|e| PluginError::LoadError(format!("create_plugin symbol not found: {}", e)))?;
            
            let plugin_ptr = create_plugin();
            if plugin_ptr.is_null() {
                return Err(PluginError::LoadError("create_plugin returned null".to_string()));
            }
            
            // Convert raw pointer to Box, then to Arc<Mutex<dyn NativePlugin>>
            // Note: This is a simplified approach - production code needs proper memory management
            let plugin = Arc::new(Mutex::new(PluginWrapper::from_raw(plugin_ptr)));
            
            let loaded = LoadedNativePlugin::from_library(path.clone(), library, plugin)?;
            let id = loaded.id.clone();
            self.plugins.push(loaded);
            
            Ok(id)
        }
    }
    
    /// Unload a plugin by ID.
    pub fn unload_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        let index = self.plugins
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        
        let plugin = self.plugins.remove(index);
        // Plugin will be unloaded when library goes out of scope
        tracing::info!("Unloaded native plugin: {}", plugin.name);
        
        Ok(())
    }
    
    /// List all loaded plugins.
    pub fn list_plugins(&self) -> Vec<NativePluginInfo> {
        self.plugins
            .iter()
            .map(|p| NativePluginInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                version: p.version.clone(),
                enabled: p.enabled,
                permissions: p.permissions.clone(),
            })
            .collect()
    }
    
    /// Enable a plugin.
    pub fn enable_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self.plugins
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.enable();
        Ok(())
    }
    
    /// Disable a plugin.
    pub fn disable_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self.plugins
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.disable();
        Ok(())
    }
    
    /// Send a message to a plugin.
    pub fn send_message(&self, id: &str, message: Value) -> Result<Value, PluginError> {
        let plugin = self.plugins
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.handle_message(message)
    }
}

impl Default for NativePluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a native plugin.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NativePluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub permissions: PermissionSet,
}

/// Wrapper to safely handle raw pointers from dynamic libraries.
struct PluginWrapper {
    inner: *mut dyn NativePlugin,
}

unsafe impl Send for PluginWrapper {}
unsafe impl Sync for PluginWrapper {}

impl PluginWrapper {
    unsafe fn from_raw(ptr: *mut dyn NativePlugin) -> Self {
        Self { inner: ptr }
    }
}

impl NativePlugin for PluginWrapper {
    fn name(&self) -> &str {
        unsafe { (*self.inner).name() }
    }
    
    fn version(&self) -> &str {
        unsafe { (*self.inner).version() }
    }
    
    fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError> {
        unsafe { (*self.inner).on_load(context) }
    }
    
    fn on_unload(&mut self) -> Result<(), PluginError> {
        unsafe { (*self.inner).on_unload() }
    }
    
    fn handle_message(&mut self, message: Value) -> Result<Value, PluginError> {
        unsafe { (*self.inner).handle_message(message) }
    }
}

impl Drop for PluginWrapper {
    fn drop(&mut self) {
        unsafe {
            // Call on_unload before dropping
            let _ = (*self.inner).on_unload();
            // Note: We don't free the pointer here as it was allocated by the plugin
            // In production, this would need proper memory management
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_default() {
        let perms = PermissionSet::default();
        assert!(!perms.system_access);
        assert!(!perms.network_access);
        assert!(!perms.file_access);
        assert!(!perms.browser_control);
    }

    #[test]
    fn test_native_plugin_manager_new() {
        let manager = NativePluginManager::new();
        assert!(manager.list_plugins().is_empty());
    }
}
