//! Network Request Interception for Exodus Browser
//! 
//! This module provides network request interception and modification capabilities.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use std::time::Duration;
/// Request method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl RequestMethod {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => RequestMethod::GET,
            "POST" => RequestMethod::POST,
            "PUT" => RequestMethod::PUT,
            "DELETE" => RequestMethod::DELETE,
            "PATCH" => RequestMethod::PATCH,
            "HEAD" => RequestMethod::HEAD,
            "OPTIONS" => RequestMethod::OPTIONS,
            "CONNECT" => RequestMethod::CONNECT,
            "TRACE" => RequestMethod::TRACE,
            _ => RequestMethod::GET,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            RequestMethod::GET => "GET",
            RequestMethod::POST => "POST",
            RequestMethod::PUT => "PUT",
            RequestMethod::DELETE => "DELETE",
            RequestMethod::PATCH => "PATCH",
            RequestMethod::HEAD => "HEAD",
            RequestMethod::OPTIONS => "OPTIONS",
            RequestMethod::CONNECT => "CONNECT",
            RequestMethod::TRACE => "TRACE",
        }
    }
}

/// Request type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    MainFrame,
    SubFrame,
    Stylesheet,
    Script,
    Image,
    Font,
    Media,
    WebSocket,
    XHR,
    Fetch,
    Other,
}

impl ResourceType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "main_frame" => ResourceType::MainFrame,
            "sub_frame" => ResourceType::SubFrame,
            "stylesheet" => ResourceType::Stylesheet,
            "script" => ResourceType::Script,
            "image" => ResourceType::Image,
            "font" => ResourceType::Font,
            "media" => ResourceType::Media,
            "websocket" => ResourceType::WebSocket,
            "xhr" => ResourceType::XHR,
            "fetch" => ResourceType::Fetch,
            _ => ResourceType::Other,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::MainFrame => "main_frame",
            ResourceType::SubFrame => "sub_frame",
            ResourceType::Stylesheet => "stylesheet",
            ResourceType::Script => "script",
            ResourceType::Image => "image",
            ResourceType::Font => "font",
            ResourceType::Media => "media",
            ResourceType::WebSocket => "websocket",
            ResourceType::XHR => "xhr",
            ResourceType::Fetch => "fetch",
            ResourceType::Other => "other",
        }
    }
}

/// Intercepted request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptedRequest {
    /// Request ID
    pub id: String,
    /// Request URL
    pub url: String,
    /// Request method
    pub method: RequestMethod,
    /// Resource type
    pub resource_type: ResourceType,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (for POST/PUT)
    pub body: Option<String>,
    /// Tab ID
    pub tab_id: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl InterceptedRequest {
    #[allow(dead_code)]
    pub fn new(url: String, method: RequestMethod, resource_type: ResourceType) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            method,
            resource_type,
            headers: HashMap::new(),
            body: None,
            tab_id: None,
            timestamp: now,
        }
    }
}

/// Intercepted response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptedResponse {
    /// Request ID
    pub request_id: String,
    /// Response status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl InterceptedResponse {
    #[allow(dead_code)]
    pub fn new(request_id: String, status_code: u16) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            request_id,
            status_code,
            headers: HashMap::new(),
            body: None,
            timestamp: now,
        }
    }
}

/// Interception rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptionRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// URL pattern (supports wildcards)
    pub url_pattern: String,
    /// Resource types to match
    pub resource_types: Vec<ResourceType>,
    /// Action to take
    pub action: InterceptionAction,
    /// Is enabled
    pub enabled: bool,
    /// Priority (higher = evaluated first)
    pub priority: u32,
    /// Created timestamp
    pub created_at: u64,
}

/// Interception action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterceptionAction {
    Block,
    Allow,
    Redirect(String),
    ModifyHeaders(HashMap<String, String>),
    ModifyBody(String),
}

impl InterceptionRule {
    #[allow(dead_code)]
    pub fn new(name: String, url_pattern: String, action: InterceptionAction) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            url_pattern,
            resource_types: vec![],
            action,
            enabled: true,
            priority: 0,
            created_at: now,
        }
    }
    
    /// Check if this rule matches a request
    pub fn matches(&self, request: &InterceptedRequest) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check URL pattern
        if !self.matches_url_pattern(&request.url) {
            return false;
        }
        
        // Check resource types
        if !self.resource_types.is_empty() && !self.resource_types.contains(&request.resource_type) {
            return false;
        }
        
        true
    }
    
    fn matches_url_pattern(&self, url: &str) -> bool {
        let pattern = &self.url_pattern;
        
        // Simple wildcard matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return url.starts_with(prefix) && url.ends_with(suffix);
            }
        }
        
        url == pattern || url.starts_with(pattern)
    }
}

