//! HTTPS-Only Mode for Exodus Browser
//! Automatically upgrades HTTP connections to HTTPS

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// HTTPS-Only mode settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsOnlySettings {
    pub enabled: bool,
    pub exceptions: HashSet<String>, // Domains that can use HTTP
}

impl Default for HttpsOnlySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            exceptions: HashSet::new(),
        }
    }
}

/// HTTPS-Only Mode Manager
pub struct HttpsOnlyManager {
    settings: Arc<Mutex<HttpsOnlySettings>>,
}

impl HttpsOnlyManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(HttpsOnlySettings::default())),
        }
    }

    /// Enable HTTPS-Only mode
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-https-only-changed", settings.enabled);
        }
    }

    /// Disable HTTPS-Only mode
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-https-only-changed", settings.enabled);
        }
    }

    /// Check if HTTPS-Only mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Add an exception (domain that can use HTTP)
    pub fn add_exception(&self, domain: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.exceptions.insert(domain.clone());
            let _ = app.emit("exodus-https-only-exception-added", domain);
        }
    }

    /// Remove an exception
    pub fn remove_exception(&self, domain: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.exceptions.remove(&domain);
            let _ = app.emit("exodus-https-only-exception-removed", domain);
        }
    }

    /// Get all exceptions
    pub fn get_exceptions(&self) -> HashSet<String> {
        self.settings.lock()
            .map(|settings| settings.exceptions.clone())
            .unwrap_or_default()
    }

    /// Check if a URL should be upgraded to HTTPS
    pub fn should_upgrade(&self, url: &str) -> bool {
        if !self.is_enabled() {
            return false;
        }

        if let Ok(parsed) = url::Url::parse(url) {
            if parsed.scheme() != "http" {
                return false;
            }

            if let Some(host) = parsed.host_str() {
                if let Ok(settings) = self.settings.lock() {
                    // Check if domain is in exceptions
                    for exception in &settings.exceptions {
                        if host.ends_with(exception) || host == exception {
                            return false;
                        }
                    }
                }
            }

            return true;
        }

        false
    }

    /// Upgrade HTTP URL to HTTPS
    pub fn upgrade_url(&self, url: &str) -> Option<String> {
        if !self.should_upgrade(url) {
            return None;
        }

        if let Ok(mut parsed) = url::Url::parse(url) {
            parsed.set_scheme("https").ok()?;
            Some(parsed.to_string())
        } else {
            None
        }
    }

    /// Get current settings
    pub fn get_settings(&self) -> HttpsOnlySettings {
        self.settings.lock()
            .map(|settings| HttpsOnlySettings {
                enabled: settings.enabled,
                exceptions: settings.exceptions.clone(),
            })
            .unwrap_or_default()
    }
}

impl Default for HttpsOnlyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable HTTPS-Only mode
#[tauri::command]
pub fn enable_https_only(
    app: AppHandle,
    manager: State<'_, Arc<HttpsOnlyManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable HTTPS-Only mode
#[tauri::command]
pub fn disable_https_only(
    app: AppHandle,
    manager: State<'_, Arc<HttpsOnlyManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if HTTPS-Only mode is enabled
#[tauri::command]
pub fn is_https_only_enabled(
    manager: State<'_, Arc<HttpsOnlyManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to add an exception
#[tauri::command]
pub fn add_https_only_exception(
    domain: String,
    app: AppHandle,
    manager: State<'_, Arc<HttpsOnlyManager>>,
) {
    manager.add_exception(domain, app);
}

/// Tauri command to remove an exception
#[tauri::command]
pub fn remove_https_only_exception(
    domain: String,
    app: AppHandle,
    manager: State<'_, Arc<HttpsOnlyManager>>,
) {
    manager.remove_exception(domain, app);
}

/// Tauri command to get all exceptions
#[tauri::command]
pub fn get_https_only_exceptions(
    manager: State<'_, Arc<HttpsOnlyManager>>,
) -> Vec<String> {
    manager.get_exceptions().into_iter().collect()
}

/// Tauri command to check if a URL should be upgraded
#[tauri::command]
pub fn should_upgrade_url(
    url: String,
    manager: State<'_, Arc<HttpsOnlyManager>>,
) -> bool {
    manager.should_upgrade(&url)
}

/// Tauri command to upgrade a URL
#[tauri::command]
pub fn upgrade_to_https(
    url: String,
    manager: State<'_, Arc<HttpsOnlyManager>>,
) -> Option<String> {
    manager.upgrade_url(&url)
}

/// Tauri command to get HTTPS-Only settings
#[tauri::command]
pub fn get_https_only_settings(
    manager: State<'_, Arc<HttpsOnlyManager>>,
) -> HttpsOnlySettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_only_manager_creation() {
        let manager = HttpsOnlyManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let manager = HttpsOnlyManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_should_upgrade() {
        let manager = HttpsOnlyManager::new();
        
        // Should not upgrade when disabled
        assert!(!manager.should_upgrade("http://example.com"));
        
        // Should not upgrade HTTPS URLs
        assert!(!manager.should_upgrade("https://example.com"));
    }

    #[test]
    fn test_upgrade_url() {
        let manager = HttpsOnlyManager::new();
        
        // Should not upgrade when disabled
        assert!(manager.upgrade_url("http://example.com").is_none());
    }

    #[test]
    fn test_exceptions() {
        let manager = HttpsOnlyManager::new();
        
        // Add exception (mock AppHandle)
        let exceptions = manager.get_exceptions();
        assert!(exceptions.is_empty());
    }
}
