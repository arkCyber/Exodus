//! Exodus Browser — pending `chrome.permissions.request` prompts (UI resolution).

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Outbound permission request from an extension webview flush.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequestOutbound {
    pub extension_id: String,
    pub request_id: String,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub source_webview_label: Option<String>,
}

/// Payload emitted to the frontend when extensions request permissions.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionPermissionRequestEvent {
    pub extension_id: String,
    pub extension_name: String,
    pub request_id: String,
    pub permissions: Vec<String>,
    pub source_webview_label: Option<String>,
}

/// Pending permission request entry.
#[derive(Debug, Clone)]
pub struct PendingEntry {
    pub extension_id: String,
    pub permissions: Vec<String>,
    pub source_webview_label: Option<String>,
}

/// In-memory queue for permission prompts.
pub struct PermissionPendingStore {
    pending: Mutex<HashMap<String, PendingEntry>>,
}

impl PermissionPendingStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// Register requests from a page flush; returns events to emit.
    pub fn register_batch<F>(
        &self,
        requests: &[PermissionRequestOutbound],
        display_name: F,
    ) -> Vec<ExtensionPermissionRequestEvent>
    where
        F: Fn(&str) -> String,
    {
        let mut guard = match self.pending.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let mut events = Vec::new();
        for req in requests {
            guard.insert(
                req.request_id.clone(),
                PendingEntry {
                    extension_id: req.extension_id.clone(),
                    permissions: req.permissions.clone(),
                    source_webview_label: req.source_webview_label.clone(),
                },
            );
            events.push(ExtensionPermissionRequestEvent {
                extension_id: req.extension_id.clone(),
                extension_name: display_name(&req.extension_id),
                request_id: req.request_id.clone(),
                permissions: req.permissions.clone(),
                source_webview_label: req.source_webview_label.clone(),
            });
        }
        events
    }

    /// Take a pending entry by request id (after UI resolves).
    pub fn take(&self, request_id: &str) -> Option<PendingEntry> {
        let mut guard = self.pending.lock().ok()?;
        guard.remove(request_id)
    }
}

impl Default for PermissionPendingStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_batch_includes_extension_display_name() {
        let store = PermissionPendingStore::new();
        let batch = vec![PermissionRequestOutbound {
            extension_id: "ext-abc".into(),
            request_id: "req-1".into(),
            permissions: vec!["storage".into()],
            source_webview_label: None,
        }];
        let events = store.register_batch(&batch, |id| {
            if id == "ext-abc" {
                "Friendly Name".into()
            } else {
                id.into()
            }
        });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].extension_name, "Friendly Name");
        assert_eq!(events[0].extension_id, "ext-abc");
    }
}
