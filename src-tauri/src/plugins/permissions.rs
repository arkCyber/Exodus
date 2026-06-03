//! Exodus Browser — Web Extension permission parsing and enforcement.

use super::error::PluginError;
use super::manifest::WebExtensionManifest;

/// Known manifest permission strings (MVP subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Storage,
    Tabs,
    ActiveTab,
    Scripting,
    Notifications,
    DeclarativeNetRequest,
    WebRequest,
}

impl Permission {
    /// Manifest permission name.
    pub fn as_str(self) -> &'static str {
        match self {
            Permission::Storage => "storage",
            Permission::Tabs => "tabs",
            Permission::ActiveTab => "activeTab",
            Permission::Scripting => "scripting",
            Permission::Notifications => "notifications",
            Permission::DeclarativeNetRequest => "declarativeNetRequest",
            Permission::WebRequest => "webRequest",
        }
    }
}

/// Parse manifest permission strings into known permissions.
pub fn parse_permissions(raw: &[String]) -> Vec<Permission> {
    let mut out = Vec::new();
    for p in raw {
        let lower = p.to_ascii_lowercase();
        let perm = match lower.as_str() {
            "storage" => Some(Permission::Storage),
            "tabs" => Some(Permission::Tabs),
            "activetab" => Some(Permission::ActiveTab),
            "scripting" => Some(Permission::Scripting),
            "notifications" => Some(Permission::Notifications),
            "declarativenetrequest" => Some(Permission::DeclarativeNetRequest),
            "webrequest" | "webrequestblocking" => Some(Permission::WebRequest),
            _ => None,
        };
        if let Some(perm) = perm {
            if !out.contains(&perm) {
                out.push(perm);
            }
        }
    }
    out
}

/// Collect permissions from manifest (explicit + inferred for content scripts).
pub fn effective_permissions(manifest: &WebExtensionManifest) -> Vec<Permission> {
    let mut perms = parse_permissions(&manifest.permissions);
    if !manifest.content_scripts.is_empty() && !perms.contains(&Permission::Scripting) {
        perms.push(Permission::Scripting);
    }
    perms
}

/// Ensure extension has a permission before an API call.
pub fn require_permission(
    extension_id: &str,
    granted: &[Permission],
    needed: Permission,
) -> Result<(), PluginError> {
    if granted.contains(&needed) {
        Ok(())
    } else {
        Err(PluginError::PermissionDenied {
            extension_id: extension_id.to_string(),
            permission: needed.as_str().to_string(),
        })
    }
}

/// Check if extension has a specific permission (chrome.permissions.contains).
pub fn contains_permission(granted: &[Permission], permission: Permission) -> bool {
    granted.contains(&permission)
}

/// Get all granted permissions (chrome.permissions.getAll).
pub fn get_all_permissions(granted: &[Permission]) -> Vec<String> {
    granted.iter().map(|p| p.as_str().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::manifest::{ContentScriptManifest, WebExtensionManifest};

    #[test]
    fn content_scripts_imply_scripting() {
        let m = WebExtensionManifest {
            manifest_version: 3,
            name: "t".into(),
            version: "1".into(),
            description: None,
            permissions: vec![],
            host_permissions: vec![],
            content_scripts: vec![ContentScriptManifest {
                matches: vec!["<all_urls>".into()],
                exclude_matches: vec![],
                js: vec!["a.js".into()],
                css: vec![],
                run_at: None,
                all_frames: None,
            }],
            background: None,
            action: None,
        };
        let perms = effective_permissions(&m);
        assert!(perms.contains(&Permission::Scripting));
    }
}
