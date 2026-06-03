//! Aerospace-grade automated validation for workspace `extensions/` dev samples.
//!
//! Ensures every bundled dev Web Extension:
//! - Parses as Manifest V3 and loads into `ExtensionManager`
//! - References only existing script/style/popup assets
//! - Meets minimum source-quality bar (header comment, strict mode, timestamped logging)
//! - Satisfies extension-specific behavioral contracts (markers, host_permissions, all_frames)

use std::fs;
use std::path::{Path, PathBuf};

use super::commands::dev_extensions_dir;
use super::error::PluginError;
use super::manager::ExtensionManager;
use super::manifest::{extension_id_from_dir, load_manifest};
use super::tabs::TabRegistry;

/// Quality audit failure for a single file.
#[derive(Debug, Clone)]
struct SourceAuditIssue {
    file: PathBuf,
    reason: String,
}

/// Result of auditing one dev extension directory.
#[derive(Debug, Clone)]
struct DevExtensionAudit {
    id: String,
    path: PathBuf,
    js_files: Vec<PathBuf>,
    issues: Vec<SourceAuditIssue>,
}

/// Resolve workspace `extensions/` directory (same path as runtime dev store).
fn workspace_extensions_dir() -> Option<PathBuf> {
    dev_extensions_dir()
}

/// List immediate child directories under `extensions/`.
fn list_extension_dirs(root: &Path) -> Result<Vec<PathBuf>, PluginError> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.path().is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// Extract `src` attribute values from `<script src="...">` tags in popup HTML.
fn script_srcs_from_html(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in html.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("<script") || !trimmed.contains("src=") {
            continue;
        }
        let Some(idx) = trimmed.find("src=") else {
            continue;
        };
        let mut rest = trimmed[idx + 4..].trim_start();
        if rest.starts_with('"') {
            rest = &rest[1..];
            if let Some(end) = rest.find('"') {
                out.push(rest[..end].to_string());
            }
        } else if rest.starts_with('\'') {
            rest = &rest[1..];
            if let Some(end) = rest.find('\'') {
                out.push(rest[..end].to_string());
            }
        }
    }
    out
}

/// Collect every `.js` file referenced by manifest (background, content_scripts, action popup script).
fn referenced_js_files(root: &Path, manifest: &super::manifest::WebExtensionManifest) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(bg) = &manifest.background {
        out.push(root.join(&bg.service_worker));
    }
    for cs in &manifest.content_scripts {
        for rel in &cs.js {
            out.push(root.join(rel));
        }
    }
    if let Some(action) = &manifest.action {
        if let Some(popup) = &action.default_popup {
            let popup_path = root.join(popup);
            if popup.ends_with(".html") {
                if let Ok(html) = fs::read_to_string(&popup_path) {
                    for src in script_srcs_from_html(&html) {
                        out.push(root.join(src));
                    }
                }
            } else if popup.ends_with(".js") {
                out.push(popup_path);
            }
        }
    }
    out
}

/// Collect CSS files referenced by content_scripts.
fn referenced_css_files(root: &Path, manifest: &super::manifest::WebExtensionManifest) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for cs in &manifest.content_scripts {
        for rel in &cs.css {
            out.push(root.join(rel));
        }
    }
    out
}

/// Aerospace source-quality checks for extension JavaScript.
fn audit_js_source(path: &Path, body: &str) -> Vec<SourceAuditIssue> {
    let mut issues = Vec::new();
    let trimmed = body.trim_start();
    if !trimmed.starts_with("/**") {
        issues.push(SourceAuditIssue {
            file: path.to_path_buf(),
            reason: "missing file header block comment (/** ... */)".into(),
        });
    }
    if !body.contains("'use strict'") && !body.contains("\"use strict\"") {
        issues.push(SourceAuditIssue {
            file: path.to_path_buf(),
            reason: "missing 'use strict'".into(),
        });
    }
    if !body.contains("tsLog") && !body.contains("LOG_PREFIX") {
        issues.push(SourceAuditIssue {
            file: path.to_path_buf(),
            reason: "missing timestamped logging (tsLog or LOG_PREFIX)".into(),
        });
    }
    if !body.contains("toISOString") {
        issues.push(SourceAuditIssue {
            file: path.to_path_buf(),
            reason: "missing ISO timestamp in logs (toISOString)".into(),
        });
    }
    issues
}

