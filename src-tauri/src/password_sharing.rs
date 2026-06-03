//! Password Sharing for Exodus Browser
//! Provides secure password sharing with trusted contacts

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Trusted contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedContact {
    pub id: String,
    pub name: String,
    pub email: String,
    pub public_key: String,
    pub trusted_since: String,
}

/// Shared password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedPassword {
    pub id: String,
    pub title: String,
    pub url: String,
    pub username: String,
    pub encrypted_password: String,
    pub shared_with: Vec<String>, // contact IDs
    pub shared_at: String,
    pub expires_at: Option<String>,
}

/// Password sharing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordSharingSettings {
    pub enabled: bool,
    pub require_approval: bool,
    pub auto_expire_days: u32,
}

impl Default for PasswordSharingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            require_approval: true,
            auto_expire_days: 30,
        }
    }
}

/// Password Sharing Manager
pub struct PasswordSharingManager {
    contacts: Arc<Mutex<Vec<TrustedContact>>>,
    shared_passwords: Arc<Mutex<Vec<SharedPassword>>>,
    settings: Arc<Mutex<PasswordSharingSettings>>,
}

impl PasswordSharingManager {
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(Mutex::new(vec![])),
            shared_passwords: Arc::new(Mutex::new(vec![])),
            settings: Arc::new(Mutex::new(PasswordSharingSettings::default())),
        }
    }

    /// Enable password sharing
    pub fn enable(&self, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.enabled = true;
        let _ = app.emit("exodus-password-sharing-enabled", true);
    }

    /// Disable password sharing
    pub fn disable(&self, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.enabled = false;
        let _ = app.emit("exodus-password-sharing-enabled", false);
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        settings.enabled
    }

    /// Add trusted contact
    pub fn add_contact(&self, name: String, email: String, public_key: String, app: AppHandle) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let contact = TrustedContact {
            id: id.clone(),
            name,
            email,
            public_key,
            trusted_since: chrono::Utc::now().to_rfc3339(),
        };
        
        let mut contacts = self.contacts.lock().unwrap();
        contacts.push(contact.clone());
        let _ = app.emit("exodus-trusted-contact-added", contact);
        id
    }

    /// Remove trusted contact
    pub fn remove_contact(&self, id: String, app: AppHandle) {
        let mut contacts = self.contacts.lock().unwrap();
        contacts.retain(|c| c.id != id);
        let _ = app.emit("exodus-trusted-contact-removed", id);
    }

    /// Get all contacts
    pub fn get_contacts(&self) -> Vec<TrustedContact> {
        let contacts = self.contacts.lock().unwrap();
        contacts.clone()
    }

    /// Share password with contacts
    pub fn share_password(&self, title: String, url: String, username: String, encrypted_password: String, contact_ids: Vec<String>, app: AppHandle) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let shared_password = SharedPassword {
            id: id.clone(),
            title,
            url,
            username,
            encrypted_password,
            shared_with: contact_ids.clone(),
            shared_at: chrono::Utc::now().to_rfc3339(),
            expires_at: None,
        };
        
        let mut shared_passwords = self.shared_passwords.lock().unwrap();
        shared_passwords.push(shared_password.clone());
        let _ = app.emit("exodus-password-shared", shared_password);
        id
    }

    /// Revoke shared password
    pub fn revoke_shared_password(&self, id: String, app: AppHandle) {
        let mut shared_passwords = self.shared_passwords.lock().unwrap();
        shared_passwords.retain(|p| p.id != id);
        let _ = app.emit("exodus-password-share-revoked", id);
    }

    /// Get shared passwords
    pub fn get_shared_passwords(&self) -> Vec<SharedPassword> {
        let shared_passwords = self.shared_passwords.lock().unwrap();
        shared_passwords.clone()
    }

    /// Set require approval
    pub fn set_require_approval(&self, require: bool, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.require_approval = require;
        let _ = app.emit("exodus-password-sharing-approval-changed", require);
    }

    /// Set auto expire days
    pub fn set_auto_expire_days(&self, days: u32, app: AppHandle) {
        let mut settings = self.settings.lock().unwrap();
        settings.auto_expire_days = days;
        let _ = app.emit("exodus-password-sharing-expire-changed", days);
    }

    /// Get settings
    pub fn get_settings(&self) -> PasswordSharingSettings {
        let settings = self.settings.lock().unwrap();
        PasswordSharingSettings {
            enabled: settings.enabled,
            require_approval: settings.require_approval,
            auto_expire_days: settings.auto_expire_days,
        }
    }
}

impl Default for PasswordSharingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to enable password sharing
#[tauri::command]
pub fn enable_password_sharing(
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable password sharing
#[tauri::command]
pub fn disable_password_sharing(
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_password_sharing_enabled(
    manager: State<'_, Arc<PasswordSharingManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to add trusted contact
#[tauri::command]
pub fn add_trusted_contact(
    name: String,
    email: String,
    public_key: String,
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) -> String {
    manager.add_contact(name, email, public_key, app)
}

/// Tauri command to remove trusted contact
#[tauri::command]
pub fn remove_trusted_contact(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) {
    manager.remove_contact(id, app);
}

/// Tauri command to get trusted contacts
#[tauri::command]
pub fn get_trusted_contacts(
    manager: State<'_, Arc<PasswordSharingManager>>,
) -> Vec<TrustedContact> {
    manager.get_contacts()
}

/// Tauri command to share password
#[tauri::command]
pub fn share_password(
    title: String,
    url: String,
    username: String,
    encrypted_password: String,
    contact_ids: Vec<String>,
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) -> String {
    manager.share_password(title, url, username, encrypted_password, contact_ids, app)
}

/// Tauri command to revoke shared password
#[tauri::command]
pub fn revoke_shared_password(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) {
    manager.revoke_shared_password(id, app);
}

/// Tauri command to get shared passwords
#[tauri::command]
pub fn get_shared_passwords(
    manager: State<'_, Arc<PasswordSharingManager>>,
) -> Vec<SharedPassword> {
    manager.get_shared_passwords()
}

/// Tauri command to set require approval
#[tauri::command]
pub fn set_password_sharing_require_approval(
    require: bool,
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) {
    manager.set_require_approval(require, app);
}

/// Tauri command to set auto expire days
#[tauri::command]
pub fn set_password_sharing_auto_expire_days(
    days: u32,
    app: AppHandle,
    manager: State<'_, Arc<PasswordSharingManager>>,
) {
    manager.set_auto_expire_days(days, app);
}

/// Tauri command to get password sharing settings
#[tauri::command]
pub fn get_password_sharing_settings(
    manager: State<'_, Arc<PasswordSharingManager>>,
) -> PasswordSharingSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_sharing_manager_creation() {
        let manager = PasswordSharingManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_contacts() {
        let manager = PasswordSharingManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(manager.get_contacts().is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = PasswordSharingManager::new();
        
        let settings = manager.get_settings();
        assert!(!settings.enabled);
    }
}
