//! Extension API Improvements for Exodus Browser
//! 
//! This module provides enhanced extension API capabilities similar to Chrome.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// Extension API capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExtensionCapability {
    Tabs,
    Bookmarks,
    History,
    Cookies,
    Storage,
    Notifications,
    WebRequest,
    WebNavigation,
    Runtime,
    ActiveTab,
    Background,
    DevTools,
}

impl ExtensionCapability {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tabs" => ExtensionCapability::Tabs,
            "bookmarks" => ExtensionCapability::Bookmarks,
            "history" => ExtensionCapability::History,
            "cookies" => ExtensionCapability::Cookies,
            "storage" => ExtensionCapability::Storage,
            "notifications" => ExtensionCapability::Notifications,
            "webrequest" => ExtensionCapability::WebRequest,
            "webnavigation" => ExtensionCapability::WebNavigation,
            "runtime" => ExtensionCapability::Runtime,
            "activetab" => ExtensionCapability::ActiveTab,
            "background" => ExtensionCapability::Background,
            "devtools" => ExtensionCapability::DevTools,
            _ => ExtensionCapability::Storage,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            ExtensionCapability::Tabs => "tabs",
            ExtensionCapability::Bookmarks => "bookmarks",
            ExtensionCapability::History => "history",
            ExtensionCapability::Cookies => "cookies",
            ExtensionCapability::Storage => "storage",
            ExtensionCapability::Notifications => "notifications",
            ExtensionCapability::WebRequest => "webRequest",
            ExtensionCapability::WebNavigation => "webNavigation",
            ExtensionCapability::Runtime => "runtime",
            ExtensionCapability::ActiveTab => "activeTab",
            ExtensionCapability::Background => "background",
            ExtensionCapability::DevTools => "devtools",
        }
    }
}

/// Extension API method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionApiMethod {
    /// API name
    pub api_name: String,
    /// Method name
    pub method_name: String,
    /// Description
    pub description: String,
    /// Parameters
    pub parameters: Vec<String>,
    /// Return type
    pub return_type: String,
}

/// Extension API registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionApiRegistration {
    /// Extension ID
    pub extension_id: String,
    /// Capabilities
    pub capabilities: Vec<ExtensionCapability>,
    /// Registered methods
    pub methods: Vec<ExtensionApiMethod>,
    /// Registration timestamp
    pub registered_at: u64,
}

impl ExtensionApiRegistration {
    pub fn new(extension_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            extension_id,
            capabilities: Vec::new(),
            methods: Vec::new(),
            registered_at: now,
        }
    }
    
    pub fn add_capability(&mut self, capability: ExtensionCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }
    
    pub fn add_method(&mut self, method: ExtensionApiMethod) {
        self.methods.push(method);
    }
}

/// Extension API manager
pub struct ExtensionApiManager {
    registrations: Arc<Mutex<HashMap<String, ExtensionApiRegistration>>>,
    available_apis: Arc<Mutex<HashMap<ExtensionCapability, Vec<ExtensionApiMethod>>>>,
    storage_path: PathBuf,
}

