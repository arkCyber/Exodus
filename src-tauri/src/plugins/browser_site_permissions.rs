//! Exodus Browser — per-origin site permissions (camera, microphone, geolocation).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// Browser-level permission kinds (distinct from extension host_permissions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowserPermissionKind {
    Camera,
    Microphone,
    Geolocation,
    Notifications,
}

impl BrowserPermissionKind {
    /// Parse kind string from the injected bridge.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "camera" => Some(Self::Camera),
            "microphone" | "mic" => Some(Self::Microphone),
            "geolocation" | "location" => Some(Self::Geolocation),
            "notifications" => Some(Self::Notifications),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::Geolocation => "geolocation",
            Self::Notifications => "notifications",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum GrantState {
    Granted,
    Denied,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct OriginGrants {
    #[serde(default)]
    camera: Option<GrantState>,
    #[serde(default)]
    microphone: Option<GrantState>,
    #[serde(default)]
    geolocation: Option<GrantState>,
    #[serde(default)]
    notifications: Option<GrantState>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BrowserSitePermissionFile {
    #[serde(default)]
    origins: HashMap<String, OriginGrants>,
}

/// Outbound permission request from page bridge (flush).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserPermissionRequestOutbound {
    pub request_id: String,
    pub kind: String,
    pub origin: String,
    pub webview_label: String,
}

/// One stored grant/deny row for settings UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSitePermissionEntry {
    pub origin: String,
    pub kind: String,
    pub granted: bool,
}

/// Emitted to UI for a site permission prompt.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSitePermissionRequestEvent {
    pub request_id: String,
    pub kind: String,
    pub origin: String,
    pub webview_label: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingBrowserPerm {
    pub kind: BrowserPermissionKind,
    pub origin: String,
    pub webview_label: String,
}

/// Disk-backed per-origin browser permission decisions.
pub struct BrowserSitePermissionStore {
    origins: Mutex<HashMap<String, OriginGrants>>,
    pending: Mutex<HashMap<String, PendingBrowserPerm>>,
    path: PathBuf,
}

impl BrowserSitePermissionStore {
    /// Open store under app data.
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let path = app_data_dir.join("browser").join("site_permissions.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let origins = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|raw| serde_json::from_str::<BrowserSitePermissionFile>(&raw).ok())
                .map(|f| f.origins)
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Self {
            origins: Mutex::new(origins),
            pending: Mutex::new(HashMap::new()),
            path,
        })
    }

