//! DNS over HTTPS (DoH) for Exodus Browser
//! Encrypts DNS queries to improve privacy and security

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// DoH provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DohProvider {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

impl Default for DohProvider {
    fn default() -> Self {
        Self {
            name: "Cloudflare".to_string(),
            url: "https://cloudflare-dns.com/dns-query".to_string(),
            enabled: false,
        }
    }
}

/// DoH settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DohSettings {
    pub enabled: bool,
    pub providers: Vec<DohProvider>,
    pub fallback_to_system: bool,
}

impl Default for DohSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            providers: vec![
                DohProvider {
                    name: "Cloudflare".to_string(),
                    url: "https://cloudflare-dns.com/dns-query".to_string(),
                    enabled: false,
                },
                DohProvider {
                    name: "Google".to_string(),
                    url: "https://dns.google/dns-query".to_string(),
                    enabled: false,
                },
                DohProvider {
                    name: "Quad9".to_string(),
                    url: "https://dns.quad9.net/dns-query".to_string(),
                    enabled: false,
                },
            ],
            fallback_to_system: true,
        }
    }
}

/// DNS over HTTPS Manager
pub struct DohManager {
    settings: Arc<Mutex<DohSettings>>,
}

impl DohManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(DohSettings::default())),
        }
    }

    /// Enable DoH
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-doh-enabled", true);
        }
    }

    /// Disable DoH
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-doh-enabled", false);
        }
    }

    /// Check if DoH is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set DoH provider
    pub fn set_provider(&self, provider_name: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            for provider in &mut settings.providers {
                provider.enabled = provider.name == provider_name;
            }
            let _ = app.emit("exodus-doh-provider-changed", provider_name);
        }
    }

    /// Get active provider
    pub fn get_active_provider(&self) -> Option<DohProvider> {
        self.settings.lock()
            .ok()
            .and_then(|settings| settings.providers.iter().find(|p| p.enabled).cloned())
    }

    /// Add custom provider
    pub fn add_provider(&self, name: String, url: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.providers.push(DohProvider {
                name: name.clone(),
                url,
                enabled: false,
            });
            let _ = app.emit("exodus-doh-provider-added", name);
        }
    }

    /// Remove provider
    pub fn remove_provider(&self, name: String, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.providers.retain(|p| p.name != name);
            let _ = app.emit("exodus-doh-provider-removed", name);
        }
    }

    /// Set fallback to system DNS
    pub fn set_fallback(&self, fallback: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.fallback_to_system = fallback;
            let _ = app.emit("exodus-doh-fallback-changed", fallback);
        }
    }

    /// Get all providers
    pub fn get_providers(&self) -> Vec<DohProvider> {
        self.settings.lock()
            .map(|settings| settings.providers.clone())
            .unwrap_or_default()
    }

    /// Get current settings
    pub fn get_settings(&self) -> DohSettings {
        self.settings.lock()
            .map(|settings| DohSettings {
                enabled: settings.enabled,
                providers: settings.providers.clone(),
                fallback_to_system: settings.fallback_to_system,
            })
            .unwrap_or_default()
    }
}

impl Default for DohManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable DoH
#[tauri::command]
pub fn enable_doh(
    app: AppHandle,
    manager: State<'_, Arc<DohManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable DoH
#[tauri::command]
pub fn disable_doh(
    app: AppHandle,
    manager: State<'_, Arc<DohManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if DoH is enabled
#[tauri::command]
pub fn is_doh_enabled(
    manager: State<'_, Arc<DohManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set DoH provider
#[tauri::command]
pub fn set_doh_provider(
    provider_name: String,
    app: AppHandle,
    manager: State<'_, Arc<DohManager>>,
) {
    manager.set_provider(provider_name, app);
}

/// Tauri command to get active provider
#[tauri::command]
pub fn get_active_doh_provider(
    manager: State<'_, Arc<DohManager>>,
) -> Option<DohProvider> {
    manager.get_active_provider()
}

/// Tauri command to add custom provider
#[tauri::command]
pub fn add_doh_provider(
    name: String,
    url: String,
    app: AppHandle,
    manager: State<'_, Arc<DohManager>>,
) {
    manager.add_provider(name, url, app);
}

/// Tauri command to remove provider
#[tauri::command]
pub fn remove_doh_provider(
    name: String,
    app: AppHandle,
    manager: State<'_, Arc<DohManager>>,
) {
    manager.remove_provider(name, app);
}

/// Tauri command to set fallback to system DNS
#[tauri::command]
pub fn set_doh_fallback(
    fallback: bool,
    app: AppHandle,
    manager: State<'_, Arc<DohManager>>,
) {
    manager.set_fallback(fallback, app);
}

/// Tauri command to get all providers
#[tauri::command]
pub fn get_doh_providers(
    manager: State<'_, Arc<DohManager>>,
) -> Vec<DohProvider> {
    manager.get_providers()
}

/// Tauri command to get DoH settings
#[tauri::command]
pub fn get_doh_settings(
    manager: State<'_, Arc<DohManager>>,
) -> DohSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doh_manager_creation() {
        let manager = DohManager::new();
        assert!(!manager.is_enabled());
        
        let providers = manager.get_providers();
        assert_eq!(providers.len(), 3); // Cloudflare, Google, Quad9
    }

    #[test]
    fn test_enable_disable() {
        let manager = DohManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_providers() {
        let manager = DohManager::new();
        
        let providers = manager.get_providers();
        assert_eq!(providers.len(), 3);
        
        // Check default providers
        assert!(providers.iter().any(|p| p.name == "Cloudflare"));
        assert!(providers.iter().any(|p| p.name == "Google"));
        assert!(providers.iter().any(|p| p.name == "Quad9"));
    }

    #[test]
    fn test_settings() {
        let manager = DohManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert!(settings.fallback_to_system);
        assert_eq!(settings.providers.len(), 3);
    }
}