impl ExtensionApiManager {
    /// Create a new extension API manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            registrations: Arc::new(Mutex::new(HashMap::new())),
            available_apis: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.initialize_available_apis()?;
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Initialize available APIs
    fn initialize_available_apis(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut apis = self.available_apis.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Tabs API
        apis.insert(ExtensionCapability::Tabs, vec![
            ExtensionApiMethod {
                api_name: "tabs".to_string(),
                method_name: "create".to_string(),
                description: "Create a new tab".to_string(),
                parameters: vec!["url".to_string(), "active".to_string()],
                return_type: "Tab".to_string(),
            },
            ExtensionApiMethod {
                api_name: "tabs".to_string(),
                method_name: "query".to_string(),
                description: "Query tabs".to_string(),
                parameters: vec!["queryInfo".to_string()],
                return_type: "Tab[]".to_string(),
            },
            ExtensionApiMethod {
                api_name: "tabs".to_string(),
                method_name: "update".to_string(),
                description: "Update tab properties".to_string(),
                parameters: vec!["tabId".to_string(), "updateProperties".to_string()],
                return_type: "Tab".to_string(),
            },
            ExtensionApiMethod {
                api_name: "tabs".to_string(),
                method_name: "remove".to_string(),
                description: "Close a tab".to_string(),
                parameters: vec!["tabId".to_string()],
                return_type: "void".to_string(),
            },
        ]);
        
        // Bookmarks API
        apis.insert(ExtensionCapability::Bookmarks, vec![
            ExtensionApiMethod {
                api_name: "bookmarks".to_string(),
                method_name: "create".to_string(),
                description: "Create a bookmark".to_string(),
                parameters: vec!["bookmark".to_string()],
                return_type: "BookmarkTreeNode".to_string(),
            },
            ExtensionApiMethod {
                api_name: "bookmarks".to_string(),
                method_name: "get".to_string(),
                description: "Get bookmarks".to_string(),
                parameters: vec!["idOrIdList".to_string()],
                return_type: "BookmarkTreeNode[]".to_string(),
            },
            ExtensionApiMethod {
                api_name: "bookmarks".to_string(),
                method_name: "remove".to_string(),
                description: "Remove a bookmark".to_string(),
                parameters: vec!["id".to_string()],
                return_type: "void".to_string(),
            },
        ]);
        
        // Storage API
        apis.insert(ExtensionCapability::Storage, vec![
            ExtensionApiMethod {
                api_name: "storage".to_string(),
                method_name: "local.get".to_string(),
                description: "Get items from local storage".to_string(),
                parameters: vec!["keys".to_string()],
                return_type: "Object".to_string(),
            },
            ExtensionApiMethod {
                api_name: "storage".to_string(),
                method_name: "local.set".to_string(),
                description: "Set items in local storage".to_string(),
                parameters: vec!["items".to_string()],
                return_type: "void".to_string(),
            },
            ExtensionApiMethod {
                api_name: "storage".to_string(),
                method_name: "local.remove".to_string(),
                description: "Remove items from local storage".to_string(),
                parameters: vec!["keys".to_string()],
                return_type: "void".to_string(),
            },
        ]);
        
        // Notifications API
        apis.insert(ExtensionCapability::Notifications, vec![
            ExtensionApiMethod {
                api_name: "notifications".to_string(),
                method_name: "create".to_string(),
                description: "Create a notification".to_string(),
                parameters: vec!["notificationId".to_string(), "options".to_string()],
                return_type: "void".to_string(),
            },
            ExtensionApiMethod {
                api_name: "notifications".to_string(),
                method_name: "clear".to_string(),
                description: "Clear a notification".to_string(),
                parameters: vec!["notificationId".to_string()],
                return_type: "boolean".to_string(),
            },
        ]);
        
        // Runtime API
        apis.insert(ExtensionCapability::Runtime, vec![
            ExtensionApiMethod {
                api_name: "runtime".to_string(),
                method_name: "getManifest".to_string(),
                description: "Get extension manifest".to_string(),
                parameters: vec![],
                return_type: "Manifest".to_string(),
            },
            ExtensionApiMethod {
                api_name: "runtime".to_string(),
                method_name: "sendMessage".to_string(),
                description: "Send message to extension".to_string(),
                parameters: vec!["message".to_string()],
                return_type: "Promise".to_string(),
            },
            ExtensionApiMethod {
                api_name: "runtime".to_string(),
                method_name: "openOptionsPage".to_string(),
                description: "Open extension options page".to_string(),
                parameters: vec![],
                return_type: "void".to_string(),
            },
        ]);
        
        Ok(())
    }
    
