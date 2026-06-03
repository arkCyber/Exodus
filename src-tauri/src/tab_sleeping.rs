//! Tab Sleeping for Memory Optimization
//!
//! Automatically suspend inactive tabs to reduce memory usage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Tab sleep state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabSleepState {
    /// Tab is active and running
    Active,
    /// Tab is sleeping (suspended)
    Sleeping,
    /// Tab is pinned and won't sleep
    Pinned,
}

/// Tab metadata for sleep management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabMetadata {
    pub tab_id: String,
    pub label: String,
    pub url: String,
    pub title: String,
    pub state: TabSleepState,
    pub last_active: u64,
    pub memory_estimate_mb: f64,
    pub is_pinned: bool,
    pub is_playing_audio: bool,
    pub is_playing_video: bool,
}

impl TabMetadata {
    pub fn new(tab_id: impl Into<String>, label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            tab_id: tab_id.into(),
            label: label.into(),
            url: url.into(),
            title: String::new(),
            state: TabSleepState::Active,
            last_active: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            memory_estimate_mb: 0.0,
            is_pinned: false,
            is_playing_audio: false,
            is_playing_video: false,
        }
    }

    pub fn should_sleep(&self, inactive_threshold_secs: u64) -> bool {
        if self.is_pinned || self.is_playing_audio || self.is_playing_video {
            return false;
        }
        
        if self.state == TabSleepState::Sleeping {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        now - self.last_active > inactive_threshold_secs
    }
}

/// Tab sleeping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSleepConfig {
    /// Enable tab sleeping
    pub enabled: bool,
    /// Time in seconds before a tab is put to sleep
    pub inactive_threshold_secs: u64,
    /// Maximum number of active tabs before forcing sleep
    pub max_active_tabs: usize,
    /// Exclude tabs playing media
    pub exclude_media: bool,
    /// Exclude pinned tabs
    pub exclude_pinned: bool,
}

impl Default for TabSleepConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            inactive_threshold_secs: 300, // 5 minutes
            max_active_tabs: 10,
            exclude_media: true,
            exclude_pinned: true,
        }
    }
}

/// Tab sleeping manager
pub struct TabSleepManager {
    tabs: Arc<RwLock<HashMap<String, TabMetadata>>>,
    config: Arc<RwLock<TabSleepConfig>>,
}

