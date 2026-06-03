//! Resource preloader for intelligent prefetching and caching

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Resource type for prioritization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Script,
    Image,
    Font,
    Media,
    Other,
}

/// Preload priority
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreloadPriority {
    Critical = 3,
    High = 2,
    Medium = 1,
    Low = 0,
}

/// Resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub url: String,
    pub resource_type: ResourceType,
    pub priority: PreloadPriority,
    pub size_bytes: Option<u64>,
    pub last_accessed: u64,
    pub access_count: u32,
    pub cache_hit: bool,
}

/// Preload hint from page analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloadHint {
    pub url: String,
    pub resource_type: ResourceType,
    pub priority: PreloadPriority,
    pub source_url: String,
}

/// Resource cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    url: String,
    data: Vec<u8>,
    resource_type: ResourceType,
    cached_at: u64,
    last_accessed: u64,
    access_count: u32,
    size_bytes: u64,
}

/// Configuration for resource preloader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloaderConfig {
    pub enabled: bool,
    pub max_cache_size_mb: u64,
    pub max_concurrent_preloads: usize,
    pub preload_timeout_secs: u64,
    pub cache_ttl_secs: u64,
    pub enable_predictive: bool,
}

impl Default for PreloaderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_cache_size_mb: 100,
            max_concurrent_preloads: 6,
            preload_timeout_secs: 10,
            cache_ttl_secs: 3600,
            enable_predictive: true,
        }
    }
}

/// Statistics for resource preloader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloaderStats {
    pub total_preloads: u64,
    pub successful_preloads: u64,
    pub failed_preloads: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_bytes_cached: u64,
    pub total_bytes_saved: u64,
    pub cache_entries: usize,
    pub hit_rate: f64,
}

/// Resource preloader manager
pub struct ResourcePreloader {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    preload_queue: Arc<RwLock<VecDeque<PreloadHint>>>,
    config: Arc<RwLock<PreloaderConfig>>,
    stats: Arc<RwLock<PreloaderStats>>,
    url_patterns: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ResourcePreloader {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            preload_queue: Arc::new(RwLock::new(VecDeque::new())),
            config: Arc::new(RwLock::new(PreloaderConfig::default())),
            stats: Arc::new(RwLock::new(PreloaderStats {
                total_preloads: 0,
                successful_preloads: 0,
                failed_preloads: 0,
                cache_hits: 0,
                cache_misses: 0,
                total_bytes_cached: 0,
                total_bytes_saved: 0,
                cache_entries: 0,
                hit_rate: 0.0,
            })),
            url_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add preload hint
    pub async fn add_hint(&self, hint: PreloadHint) {
        let config = self.config.read().await;
        if !config.enabled {
            return;
        }
        drop(config);

        let mut queue = self.preload_queue.write().await;
        
        // Check if already in queue
        if !queue.iter().any(|h| h.url == hint.url) {
            queue.push_back(hint);
        }
    }

    /// Process preload queue
    pub async fn process_queue(&self) -> Result<usize, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Ok(0);
        }
        
        let max_concurrent = config.max_concurrent_preloads;
        drop(config);

        let mut processed = 0;
        let mut queue = self.preload_queue.write().await;

        while processed < max_concurrent && !queue.is_empty() {
            if let Some(hint) = queue.pop_front() {
                drop(queue);
                
                // Simulate resource fetch (in real implementation, use reqwest)
                let result = self.fetch_resource(&hint.url, hint.resource_type.clone()).await;
                
                match result {
                    Ok(data) => {
                        self.cache_resource(&hint.url, data, hint.resource_type).await;
                        
                        let mut stats = self.stats.write().await;
                        stats.successful_preloads += 1;
                    }
                    Err(_) => {
                        let mut stats = self.stats.write().await;
                        stats.failed_preloads += 1;
                    }
                }

                processed += 1;
                queue = self.preload_queue.write().await;
            }
        }

