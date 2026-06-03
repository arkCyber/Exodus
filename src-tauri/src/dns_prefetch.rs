//! DNS Prefetching for Performance Optimization
//! 
//! This module provides DNS prefetching to reduce latency when navigating to links.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use tokio::runtime::Runtime;

/// DNS cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsCacheEntry {
    /// Domain name
    pub domain: String,
    /// Resolved IP addresses
    pub ips: Vec<String>,
    /// Cache timestamp
    pub cached_at: u64,
    /// TTL in seconds
    pub ttl: u64,
    /// Number of times used
    pub use_count: u32,
}

impl DnsCacheEntry {
    pub fn new(domain: String, ips: Vec<String>, ttl: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            domain,
            ips,
            cached_at: now,
            ttl,
            use_count: 0,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        now - self.cached_at > self.ttl
    }
    
    pub fn record_use(&mut self) {
        self.use_count += 1;
    }
}

/// DNS prefetch settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsPrefetchSettings {
    /// Enable DNS prefetching
    pub enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Maximum cache entries
    pub max_cache_entries: usize,
    /// Prefetch on hover
    pub prefetch_on_hover: bool,
    /// Prefetch on page load
    pub prefetch_on_page_load: bool,
}

impl Default for DnsPrefetchSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_ttl: 3600, // 1 hour
            max_cache_entries: 1000,
            prefetch_on_hover: true,
            prefetch_on_page_load: true,
        }
    }
}

/// DNS prefetch manager
pub struct DnsPrefetchManager {
    cache: Arc<Mutex<HashMap<String, DnsCacheEntry>>>,
    settings: Arc<Mutex<DnsPrefetchSettings>>,
    prefetch_queue: Arc<Mutex<HashSet<String>>>,
    runtime: Arc<Runtime>,
}

impl DnsPrefetchManager {
    /// Create a new DNS prefetch manager
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(DnsPrefetchSettings::default())),
            prefetch_queue: Arc::new(Mutex::new(HashSet::new())),
            runtime: Arc::new(Runtime::new().expect("Failed to create Tokio runtime")),
        }
    }
    
    /// Resolve a domain and cache the result
    pub fn resolve_domain(&self, domain: String) -> Result<Vec<String>, String> {
        // Check cache first
        if let Some(entry) = self.get_cached(&domain) {
            return Ok(entry.ips);
        }
        
        // Resolve domain
        let ips = self.resolve_domain_sync(&domain)?;
        
        // Cache the result
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let entry = DnsCacheEntry::new(domain.clone(), ips.clone(), settings.cache_ttl);
        
        let mut cache = self.cache.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Evict if needed
        if cache.len() >= settings.max_cache_entries {
            // Remove oldest entry
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(domain.clone(), entry);
        
        Ok(ips)
    }
    
    /// Resolve domain synchronously
    fn resolve_domain_sync(&self, domain: &str) -> Result<Vec<String>, String> {
        let domain_with_port = if domain.contains(':') {
            domain.to_string()
        } else {
            format!("{}:443", domain)
        };
        
        match domain_with_port.to_socket_addrs() {
            Ok(addrs) => {
                let ips: Vec<String> = addrs
                    .map(|addr| addr.ip().to_string())
                    .collect();
                
                if ips.is_empty() {
                    Err(format!("No IP addresses found for domain: {}", domain))
                } else {
                    Ok(ips)
                }
            }
            Err(e) => Err(format!("DNS resolution failed for {}: {}", domain, e))
        }
    }
    
    /// Get cached DNS entry
    pub fn get_cached(&self, domain: &str) -> Option<DnsCacheEntry> {
        let mut cache = self.cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(mut entry) = cache.get(domain).cloned() {
            if entry.is_expired() {
                cache.remove(domain);
                return None;
            }
            
            entry.record_use();
            cache.insert(domain.to_string(), entry.clone());
            return Some(entry);
        }
        
        None
    }
    
    /// Prefetch a domain (async)
    pub fn prefetch_domain(&self, domain: String) {
        let cache = self.cache.clone();
        let settings = self.settings.clone();
        
        self.runtime.spawn(async move {
            // Check if already cached
            {
                let cache_guard = cache.lock();
                if cache_guard.as_ref().ok().map(|g| g.contains_key(&domain)).unwrap_or(false) {
                    return;
                }
            }
            
            // Resolve domain
            if let Ok(ips) = Self::resolve_domain_async(&domain).await {
                let settings_guard = settings.lock();
                if let Ok(settings) = settings_guard {
                    let entry = DnsCacheEntry::new(domain.clone(), ips, settings.cache_ttl);
                    
                    let mut cache_guard = cache.lock();
                    if let Ok(mut cache) = cache_guard {
                        // Evict if needed
                        if cache.len() >= settings.max_cache_entries {
                            if let Some(oldest_key) = cache.keys().next().cloned() {
                                cache.remove(&oldest_key);
                            }
                        }
                        
                        cache.insert(domain, entry);
                    }
                }
            }
        });
    }
    
    /// Resolve domain asynchronously
    async fn resolve_domain_async(domain: &str) -> Result<Vec<String>, String> {
        let domain_with_port = if domain.contains(':') {
            domain.to_string()
        } else {
            format!("{}:443", domain)
        };
        
        let result = match tokio::net::lookup_host(&domain_with_port).await {
            Ok(addrs) => {
                let ips: Vec<String> = addrs
                    .map(|addr| addr.ip().to_string())
                    .collect();
                
                if ips.is_empty() {
                    Err(format!("No IP addresses found for domain: {}", domain))
                } else {
                    Ok(ips)
                }
            }
            Err(e) => Err(format!("DNS resolution failed for {}: {}", domain, e))
        };
        
        result
    }
    
    /// Prefetch domains from a page
    pub fn prefetch_from_page(&self, html: &str) {
        let domains = Self::extract_domains_from_html(html);
        
        for domain in domains {
            self.prefetch_domain(domain);
        }
    }
    
    /// Extract domains from HTML
    fn extract_domains_from_html(html: &str) -> Vec<String> {
        let mut domains = HashSet::new();
        
        // Extract href attributes
        for line in html.lines() {
            if let Some(start) = line.find("href=\"") {
                let start = start + 6;
                if let Some(end) = line[start..].find('"') {
                    let url = &line[start..start + end];
                    if let Ok(parsed) = url::Url::parse(url) {
                        if let Some(host) = parsed.host_str() {
                            domains.insert(host.to_string());
                        }
                    }
                }
            }
            
            // Extract src attributes
            if let Some(start) = line.find("src=\"") {
                let start = start + 5;
                if let Some(end) = line[start..].find('"') {
                    let url = &line[start..start + end];
                    if let Ok(parsed) = url::Url::parse(url) {
                        if let Some(host) = parsed.host_str() {
                            domains.insert(host.to_string());
                        }
                    }
                }
            }
        }
        
        domains.into_iter().collect()
    }
    
    /// Add domain to prefetch queue
    pub fn queue_prefetch(&self, domain: String) {
        let mut queue = self.prefetch_queue.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        queue.insert(domain);
    }
    
    /// Process prefetch queue
    pub fn process_prefetch_queue(&self) {
        let domains: Vec<String> = {
            let queue = self.prefetch_queue.lock()
                .unwrap_or_else(|_| panic!("Lock error"));
            queue.iter().cloned().collect()
        };
        
        for domain in domains {
            self.prefetch_domain(domain);
        }
        
        let mut queue = self.prefetch_queue.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        queue.clear();
    }
    
    /// Clear DNS cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        cache.clear();
    }
    
    /// Get DNS cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, usize> {
        let cache = self.cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), cache.len());
        stats.insert("total_lookups".to_string(), cache.values().map(|e| e.use_count as usize).sum());
        
        stats
    }
    
    /// Get DNS prefetch settings
    pub fn get_settings(&self) -> DnsPrefetchSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update DNS prefetch settings
    pub fn update_settings(&self, settings: DnsPrefetchSettings) {
        let mut current = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        *current = settings;
    }
}

