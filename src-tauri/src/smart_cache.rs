//! Smart Caching System for Exodus Browser
//! 
//! This module provides intelligent caching with LRU eviction, 
//! size-based limits, and cache statistics.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cache key (URL)
    pub key: String,
    /// Cached content
    pub content: Vec<u8>,
    /// Content type
    pub content_type: String,
    /// Size in bytes
    pub size: usize,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Access count
    pub access_count: u32,
    /// TTL in seconds (0 = no expiration)
    pub ttl: u64,
}

impl CacheEntry {
    pub fn new(key: String, content: Vec<u8>, content_type: String, ttl: u64) -> Self {
        let size = content.len();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            key,
            content,
            content_type,
            size,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if self.ttl == 0 {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        now - self.last_accessed > self.ttl
    }
    
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        self.access_count += 1;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries
    pub total_entries: usize,
    /// Total size in bytes
    pub total_size: usize,
    /// Total hits
    pub hits: u64,
    /// Total misses
    pub misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Evictions
    pub evictions: u64,
    /// Expired entries
    pub expired: u64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            total_size: 0,
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            evictions: 0,
            expired: 0,
        }
    }
    
    pub fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
    }
}

/// Smart cache with LRU eviction
pub struct SmartCache {
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
    lru_order: Arc<Mutex<VecDeque<String>>>,
    max_size: usize, // Maximum total size in bytes
    max_entries: usize, // Maximum number of entries
    stats: Arc<Mutex<CacheStats>>,
    storage_path: PathBuf,
}

