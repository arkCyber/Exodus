//! Site Isolation Manager for Exodus Browser
//!
//! This module implements aerospace-grade site isolation to ensure that
//! each security origin runs in its own process, preventing cross-site
//! data leaks and containing crashes.
//!
//! Architecture:
//! - SiteInstance: Represents a security origin (scheme + eTLD+1)
//! - SiteProcess: Manages a process for a site instance
//! - SiteIsolationManager: Coordinates site instances and processes
//! - ProcessCrashHandler: Handles process crashes with isolation
//! - ProcessPool: Manages a pool of isolated processes
//! - SiteBoundaryPolicy: Enforces site boundary policies

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};
use url::Url;
use crate::ipc_security::{IpcSecurityManager, IpcMessage, IpcMessageType, IpcPriority, SecurityLevel, SecurityPolicy};

/// Message for site-to-site communication
///
/// This structure represents a message sent from one site to another
/// in the isolated process architecture. Messages are queued and
/// delivered based on security context policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMessage {
    /// The site sending the message
    pub from_site: SiteId,
    /// The site receiving the message
    pub to_site: SiteId,
    /// Type of message (e.g., "postMessage", "shared-worker")
    pub message_type: String,
    /// Message payload (JSON-serializable data)
    pub payload: serde_json::Value,
    /// Unix timestamp when message was created
    pub timestamp: u64,
}

impl SiteMessage {
    pub fn new(from_site: SiteId, to_site: SiteId, message_type: String, payload: serde_json::Value) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            from_site,
            to_site,
            message_type,
            payload,
            timestamp,
        }
    }
}

/// Security context for site communication
///
/// Defines the security policies for a site regarding cross-origin
/// communication and worker capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Whether cross-origin communication is allowed
    pub allow_cross_origin: bool,
    /// Whether postMessage API is allowed
    pub allow_post_message: bool,
    /// Whether shared workers are allowed
    pub allow_shared_workers: bool,
    /// Whether service workers are allowed
    pub allow_service_workers: bool,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            allow_cross_origin: false,
            allow_post_message: true,
            allow_shared_workers: false,
            allow_service_workers: true,
        }
    }
}

/// Site identifier (scheme + eTLD+1)
///
/// Represents a security origin using the effective top-level domain plus one (eTLD+1).
/// This ensures that subdomains of the same domain are considered the same site
/// for isolation purposes (e.g., sub.example.com and example.com share the same SiteId).
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SiteId {
    /// URL scheme (e.g., "https", "http")
    pub scheme: String,
    /// Effective top-level domain plus one (e.g., "example.com")
    pub etld_plus_one: String,
}

impl SiteId {
    /// Create a SiteId from a URL
    pub fn from_url(url: &str) -> Result<Self, String> {
        let parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        
        let scheme = parsed.scheme().to_string();
        let host = parsed.host_str().ok_or("URL has no host")?;
        
        // Extract eTLD+1 (effective top-level domain + 1)
        // For now, use a simple implementation
        // TODO: Use proper Public Suffix List
        let etld_plus_one = extract_etld_plus_one(host);
        
        Ok(SiteId {
            scheme,
            etld_plus_one,
        })
    }
}

/// Extract eTLD+1 from a hostname
/// Uses a simplified Public Suffix List for common TLDs
fn extract_etld_plus_one(host: &str) -> String {
    // Common public suffixes (simplified list)
    const PUBLIC_SUFFIXES: &[&str] = &[
        "com", "org", "net", "edu", "gov", "mil", "io", "co", "uk", "us", "ca",
        "au", "de", "fr", "jp", "cn", "ru", "br", "in", "mx", "es", "it", "nl",
        "se", "pl", "kr", "za", "ch", "at", "be", "dk", "no", "fi", "gr", "pt",
        "cz", "hu", "ro", "il", "hk", "sg", "nz", "ie", "th", "vn", "id", "my",
        "ph", "pk", "bd", "lk", "np", "mm", "kh", "la", "vn", "tw", "mo",
        // Co.uk style two-part TLDs
        "co.uk", "org.uk", "ac.uk", "gov.uk", "edu.uk", "nhs.uk",
        "co.jp", "ac.jp", "go.jp", "ne.jp",
        "com.au", "net.au", "org.au", "edu.au", "gov.au",
        "co.nz", "org.nz", "ac.nz", "gov.nz",
        "com.cn", "net.cn", "org.cn", "edu.cn", "gov.cn",
        "co.in", "net.in", "org.in", "edu.in", "gov.in",
        "com.br", "net.br", "org.br", "edu.br", "gov.br",
        "com.mx", "net.mx", "org.mx", "edu.mx", "gov.mx",
        "com.tw", "net.tw", "org.tw", "edu.tw", "gov.tw",
        "com.hk", "net.hk", "org.hk", "edu.hk", "gov.hk",
        "com.sg", "net.sg", "org.sg", "edu.sg", "gov.sg",
    ];
    
    let parts: Vec<&str> = host.split('.').collect();
    
    if parts.len() == 1 {
        return host.to_string();
    }
    
    // Check for two-part TLDs
    if parts.len() >= 3 {
        let two_part = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        if PUBLIC_SUFFIXES.contains(&two_part.as_str()) {
            return format!("{}.{}", parts[parts.len() - 3], two_part);
        }
    }
    
    // Check for single-part TLDs
    let tld = parts[parts.len() - 1];
    if PUBLIC_SUFFIXES.contains(&tld) && parts.len() >= 2 {
        return format!("{}.{}", parts[parts.len() - 2], tld);
    }
    
    // Default to last two parts
    if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        host.to_string()
    }
}