/// Audit one extension directory (manifest + assets + JS quality).
fn audit_extension_dir(path: &Path) -> Result<DevExtensionAudit, PluginError> {
    let manifest = load_manifest(path)?;
    let id = extension_id_from_dir(
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown"),
    );
    let mut issues = Vec::new();
    let mut js_files = Vec::new();

    if manifest.manifest_version != 3 {
        issues.push(SourceAuditIssue {
            file: path.join("manifest.json"),
            reason: format!("manifest_version must be 3, got {}", manifest.manifest_version),
        });
    }
    if manifest.version.trim().is_empty() {
        issues.push(SourceAuditIssue {
            file: path.join("manifest.json"),
            reason: "version must be non-empty".into(),
        });
    }

    for rel in referenced_js_files(path, &manifest) {
        if !rel.exists() {
            issues.push(SourceAuditIssue {
                file: rel.clone(),
                reason: "referenced JS file missing".into(),
            });
            continue;
        }
        let body = fs::read_to_string(&rel)?;
        js_files.push(rel.clone());
        issues.extend(audit_js_source(&rel, &body));
    }

    for rel in referenced_css_files(path, &manifest) {
        if !rel.exists() {
            issues.push(SourceAuditIssue {
                file: rel,
                reason: "referenced CSS file missing".into(),
            });
            continue;
        }
        let body = fs::read_to_string(&rel)?;
        if !body.trim_start().starts_with("/**") {
            issues.push(SourceAuditIssue {
                file: rel,
                reason: "CSS file missing header block comment".into(),
            });
        }
    }

    if let Some(action) = &manifest.action {
        if let Some(popup) = &action.default_popup {
            let popup_path = path.join(popup);
            if popup.ends_with(".html") && !popup_path.exists() {
                issues.push(SourceAuditIssue {
                    file: popup_path,
                    reason: "action.default_popup HTML missing".into(),
                });
            }
        }
    }

    Ok(DevExtensionAudit {
        id,
        path: path.to_path_buf(),
        js_files,
        issues,
    })
}

