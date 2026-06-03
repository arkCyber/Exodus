//! Exodus Browser — Web Extension plugin error types.

use thiserror::Error;

/// Errors raised by the Web Extension subsystem.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Filesystem or database failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON or manifest parse failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Custom parse error.
    #[error("Parse error: {0}")]
    Parse(String),
    /// Extension id or path not found.
    #[error("Not found: {0}")]
    NotFound(String),
    /// Extension lacks a required manifest permission.
    #[error("Extension '{extension_id}' lacks permission '{permission}'")]
    PermissionDenied {
        extension_id: String,
        permission: String,
    },
    /// Manifest failed validation.
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    /// Extension id already registered.
    #[error("Already installed: {0}")]
    AlreadyInstalled(String),
    /// Manifest error.
    #[error("Manifest error: {0}")]
    ManifestError(String),
    /// Load error.
    #[error("Load error: {0}")]
    LoadError(String),
    /// Plugin disabled.
    #[error("Plugin disabled")]
    Disabled,
}

impl From<PluginError> for String {
    fn from(value: PluginError) -> Self {
        value.to_string()
    }
}