/// Site instance state
///
/// Represents an active instance of a site with associated webviews.
/// Tracks usage statistics and process assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInstance {
    /// The site identifier
    pub site_id: SiteId,
    /// Assigned process ID (if any)
    pub process_id: Option<String>,
    /// Unix timestamp when instance was created
    pub created_at: u64,
    /// Unix timestamp when instance was last used
    pub last_used: u64,
    /// Number of active webviews for this site
    pub webview_count: usize,
}

impl SiteInstance {
    pub fn new(site_id: SiteId) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            site_id,
            process_id: None,
            created_at: now,
            last_used: now,
            webview_count: 0,
        }
    }
    
    pub fn touch(&mut self) {
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

/// Process information
///
/// Tracks the state and metrics of a site process including
/// crash history and resource usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Unique process identifier
    pub process_id: String,
    /// Site this process belongs to
    pub site_id: SiteId,
    /// OS process ID (if available)
    pub pid: Option<u32>,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage (0.0 to 1.0)
    pub cpu_usage: f32,
    /// Unix timestamp when process was created
    pub created_at: u64,
    /// Number of times this process has crashed
    pub crash_count: u32,
    /// Unix timestamp of last crash (if any)
    pub last_crash: Option<u64>,
    /// Whether process is currently crashed
    pub is_crashed: bool,
}

impl ProcessInfo {
    pub fn new(process_id: String, site_id: SiteId) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            process_id,
            site_id,
            pid: None,
            memory_usage: 0,
            cpu_usage: 0.0,
            created_at: now,
            crash_count: 0,
            last_crash: None,
            is_crashed: false,
        }
    }
    
    pub fn mark_crashed(&mut self) {
        self.is_crashed = true;
        self.crash_count += 1;
        self.last_crash = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
    }
    
    pub fn mark_recovered(&mut self) {
        self.is_crashed = false;
    }
}

/// Site isolation policy
///
/// Configuration for site isolation behavior including process limits,
/// resource constraints, and strictness settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationPolicy {
    /// Whether site isolation is enabled
    pub enabled: bool,
    /// If true, every subdomain gets its own process (strict mode)
    pub strict_mode: bool,
    /// Maximum number of processes allowed
    pub max_processes: usize,
    /// Memory limit per process in bytes
    pub process_memory_limit: u64,
    /// CPU usage limit per process (0.0 to 1.0)
    pub process_cpu_limit: f32,
    /// Idle process timeout in seconds before eviction
    pub idle_process_timeout: u64,
    /// Whether to enable process pooling
    pub enable_process_pooling: bool,
    /// Minimum pool size
    pub min_pool_size: usize,
    /// Maximum pool size
    pub max_pool_size: usize,
    /// Whether to enable Spectre/Meltdown mitigations
    pub enable_spectre_mitigations: bool,
    /// Whether to enforce same-origin policy strictly
    pub strict_same_origin: bool,
}

impl Default for IsolationPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            strict_mode: false,
            max_processes: 50,
            process_memory_limit: 2 * 1024 * 1024 * 1024, // 2GB
            process_cpu_limit: 0.8, // 80%
            idle_process_timeout: 300, // 5 minutes
            enable_process_pooling: true,
            min_pool_size: 3,
            max_pool_size: 10,
            enable_spectre_mitigations: true,
            strict_same_origin: true,
        }
    }
}

/// Process pool entry
#[derive(Debug, Clone)]
struct ProcessPoolEntry {
    process_id: String,
    site_id: Option<SiteId>,
    is_available: bool,
    created_at: u64,
    last_used: u64,
}

/// Process pool manager
///
/// Manages a pool of pre-allocated processes for efficient site isolation.
/// Reduces process creation overhead by reusing idle processes.
struct ProcessPool {
    pool: Arc<Mutex<Vec<ProcessPoolEntry>>>,
    min_size: usize,
    max_size: usize,
}

impl ProcessPool {
    fn new(min_size: usize, max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::new())),
            min_size,
            max_size,
        }
    }

    /// Acquire a process from the pool or create a new one
    fn acquire(&self, site_id: Option<SiteId>) -> Result<String, String> {
        let mut pool = self.pool.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        // Try to find an available process
        if let Some(entry) = pool.iter_mut().find(|e| e.is_available) {
            entry.is_available = false;
            entry.site_id = site_id.clone();
            entry.last_used = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Ok(entry.process_id.clone());
        }

        // Check if we can create a new process
        if pool.len() < self.max_size {
            let process_id = format!("pool-{}-{}", 
                pool.len(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            
            pool.push(ProcessPoolEntry {
                process_id: process_id.clone(),
                site_id,
                is_available: false,
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                last_used: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });
            
            Ok(process_id)
        } else {
            Err("Process pool exhausted".to_string())
        }
    }

    /// Release a process back to the pool
    fn release(&self, process_id: &str) -> Result<(), String> {
        let mut pool = self.pool.lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(entry) = pool.iter_mut().find(|e| e.process_id == process_id) {
            entry.is_available = true;
            entry.site_id = None;
            entry.last_used = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }

        Ok(())
    }

    /// Get pool statistics
    fn get_stats(&self) -> (usize, usize, usize) {
        let pool = self.pool.lock().unwrap();
        let total = pool.len();
        let available = pool.iter().filter(|e| e.is_available).count();
        let in_use = total - available;
        (total, available, in_use)
    }
}

/// Site boundary policy
///
/// Enforces strict site boundary policies to prevent cross-site data leaks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteBoundaryPolicy {
    /// Whether to enforce strict same-origin policy
    pub strict_same_origin: bool,
    /// Blocked cross-origin navigations
    pub blocked_navigations: HashSet<String>,
    /// Allowed cross-origin navigations (whitelist)
    pub allowed_navigations: HashSet<String>,
    /// Whether to block subdomain access
    pub block_subdomain_access: bool,
    /// Whether to block third-party cookies
    pub block_third_party_cookies: bool,
    /// Whether to enable Spectre mitigations
    pub enable_spectre_mitigations: bool,
}

