//! Exodus Browser — Chrome Extension Manifest V3 parsing and validation.

use serde::Deserialize;

use super::error::PluginError;

/// Parsed Manifest V3 (subset used by Exodus; JSON keys follow Chrome snake_case).
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct WebExtensionManifest {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub host_permissions: Vec<String>,
    #[serde(default)]
    pub content_scripts: Vec<ContentScriptManifest>,
    #[serde(default)]
    pub action: Option<ExtensionAction>,
    #[serde(default)]
    pub background: Option<BackgroundManifest>,
}

/// MV3 background service worker entry.
#[derive(Debug, Clone, Deserialize)]
pub struct BackgroundManifest {
    pub service_worker: String,
}

/// Content script entry from manifest.json.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ContentScriptManifest {
    #[serde(default)]
    pub matches: Vec<String>,
    #[serde(default)]
    pub exclude_matches: Vec<String>,
    #[serde(default)]
    pub js: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub run_at: Option<String>,
    #[serde(default)]
    pub all_frames: Option<bool>,
}

/// Toolbar action metadata (optional).
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ExtensionAction {
    #[serde(default)]
    pub default_title: Option<String>,
    #[serde(default)]
    pub default_popup: Option<String>,
}

/// When to inject content scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum RunAt {
    DocumentStart,
    DocumentEnd,
    DocumentIdle,
}

impl RunAt {
    /// Parse manifest `run_at` string.
    pub fn from_manifest(value: Option<&str>) -> Self {
        match value.unwrap_or("document_idle") {
            "document_start" => RunAt::DocumentStart,
            "document_end" => RunAt::DocumentEnd,
            _ => RunAt::DocumentIdle,
        }
    }
}

/// Load and validate manifest.json from an extension directory.
pub fn load_manifest(extension_dir: &std::path::Path) -> Result<WebExtensionManifest, PluginError> {
    let path = extension_dir.join("manifest.json");
    let raw = std::fs::read_to_string(&path)
        .map_err(PluginError::Io)?;
    let manifest: WebExtensionManifest = serde_json::from_str(&raw)
        .map_err(|e| PluginError::Parse(format!("manifest.json: {e}")))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Validate manifest fields required by Exodus Web Extension MVP.
pub fn validate_manifest(manifest: &WebExtensionManifest) -> Result<(), PluginError> {
    if manifest.manifest_version != 3 {
        return Err(PluginError::InvalidManifest(format!(
            "manifest_version must be 3, got {}",
            manifest.manifest_version
        )));
    }
    if manifest.name.trim().is_empty() {
        return Err(PluginError::InvalidManifest("name is required".into()));
    }
    if manifest.version.trim().is_empty() {
        return Err(PluginError::InvalidManifest("version is required".into()));
    }
    for cs in &manifest.content_scripts {
        if cs.matches.is_empty() {
            return Err(PluginError::InvalidManifest(
                "content_scripts entry requires non-empty matches".into(),
            ));
        }
    }
    let has_background = manifest
        .background
        .as_ref()
        .map(|b| !b.service_worker.trim().is_empty())
        .unwrap_or(false);
    if manifest.content_scripts.is_empty() && !has_background {
        return Err(PluginError::InvalidManifest(
            "extension requires content_scripts and/or background.service_worker".into(),
        ));
    }
    if let Some(bg) = &manifest.background {
        if bg.service_worker.trim().is_empty() {
            return Err(PluginError::InvalidManifest(
                "background.service_worker must be non-empty".into(),
            ));
        }
    }
    Ok(())
}

/// Derive a stable extension id from directory name.
pub fn extension_id_from_dir(dir_name: &str) -> String {
    dir_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_manifest_v2() {
        let m = WebExtensionManifest {
            manifest_version: 2,
            name: "x".into(),
            version: "1.0".into(),
            description: None,
            permissions: vec![],
            host_permissions: vec![],
            content_scripts: vec![],
            background: None,
            action: None,
        };
        assert!(validate_manifest(&m).is_err());
    }

    #[test]
    fn validate_accepts_background_only() {
        let m = WebExtensionManifest {
            manifest_version: 3,
            name: "bg".into(),
            version: "1.0".into(),
            description: None,
            permissions: vec![],
            host_permissions: vec![],
            content_scripts: vec![],
            background: Some(BackgroundManifest {
                service_worker: "background.js".into(),
            }),
            action: None,
        };
        assert!(validate_manifest(&m).is_ok());
    }

    #[test]
    fn run_at_parsing() {
        assert_eq!(
            RunAt::from_manifest(Some("document_start")),
            RunAt::DocumentStart
        );
        assert_eq!(RunAt::from_manifest(None), RunAt::DocumentIdle);
    }
}