// Tauri Commands

/// Resolve a domain
#[tauri::command]
pub fn resolve_domain(
    domain: String,
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<Vec<String>, String> {
    manager.resolve_domain(domain)
}

/// Get cached DNS entry
#[tauri::command]
pub fn get_cached_dns(
    domain: String,
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<Option<DnsCacheEntry>, String> {
    Ok(manager.get_cached(&domain))
}

/// Prefetch a domain
#[tauri::command]
pub fn prefetch_domain(
    domain: String,
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<(), String> {
    manager.prefetch_domain(domain);
    Ok(())
}

/// Prefetch domains from page HTML
#[tauri::command]
pub fn prefetch_from_page(
    html: String,
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<(), String> {
    manager.prefetch_from_page(&html);
    Ok(())
}

/// Queue domain for prefetch
#[tauri::command]
pub fn queue_prefetch(
    domain: String,
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<(), String> {
    manager.queue_prefetch(domain);
    Ok(())
}

/// Process prefetch queue
#[tauri::command]
pub fn process_prefetch_queue(
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<(), String> {
    manager.process_prefetch_queue();
    Ok(())
}

/// Clear DNS cache
#[tauri::command]
pub fn clear_dns_cache(
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<(), String> {
    manager.clear_cache();
    Ok(())
}

/// Get DNS cache statistics
#[tauri::command]
pub fn get_dns_cache_stats(
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<HashMap<String, usize>, String> {
    Ok(manager.get_cache_stats())
}

/// Get DNS prefetch settings
#[tauri::command]
pub fn get_dns_prefetch_settings(
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<DnsPrefetchSettings, String> {
    Ok(manager.get_settings())
}

/// Update DNS prefetch settings
#[tauri::command]
pub fn update_dns_prefetch_settings(
    settings: DnsPrefetchSettings,
    manager: State<'_, Arc<DnsPrefetchManager>>,
) -> Result<(), String> {
    manager.update_settings(settings);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dns_cache_entry() {
        let entry = DnsCacheEntry::new(
            "example.com".to_string(),
            vec!["1.2.3.4".to_string()],
            3600,
        );
        
        assert_eq!(entry.domain, "example.com");
        assert_eq!(entry.ips.len(), 1);
        assert!(!entry.is_expired());
    }
    
    #[test]
    fn test_dns_prefetch_manager() {
        let manager = DnsPrefetchManager::new();
        
        // Test with a real domain
        let result = manager.resolve_domain("localhost".to_string());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_extract_domains_from_html() {
        let html = r#"
            <html>
                <a href="https://example.com">Link</a>
                <img src="https://cdn.example.com/image.jpg">
            </html>
        "#;
        
        let domains = DnsPrefetchManager::extract_domains_from_html(html);
        assert!(domains.contains(&"example.com".to_string()));
        assert!(domains.contains(&"cdn.example.com".to_string()));
    }
    
    #[test]
    fn test_dns_prefetch_settings() {
        let settings = DnsPrefetchSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.cache_ttl, 3600);
    }
}