        Ok(processed)
    }

    /// Fetch resource (simulated)
    async fn fetch_resource(&self, url: &str, _resource_type: ResourceType) -> Result<Vec<u8>, String> {
        // In real implementation, use reqwest to fetch
        // For now, simulate with empty data
        let config = self.config.read().await;
        let timeout = Duration::from_secs(config.preload_timeout_secs);
        drop(config);

        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Simulate data (in real app, fetch from network)
        let simulated_data = format!("Resource data for {}", url).into_bytes();
        
        Ok(simulated_data)
    }

    /// Cache resource
    async fn cache_resource(&self, url: &str, data: Vec<u8>, resource_type: ResourceType) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let size_bytes = data.len() as u64;
        
        let entry = CacheEntry {
            url: url.to_string(),
            data,
            resource_type,
            cached_at: now,
            last_accessed: now,
            access_count: 0,
            size_bytes,
        };

        let mut cache = self.cache.write().await;
        cache.insert(url.to_string(), entry);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_bytes_cached += size_bytes;
        stats.cache_entries = cache.len();

        // Cleanup if over limit
        drop(cache);
        drop(stats);
        self.cleanup_cache().await;
    }

    /// Get cached resource
    pub async fn get_cached(&self, url: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(url) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();

            // Check TTL
            let config = self.config.read().await;
            let ttl = config.cache_ttl_secs;
            drop(config);

            if now - entry.cached_at > ttl {
                // Expired
                cache.remove(url);
                
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
                return None;
            }

            // Update access info
            entry.last_accessed = now;
            entry.access_count += 1;

            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;
            stats.total_bytes_saved += entry.size_bytes;
            stats.hit_rate = stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64;

            Some(entry.data.clone())
        } else {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
            stats.hit_rate = stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64;
            None
        }
    }

    /// Cleanup cache based on size limit
    async fn cleanup_cache(&self) {
        let config = self.config.read().await;
        let max_size = config.max_cache_size_mb * 1024 * 1024;
        drop(config);

        let mut cache = self.cache.write().await;
        let mut total_size: u64 = cache.values().map(|e| e.size_bytes).sum();

        if total_size <= max_size {
            return;
        }

        // Sort by LRU (least recently used)
        let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.last_accessed)).collect();
        entries.sort_by_key(|(_, accessed)| *accessed);

        // Remove oldest until under limit
        for (url, _) in entries {
            if total_size <= max_size {
                break;
            }

            if let Some(entry) = cache.remove(&url) {
                total_size -= entry.size_bytes;
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.cache_entries = cache.len();
        stats.total_bytes_cached = total_size;
    }

    /// Learn URL patterns for predictive preloading
    pub async fn learn_pattern(&self, source_url: &str, resource_url: &str) {
        let config = self.config.read().await;
        if !config.enable_predictive {
            return;
        }
        drop(config);

        let mut patterns = self.url_patterns.write().await;
        patterns
            .entry(source_url.to_string())
            .or_insert_with(Vec::new)
            .push(resource_url.to_string());
    }

    /// Get predictive preload hints
    pub async fn get_predictive_hints(&self, url: &str) -> Vec<String> {
        let patterns = self.url_patterns.read().await;
        patterns.get(url).cloned().unwrap_or_default()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> PreloaderStats {
        self.stats.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: PreloaderConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// Get configuration
    pub async fn get_config(&self) -> PreloaderConfig {
        self.config.read().await.clone()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        let mut stats = self.stats.write().await;
        stats.cache_entries = 0;
        stats.total_bytes_cached = 0;
    }
}

impl Default for ResourcePreloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_preloader_creation() {
        let preloader = ResourcePreloader::new();
        let stats = preloader.get_stats().await;
        assert_eq!(stats.cache_entries, 0);
    }

    #[tokio::test]
    async fn test_add_hint() {
        let preloader = ResourcePreloader::new();
        
        let hint = PreloadHint {
            url: "https://example.com/style.css".to_string(),
            resource_type: ResourceType::Stylesheet,
            priority: PreloadPriority::High,
            source_url: "https://example.com".to_string(),
        };

        preloader.add_hint(hint).await;
        
        let queue = preloader.preload_queue.read().await;
        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let preloader = ResourcePreloader::new();
        
        let data = b"test data".to_vec();
        preloader.cache_resource(
            "https://example.com/test.js",
            data.clone(),
            ResourceType::Script,
        ).await;

        let cached = preloader.get_cached("https://example.com/test.js").await;
        assert!(cached.is_some());
        assert_eq!(cached.expect("Expected cached data"), data);

        let stats = preloader.get_stats().await;
        assert_eq!(stats.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_pattern_learning() {
        let preloader = ResourcePreloader::new();
        
        preloader.learn_pattern(
            "https://example.com",
            "https://example.com/style.css",
        ).await;

        let hints = preloader.get_predictive_hints("https://example.com").await;
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0], "https://example.com/style.css");
    }
}
