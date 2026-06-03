//! Exodus Browser — pending host_permissions confirmation on extension install.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::Serialize;

/// Emitted when an extension install needs host permission approval.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionHostInstallRequestEvent {
    pub request_id: String,
    pub extension_id: String,
    pub extension_name: String,
    pub host_permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingHostInstall {
    pub extension_id: String,
    pub host_permissions: Vec<String>,
}

/// Queue for install-time host permission prompts.
pub struct HostInstallPendingStore {
    pending: Mutex<HashMap<String, PendingHostInstall>>,
}

impl HostInstallPendingStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new install host prompt; returns event for the UI.
    pub fn register_install(
        &self,
        extension_id: &str,
        extension_name: &str,
        host_permissions: Vec<String>,
    ) -> ExtensionHostInstallRequestEvent {
        let request_id = format!(
            "host-install-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        let mut guard = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        guard.insert(
            request_id.clone(),
            PendingHostInstall {
                extension_id: extension_id.to_string(),
                host_permissions: host_permissions.clone(),
            },
        );
        ExtensionHostInstallRequestEvent {
            request_id,
            extension_id: extension_id.to_string(),
            extension_name: extension_name.to_string(),
            host_permissions,
        }
    }

    /// Take pending entry after UI resolves.
    pub fn take(&self, request_id: &str) -> Option<PendingHostInstall> {
        let mut guard = self.pending.lock().ok()?;
        guard.remove(request_id)
    }
}

impl Default for HostInstallPendingStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_install_emits_distinct_extension_names() {
        let store = HostInstallPendingStore::new();
        let a = store.register_install("ext-a", "Test Host Perms A", vec!["https://a/*".into()]);
        let b = store.register_install("ext-b", "Test Host Perms B", vec!["https://b/*".into()]);
        assert_eq!(a.extension_id, "ext-a");
        assert_eq!(b.extension_id, "ext-b");
        assert_eq!(a.extension_name, "Test Host Perms A");
        assert_eq!(b.extension_name, "Test Host Perms B");
        assert!(!a.request_id.is_empty());
        assert!(!b.request_id.is_empty());
    }
}
