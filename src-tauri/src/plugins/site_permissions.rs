//! Exodus Browser — per-extension host / origin permissions (Chrome host_permissions).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::match_patterns::url_matches_pattern;

/// Persisted grants: extension id → match patterns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SitePermissionFile {
    #[serde(default)]
    grants: HashMap<String, Vec<String>>,
}

/// Thread-safe host permission store (disk-backed).
pub struct SitePermissionStore {
    grants: Mutex<HashMap<String, Vec<String>>>,
    path: PathBuf,
}

impl SitePermissionStore {
    /// Open or create store under app data.
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let path = app_data_dir.join("extensions").join("site_permissions.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let grants = if path.exists() {
            let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str::<SitePermissionFile>(&raw)
                .map(|f| f.grants)
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Self {
            grants: Mutex::new(grants),
            path,
        })
    }

    fn save(&self) -> Result<(), String> {
        let guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        let file = SitePermissionFile {
            grants: guard.clone(),
        };
        let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    /// Replace all granted host patterns for an extension.
    #[allow(dead_code)]
    pub fn set_hosts(&self, extension_id: &str, patterns: Vec<String>) -> Result<(), String> {
        let mut guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        guard.insert(extension_id.to_string(), patterns);
        drop(guard);
        self.save()
    }

    /// Merge additional host patterns (deduped).
    pub fn grant_hosts(&self, extension_id: &str, patterns: &[String]) -> Result<(), String> {
        let mut guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        let entry = guard
            .entry(extension_id.to_string())
            .or_insert_with(Vec::new);
        for p in patterns {
            if !entry.contains(p) {
                entry.push(p.clone());
            }
        }
        drop(guard);
        self.save()
    }

    /// List granted patterns for an extension.
    pub fn hosts_for(&self, extension_id: &str) -> Vec<String> {
        let guard = match self.grants.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        guard.get(extension_id).cloned().unwrap_or_default()
    }

    /// Returns true when `url` matches any granted host pattern for the extension.
    pub fn url_allowed(&self, extension_id: &str, url: &str) -> bool {
        let patterns = self.hosts_for(extension_id);
        if patterns.is_empty() {
            return false;
        }
        patterns.iter().any(|p| url_matches_pattern(url, p))
    }

    /// Remove grants when an extension is uninstalled.
    pub fn remove_extension(&self, extension_id: &str) -> Result<(), String> {
        let mut guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        guard.remove(extension_id);
        drop(guard);
        self.save()
    }

    /// Revoke all host patterns granted to an extension.
    pub fn revoke_all_hosts(&self, extension_id: &str) -> Result<(), String> {
        let mut guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        guard.remove(extension_id);
        drop(guard);
        self.save()
    }

    /// Revoke specific host patterns for an extension.
    pub fn revoke_hosts(&self, extension_id: &str, patterns: &[String]) -> Result<(), String> {
        let mut guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        if let Some(entry) = guard.get_mut(extension_id) {
            entry.retain(|p| !patterns.contains(p));
            if entry.is_empty() {
                guard.remove(extension_id);
            }
        }
        drop(guard);
        self.save()
    }

    /// Clear all grants (for testing or reset).
    #[allow(dead_code)]
    pub fn clear_all(&self) -> Result<(), String> {
        let mut guard = self
            .grants
            .lock()
            .map_err(|e| format!("Site permission lock error: {e}"))?;
        guard.clear();
        drop(guard);
        self.save()
    }

    /// Get count of extensions with granted permissions.
    #[allow(dead_code)]
    pub fn extension_count(&self) -> usize {
        self.grants
            .lock()
            .map(|g| g.len())
            .unwrap_or(0)
    }

    /// Get total pattern count across all extensions.
    #[allow(dead_code)]
    pub fn total_pattern_count(&self) -> usize {
        self.grants
            .lock()
            .map(|g| g.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }

    /// Split API permission strings from host match patterns.
    pub fn split_permission_strings(perms: &[String]) -> (Vec<String>, Vec<String>) {
        let mut api = Vec::new();
        let mut hosts = Vec::new();
        for p in perms {
            if is_host_permission_pattern(p) {
                hosts.push(p.clone());
            } else {
                api.push(p.clone());
            }
        }
        (api, hosts)
    }
}

/// Host permissions use URL match patterns or `<all_urls>`.
pub fn is_host_permission_pattern(perm: &str) -> bool {
    perm == "<all_urls>"
        || perm.contains("://")
        || perm.starts_with('*')
        || perm.starts_with("*.") 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_host_and_api_permissions() {
        let (api, hosts) = SitePermissionStore::split_permission_strings(&[
            "storage".into(),
            "https://*.example.com/*".into(),
        ]);
        assert_eq!(api, vec!["storage"]);
        assert_eq!(hosts.len(), 1);
    }

    #[test]
    fn url_allowed_matches_pattern() {
        let dir = std::env::temp_dir().join(format!("exodus_site_{}", uuid::Uuid::new_v4()));
        let store = SitePermissionStore::new(&dir).expect("store");
        store
            .grant_hosts("ext", &["https://*.example.com/*".to_string()])
            .expect("grant");
        assert!(store.url_allowed("ext", "https://www.example.com/path"));
        assert!(!store.url_allowed("ext", "https://other.com/"));
    }

    #[test]
    fn revoke_all_hosts_clears_extension() {
        let dir = std::env::temp_dir().join(format!("exodus_site_all_{}", uuid::Uuid::new_v4()));
        let store = SitePermissionStore::new(&dir).expect("store");
        store
            .grant_hosts(
                "ext",
                &[
                    "https://*.example.com/*".to_string(),
                    "https://*.test.com/*".to_string(),
                ],
            )
            .expect("grant");
        store.revoke_all_hosts("ext").expect("revoke all");
        assert!(!store.url_allowed("ext", "https://www.example.com/path"));
        assert!(!store.url_allowed("ext", "https://www.test.com/path"));
    }

    #[test]
    fn revoke_specific_patterns() {
        let dir = std::env::temp_dir().join(format!("exodus_site_{}", uuid::Uuid::new_v4()));
        let store = SitePermissionStore::new(&dir).expect("store");
        store
            .grant_hosts("ext", &["https://*.example.com/*".to_string(), "https://*.test.com/*".to_string()])
            .expect("grant");
        store
            .revoke_hosts("ext", &["https://*.example.com/*".to_string()])
            .expect("revoke");
        assert!(!store.url_allowed("ext", "https://www.example.com/path"));
        assert!(store.url_allowed("ext", "https://www.test.com/path"));
    }

    #[test]
    fn clear_all_permissions() {
        let dir = std::env::temp_dir().join(format!("exodus_site_{}", uuid::Uuid::new_v4()));
        let store = SitePermissionStore::new(&dir).expect("store");
        store
            .grant_hosts("ext1", &["https://*.example.com/*".to_string()])
            .expect("grant");
        store
            .grant_hosts("ext2", &["https://*.test.com/*".to_string()])
            .expect("grant");
        assert_eq!(store.extension_count(), 2);
        store.clear_all().expect("clear");
        assert_eq!(store.extension_count(), 0);
    }

    #[test]
    fn all_urls_pattern() {
        let dir = std::env::temp_dir().join(format!("exodus_site_{}", uuid::Uuid::new_v4()));
        let store = SitePermissionStore::new(&dir).expect("store");
        store
            .grant_hosts("ext", &["<all_urls>".to_string()])
            .expect("grant");
        assert!(store.url_allowed("ext", "https://any-site.com/path"));
        assert!(store.url_allowed("ext", "http://another-site.com/"));
    }
}
