//! Exodus Browser — `extension://` URL resolution for Web Extension pages.

use std::path::{Path, PathBuf};

use super::error::PluginError;
use super::manager::ExtensionManager;

/// Parsed `extension://{id}/{relative_path}` URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionUrl {
    pub extension_id: String,
    pub path: String,
}

/// Parse an extension scheme URL.
pub fn parse_extension_url(url: &str) -> Option<ExtensionUrl> {
    let rest = url.strip_prefix("extension://")?;
    let (id, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i + 1..]),
        None => (rest, ""),
    };
    if id.is_empty() {
        return None;
    }
    Some(ExtensionUrl {
        extension_id: id.to_string(),
        path: if path.is_empty() {
            "index.html".to_string()
        } else {
            path.to_string()
        },
    })
}

/// Resolve to an on-disk file path for an installed extension.
pub fn resolve_extension_file(
    mgr: &ExtensionManager,
    ext: &ExtensionUrl,
) -> Result<PathBuf, PluginError> {
    let root = mgr.extension_root(&ext.extension_id)?;
    let clean = ext
        .path
        .trim_start_matches('/')
        .replace("../", "");
    let path = root.join(&clean);
    if !path.starts_with(&root) {
        return Err(PluginError::InvalidManifest("Path traversal denied".into()));
    }
    if !path.exists() {
        return Err(PluginError::NotFound(format!(
            "Extension resource not found: {}",
            ext.path
        )));
    }
    Ok(path)
}

/// Build a `file://` navigation target for a resolved extension page.
pub fn file_url_for_path(path: &Path) -> Result<String, PluginError> {
    let canonical = path.canonicalize().map_err(PluginError::Io)?;
    Ok(format!("file://{}", canonical.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_extension_scheme() {
        let u = parse_extension_url("extension://hello/popup.html").expect("parse");
        assert_eq!(u.extension_id, "hello");
        assert_eq!(u.path, "popup.html");
    }
}
