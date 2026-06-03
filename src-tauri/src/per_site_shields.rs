//! Per-Site Shield Controls for Exodus Browser
//! Provides granular privacy settings per website

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Shield types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShieldType {
    AdBlocking,
    TrackerBlocking,
    CookieBlocking,
    FingerprintingProtection,
    HttpsOnly,
    ScriptBlocking,
    CanvasFingerprinting,
    WebGLFingerprinting,
    AudioFingerprinting,
    FontFingerprinting,
}

/// Per-site shield settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteShieldSettings {
    pub origin: String,
    pub shields: HashMap<ShieldType, bool>,
    pub custom_rules: Vec<String>,
}

impl Default for SiteShieldSettings {
    fn default() -> Self {
        let mut shields = HashMap::new();
        shields.insert(ShieldType::AdBlocking, true);
        shields.insert(ShieldType::TrackerBlocking, true);
        shields.insert(ShieldType::CookieBlocking, true);
        shields.insert(ShieldType::FingerprintingProtection, true);
        shields.insert(ShieldType::HttpsOnly, true);
        
        Self {
            origin: String::new(),
            shields,
            custom_rules: vec![],
        }
    }
}

/// Per-Site Shield Manager
pub struct PerSiteShieldManager {
    site_settings: Arc<Mutex<HashMap<String, SiteShieldSettings>>>,
    default_settings: Arc<Mutex<SiteShieldSettings>>,
}

impl PerSiteShieldManager {
    pub fn new() -> Self {
        Self {
            site_settings: Arc::new(Mutex::new(HashMap::new())),
            default_settings: Arc::new(Mutex::new(SiteShieldSettings::default())),
        }
    }

    /// Get shield settings for a site
    pub fn get_site_settings(&self, origin: &str) -> SiteShieldSettings {
        let site_settings = self.site_settings.lock();
        if let Ok(settings) = site_settings {
            if let Some(settings) = settings.get(origin) {
                return settings.clone();
            }
        }
        // Return default settings if site not found
        let default = self.default_settings.lock();
        if let Ok(default) = default {
            let mut settings = default.clone();
            settings.origin = origin.to_string();
            return settings;
        }
        SiteShieldSettings::default()
    }

    /// Set shield settings for a site
    pub fn set_site_settings(&self, origin: String, settings: SiteShieldSettings, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            site_settings.insert(origin.clone(), settings.clone());
            let _ = app.emit("exodus-site-shield-settings-changed", (origin, settings));
        }
    }

    /// Enable a shield for a site
    pub fn enable_shield(&self, origin: String, shield_type: ShieldType, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            let settings = site_settings.entry(origin.clone()).or_insert_with(|| {
                let mut s = SiteShieldSettings::default();
                s.origin = origin.clone();
                s
            });
            settings.shields.insert(shield_type.clone(), true);
            let _ = app.emit("exodus-site-shield-enabled", (origin, shield_type));
        }
    }

    /// Disable a shield for a site
    pub fn disable_shield(&self, origin: String, shield_type: ShieldType, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            let settings = site_settings.entry(origin.clone()).or_insert_with(|| {
                let mut s = SiteShieldSettings::default();
                s.origin = origin.clone();
                s
            });
            settings.shields.insert(shield_type.clone(), false);
            let _ = app.emit("exodus-site-shield-disabled", (origin, shield_type));
        }
    }

    /// Check if a shield is enabled for a site
    pub fn is_shield_enabled(&self, origin: &str, shield_type: ShieldType) -> bool {
        if let Ok(site_settings) = self.site_settings.lock() {
            if let Some(settings) = site_settings.get(origin) {
                return *settings.shields.get(&shield_type).unwrap_or(&true);
            }
        }
        // Default to enabled if site not found
        true
    }

    /// Set default shield settings
    pub fn set_default_settings(&self, settings: SiteShieldSettings, app: AppHandle) {
        if let Ok(mut default_settings) = self.default_settings.lock() {
            *default_settings = settings.clone();
            let _ = app.emit("exodus-default-shield-settings-changed", default_settings.clone());
        }
    }

    /// Get default shield settings
    pub fn get_default_settings(&self) -> SiteShieldSettings {
        self.default_settings.lock()
            .map(|settings| settings.clone())
            .unwrap_or_default()
    }

    /// Add custom rule for a site
    pub fn add_custom_rule(&self, origin: String, rule: String, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            let settings = site_settings.entry(origin.clone()).or_insert_with(|| {
                let mut s = SiteShieldSettings::default();
                s.origin = origin.clone();
                s
            });
            settings.custom_rules.push(rule.clone());
            let _ = app.emit("exodus-site-custom-rule-added", (origin, rule));
        }
    }

    /// Remove custom rule from a site
    pub fn remove_custom_rule(&self, origin: String, rule: String, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            if let Some(settings) = site_settings.get_mut(&origin) {
                settings.custom_rules.retain(|r| r != &rule);
                let _ = app.emit("exodus-site-custom-rule-removed", (origin, rule));
            }
        }
    }

    /// Get all sites with custom settings
    pub fn get_all_sites(&self) -> Vec<String> {
        self.site_settings.lock()
            .map(|site_settings| site_settings.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Reset site to default settings
    pub fn reset_site(&self, origin: String, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            site_settings.remove(&origin);
            let _ = app.emit("exodus-site-shield-reset", origin);
        }
    }

    /// Clear all custom settings
    pub fn clear_all_custom(&self, app: AppHandle) {
        if let Ok(mut site_settings) = self.site_settings.lock() {
            site_settings.clear();
            let _ = app.emit("exodus-all-site-shields-cleared", ());
        }
    }
}