impl Default for SiteBoundaryPolicy {
    fn default() -> Self {
        Self {
            strict_same_origin: true,
            blocked_navigations: HashSet::new(),
            allowed_navigations: HashSet::new(),
            block_subdomain_access: false,
            block_third_party_cookies: true,
            enable_spectre_mitigations: true,
        }
    }
}

impl SiteBoundaryPolicy {
    /// Check if navigation is allowed between sites
    pub fn is_navigation_allowed(&self, from: &SiteId, to: &SiteId) -> bool {
        // Same-origin is always allowed
        if from == to {
            return true;
        }

        // Check strict same-origin policy
        if self.strict_same_origin {
            return false;
        }

        // Check whitelist
        let nav_key = format!("{} -> {}", from.etld_plus_one, to.etld_plus_one);
        if self.allowed_navigations.contains(&nav_key) {
            return true;
        }

        // Check blacklist
        if self.blocked_navigations.contains(&nav_key) {
            return false;
        }

        // Check subdomain access
        if self.block_subdomain_access {
            if from.etld_plus_one == to.etld_plus_one {
                return false;
            }
        }

        // Default: allow if not explicitly blocked
        true
    }

    /// Check if cookies should be blocked for third-party context
    pub fn should_block_third_party_cookies(&self, first_party: &SiteId, third_party: &SiteId) -> bool {
        if !self.block_third_party_cookies {
            return false;
        }
        first_party != third_party
    }
}

/// Site Isolation Manager
///
/// Central coordinator for site isolation, managing site instances,
/// process assignment, security contexts, and inter-site communication.
/// Thread-safe via Arc<Mutex<>> wrappers.
pub struct SiteIsolationManager {
    /// Isolation policy configuration
    policy: Arc<Mutex<IsolationPolicy>>,
    /// Active site instances keyed by SiteId
    site_instances: Arc<Mutex<HashMap<SiteId, SiteInstance>>>,
    /// Process information keyed by process ID
    pub(crate) processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
    /// URL to SiteId mapping for quick lookups
    url_to_site: Arc<Mutex<HashMap<String, SiteId>>>,
    /// Security contexts per site
    security_contexts: Arc<Mutex<HashMap<SiteId, SecurityContext>>>,
    /// Pending site-to-site messages
    message_queue: Arc<Mutex<Vec<SiteMessage>>>,
    /// Process pool for efficient process management
    process_pool: Option<ProcessPool>,
    /// Site boundary policy
    boundary_policy: Arc<Mutex<SiteBoundaryPolicy>>,
    /// IPC security manager for secure inter-process communication
    ipc_security: Arc<IpcSecurityManager>,
    /// Blacklisted sites (crashed too many times)
    blacklisted_sites: Arc<Mutex<HashSet<SiteId>>>,
}