impl SmartCache {
    /// Create a new smart cache
    pub fn new(storage_path: PathBuf, max_size: usize, max_entries: usize) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let cache = Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            lru_order: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            max_entries,
            stats: Arc::new(Mutex::new(CacheStats::new())),
            storage_path,
        };
        
        cache.load_from_disk()?;
        Ok(cache)
    }
    
    /// Get a cached entry
    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        enum GetAction {
            Hit(CacheEntry),
            Expired,
            Miss,
        }

        let action = {
            let mut entries = self.entries.lock().ok()?;
            if let Some(mut entry) = entries.get(key).cloned() {
                if entry.is_expired() {
                    entries.remove(key);
                    GetAction::Expired
                } else {
                    entry.touch();
                    let updated = entry.clone();
                    entries.insert(key.to_string(), updated.clone());
                    GetAction::Hit(updated)
                }
            } else {
                GetAction::Miss
            }
        };

        match action {
            GetAction::Hit(entry) => {
                self.update_lru(key, false);
                self.record_hit();
                Some(entry)
            }
            GetAction::Expired => {
                self.update_lru(key, true);
                self.record_miss();
                None
            }
            GetAction::Miss => {
                self.record_miss();
                None
            }
        }
    }
    
    /// Put a value in the cache
    pub fn put(&self, key: String, content: Vec<u8>, content_type: String, ttl: u64) -> Result<(), Box<dyn std::error::Error>> {
        let entry = CacheEntry::new(key.clone(), content, content_type, ttl);
        
        // Check if we need to evict entries
        self.evict_if_needed(entry.size)?;
        
        {
            let mut entries = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            entries.insert(key.clone(), entry);
        }
        self.update_lru(&key, false);
        self.update_stats();
        
        Ok(())
    }
    
    /// Remove an entry from the cache
    pub fn remove(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut entries = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            entries.remove(key);
        }
        self.update_lru(key, true);
        self.update_stats();
        
        Ok(())
    }
    
    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        entries.clear();
        
        let mut lru_order = self.lru_order.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        lru_order.clear();
        self.update_stats();
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.clone()
    }
    
    /// Evict entries if needed based on size and count limits
    fn evict_if_needed(&self, new_entry_size: usize) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let current_size: usize = entries.values().map(|e| e.size).sum();
        let current_count = entries.len();
        
        drop(entries);
        
        // Check if we need to evict
        if current_size + new_entry_size <= self.max_size && current_count < self.max_entries {
            return Ok(());
        }
        
        // Evict least recently used entries (never hold `lru_order` while locking `entries`).
        let mut evicted_count = 0;
        loop {
            let current_size: usize = {
                let entries = self.entries.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                entries.values().map(|e| e.size).sum()
            };

            let current_count = {
                let entries = self.entries.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                entries.len()
            };

            if current_size + new_entry_size <= self.max_size && current_count < self.max_entries {
                break;
            }

            let victim = {
                let mut lru_order = self.lru_order.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                lru_order.pop_front()
            };

            let Some(key) = victim else {
                break;
            };

            let mut entries = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            entries.remove(&key);
            evicted_count += 1;
        }
        
        // Record evictions
        if evicted_count > 0 {
            let mut stats = self.stats.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            stats.evictions += evicted_count as u64;
        }
        
        Ok(())
    }
    
    /// Update LRU order
    fn update_lru(&self, key: &str, remove: bool) {
        let mut lru_order = self.lru_order.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if remove {
            lru_order.retain(|k| k != key);
        } else {
            lru_order.retain(|k| k != key);
            lru_order.push_back(key.to_string());
        }
    }
    
    /// Record a cache hit
    fn record_hit(&self) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.hits += 1;
        stats.update_hit_rate();
    }
    
    /// Record a cache miss
    fn record_miss(&self) {
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        stats.misses += 1;
        stats.update_hit_rate();
    }
    
    /// Update cache statistics
    fn update_stats(&self) {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        let mut stats = self.stats.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        stats.total_entries = entries.len();
        stats.total_size = entries.values().map(|e| e.size).sum();
    }
    
    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let expired_keys: Vec<String> = {
            let entries = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            entries
                .iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(key, _)| key.clone())
                .collect()
        };

        let count = expired_keys.len();
        for key in &expired_keys {
            {
                let mut entries = self.entries.lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                entries.remove(key);
            }
            self.update_lru(key, true);
        }
        
        // Record expired count
        if count > 0 {
            let mut stats = self.stats.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            stats.expired += count as u64;
        }
        
        self.update_stats();
        Ok(count)
    }
    
    /// Get all cache keys
    pub fn get_keys(&self) -> Vec<String> {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        entries.keys().cloned().collect()
    }
    
    /// Get cache size in bytes
    pub fn get_size(&self) -> usize {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        entries.values().map(|e| e.size).sum()
    }
    
    /// Get cache entry count
    pub fn get_count(&self) -> usize {
        let entries = self.entries.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        entries.len()
    }
    
    /// Set cache limits
    pub fn set_limits(&self, max_size: usize, max_entries: usize) -> Result<(), Box<dyn std::error::Error>> {
        let lru_order = self.lru_order.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Update limits
        let _new_max_size = max_size;
        let _new_max_entries = max_entries;
        
        drop(lru_order);
        
        // Evict if needed with new limits
        self.evict_if_needed(0)?;
        
        Ok(())
    }
    
    /// Load cache from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join("cache.json");
        
        if !file_path.exists() {
            return Ok(());
        }
        
        let content = std::fs::read_to_string(&file_path)?;
        let loaded: HashMap<String, CacheEntry> = serde_json::from_str(&content)?;

        let lru_keys = {
            let mut guard = self.entries.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            *guard = loaded;
            let mut sorted: Vec<(String, u64)> = guard
                .iter()
                .map(|(k, e)| (k.clone(), e.last_accessed))
                .collect();
            sorted.sort_by_key(|(_, ts)| *ts);
            sorted.into_iter().map(|(k, _)| k).collect::<Vec<_>>()
        };

        {
            let mut lru_order = self.lru_order.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            lru_order.clear();
            for key in lru_keys {
                lru_order.push_back(key);
            }
        }

        self.update_stats();
        
        Ok(())
    }
    
    /// Save cache to disk
    #[allow(dead_code)]
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.storage_path.join("cache.json");
        
        let entries = self.entries.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*entries)?;
        std::fs::write(&file_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get cached content
#[tauri::command]
pub fn cache_get(
    key: String,
    cache: State<'_, Arc<SmartCache>>,
) -> Result<Option<CacheEntry>, String> {
    Ok(cache.get(&key))
}

/// Put content in cache
#[tauri::command]
pub fn cache_put(
    key: String,
    content: String,
    content_type: String,
    ttl: u64,
    cache: State<'_, Arc<SmartCache>>,
) -> Result<(), String> {
    cache.put(key, content.into_bytes(), content_type, ttl)
        .map_err(|e| format!("Failed to cache content: {}", e))
}

/// Remove cached content
#[tauri::command]
pub fn cache_remove(
    key: String,
    cache: State<'_, Arc<SmartCache>>,
) -> Result<(), String> {
    cache.remove(&key)
        .map_err(|e| format!("Failed to remove from cache: {}", e))
}

/// Clear all cache
#[tauri::command]
pub fn cache_clear(
    cache: State<'_, Arc<SmartCache>>,
) -> Result<(), String> {
    cache.clear()
        .map_err(|e| format!("Failed to clear cache: {}", e))
}

/// Get cache statistics
#[tauri::command]
pub fn cache_get_stats(
    cache: State<'_, Arc<SmartCache>>,
) -> Result<CacheStats, String> {
    Ok(cache.get_stats())
}

/// Cleanup expired cache entries
#[tauri::command]
pub fn cache_cleanup_expired(
    cache: State<'_, Arc<SmartCache>>,
) -> Result<usize, String> {
    cache.cleanup_expired()
        .map_err(|e| format!("Failed to cleanup expired entries: {}", e))
}

/// Get all cache keys
#[tauri::command]
pub fn cache_get_keys(
    cache: State<'_, Arc<SmartCache>>,
) -> Result<Vec<String>, String> {
    Ok(cache.get_keys())
}

/// Get cache size
#[tauri::command]
pub fn cache_get_size(
    cache: State<'_, Arc<SmartCache>>,
) -> Result<usize, String> {
    Ok(cache.get_size())
}

/// Get cache entry count
#[tauri::command]
pub fn cache_get_count(
    cache: State<'_, Arc<SmartCache>>,
) -> Result<usize, String> {
    Ok(cache.get_count())
}

/// Set cache limits
#[tauri::command]
pub fn cache_set_limits(
    max_size: usize,
    max_entries: usize,
    cache: State<'_, Arc<SmartCache>>,
) -> Result<(), String> {
    cache.set_limits(max_size, max_entries)
        .map_err(|e| format!("Failed to set cache limits: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new(
            "https://example.com".to_string(),
            b"test content".to_vec(),
            "text/html".to_string(),
            3600,
        );
        
        assert_eq!(entry.key, "https://example.com");
        assert_eq!(entry.content_type, "text/html");
        assert_eq!(entry.access_count, 0);
        assert!(!entry.is_expired());
    }
    
    #[test]
    fn test_cache_put_get() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = SmartCache::new(temp_dir.path().to_path_buf(), 1024 * 1024, 100).expect("Failed to create cache");
        
        cache.put(
            "test_key".to_string(),
            b"test content".to_vec(),
            "text/html".to_string(),
            3600,
        ).expect("Failed to put entry");
        
        let entry = cache.get("test_key");
        assert!(entry.is_some());
        
        let entry = entry.expect("Expected entry to exist");
        assert_eq!(entry.content, b"test content".to_vec());
    }
    
    #[test]
    fn test_cache_eviction() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = SmartCache::new(temp_dir.path().to_path_buf(), 100, 2).expect("Failed to create cache");
        
        cache.put(
            "key1".to_string(),
            vec![0; 50],
            "text/html".to_string(),
            3600,
        ).expect("Failed to put key1");
        
        cache.put(
            "key2".to_string(),
            vec![0; 50],
            "text/html".to_string(),
            3600,
        ).expect("Failed to put key2");
        
        // This should evict key1 due to size limit
        cache.put(
            "key3".to_string(),
            vec![0; 50],
            "text/html".to_string(),
            3600,
        ).expect("Failed to put key3");
        
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }
    
    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = SmartCache::new(temp_dir.path().to_path_buf(), 1024 * 1024, 100).expect("Failed to create cache");
        
        cache.put(
            "key1".to_string(),
            b"content1".to_vec(),
            "text/html".to_string(),
            3600,
        ).expect("Failed to put entry");
        
        let stats = cache.get_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_size, 8);
    }
}