    /// Register extension API
    pub fn register_extension(&self, extension_id: String, capabilities: Vec<ExtensionCapability>) -> Result<(), Box<dyn std::error::Error>> {
        let mut registration = ExtensionApiRegistration::new(extension_id.clone());
        
        let apis = self.available_apis.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        for capability in &capabilities {
            registration.add_capability(capability.clone());
            if let Some(methods) = apis.get(capability) {
                for method in methods {
                    registration.add_method(method.clone());
                }
            }
        }
        
        let mut registrations = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        registrations.insert(extension_id, registration);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Unregister extension API
    pub fn unregister_extension(&self, extension_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut registrations = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        registrations.remove(extension_id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get extension registration
    pub fn get_registration(&self, extension_id: &str) -> Option<ExtensionApiRegistration> {
        let registrations = self.registrations.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        registrations.get(extension_id).cloned()
    }
    
    /// Get all registrations
    pub fn get_all_registrations(&self) -> Vec<ExtensionApiRegistration> {
        let registrations = self.registrations.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        registrations.values().cloned().collect()
    }
    
    /// Get available APIs for a capability
    pub fn get_available_apis(&self, capability: ExtensionCapability) -> Vec<ExtensionApiMethod> {
        let apis = self.available_apis.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        apis.get(&capability).cloned().unwrap_or_default()
    }
    
    /// Check if extension has capability
    pub fn has_capability(&self, extension_id: &str, capability: ExtensionCapability) -> bool {
        let registrations = self.registrations.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(registration) = registrations.get(extension_id) {
            registration.capabilities.contains(&capability)
        } else {
            false
        }
    }
    
    /// Get all available capabilities
    pub fn get_all_capabilities(&self) -> Vec<ExtensionCapability> {
        let apis = self.available_apis.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        apis.keys().cloned().collect()
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("extension_api_registrations.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let registrations: HashMap<String, ExtensionApiRegistration> = serde_json::from_str(&content)?;
            *self.registrations.lock().unwrap() = registrations;
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("extension_api_registrations.json");
        
        let registrations = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*registrations)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Register extension API
#[tauri::command]
pub fn register_extension_api(
    extension_id: String,
    capabilities: Vec<String>,
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<(), String> {
    let caps: Vec<ExtensionCapability> = capabilities.iter()
        .map(|c| ExtensionCapability::from_str(c))
        .collect();
    
    manager.register_extension(extension_id, caps)
        .map_err(|e| format!("Failed to register extension API: {}", e))
}

/// Unregister extension API
#[tauri::command]
pub fn unregister_extension_api(
    extension_id: String,
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<(), String> {
    manager.unregister_extension(&extension_id)
        .map_err(|e| format!("Failed to unregister extension API: {}", e))
}

/// Get extension registration
#[tauri::command]
pub fn get_extension_registration(
    extension_id: String,
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<Option<ExtensionApiRegistration>, String> {
    Ok(manager.get_registration(&extension_id))
}

/// Get all extension registrations
#[tauri::command]
pub fn get_all_extension_registrations(
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<Vec<ExtensionApiRegistration>, String> {
    Ok(manager.get_all_registrations())
}

/// Get available APIs for a capability
#[tauri::command]
pub fn get_available_apis(
    capability: String,
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<Vec<ExtensionApiMethod>, String> {
    let cap = ExtensionCapability::from_str(&capability);
    Ok(manager.get_available_apis(cap))
}

/// Check if extension has capability
#[tauri::command]
pub fn check_extension_capability(
    extension_id: String,
    capability: String,
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<bool, String> {
    let cap = ExtensionCapability::from_str(&capability);
    Ok(manager.has_capability(&extension_id, cap))
}

/// Get all available capabilities
#[tauri::command]
pub fn get_all_capabilities(
    manager: State<'_, Arc<ExtensionApiManager>>,
) -> Result<Vec<String>, String> {
    let caps = manager.get_all_capabilities();
    Ok(caps.iter().map(|c| c.as_str().to_string()).collect())
}