/// Format audit failures for test panic messages.
fn format_audit_failures(audits: &[DevExtensionAudit]) -> String {
    let mut lines = Vec::new();
    for a in audits {
        if a.issues.is_empty() {
            continue;
        }
        lines.push(format!("Extension `{}` ({})", a.id, a.path.display()));
        for i in &a.issues {
            lines.push(format!("  - {}: {}", i.file.display(), i.reason));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn require_workspace_extensions() -> PathBuf {
        workspace_extensions_dir()
            .filter(|p| p.exists())
            .expect("workspace extensions/ directory must exist for dev extension tests")
    }

    #[test]
    fn workspace_extensions_dir_exists() {
        let dir = require_workspace_extensions();
        assert!(dir.is_dir(), "extensions dir: {}", dir.display());
    }

    #[test]
    fn all_workspace_extensions_pass_source_audit() {
        let root = require_workspace_extensions();
        let dirs = list_extension_dirs(&root).expect("list dirs");
        assert!(!dirs.is_empty(), "expected at least one dev extension under extensions/");
        let mut audits = Vec::new();
        for dir in &dirs {
            audits.push(audit_extension_dir(dir).expect("audit dir"));
        }
        let failed: Vec<_> = audits.iter().filter(|a| !a.issues.is_empty()).collect();
        assert!(
            failed.is_empty(),
            "Dev extension source audit failed:\n{}",
            format_audit_failures(&audits)
        );
    }

    #[test]
    fn all_workspace_extensions_load_via_manager() {
        let root = require_workspace_extensions();
        let base = std::env::temp_dir().join(format!("exodus_dev_ext_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        let count = mgr
            .scan_and_load(Some(&root))
            .expect("scan dev extensions");
        let dirs = list_extension_dirs(&root).expect("list");
        assert_eq!(
            count,
            dirs.len(),
            "every extensions/ folder must load (check warnings for parse errors)"
        );
        assert_eq!(mgr.list().len(), dirs.len());
    }

    #[test]
    fn sample_hello_injects_content_marker() {
        let root = require_workspace_extensions();
        let base = std::env::temp_dir().join(format!("exodus_dev_hello_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        mgr.scan_and_load(Some(&root)).expect("scan");
        let tabs = TabRegistry::default();
        let script = mgr.document_start_script("https://example.com/page", &tabs, "exodus-tab-test");
        assert!(
            script.contains("exodusSampleHello") || script.contains("dataset.exodusSampleHello"),
            "sample-hello content script must set page marker"
        );
        let ext = mgr
            .list()
            .into_iter()
            .find(|e| e.id == "sample-hello")
            .expect("sample-hello must be loaded");
        assert!(
            ext.permissions.iter().any(|p| p == "storage"),
            "sample-hello needs storage permission"
        );
        assert!(
            ext.permissions.iter().any(|p| p == "tabs"),
            "sample-hello needs tabs permission"
        );
    }

    #[test]
    fn sample_all_frames_sets_all_frames_flag() {
        let root = require_workspace_extensions();
        let ext_dir = root.join("sample-all-frames");
        let manifest = load_manifest(&ext_dir).expect("manifest");
        let cs = manifest
            .content_scripts
            .first()
            .expect("content_scripts entry");
        assert_eq!(cs.all_frames, Some(true));
        let base = std::env::temp_dir().join(format!("exodus_dev_af_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        mgr.scan_and_load(Some(&root)).expect("scan");
        assert!(
            mgr.list().iter().any(|e| e.id == "sample-all-frames"),
            "sample-all-frames must load"
        );
        let tabs = TabRegistry::default();
        let script = mgr.document_start_script("https://example.com/", &tabs, "exodus-tab-test");
        assert!(
            script.contains("exodusAllFrames"),
            "sample-all-frames must inject frame role marker"
        );
    }

    #[test]
    fn host_perms_test_extensions_declare_distinct_hosts() {
        let root = require_workspace_extensions();
        let a = load_manifest(&root.join("test-host-perms-a")).expect("manifest a");
        let b = load_manifest(&root.join("test-host-perms-b")).expect("manifest b");
        assert!(
            a.host_permissions.iter().any(|h| h.contains("example-a")),
            "test-host-perms-a must declare example-a host"
        );
        assert!(
            b.host_permissions.iter().any(|h| h.contains("example-b")),
            "test-host-perms-b must declare example-b host"
        );
        let hosts_a: HashSet<_> = a.host_permissions.iter().collect();
        let hosts_b: HashSet<_> = b.host_permissions.iter().collect();
        assert!(hosts_a.is_disjoint(&hosts_b), "host patterns must differ for install prompt tests");
    }

    #[test]
    fn sample_net_rules_declares_net_permissions() {
        let root = require_workspace_extensions();
        let manifest = load_manifest(&root.join("sample-net-rules")).expect("manifest");
        let perms: std::collections::HashSet<String> = manifest
            .permissions
            .iter()
            .map(|p| p.to_ascii_lowercase())
            .collect();
        assert!(
            perms.contains("declarativenetrequest") || perms.contains("declarative_net_request"),
            "sample-net-rules needs declarativeNetRequest"
        );
        assert!(
            perms.contains("webrequest") || perms.contains("webrequestblocking"),
            "sample-net-rules needs webRequest"
        );
        let base = std::env::temp_dir().join(format!("exodus_dev_net_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        mgr.scan_and_load(Some(&root)).expect("scan");
        let ext = mgr
            .list()
            .into_iter()
            .find(|e| e.id == "sample-net-rules")
            .expect("sample-net-rules loaded");
        assert!(
            ext.permissions.iter().any(|p| p == "declarativeNetRequest"),
            "manager must parse declarativeNetRequest permission"
        );
        assert!(
            ext.permissions.iter().any(|p| p == "webRequest"),
            "manager must parse webRequest permission"
        );
    }

    #[test]
    fn sample_net_rules_background_registers_net_apis() {
        let root = require_workspace_extensions();
        let base = std::env::temp_dir().join(format!("exodus_dev_net_bg_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        mgr.scan_and_load(Some(&root)).expect("scan");
        let tabs = TabRegistry::default();
        let boot = mgr
            .background_boot_script("sample-net-rules", &tabs)
            .expect("sample-net-rules background");
        assert!(
            boot.contains("declarativeNetRequest") || boot.contains("updateDynamicRules"),
            "background must call DNR API"
        );
        assert!(
            boot.contains("webRequest") && boot.contains("onBeforeRequest"),
            "background must register webRequest listener"
        );
        assert!(
            boot.contains("exodus-blocked.test"),
            "background must target fake test host only"
        );
    }

    #[test]
    fn sample_hello_background_boot_contains_handlers() {
        let root = require_workspace_extensions();
        let base = std::env::temp_dir().join(format!("exodus_dev_bg_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        mgr.scan_and_load(Some(&root)).expect("scan");
        let tabs = TabRegistry::default();
        let boot = mgr
            .background_boot_script("sample-hello", &tabs)
            .expect("sample-hello has background");
        assert!(boot.contains("onMessage"), "background must register onMessage");
        assert!(boot.contains("alarms"), "background must use alarms API");
    }
}