    fn save(&self) -> Result<(), String> {
        let guard = self
            .origins
            .lock()
            .map_err(|e| format!("Browser site permission lock error: {e}"))?;
        let file = BrowserSitePermissionFile {
            origins: guard.clone(),
        };
        let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    fn state_for(origin: &OriginGrants, kind: BrowserPermissionKind) -> Option<GrantState> {
        match kind {
            BrowserPermissionKind::Camera => origin.camera,
            BrowserPermissionKind::Microphone => origin.microphone,
            BrowserPermissionKind::Geolocation => origin.geolocation,
            BrowserPermissionKind::Notifications => origin.notifications,
        }
    }

    fn set_state(grants: &mut OriginGrants, kind: BrowserPermissionKind, state: GrantState) {
        match kind {
            BrowserPermissionKind::Camera => grants.camera = Some(state),
            BrowserPermissionKind::Microphone => grants.microphone = Some(state),
            BrowserPermissionKind::Geolocation => grants.geolocation = Some(state),
            BrowserPermissionKind::Notifications => grants.notifications = Some(state),
        }
    }

    fn clear_state(grants: &mut OriginGrants, kind: BrowserPermissionKind) {
        match kind {
            BrowserPermissionKind::Camera => grants.camera = None,
            BrowserPermissionKind::Microphone => grants.microphone = None,
            BrowserPermissionKind::Geolocation => grants.geolocation = None,
            BrowserPermissionKind::Notifications => grants.notifications = None,
        }
    }

    /// Returns stored decision when origin/kind was decided before.
    pub fn decision(&self, origin: &str, kind: BrowserPermissionKind) -> Option<bool> {
        let guard = self.origins.lock().ok()?;
        let entry = guard.get(origin)?;
        Self::state_for(entry, kind).map(|s| s == GrantState::Granted)
    }

    /// Persist grant or deny for an origin/kind.
    pub fn set_decision(
        &self,
        origin: &str,
        kind: BrowserPermissionKind,
        granted: bool,
    ) -> Result<(), String> {
        let mut guard = self
            .origins
            .lock()
            .map_err(|e| format!("Browser site permission lock error: {e}"))?;
        let entry = guard.entry(origin.to_string()).or_default();
        Self::set_state(
            entry,
            kind,
            if granted {
                GrantState::Granted
            } else {
                GrantState::Denied
            },
        );
        drop(guard);
        self.save()
    }

    /// Register flush queue entries; returns UI events (skips already-decided).
    pub fn register_batch(
        &self,
        _webview_label: &str,
        requests: &[BrowserPermissionRequestOutbound],
    ) -> Vec<BrowserSitePermissionRequestEvent> {
        let mut out = Vec::new();
        let mut pending = match self.pending.lock() {
            Ok(g) => g,
            Err(_) => return out,
        };
        for req in requests {
            let Some(kind) = BrowserPermissionKind::parse(&req.kind) else {
                continue;
            };
            if let Some(decided) = self.decision(&req.origin, kind) {
                // Auto-resolve immediately via promise map in deliver script path
                let _ = decided;
                continue;
            }
            pending.insert(
                req.request_id.clone(),
                PendingBrowserPerm {
                    kind,
                    origin: req.origin.clone(),
                    webview_label: req.webview_label.clone(),
                },
            );
            out.push(BrowserSitePermissionRequestEvent {
                request_id: req.request_id.clone(),
                kind: kind.as_str().to_string(),
                origin: req.origin.clone(),
                webview_label: req.webview_label.clone(),
            });
        }
        out
    }

    /// Take pending entry on UI resolve.
    pub fn take_pending(&self, request_id: &str) -> Option<PendingBrowserPerm> {
        let mut guard = self.pending.lock().ok()?;
        guard.remove(request_id)
    }

    /// Flatten stored decisions for settings UI.
    pub fn list_grants(&self) -> Vec<BrowserSitePermissionEntry> {
        let guard = match self.origins.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let mut out = Vec::new();
        for (origin, grants) in guard.iter() {
            Self::push_entry(&mut out, origin, BrowserPermissionKind::Camera, grants.camera);
            Self::push_entry(
                &mut out,
                origin,
                BrowserPermissionKind::Microphone,
                grants.microphone,
            );
            Self::push_entry(
                &mut out,
                origin,
                BrowserPermissionKind::Geolocation,
                grants.geolocation,
            );
            Self::push_entry(
                &mut out,
                origin,
                BrowserPermissionKind::Notifications,
                grants.notifications,
            );
        }
        out.sort_by(|a, b| a.origin.cmp(&b.origin).then(a.kind.cmp(&b.kind)));
        out
    }

    fn push_entry(
        out: &mut Vec<BrowserSitePermissionEntry>,
        origin: &str,
        kind: BrowserPermissionKind,
        state: Option<GrantState>,
    ) {
        let Some(state) = state else {
            return;
        };
        out.push(BrowserSitePermissionEntry {
            origin: origin.to_string(),
            kind: kind.as_str().to_string(),
            granted: state == GrantState::Granted,
        });
    }

    /// Revoke stored decisions for an origin (optional kinds filter).
    pub fn revoke(&self, origin: &str, kinds: Option<Vec<String>>) -> Result<(), String> {
        let mut guard = self
            .origins
            .lock()
            .map_err(|e| format!("Browser site permission lock error: {e}"))?;
        let Some(entry) = guard.get_mut(origin) else {
            return Ok(());
        };
        let filter: Option<Vec<BrowserPermissionKind>> = kinds.map(|list| {
            list.iter()
                .filter_map(|k| BrowserPermissionKind::parse(k))
                .collect()
        });
        match filter {
            None => {
                guard.remove(origin);
            }
            Some(ref kinds) if kinds.is_empty() => {
                guard.remove(origin);
            }
            Some(kinds) => {
                for kind in kinds {
                    Self::clear_state(entry, kind);
                }
                if entry.camera.is_none()
                    && entry.microphone.is_none()
                    && entry.geolocation.is_none()
                    && entry.notifications.is_none()
                {
                    guard.remove(origin);
                }
            }
        }
        drop(guard);
        self.save()
    }
}

/// JavaScript bridge installed in every tab (wraps getUserMedia / geolocation).
pub fn browser_permission_bridge_script() -> String {
    r#"(function() {
  if (window.__exodusBrowserPermBridge) return;
  window.__exodusBrowserPermBridge = true;
  window.__exodusBrowserPermQueue = window.__exodusBrowserPermQueue || [];
  window.__exodusPendingPromises = window.__exodusPendingPromises || {};
  if (!window.__exodusResolveReply) {
    window.__exodusResolveReply = function(reqId, resp) {
      var e = window.__exodusPendingPromises[reqId];
      if (e) { e.resolve(resp); delete window.__exodusPendingPromises[reqId]; }
    };
  }
  function queuePerm(kind) {
    return new Promise(function(resolve) {
      var reqId = 'bp' + Math.random().toString(36).slice(2);
      window.__exodusBrowserPermQueue.push({
        requestId: reqId,
        kind: kind,
        origin: location.origin || '',
        sourceWebviewLabel: window.__exodusWebviewLabel || ''
      });
      window.__exodusPendingPromises[reqId] = { resolve: resolve };
      if (window.__exodusStorageDirty !== undefined) window.__exodusStorageDirty = true;
      setTimeout(function() { resolve(false); }, 120000);
    });
  }
  if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
    var origGum = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
    navigator.mediaDevices.getUserMedia = function(constraints) {
      var needs = [];
      if (constraints && constraints.video) needs.push('camera');
      if (constraints && constraints.audio) needs.push('microphone');
      if (needs.length === 0) return origGum(constraints);
      return Promise.all(needs.map(queuePerm)).then(function(results) {
        if (results.some(function(r) { return r !== true; })) {
          return Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
        }
        return origGum(constraints);
      });
    };
  }
  if (navigator.geolocation && navigator.geolocation.getCurrentPosition) {
    var origGet = navigator.geolocation.getCurrentPosition.bind(navigator.geolocation);
    navigator.geolocation.getCurrentPosition = function(success, error, options) {
      queuePerm('geolocation').then(function(granted) {
        if (granted) origGet(success, error, options);
        else if (error) error({ code: 1, message: 'Permission denied' });
      });
    };
  }
})();"#
    .to_string()
}

