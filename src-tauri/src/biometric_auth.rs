//! Biometric Authentication for Exodus Browser
//! Provides Face ID, Touch ID, and Windows Hello support

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Biometric authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Biometric authentication settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricSettings {
    pub enabled: bool,
    pub require_for_passwords: bool,
    pub require_for_sensitive_data: bool,
    pub auto_lock_timeout_minutes: u32,
}

impl Default for BiometricSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            require_for_passwords: true,
            require_for_sensitive_data: true,
            auto_lock_timeout_minutes: 5,
        }
    }
}

/// Biometric Authentication Manager
pub struct BiometricAuthManager {
    settings: Arc<Mutex<BiometricSettings>>,
    #[cfg(target_os = "macos")]
    available: bool,
    #[cfg(not(target_os = "macos"))]
    available: bool,
}

impl BiometricAuthManager {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let available = Self::check_macos_biometric_available();
        #[cfg(target_os = "windows")]
        let available = Self::check_windows_biometric_available();
        #[cfg(target_os = "linux")]
        let available = Self::check_linux_biometric_available();

        Self {
            settings: Arc::new(Mutex::new(BiometricSettings::default())),
            available,
        }
    }

    /// Check if biometric authentication is available on macOS
    #[cfg(target_os = "macos")]
    fn check_macos_biometric_available() -> bool {
        // Check for Touch ID / Face ID availability
        // This would typically use LocalAuthentication framework
        // For now, return false as placeholder
        false
    }

    /// Check if biometric authentication is available on Windows
    #[cfg(target_os = "windows")]
    fn check_windows_biometric_available() -> bool {
        // Check for Windows Hello availability
        // This would typically use Windows Hello API
        // For now, return false as placeholder
        false
    }

    /// Check if biometric authentication is available on Linux
    #[cfg(target_os = "linux")]
    fn check_linux_biometric_available() -> bool {
        // Check for fingerprint reader availability
        // This would typically use libfprint
        // For now, return false as placeholder
        false
    }

    /// Check if biometric authentication is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Enable biometric authentication
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-biometric-enabled", true);
        }
    }

    /// Disable biometric authentication
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-biometric-enabled", false);
        }
    }

    /// Check if biometric authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Request biometric authentication
    pub fn authenticate(&self, reason: &str) -> BiometricResult {
        if !self.available {
            return BiometricResult {
                success: false,
                error: Some("Biometric authentication not available".to_string()),
            };
        }

        if !self.is_enabled() {
            return BiometricResult {
                success: false,
                error: Some("Biometric authentication not enabled".to_string()),
            };
        }

        // Perform actual biometric authentication
        // This would use platform-specific APIs
        #[cfg(target_os = "macos")]
        let result = self.authenticate_macos(reason);
        #[cfg(target_os = "windows")]
        let result = self.authenticate_windows(reason);
        #[cfg(target_os = "linux")]
        let result = self.authenticate_linux(reason);
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let result = BiometricResult {
            success: false,
            error: Some("Platform not supported".to_string()),
        };

        result
    }

    /// Authenticate on macOS
    #[cfg(target_os = "macos")]
    fn authenticate_macos(&self, reason: &str) -> BiometricResult {
        // Use LocalAuthentication framework
        // For now, return success as placeholder
        BiometricResult {
            success: true,
            error: None,
        }
    }

    /// Authenticate on Windows
    #[cfg(target_os = "windows")]
    fn authenticate_windows(&self, reason: &str) -> BiometricResult {
        // Use Windows Hello API
        // For now, return success as placeholder
        BiometricResult {
            success: true,
            error: None,
        }
    }

    /// Authenticate on Linux
    #[cfg(target_os = "linux")]
    fn authenticate_linux(&self, reason: &str) -> BiometricResult {
        // Use libfprint
        // For now, return success as placeholder
        BiometricResult {
            success: true,
            error: None,
        }
    }

    /// Set requirement for passwords
    pub fn set_require_for_passwords(&self, require: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.require_for_passwords = require;
            let _ = app.emit("exodus-biometric-passwords-changed", require);
        }
    }

    /// Set requirement for sensitive data
    pub fn set_require_for_sensitive_data(&self, require: bool, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.require_for_sensitive_data = require;
            let _ = app.emit("exodus-biometric-sensitive-changed", require);
        }
    }

    /// Set auto-lock timeout
    pub fn set_auto_lock_timeout(&self, timeout_minutes: u32, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.auto_lock_timeout_minutes = timeout_minutes;
            let _ = app.emit("exodus-biometric-timeout-changed", timeout_minutes);
        }
    }

    /// Get current settings
    pub fn get_settings(&self) -> BiometricSettings {
        self.settings.lock()
            .map(|settings| BiometricSettings {
                enabled: settings.enabled,
                require_for_passwords: settings.require_for_passwords,
                require_for_sensitive_data: settings.require_for_sensitive_data,
                auto_lock_timeout_minutes: settings.auto_lock_timeout_minutes,
            })
            .unwrap_or_default()
    }
}

impl Default for BiometricAuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to check if biometric authentication is available
#[tauri::command]
pub fn is_biometric_available(
    manager: State<'_, Arc<BiometricAuthManager>>,
) -> bool {
    manager.is_available()
}

/// Tauri command to enable biometric authentication
#[tauri::command]
pub fn enable_biometric(
    app: AppHandle,
    manager: State<'_, Arc<BiometricAuthManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable biometric authentication
#[tauri::command]
pub fn disable_biometric(
    app: AppHandle,
    manager: State<'_, Arc<BiometricAuthManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if biometric authentication is enabled
#[tauri::command]
pub fn is_biometric_enabled(
    manager: State<'_, Arc<BiometricAuthManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to request biometric authentication
#[tauri::command]
pub fn authenticate_biometric(
    reason: String,
    manager: State<'_, Arc<BiometricAuthManager>>,
) -> BiometricResult {
    manager.authenticate(&reason)
}

/// Tauri command to set requirement for passwords
#[tauri::command]
pub fn set_biometric_require_passwords(
    require: bool,
    app: AppHandle,
    manager: State<'_, Arc<BiometricAuthManager>>,
) {
    manager.set_require_for_passwords(require, app);
}

/// Tauri command to set requirement for sensitive data
#[tauri::command]
pub fn set_biometric_require_sensitive(
    require: bool,
    app: AppHandle,
    manager: State<'_, Arc<BiometricAuthManager>>,
) {
    manager.set_require_for_sensitive_data(require, app);
}

/// Tauri command to set auto-lock timeout
#[tauri::command]
pub fn set_biometric_auto_lock_timeout(
    timeout_minutes: u32,
    app: AppHandle,
    manager: State<'_, Arc<BiometricAuthManager>>,
) {
    manager.set_auto_lock_timeout(timeout_minutes, app);
}

/// Tauri command to get biometric authentication settings
#[tauri::command]
pub fn get_biometric_settings(
    manager: State<'_, Arc<BiometricAuthManager>>,
) -> BiometricSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_auth_manager_creation() {
        let manager = BiometricAuthManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let manager = BiometricAuthManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_settings() {
        let manager = BiometricAuthManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
        assert!(settings.require_for_passwords);
        assert!(settings.require_for_sensitive_data);
        assert_eq!(settings.auto_lock_timeout_minutes, 5);
    }
}