impl TabSleepManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(TabSleepConfig::default())),
        }
    }

    pub fn with_config(config: TabSleepConfig) -> Self {
        Self {
            tabs: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Register a tab for sleep management
    pub async fn register_tab(&self, mut metadata: TabMetadata) {
        if metadata.is_pinned {
            metadata.state = TabSleepState::Pinned;
        }
        
        let mut tabs = self.tabs.write().await;
        tabs.insert(metadata.tab_id.clone(), metadata);
    }

    /// Unregister a tab (when closed)
    pub async fn unregister_tab(&self, tab_id: &str) {
        let mut tabs = self.tabs.write().await;
        tabs.remove(tab_id);
    }

    /// Mark a tab as active (user switched to it)
    pub async fn mark_active(&self, tab_id: &str) {
        let mut tabs = self.tabs.write().await;
        
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.last_active = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            
            if tab.state == TabSleepState::Sleeping {
                tab.state = TabSleepState::Active;
            }
        }
    }

    /// Update tab metadata
    pub async fn update_tab(&self, tab_id: &str, update: impl FnOnce(&mut TabMetadata)) {
        let mut tabs = self.tabs.write().await;
        
        if let Some(tab) = tabs.get_mut(tab_id) {
            update(tab);
        }
    }

    /// Get tabs that should be put to sleep
    pub async fn get_tabs_to_sleep(&self) -> Vec<String> {
        let tabs = self.tabs.read().await;
        let config = self.config.read().await;
        
        if !config.enabled {
            return Vec::new();
        }
        
        let mut candidates: Vec<_> = tabs
            .values()
            .filter(|t| t.should_sleep(config.inactive_threshold_secs))
            .collect();
        
        // Sort by last active time (oldest first)
        candidates.sort_by_key(|t| t.last_active);
        
        // Count currently active tabs
        let active_count = tabs.values().filter(|t| t.state == TabSleepState::Active).count();
        
        // If we're over the limit, force sleep on oldest tabs
        let to_sleep_count = if active_count > config.max_active_tabs {
            active_count - config.max_active_tabs
        } else {
            candidates.len()
        };
        
        candidates
            .into_iter()
            .take(to_sleep_count)
            .map(|t| t.tab_id.clone())
            .collect()
    }

    /// Mark a tab as sleeping
    pub async fn mark_sleeping(&self, tab_id: &str) -> Result<(), String> {
        let mut tabs = self.tabs.write().await;
        
        if let Some(tab) = tabs.get_mut(tab_id) {
            if tab.is_pinned || tab.is_playing_audio || tab.is_playing_video {
                return Err("Cannot sleep this tab".to_string());
            }
            
            tab.state = TabSleepState::Sleeping;
            Ok(())
        } else {
            Err(format!("Tab {} not found", tab_id))
        }
    }

    /// Wake up a sleeping tab
    pub async fn wake_tab(&self, tab_id: &str) -> Result<(), String> {
        let mut tabs = self.tabs.write().await;
        
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.state = TabSleepState::Active;
            tab.last_active = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            Ok(())
        } else {
            Err(format!("Tab {} not found", tab_id))
        }
    }

    /// Get all tab metadata
    pub async fn get_all_tabs(&self) -> Vec<TabMetadata> {
        let tabs = self.tabs.read().await;
        tabs.values().cloned().collect()
    }

    /// Get tab statistics
    pub async fn get_stats(&self) -> TabSleepStats {
        let tabs = self.tabs.read().await;
        
        let total_tabs = tabs.len();
        let active_tabs = tabs.values().filter(|t| t.state == TabSleepState::Active).count();
        let sleeping_tabs = tabs.values().filter(|t| t.state == TabSleepState::Sleeping).count();
        let pinned_tabs = tabs.values().filter(|t| t.state == TabSleepState::Pinned).count();
        
        let total_memory_mb: f64 = tabs.values().map(|t| t.memory_estimate_mb).sum();
        let active_memory_mb: f64 = tabs
            .values()
            .filter(|t| t.state == TabSleepState::Active)
            .map(|t| t.memory_estimate_mb)
            .sum();
        
        TabSleepStats {
            total_tabs,
            active_tabs,
            sleeping_tabs,
            pinned_tabs,
            total_memory_mb,
            active_memory_mb,
            saved_memory_mb: total_memory_mb - active_memory_mb,
        }
    }

    /// Update configuration
    pub async fn update_config(&self, config: TabSleepConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> TabSleepConfig {
        self.config.read().await.clone()
    }
}

impl Default for TabSleepManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab sleeping statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSleepStats {
    pub total_tabs: usize,
    pub active_tabs: usize,
    pub sleeping_tabs: usize,
    pub pinned_tabs: usize,
    pub total_memory_mb: f64,
    pub active_memory_mb: f64,
    pub saved_memory_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tab_sleep_threshold() {
        let manager = TabSleepManager::new();
        
        let mut tab = TabMetadata::new("tab1", "label1", "https://example.com");
        tab.last_active = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            - 400; // 400 seconds ago
        
        manager.register_tab(tab).await;
        
        let to_sleep = manager.get_tabs_to_sleep().await;
        assert_eq!(to_sleep.len(), 1);
        assert_eq!(to_sleep[0], "tab1");
    }

    #[tokio::test]
    async fn test_pinned_tabs_dont_sleep() {
        let manager = TabSleepManager::new();
        
        let mut tab = TabMetadata::new("tab1", "label1", "https://example.com");
        tab.is_pinned = true;
        tab.last_active = 0; // Very old
        
        manager.register_tab(tab).await;
        
        let to_sleep = manager.get_tabs_to_sleep().await;
        assert_eq!(to_sleep.len(), 0);
    }

    #[tokio::test]
    async fn test_media_tabs_dont_sleep() {
        let manager = TabSleepManager::new();
        
        let mut tab = TabMetadata::new("tab1", "label1", "https://example.com");
        tab.is_playing_audio = true;
        tab.last_active = 0;
        
        manager.register_tab(tab).await;
        
        let to_sleep = manager.get_tabs_to_sleep().await;
        assert_eq!(to_sleep.len(), 0);
    }
}