/// Script to read and clear browser permission queue from a tab.
pub fn browser_permission_flush_script() -> &'static str {
    r#"JSON.stringify(window.__exodusBrowserPermQueue || [])"#
}

/// Deliver grant/deny to the requesting tab.
pub fn deliver_browser_permission_reply_script(request_id: &str, granted: bool) -> String {
    let escaped_id = request_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  if (window.__exodusResolveReply) window.__exodusResolveReply('{escaped_id}', {granted});
}})();"#
    )
}

/// Process queued browser permission requests after tab flush.
pub fn process_browser_permission_queue(
    app: &AppHandle,
    store: &BrowserSitePermissionStore,
    webview_label: &str,
    requests: &[BrowserPermissionRequestOutbound],
) -> Result<(), String> {
    let mut labeled: Vec<BrowserPermissionRequestOutbound> = requests
        .iter()
        .map(|r| BrowserPermissionRequestOutbound {
            webview_label: if r.webview_label.is_empty() {
                webview_label.to_string()
            } else {
                r.webview_label.clone()
            },
            ..r.clone()
        })
        .collect();

    // Auto-apply remembered decisions without UI
    labeled.retain(|req| {
        let Some(kind) = BrowserPermissionKind::parse(&req.kind) else {
            return false;
        };
        if let Some(granted) = store.decision(&req.origin, kind) {
            if let Some(wv) = app.get_webview(&req.webview_label) {
                let script = deliver_browser_permission_reply_script(&req.request_id, granted);
                let _ = wv.eval(&script);
            }
            return false;
        }
        true
    });

    let events = store.register_batch(webview_label, &labeled);
    for ev in events {
        let _ = app.emit("exodus-browser-site-permission-request", ev);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_permission_kinds() {
        assert_eq!(
            BrowserPermissionKind::parse("microphone"),
            Some(BrowserPermissionKind::Microphone)
        );
        assert_eq!(
            BrowserPermissionKind::parse("geolocation"),
            Some(BrowserPermissionKind::Geolocation)
        );
    }

    #[test]
    fn list_and_revoke_grants() {
        let dir = std::env::temp_dir().join(format!("exodus_bsp_list_{}", uuid::Uuid::new_v4()));
        let store = BrowserSitePermissionStore::new(&dir).expect("store");
        store
            .set_decision("https://a.test", BrowserPermissionKind::Camera, true)
            .expect("set");
        store
            .set_decision("https://a.test", BrowserPermissionKind::Microphone, false)
            .expect("set");
        let list = store.list_grants();
        assert_eq!(list.len(), 2);
        store
            .revoke("https://a.test", Some(vec!["camera".to_string()]))
            .expect("revoke one");
        assert_eq!(store.list_grants().len(), 1);
        store.revoke("https://a.test", None).expect("revoke all");
        assert!(store.list_grants().is_empty());
    }

    #[test]
    fn decision_roundtrip() {
        let dir = std::env::temp_dir().join(format!("exodus_bsp_{}", uuid::Uuid::new_v4()));
        let store = BrowserSitePermissionStore::new(&dir).expect("store");
        store
            .set_decision("https://example.com", BrowserPermissionKind::Camera, true)
            .expect("set");
        assert_eq!(
            store.decision("https://example.com", BrowserPermissionKind::Camera),
            Some(true)
        );
    }
}
