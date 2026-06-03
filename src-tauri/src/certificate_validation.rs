//! Certificate Validation for Exodus Browser
//! 
//! This module provides SSL/TLS certificate validation and transparency.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tauri::State;

/// Certificate validation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Expired,
    NotYetValid,
    Revoked,
    SelfSigned,
    InvalidChain,
    HostnameMismatch,
    UnknownError(String),
}

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Subject
    pub subject: String,
    /// Issuer
    pub issuer: String,
    /// Valid from
    pub valid_from: u64,
    /// Valid to
    pub valid_to: u64,
    /// Serial number
    pub serial_number: String,
    /// Fingerprint
    pub fingerprint: String,
    /// Validation result
    pub validation_result: ValidationResult,
}

impl CertificateInfo {
    #[allow(dead_code)]
    pub fn new(subject: String, issuer: String, valid_from: u64, valid_to: u64) -> Self {
        Self {
            subject,
            issuer,
            valid_from,
            valid_to,
            serial_number: String::new(),
            fingerprint: String::new(),
            validation_result: ValidationResult::Valid,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        now > self.valid_to
    }
    
    pub fn is_not_yet_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        now < self.valid_from
    }
}

/// Certificate validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidationSettings {
    /// Enable strict validation
    pub strict_validation: bool,
    /// Allow self-signed certificates
    pub allow_self_signed: bool,
    /// Allow expired certificates
    pub allow_expired: bool,
    /// Enable certificate transparency
    pub enable_transparency: bool,
    /// Check CRL (Certificate Revocation List)
    pub check_crl: bool,
    /// Check OCSP (Online Certificate Status Protocol)
    pub check_ocsp: bool,
}

impl Default for CertificateValidationSettings {
    fn default() -> Self {
        Self {
            strict_validation: true,
            allow_self_signed: false,
            allow_expired: false,
            enable_transparency: true,
            check_crl: true,
            check_ocsp: true,
        }
    }
}

/// Certificate validation manager
pub struct CertificateValidationManager {
    settings: Arc<Mutex<CertificateValidationSettings>>,
    certificate_cache: Arc<Mutex<HashMap<String, CertificateInfo>>>,
    revoked_certificates: Arc<Mutex<HashMap<String, u64>>>,
    storage_path: PathBuf,
}

impl CertificateValidationManager {
    /// Create a new certificate validation manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(CertificateValidationSettings::default())),
            certificate_cache: Arc::new(Mutex::new(HashMap::new())),
            revoked_certificates: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Validate a certificate (simplified implementation)
    pub fn validate_certificate(&self, host: &str, cert_info: CertificateInfo) -> ValidationResult {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        // Check if expired
        if cert_info.is_expired() {
            if settings.allow_expired {
                return ValidationResult::Valid;
            }
            return ValidationResult::Expired;
        }
        
        // Check if not yet valid
        if cert_info.is_not_yet_valid() {
            return ValidationResult::NotYetValid;
        }
        
        // Check if revoked
        let revoked = self.revoked_certificates.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(&_revoked_at) = revoked.get(&cert_info.serial_number) {
            return ValidationResult::Revoked;
        }
        
        // Check hostname match (simplified)
        if !host.contains(&cert_info.subject) && !cert_info.subject.contains(host) {
            if settings.strict_validation {
                return ValidationResult::HostnameMismatch;
            }
        }
        
        ValidationResult::Valid
    }
    
    /// Cache certificate information
    pub fn cache_certificate(&self, host: String, cert_info: CertificateInfo) {
        let mut cache = self.certificate_cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        cache.insert(host, cert_info);
    }
    
    /// Get cached certificate
    pub fn get_cached_certificate(&self, host: &str) -> Option<CertificateInfo> {
        let cache = self.certificate_cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        cache.get(host).cloned()
    }
    
    /// Clear certificate cache
    pub fn clear_cache(&self) {
        let mut cache = self.certificate_cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        cache.clear();
    }
    
    /// Add revoked certificate
    pub fn add_revoked_certificate(&self, serial_number: String, revoked_at: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut revoked = self.revoked_certificates.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        revoked.insert(serial_number, revoked_at);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get revoked certificates
    pub fn get_revoked_certificates(&self) -> HashMap<String, u64> {
        let revoked = self.revoked_certificates.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        revoked.clone()
    }
    
    /// Get certificate validation settings
    pub fn get_settings(&self) -> CertificateValidationSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update certificate validation settings
    pub fn update_settings(&self, settings: CertificateValidationSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get certificate cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.certificate_cache.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        let revoked = self.revoked_certificates.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        (cache.len(), revoked.len())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("certificate_settings.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: CertificateValidationSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("certificate_settings.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&settings_path, content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Validate a certificate
#[tauri::command]
pub fn validate_certificate(
    host: String,
    cert_info: CertificateInfo,
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<ValidationResult, String> {
    Ok(manager.validate_certificate(&host, cert_info))
}

/// Cache certificate information
#[tauri::command]
pub fn cache_certificate(
    host: String,
    cert_info: CertificateInfo,
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<(), String> {
    manager.cache_certificate(host, cert_info);
    Ok(())
}

/// Get cached certificate
#[tauri::command]
pub fn get_cached_certificate(
    host: String,
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<Option<CertificateInfo>, String> {
    Ok(manager.get_cached_certificate(&host))
}

/// Clear certificate cache
#[tauri::command]
pub fn clear_certificate_cache(
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<(), String> {
    manager.clear_cache();
    Ok(())
}

/// Add revoked certificate
#[tauri::command]
pub fn add_revoked_certificate(
    serial_number: String,
    revoked_at: u64,
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<(), String> {
    manager.add_revoked_certificate(serial_number, revoked_at)
        .map_err(|e| format!("Failed to add revoked certificate: {}", e))
}

/// Get revoked certificates
#[tauri::command]
pub fn get_revoked_certificates(
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<HashMap<String, u64>, String> {
    Ok(manager.get_revoked_certificates())
}

/// Get certificate validation settings
#[tauri::command]
pub fn get_certificate_settings(
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<CertificateValidationSettings, String> {
    Ok(manager.get_settings())
}

/// Update certificate validation settings
#[tauri::command]
pub fn update_certificate_settings(
    settings: CertificateValidationSettings,
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get certificate cache statistics
#[tauri::command]
pub fn get_certificate_cache_stats(
    manager: State<'_, Arc<CertificateValidationManager>>,
) -> Result<(usize, usize), String> {
    Ok(manager.get_cache_stats())
}