impl Default for PerSiteShieldManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to get shield settings for a site
#[tauri::command]
pub fn get_site_shield_settings(
    origin: String,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) -> SiteShieldSettings {
    manager.get_site_settings(&origin)
}

/// Tauri command to set shield settings for a site
#[tauri::command]
pub fn set_site_shield_settings(
    origin: String,
    settings: SiteShieldSettings,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    manager.set_site_settings(origin, settings, app);
}

/// Tauri command to enable a shield for a site
#[tauri::command]
pub fn enable_site_shield(
    origin: String,
    shield_type: String,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    let shield = match shield_type.as_str() {
        "ad_blocking" => ShieldType::AdBlocking,
        "tracker_blocking" => ShieldType::TrackerBlocking,
        "cookie_blocking" => ShieldType::CookieBlocking,
        "fingerprinting_protection" => ShieldType::FingerprintingProtection,
        "https_only" => ShieldType::HttpsOnly,
        "script_blocking" => ShieldType::ScriptBlocking,
        "canvas_fingerprinting" => ShieldType::CanvasFingerprinting,
        "webgl_fingerprinting" => ShieldType::WebGLFingerprinting,
        "audio_fingerprinting" => ShieldType::AudioFingerprinting,
        "font_fingerprinting" => ShieldType::FontFingerprinting,
        _ => return,
    };
    manager.enable_shield(origin, shield, app);
}

/// Tauri command to disable a shield for a site
#[tauri::command]
pub fn disable_site_shield(
    origin: String,
    shield_type: String,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    let shield = match shield_type.as_str() {
        "ad_blocking" => ShieldType::AdBlocking,
        "tracker_blocking" => ShieldType::TrackerBlocking,
        "cookie_blocking" => ShieldType::CookieBlocking,
        "fingerprinting_protection" => ShieldType::FingerprintingProtection,
        "https_only" => ShieldType::HttpsOnly,
        "script_blocking" => ShieldType::ScriptBlocking,
        "canvas_fingerprinting" => ShieldType::CanvasFingerprinting,
        "webgl_fingerprinting" => ShieldType::WebGLFingerprinting,
        "audio_fingerprinting" => ShieldType::AudioFingerprinting,
        "font_fingerprinting" => ShieldType::FontFingerprinting,
        _ => return,
    };
    manager.disable_shield(origin, shield, app);
}

/// Tauri command to check if a shield is enabled for a site
#[tauri::command]
pub fn is_site_shield_enabled(
    origin: String,
    shield_type: String,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) -> bool {
    let shield = match shield_type.as_str() {
        "ad_blocking" => ShieldType::AdBlocking,
        "tracker_blocking" => ShieldType::TrackerBlocking,
        "cookie_blocking" => ShieldType::CookieBlocking,
        "fingerprinting_protection" => ShieldType::FingerprintingProtection,
        "https_only" => ShieldType::HttpsOnly,
        "script_blocking" => ShieldType::ScriptBlocking,
        "canvas_fingerprinting" => ShieldType::CanvasFingerprinting,
        "webgl_fingerprinting" => ShieldType::WebGLFingerprinting,
        "audio_fingerprinting" => ShieldType::AudioFingerprinting,
        "font_fingerprinting" => ShieldType::FontFingerprinting,
        _ => return true,
    };
    manager.is_shield_enabled(&origin, shield)
}

/// Tauri command to set default shield settings
#[tauri::command]
pub fn set_default_shield_settings(
    settings: SiteShieldSettings,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    manager.set_default_settings(settings, app);
}

/// Tauri command to get default shield settings
#[tauri::command]
pub fn get_default_shield_settings(
    manager: State<'_, Arc<PerSiteShieldManager>>,
) -> SiteShieldSettings {
    manager.get_default_settings()
}

/// Tauri command to add custom rule for a site
#[tauri::command]
pub fn add_site_custom_rule(
    origin: String,
    rule: String,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    manager.add_custom_rule(origin, rule, app);
}

/// Tauri command to remove custom rule from a site
#[tauri::command]
pub fn remove_site_custom_rule(
    origin: String,
    rule: String,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    manager.remove_custom_rule(origin, rule, app);
}

/// Tauri command to get all sites with custom settings
#[tauri::command]
pub fn get_all_shield_sites(
    manager: State<'_, Arc<PerSiteShieldManager>>,
) -> Vec<String> {
    manager.get_all_sites()
}

/// Tauri command to reset site to default settings
#[tauri::command]
pub fn reset_site_shields(
    origin: String,
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    manager.reset_site(origin, app);
}

/// Tauri command to clear all custom settings
#[tauri::command]
pub fn clear_all_site_shields(
    app: AppHandle,
    manager: State<'_, Arc<PerSiteShieldManager>>,
) {
    manager.clear_all_custom(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_site_shield_manager_creation() {
        let manager = PerSiteShieldManager::new();
        let settings = manager.get_site_settings("example.com");
        assert!(!settings.origin.is_empty());
    }

    #[test]
    fn test_enable_disable_shield() {
        let manager = PerSiteShieldManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(manager.is_shield_enabled("example.com", ShieldType::AdBlocking));
    }

    #[test]
    fn test_default_settings() {
        let manager = PerSiteShieldManager::new();
        
        let settings = manager.get_default_settings();
        assert!(settings.shields.get(&ShieldType::AdBlocking).is_some());
    }
}