impl SiteIsolationManager {
    pub fn new() -> Self {
        let policy = IsolationPolicy::default();
        let process_pool = if policy.enable_process_pooling {
            Some(ProcessPool::new(policy.min_pool_size, policy.max_pool_size))
        } else {
            None
        };

        // Initialize IPC security with default policy
        let ipc_policy = SecurityPolicy {
            max_message_size: 10 * 1024 * 1024, // 10MB
            rate_limit_per_second: 100,
            require_encryption: true,
            ..Default::default()
        };
        let ipc_security = Arc::new(IpcSecurityManager::new(ipc_policy));

        Self {
            policy: Arc::new(Mutex::new(policy)),
            site_instances: Arc::new(Mutex::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            url_to_site: Arc::new(Mutex::new(HashMap::new())),
            security_contexts: Arc::new(Mutex::new(HashMap::new())),
            message_queue: Arc::new(Mutex::new(Vec::new())),
            process_pool,
            boundary_policy: Arc::new(Mutex::new(SiteBoundaryPolicy::default())),
            ipc_security,
            blacklisted_sites: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    /// Get or create a site instance for a URL
    pub fn get_or_create_site(&self, url: &str) -> Result<SiteInstance, String> {
        // Validate URL is not empty
        if url.is_empty() {
            return Err("URL cannot be empty".to_string());
        }
        
        let site_id = SiteId::from_url(url)?;
        
        // Check if site is blacklisted
        let blacklisted = self.blacklisted_sites.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        if blacklisted.contains(&site_id) {
            return Err(format!("Site {} is blacklisted due to repeated crashes", site_id.etld_plus_one));
        }
        drop(blacklisted);
        
        let mut instances = self.site_instances.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(instance) = instances.get_mut(&site_id) {
            instance.touch();
            // Prevent overflow
            instance.webview_count = instance.webview_count.saturating_add(1);
            return Ok(instance.clone());
        }
        
        // Create new site instance
        let mut instance = SiteInstance::new(site_id.clone());
        instance.webview_count = 1;
        
        // Assign to process
        let process_id = self.assign_to_process(&site_id)?;
        instance.process_id = Some(process_id.clone());
        
        instances.insert(site_id.clone(), instance.clone());
        
        // Update URL mapping
        let mut url_map = self.url_to_site.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        url_map.insert(url.to_string(), site_id);
        
        Ok(instance)
    }
    
    /// Assign a site to a process
    fn assign_to_process(&self, site_id: &SiteId) -> Result<String, String> {
        let policy = self.policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if !policy.enabled {
            // Single process mode
            return Ok("default".to_string());
        }
        
        // Try to use process pool if enabled
        if policy.enable_process_pooling {
            if let Some(ref pool) = self.process_pool {
                match pool.acquire(Some(site_id.clone())) {
                    Ok(process_id) => {
                        // Register the process in the main process map
                        let mut processes = self.processes.lock()
                            .map_err(|e| format!("Lock error: {}", e))?;
                        if !processes.contains_key(&process_id) {
                            let process_info = ProcessInfo::new(process_id.clone(), site_id.clone());
                            processes.insert(process_id.clone(), process_info);
                        }
                        return Ok(process_id);
                    }
                    Err(_) => {
                        // Pool exhausted, fall through to regular process creation
                    }
                }
            }
        }
        
        let mut processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Check if we can reuse an existing process
        // In strict mode, always create new process
        if !policy.strict_mode {
            // Try to find a process for the same site
            for (pid, process) in processes.iter() {
                if process.site_id == *site_id && !process.is_crashed {
                    return Ok(pid.clone());
                }
            }
        }
        
        // Check process limit
        if processes.len() >= policy.max_processes {
            // Evict idle processes
            self.evict_idle_processes(&mut processes, &policy)?;
        }
        
        // Create new process
        let process_id = format!("site-{}-{}", 
            site_id.etld_plus_one, 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        
        let process_info = ProcessInfo::new(process_id.clone(), site_id.clone());
        processes.insert(process_id.clone(), process_info);
        
        Ok(process_id)
    }
    
    /// Evict idle processes
    fn evict_idle_processes(
        &self,
        processes: &mut HashMap<String, ProcessInfo>,
        policy: &IsolationPolicy,
    ) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let mut to_remove: Vec<String> = Vec::new();
        
        for (pid, process) in processes.iter() {
            // Use last_crash time if crashed, otherwise use created_at
            let last_activity = process.last_crash.unwrap_or(process.created_at);
            let idle_time = now - last_activity;
            if idle_time > policy.idle_process_timeout {
                to_remove.push(pid.clone());
            }
        }
        
        for pid in to_remove {
            processes.remove(&pid);
            tracing::info!("Evicted idle process: {}", pid);
        }
        
        Ok(())
    }
    
    /// Release a site instance (decrement webview count)
    pub fn release_site(&self, url: &str) -> Result<(), String> {
        let site_id = SiteId::from_url(url)?;
        
        let mut instances = self.site_instances.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(instance) = instances.get_mut(&site_id) {
            if instance.webview_count > 0 {
                instance.webview_count -= 1;
            }
            
            // Remove if no more webviews
            if instance.webview_count == 0 {
                instances.remove(&site_id);
                
                // Remove URL mapping
                let mut url_map = self.url_to_site.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                url_map.remove(url);
                
                // Clean up security context if no more instances
                let mut contexts = self.security_contexts.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                contexts.remove(&site_id);
            }
        }
        
        Ok(())
    }
    
    /// Handle process crash
    pub fn handle_process_crash(&self, process_id: &str, app: &AppHandle) -> Result<(), String> {
        tracing::error!("Process crash detected: {}", process_id);
        
        let mut processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(mut process) = processes.remove(process_id) {
            // Mark process as crashed
            process.mark_crashed();
            
            // Check if process has crashed too many times
            let policy = self.policy.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            if process.crash_count >= 3 {
                tracing::error!("Process {} has crashed {} times, blacklisting site: {}", 
                    process_id, process.crash_count, process.site_id.etld_plus_one);
                
                // Add site to blacklist
                let mut blacklisted = self.blacklisted_sites.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                blacklisted.insert(process.site_id.clone());
            }
            
            // Store crashed process info for analysis
            processes.insert(process_id.to_string(), process.clone());
            
            // Release from process pool if applicable
            if let Some(ref pool) = self.process_pool {
                let _ = pool.release(process_id);
            }
            
            // Find all site instances using this process
            let mut instances = self.site_instances.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            let affected_sites: Vec<SiteId> = instances
                .iter()
                .filter(|(_, inst)| inst.process_id.as_ref() == Some(&process_id.to_string()))
                .map(|(site_id, _)| site_id.clone())
                .collect();
            
            // Update affected site instances
            for site_id in &affected_sites {
                if let Some(instance) = instances.get_mut(site_id) {
                    instance.process_id = None;
                }
            }
            
            // Emit crash event with details
            let crash_info = serde_json::json!({
                "process_id": process_id,
                "site_id": process.site_id,
                "crash_count": process.crash_count,
                "last_crash": process.last_crash,
                "affected_sites": affected_sites,
            });
            let _ = app.emit("exodus-process-crashed", crash_info);
        }
        
        Ok(())
    }
    
    /// Recover a crashed process
    pub fn recover_process(&self, process_id: &str) -> Result<(), String> {
        let mut processes = self.processes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(process) = processes.get_mut(process_id) {
            process.mark_recovered();
            tracing::info!("Process recovered: {}", process_id);
        }
        
        Ok(())
    }
    
    /// Get isolation policy
    pub fn get_policy(&self) -> IsolationPolicy {
        self.policy.lock()
            .map(|p| p.clone())
            .unwrap_or_default()
    }
    
    /// Set isolation policy
    pub fn set_policy(&self, policy: IsolationPolicy) -> Result<(), String> {
        let mut p = self.policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *p = policy;
        Ok(())
    }
    
    /// Get all site instances
    pub fn get_site_instances(&self) -> Vec<SiteInstance> {
        self.site_instances.lock()
            .map(|instances| instances.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get all processes
    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        self.processes.lock()
            .map(|processes| processes.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get process for a URL
    pub fn get_process_for_url(&self, url: &str) -> Result<Option<String>, String> {
        let site_id = SiteId::from_url(url)?;
        
        let instances = self.site_instances.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(instances.get(&site_id)
            .and_then(|inst| inst.process_id.clone()))
    }
    
    /// Set security context for a site
    pub fn set_security_context(&self, site_id: SiteId, context: SecurityContext) -> Result<(), String> {
        let mut contexts = self.security_contexts.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        contexts.insert(site_id, context);
        Ok(())
    }
    
    /// Get security context for a site
    pub fn get_security_context(&self, site_id: &SiteId) -> SecurityContext {
        self.security_contexts.lock()
            .map(|contexts| contexts.get(site_id).cloned())
            .unwrap_or_default()
            .unwrap_or_default()
    }
    
    /// Send message from one site to another
    pub fn send_site_message(&self, message: SiteMessage, app: &AppHandle) -> Result<(), String> {
        let from_context = self.get_security_context(&message.from_site);
        let to_context = self.get_security_context(&message.to_site);
        
        // Check if communication is allowed
        if !from_context.allow_post_message || !to_context.allow_post_message {
            return Err("Site-to-site communication not allowed".to_string());
        }
        
        // Check if sites are same-origin (allow if strict mode is off)
        let policy = self.policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if !policy.strict_mode && message.from_site == message.to_site {
            // Same-origin communication is always allowed
        } else if !from_context.allow_cross_origin {
            return Err("Cross-origin communication not allowed".to_string());
        }
        
        // Queue message for delivery
        let mut queue = self.message_queue.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        queue.push(message.clone());
        
        // Emit event for message delivery
        let _ = app.emit("exodus-site-message", message);
        
        Ok(())
    }
    
    /// Get pending messages for a site
    pub fn get_pending_messages(&self, site_id: &SiteId) -> Vec<SiteMessage> {
        let mut queue = self.message_queue.lock()
            .map_err(|e| {
                tracing::error!("Lock error: {}", e);
                Vec::<SiteMessage>::new()
            })
            .unwrap();
        
        // Limit message queue size to prevent memory bloat
        if queue.len() > 1000 {
            tracing::warn!("Message queue size exceeds limit, truncating");
            queue.truncate(1000);
        }
        
        let messages: Vec<SiteMessage> = queue
            .iter()
            .filter(|msg| &msg.to_site == site_id)
            .cloned()
            .collect();
        
        // Remove delivered messages
        queue.retain(|msg| &msg.to_site != site_id);
        
        messages
    }
    
    /// Clean up stale data to prevent memory leaks
    /// Should be called periodically (e.g., every hour)
    pub fn cleanup_stale_data(&self) -> Result<(), String> {
        let policy = self.policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Clean up old messages
        let mut queue = self.message_queue.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let before_count = queue.len();
        queue.retain(|msg| now - msg.timestamp < 3600); // Keep messages for 1 hour
        let after_count = queue.len();
        if before_count > after_count {
            tracing::info!("Cleaned up {} old messages", before_count - after_count);
        }
        
        // Clean up stale site instances (those with webview_count == 0 but not removed)
        let mut instances = self.site_instances.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let before_instances = instances.len();
        instances.retain(|_, inst| {
            inst.webview_count > 0 || (now - inst.last_used) < policy.idle_process_timeout
        });
        let after_instances = instances.len();
        if before_instances > after_instances {
            tracing::info!("Cleaned up {} stale site instances", before_instances - after_instances);
        }
        
        Ok(())
    }
    
    /// Check if navigation is allowed between sites
    pub fn is_navigation_allowed(&self, from_url: &str, to_url: &str) -> Result<bool, String> {
        let from_site = SiteId::from_url(from_url)?;
        let to_site = SiteId::from_url(to_url)?;
        
        let policy = self.boundary_policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(policy.is_navigation_allowed(&from_site, &to_site))
    }
    
    /// Check if third-party cookies should be blocked
    pub fn should_block_third_party_cookies(&self, first_party_url: &str, third_party_url: &str) -> Result<bool, String> {
        let first_party = SiteId::from_url(first_party_url)?;
        let third_party = SiteId::from_url(third_party_url)?;
        
        let policy = self.boundary_policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        Ok(policy.should_block_third_party_cookies(&first_party, &third_party))
    }
    
    /// Get site boundary policy
    pub fn get_boundary_policy(&self) -> SiteBoundaryPolicy {
        self.boundary_policy.lock()
            .map(|p| p.clone())
            .unwrap_or_default()
    }
    
    /// Set site boundary policy
    pub fn set_boundary_policy(&self, policy: SiteBoundaryPolicy) -> Result<(), String> {
        let mut p = self.boundary_policy.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *p = policy;
        Ok(())
    }
    
    /// Get process pool statistics
    pub fn get_pool_stats(&self) -> Option<(usize, usize, usize)> {
        self.process_pool.as_ref().map(|pool| pool.get_stats())
    }
    
    /// Remove site from blacklist
    pub fn unblock_site(&self, url: &str) -> Result<(), String> {
        let site_id = SiteId::from_url(url)?;
        let mut blacklisted = self.blacklisted_sites.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        blacklisted.remove(&site_id);
        Ok(())
    }
    
    /// Get blacklisted sites
    pub fn get_blacklisted_sites(&self) -> Vec<SiteId> {
        self.blacklisted_sites.lock()
            .map(|sites| sites.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Send secure IPC message between sites
    pub fn send_ipc_message(&self, from_url: &str, to_url: &str, command: String, payload: serde_json::Value) -> Result<(), String> {
        let from_site = SiteId::from_url(from_url)?;
        let to_site = SiteId::from_url(to_url)?;
        
        // Create IPC message
        let ipc_message = IpcMessage {
            message_id: format!("msg-{}-{}", 
                from_site.etld_plus_one,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            message_type: IpcMessageType::Command,
            source: from_site.etld_plus_one.clone(),
            destination: to_site.etld_plus_one.clone(),
            command,
            payload,
            priority: IpcPriority::Normal,
            security_level: SecurityLevel::Medium,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: Some(60),
        };
        
        // Process through IPC security
        match self.ipc_security.process_message(ipc_message) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("IPC security error: {}", e)),
        }
    }
    
    /// Get isolation statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let instances = self.site_instances.lock().unwrap();
        let processes = self.processes.lock().unwrap();
        let policy = self.policy.lock().unwrap();
        let blacklisted = self.blacklisted_sites.lock().unwrap();
        
        serde_json::json!({
            "enabled": policy.enabled,
            "strict_mode": policy.strict_mode,
            "total_sites": instances.len(),
            "total_processes": processes.len(),
            "max_processes": policy.max_processes,
            "blacklisted_sites": blacklisted.len(),
            "process_pool_enabled": policy.enable_process_pooling,
            "pool_stats": self.get_pool_stats(),
            "spectre_mitigations": policy.enable_spectre_mitigations,
        })
    }
}

impl Default for SiteIsolationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to get or create a site instance
#[tauri::command]
pub fn get_or_create_site(
    url: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<SiteInstance, String> {
    manager.get_or_create_site(&url)
}

/// Tauri command to release a site instance
#[tauri::command]
pub fn release_site(
    url: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.release_site(&url)
}

/// Tauri command to handle process crash
#[tauri::command]
pub fn handle_process_crash(
    process_id: String,
    app: AppHandle,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.handle_process_crash(&process_id, &app)
}

/// Tauri command to recover a crashed process
#[tauri::command]
pub fn recover_process(
    process_id: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.recover_process(&process_id)
}

/// Tauri command to get isolation policy
#[tauri::command]
pub fn get_isolation_policy(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> IsolationPolicy {
    manager.get_policy()
}

/// Tauri command to set isolation policy
#[tauri::command]
pub fn set_isolation_policy(
    policy: IsolationPolicy,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.set_policy(policy)
}

/// Tauri command to get all site instances
#[tauri::command]
pub fn get_site_instances(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Vec<SiteInstance> {
    manager.get_site_instances()
}

/// Tauri command to get all processes
#[tauri::command]
pub fn get_processes(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Vec<ProcessInfo> {
    manager.get_processes()
}

/// Tauri command to get process for a URL
#[tauri::command]
pub fn get_process_for_url(
    url: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<Option<String>, String> {
    manager.get_process_for_url(&url)
}

/// Tauri command to set security context
#[tauri::command]
pub fn set_security_context(
    site_id: SiteId,
    context: SecurityContext,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.set_security_context(site_id, context)
}

/// Tauri command to get security context
#[tauri::command]
pub fn get_security_context(
    site_id: SiteId,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> SecurityContext {
    manager.get_security_context(&site_id)
}

/// Tauri command to send site message
#[tauri::command]
pub fn send_site_message(
    message: SiteMessage,
    app: AppHandle,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.send_site_message(message, &app)
}

/// Tauri command to get pending messages
#[tauri::command]
pub fn get_pending_messages(
    site_id: SiteId,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Vec<SiteMessage> {
    manager.get_pending_messages(&site_id)
}

/// Tauri command to clean up stale data
#[tauri::command]
pub fn cleanup_stale_data(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.cleanup_stale_data()
}

/// Tauri command to check if navigation is allowed
#[tauri::command]
pub fn is_navigation_allowed(
    from_url: String,
    to_url: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<bool, String> {
    manager.is_navigation_allowed(&from_url, &to_url)
}

/// Tauri command to check if third-party cookies should be blocked
#[tauri::command]
pub fn should_block_third_party_cookies(
    first_party_url: String,
    third_party_url: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<bool, String> {
    manager.should_block_third_party_cookies(&first_party_url, &third_party_url)
}

/// Tauri command to get site boundary policy
#[tauri::command]
pub fn get_boundary_policy(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> SiteBoundaryPolicy {
    manager.get_boundary_policy()
}

/// Tauri command to set site boundary policy
#[tauri::command]
pub fn set_boundary_policy(
    policy: SiteBoundaryPolicy,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.set_boundary_policy(policy)
}

/// Tauri command to get process pool statistics
#[tauri::command]
pub fn get_pool_stats(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Option<(usize, usize, usize)> {
    manager.get_pool_stats()
}

/// Tauri command to unblock a site
#[tauri::command]
pub fn unblock_site(
    url: String,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.unblock_site(&url)
}

/// Tauri command to get blacklisted sites
#[tauri::command]
pub fn get_blacklisted_sites(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Vec<SiteId> {
    manager.get_blacklisted_sites()
}

/// Tauri command to send IPC message
#[tauri::command]
pub fn send_ipc_message(
    from_url: String,
    to_url: String,
    command: String,
    payload: serde_json::Value,
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> Result<(), String> {
    manager.send_ipc_message(&from_url, &to_url, command, payload)
}

/// Tauri command to get isolation statistics
#[tauri::command]
pub fn get_isolation_stats(
    manager: State<'_, Arc<SiteIsolationManager>>,
) -> serde_json::Value {
    manager.get_stats()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_site_id_from_url() {
        let site_id = SiteId::from_url("https://example.com/path").unwrap();
        assert_eq!(site_id.scheme, "https");
        assert_eq!(site_id.etld_plus_one, "example.com");
    }
    
    #[test]
    fn test_site_id_from_subdomain() {
        let site_id = SiteId::from_url("https://sub.example.com/path").unwrap();
        assert_eq!(site_id.scheme, "https");
        assert_eq!(site_id.etld_plus_one, "example.com");
    }
    
    #[test]
    fn test_extract_etld_plus_one() {
        assert_eq!(extract_etld_plus_one("example.com"), "example.com");
        assert_eq!(extract_etld_plus_one("sub.example.com"), "example.com");
        assert_eq!(extract_etld_plus_one("deep.sub.example.com"), "example.com");
    }
    
    #[test]
    fn test_site_instance_creation() {
        let site_id = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let instance = SiteInstance::new(site_id);
        assert_eq!(instance.webview_count, 0);
        assert!(instance.process_id.is_none());
    }
    
    #[test]
    fn test_site_instance_touch() {
        let site_id = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let mut instance = SiteInstance::new(site_id);
        let initial_time = instance.last_used;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        instance.touch();
        
        assert!(instance.last_used > initial_time);
    }
    
    #[test]
    fn test_process_info_creation() {
        let site_id = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let process = ProcessInfo::new("test-process".to_string(), site_id);
        assert_eq!(process.process_id, "test-process");
        assert_eq!(process.crash_count, 0);
        assert!(!process.is_crashed);
    }
    
    #[test]
    fn test_process_crash_marking() {
        let site_id = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let mut process = ProcessInfo::new("test-process".to_string(), site_id);
        
        process.mark_crashed();
        assert!(process.is_crashed);
        assert_eq!(process.crash_count, 1);
        assert!(process.last_crash.is_some());
        
        process.mark_recovered();
        assert!(!process.is_crashed);
    }
    
    #[test]
    fn test_isolation_policy_default() {
        let policy = IsolationPolicy::default();
        assert!(policy.enabled);
        assert!(!policy.strict_mode);
        assert_eq!(policy.max_processes, 50);
    }
    
    #[test]
    fn test_site_isolation_manager() {
        let manager = SiteIsolationManager::new();
        
        let instance = manager.get_or_create_site("https://example.com").unwrap();
        assert_eq!(instance.webview_count, 1);
        assert!(instance.process_id.is_some());
        
        let instance2 = manager.get_or_create_site("https://example.com").unwrap();
        assert_eq!(instance2.webview_count, 2);
        
        manager.release_site("https://example.com").unwrap();
        
        let instance3 = manager.get_or_create_site("https://example.com").unwrap();
        assert_eq!(instance3.webview_count, 2);
    }
    
    #[test]
    fn test_site_isolation_different_sites() {
        let manager = SiteIsolationManager::new();
        
        let instance1 = manager.get_or_create_site("https://example.com").unwrap();
        let instance2 = manager.get_or_create_site("https://google.com").unwrap();
        
        assert_ne!(instance1.site_id.etld_plus_one, instance2.site_id.etld_plus_one);
        assert_ne!(instance1.process_id, instance2.process_id);
    }
    
    #[test]
    fn test_security_context() {
        let manager = SiteIsolationManager::new();
        
        let site_id = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        
        let context = SecurityContext {
            allow_cross_origin: true,
            allow_post_message: true,
            allow_shared_workers: false,
            allow_service_workers: true,
        };
        
        manager.set_security_context(site_id.clone(), context).unwrap();
        
        let retrieved = manager.get_security_context(&site_id);
        assert!(retrieved.allow_cross_origin);
        assert!(retrieved.allow_post_message);
    }
    
    #[test]
    fn test_site_message() {
        let from_site = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let to_site = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "google.com".to_string(),
        };
        
        let message = SiteMessage::new(
            from_site.clone(),
            to_site.clone(),
            "test-type".to_string(),
            serde_json::json!({"data": "test"}),
        );
        
        assert_eq!(message.from_site, from_site);
        assert_eq!(message.to_site, to_site);
        assert_eq!(message.message_type, "test-type");
    }
    
    #[test]
    fn test_process_crash_handling() {
        let manager = SiteIsolationManager::new();
        
        // Create a site
        let instance = manager.get_or_create_site("https://example.com").unwrap();
        let process_id = instance.process_id.clone().unwrap();
        
        // Simulate crash (manually mark process as crashed)
        {
            let mut processes = manager.processes.lock().unwrap();
            if let Some(process) = processes.get_mut(&process_id) {
                process.mark_crashed();
            }
        }
        
        // Verify process is marked as crashed
        let processes = manager.get_processes();
        let crashed_process = processes.iter().find(|p| p.process_id == process_id);
        assert!(crashed_process.is_some());
        assert!(crashed_process.unwrap().is_crashed);
    }
    
    #[test]
    fn test_process_recovery() {
        let manager = SiteIsolationManager::new();
        
        // Create a site
        let instance = manager.get_or_create_site("https://example.com").unwrap();
        let process_id = instance.process_id.clone().unwrap();
        
        // Simulate crash (manually mark process as crashed)
        {
            let mut processes = manager.processes.lock().unwrap();
            if let Some(process) = processes.get_mut(&process_id) {
                process.mark_crashed();
            }
        }
        
        // Recover
        manager.recover_process(&process_id).unwrap();
        
        // Verify process is recovered
        let processes = manager.get_processes();
        let recovered_process = processes.iter().find(|p| p.process_id == process_id);
        assert!(recovered_process.is_some());
        assert!(!recovered_process.unwrap().is_crashed);
    }
    
    #[test]
    fn test_extract_etld_plus_one_with_tld_list() {
        assert_eq!(extract_etld_plus_one("example.com"), "example.com");
        assert_eq!(extract_etld_plus_one("sub.example.com"), "example.com");
        assert_eq!(extract_etld_plus_one("example.co.uk"), "example.co.uk");
        assert_eq!(extract_etld_plus_one("sub.example.co.uk"), "example.co.uk");
        assert_eq!(extract_etld_plus_one("example.co.jp"), "example.co.jp");
        assert_eq!(extract_etld_plus_one("example.com.au"), "example.com.au");
    }
    
    #[test]
    fn test_site_boundary_policy() {
        let policy = SiteBoundaryPolicy::default();
        
        let site1 = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let site2 = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "google.com".to_string(),
        };
        
        // Same-origin should be allowed
        assert!(policy.is_navigation_allowed(&site1, &site1));
        
        // Cross-origin should be blocked in strict mode
        assert!(!policy.is_navigation_allowed(&site1, &site2));
    }
    
    #[test]
    fn test_site_boundary_policy_with_whitelist() {
        let mut policy = SiteBoundaryPolicy::default();
        policy.strict_same_origin = false;
        policy.allowed_navigations.insert("example.com -> google.com".to_string());
        
        let site1 = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let site2 = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "google.com".to_string(),
        };
        
        // Whitelisted navigation should be allowed
        assert!(policy.is_navigation_allowed(&site1, &site2));
    }
    
    #[test]
    fn test_third_party_cookie_blocking() {
        let policy = SiteBoundaryPolicy::default();
        
        let first_party = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        let third_party = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "tracker.com".to_string(),
        };
        
        // Third-party cookies should be blocked
        assert!(policy.should_block_third_party_cookies(&first_party, &third_party));
        
        // First-party cookies should not be blocked
        assert!(!policy.should_block_third_party_cookies(&first_party, &first_party));
    }
    
    #[test]
    fn test_process_pool() {
        let pool = ProcessPool::new(2, 5);
        
        let site_id = SiteId {
            scheme: "https".to_string(),
            etld_plus_one: "example.com".to_string(),
        };
        
        // Acquire a process
        let pid1 = pool.acquire(Some(site_id.clone())).unwrap();
        assert!(!pid1.is_empty());
        
        // Acquire another process
        let pid2 = pool.acquire(Some(site_id.clone())).unwrap();
        assert_ne!(pid1, pid2);
        
        // Release first process
        pool.release(&pid1).unwrap();
        
        // Acquire should reuse released process
        let pid3 = pool.acquire(Some(site_id.clone())).unwrap();
        assert_eq!(pid1, pid3);
        
        // Check stats
        let (total, available, in_use) = pool.get_stats();
        assert_eq!(total, 2);
        assert_eq!(available, 0);
        assert_eq!(in_use, 2);
    }
    
    #[test]
    fn test_site_blacklisting() {
        let manager = SiteIsolationManager::new();
        
        // Create a site
        let instance = manager.get_or_create_site("https://example.com").unwrap();
        let process_id = instance.process_id.clone().unwrap();
        
        // Manually add site to blacklist (simulate crash behavior)
        let site_id = SiteId::from_url("https://example.com").unwrap();
        let mut blacklisted = manager.blacklisted_sites.lock().unwrap();
        blacklisted.insert(site_id);
        drop(blacklisted);
        
        // Site should be blacklisted
        let blacklisted = manager.get_blacklisted_sites();
        assert!(!blacklisted.is_empty());
        
        // Trying to create site again should fail
        let result = manager.get_or_create_site("https://example.com");
        assert!(result.is_err());
        
        // Unblock the site
        manager.unblock_site("https://example.com").unwrap();
        
        // Should be able to create site again
        let result = manager.get_or_create_site("https://example.com");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_isolation_stats() {
        let manager = SiteIsolationManager::new();
        
        // Create some sites
        manager.get_or_create_site("https://example.com").unwrap();
        manager.get_or_create_site("https://google.com").unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats["total_sites"], 2);
        assert_eq!(stats["enabled"], true);
        assert!(stats["spectre_mitigations"].as_bool().unwrap());
    }
    
    #[test]
    fn test_navigation_allowed() {
        let manager = SiteIsolationManager::new();
        
        // Same-origin navigation should be allowed
        let result = manager.is_navigation_allowed("https://example.com", "https://example.com/page");
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Cross-origin navigation should be blocked in strict mode
        let result = manager.is_navigation_allowed("https://example.com", "https://google.com");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