/// Network interception settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterceptionSettings {
    /// Enable interception
    pub enable_interception: bool,
    /// Log all requests
    pub log_all_requests: bool,
    /// Log all responses
    pub log_all_responses: bool,
    /// Max log entries
    pub max_log_entries: usize,
    /// Block third-party requests
    pub block_third_party: bool,
    /// Block mixed content
    pub block_mixed_content: bool,
}

impl Default for NetworkInterceptionSettings {
    fn default() -> Self {
        Self {
            enable_interception: true,
            log_all_requests: false,
            log_all_responses: false,
            max_log_entries: 1000,
            block_third_party: false,
            block_mixed_content: false,
        }
    }
}

/// Network interceptor
pub struct NetworkInterceptor {
    rules: Arc<Mutex<Vec<InterceptionRule>>>,
    settings: Arc<Mutex<NetworkInterceptionSettings>>,
    request_log: Arc<Mutex<Vec<InterceptedRequest>>>,
    response_log: Arc<Mutex<Vec<InterceptedResponse>>>,
    blocked_domains: Arc<Mutex<HashSet<String>>>,
    storage_path: PathBuf,
}

impl NetworkInterceptor {
    /// Create a new network interceptor
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let interceptor = Self {
            rules: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(NetworkInterceptionSettings::default())),
            request_log: Arc::new(Mutex::new(Vec::new())),
            response_log: Arc::new(Mutex::new(Vec::new())),
            blocked_domains: Arc::new(Mutex::new(HashSet::new())),
            storage_path,
        };
        
        interceptor.load_from_disk()?;
        Ok(interceptor)
    }
    
    /// Add an interception rule
    pub fn add_rule(&self, rule: InterceptionRule) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        rules.push(rule);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove an interception rule
    pub fn remove_rule(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        rules.retain(|r| r.id != id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get all rules
    pub fn get_rules(&self) -> Vec<InterceptionRule> {
        let rules = self.rules.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        rules.clone()
    }
    
    /// Update a rule
    pub fn update_rule(&self, id: String, rule: InterceptionRule) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(existing) = rules.iter_mut().find(|r| r.id == id) {
            *existing = rule;
            rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Intercept a request
    pub fn intercept_request(&self, request: InterceptedRequest) -> Option<InterceptionAction> {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.enable_interception {
            return None;
        }
        
        // Log request if enabled
        if settings.log_all_requests {
            self.log_request(request.clone());
        }
        
        // Check blocked domains
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(domain) = Self::extract_domain(&request.url) {
            if blocked.contains(&domain) || blocked.iter().any(|d| domain.ends_with(&format!(".{}", d))) {
                return Some(InterceptionAction::Block);
            }
        }
        
        // Check rules
        let rules = self.rules.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        for rule in rules.iter() {
            if rule.matches(&request) {
                return Some(rule.action.clone());
            }
        }
        
        None
    }
    
    /// Log a request
    fn log_request(&self, request: InterceptedRequest) {
        let mut log = self.request_log.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        log.push(request);
        
        // Trim log if needed
        if log.len() > settings.max_log_entries {
            log.remove(0);
        }
    }
    
    /// Log a response
    pub fn log_response(&self, response: InterceptedResponse) {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if !settings.log_all_responses {
            return;
        }
        
        let mut log = self.response_log.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        log.push(response);
        
        // Trim log if needed
        if log.len() > settings.max_log_entries {
            log.remove(0);
        }
    }
    
    /// Get request log
    pub fn get_request_log(&self) -> Vec<InterceptedRequest> {
        let log = self.request_log.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        log.clone()
    }
    
    /// Get response log
    pub fn get_response_log(&self) -> Vec<InterceptedResponse> {
        let log = self.response_log.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        log.clone()
    }
    
    /// Clear logs
    pub fn clear_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut request_log = self.request_log.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let mut response_log = self.response_log.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        request_log.clear();
        response_log.clear();
        
        Ok(())
    }
    
    /// Block a domain
    pub fn block_domain(&self, domain: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_domains.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        blocked.insert(domain);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Unblock a domain
    pub fn unblock_domain(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_domains.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        blocked.remove(domain);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get blocked domains
    pub fn get_blocked_domains(&self) -> Vec<String> {
        let blocked = self.blocked_domains.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        blocked.iter().cloned().collect()
    }
    
    /// Get settings
    pub fn get_settings(&self) -> NetworkInterceptionSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update settings
    pub fn update_settings(&self, settings: NetworkInterceptionSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            parsed.host_str().map(|s| s.to_string())
        } else {
            None
        }
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules_path = self.storage_path.join("interception_rules.json");
        let settings_path = self.storage_path.join("interception_settings.json");
        let blocked_path = self.storage_path.join("blocked_domains.json");
        
        if rules_path.exists() {
            let content = std::fs::read_to_string(&rules_path)?;
            let loaded: Vec<InterceptionRule> = serde_json::from_str(&content)?;
            if let Ok(mut guard) = self.rules.lock() {
                *guard = loaded;
            }
        }
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let loaded: NetworkInterceptionSettings = serde_json::from_str(&content)?;
            if let Ok(mut guard) = self.settings.lock() {
                *guard = loaded;
            }
        }
        
        if blocked_path.exists() {
            let content = std::fs::read_to_string(&blocked_path)?;
            let loaded: HashSet<String> = serde_json::from_str(&content)?;
            if let Ok(mut guard) = self.blocked_domains.lock() {
                *guard = loaded;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules_path = self.storage_path.join("interception_rules.json");
        let settings_path = self.storage_path.join("interception_settings.json");
        let blocked_path = self.storage_path.join("blocked_domains.json");
        
        let rules = self.rules.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let blocked = self.blocked_domains.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let rules_content = serde_json::to_string_pretty(&*rules)?;
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let blocked_content = serde_json::to_string_pretty(&*blocked)?;
        
        std::fs::write(&rules_path, rules_content)?;
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&blocked_path, blocked_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Add interception rule
#[tauri::command]
pub fn add_interception_rule(
    rule: InterceptionRule,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.add_rule(rule)
        .map_err(|e| format!("Failed to add rule: {}", e))
}

/// Remove interception rule
#[tauri::command]
pub fn remove_interception_rule(
    id: String,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.remove_rule(&id)
        .map_err(|e| format!("Failed to remove rule: {}", e))
}

/// Get all interception rules
#[tauri::command]
pub fn get_interception_rules(
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<Vec<InterceptionRule>, String> {
    Ok(interceptor.get_rules())
}

/// Update interception rule
#[tauri::command]
pub fn update_interception_rule(
    id: String,
    rule: InterceptionRule,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.update_rule(id, rule)
        .map_err(|e| format!("Failed to update rule: {}", e))
}

/// Intercept a request
#[tauri::command]
pub fn intercept_request(
    request: InterceptedRequest,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<Option<String>, String> {
    let action = interceptor.intercept_request(request);
    
    // Convert action to string for serialization
    let action_str = action.map(|a| match a {
        InterceptionAction::Block => "block".to_string(),
        InterceptionAction::Allow => "allow".to_string(),
        InterceptionAction::Redirect(url) => format!("redirect:{}", url),
        InterceptionAction::ModifyHeaders(_) => "modify_headers".to_string(),
        InterceptionAction::ModifyBody(_) => "modify_body".to_string(),
    });
    
    Ok(action_str)
}

/// Log a response
#[tauri::command]
pub fn log_response(
    response: InterceptedResponse,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.log_response(response);
    Ok(())
}

/// Get request log
#[tauri::command]
pub fn get_request_log(
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<Vec<InterceptedRequest>, String> {
    Ok(interceptor.get_request_log())
}

/// Get response log
#[tauri::command]
pub fn get_response_log(
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<Vec<InterceptedResponse>, String> {
    Ok(interceptor.get_response_log())
}

/// Clear logs
#[tauri::command]
pub fn clear_network_logs(
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.clear_logs()
        .map_err(|e| format!("Failed to clear logs: {}", e))
}

/// Block a domain
#[tauri::command]
pub fn block_network_domain(
    domain: String,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.block_domain(domain)
        .map_err(|e| format!("Failed to block domain: {}", e))
}

/// Unblock a domain
#[tauri::command]
pub fn unblock_network_domain(
    domain: String,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.unblock_domain(&domain)
        .map_err(|e| format!("Failed to unblock domain: {}", e))
}

/// Get blocked domains
#[tauri::command]
pub fn get_blocked_network_domains(
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<Vec<String>, String> {
    Ok(interceptor.get_blocked_domains())
}

/// Get network interception settings
#[tauri::command]
pub fn get_network_interception_settings(
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<NetworkInterceptionSettings, String> {
    Ok(interceptor.get_settings())
}

/// Update network interception settings
#[tauri::command]
pub fn update_network_interception_settings(
    settings: NetworkInterceptionSettings,
    interceptor: State<'_, Arc<NetworkInterceptor>>,
) -> Result<(), String> {
    interceptor.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}
