//! Exodus Browser — Web Extension manager (load, enable, content scripts).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::error::PluginError;
use super::background::build_background_boot;
use super::crx::extract_extension_package;
use super::extension_url::{file_url_for_path, parse_extension_url, resolve_extension_file};
use super::runtime;
use super::chrome_bridge::{apply_storage_flush_to_disk, build_extension_prelude, parse_page_flush};
use super::inject::{build_document_end_bundle, build_document_start_bundle};
use super::tabs::TabRegistry;
use super::manifest::{
    extension_id_from_dir, load_manifest, RunAt, WebExtensionManifest,
};
use super::match_patterns::url_matches_content_script;
use super::permissions::{effective_permissions, Permission};
use super::storage::ExtensionStorage;

/// Thread-safe extension manager handle for Tauri state.
pub type ExtensionState = Arc<Mutex<ExtensionManager>>;

/// Resolved content script ready for injection.
#[derive(Debug, Clone)]
pub struct ResolvedContentScript {
    pub extension_id: String,
    pub matches: Vec<String>,
    pub exclude_matches: Vec<String>,
    pub js_bodies: Vec<String>,
    pub css_bodies: Vec<String>,
    pub all_frames: bool,
    pub run_at: RunAt,
}

impl ResolvedContentScript {
    /// Check whether this script should run on `url`.
    pub fn matches_url(&self, url: &str) -> bool {
        if url.starts_with("about:") || url.starts_with("data:") {
            return false;
        }
        url_matches_content_script(url, &self.matches, &self.exclude_matches)
    }
}

/// Loaded extension record.
#[derive(Debug, Clone)]
pub struct LoadedExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub path: PathBuf,
    pub enabled: bool,
    pub permissions: Vec<Permission>,
    pub host_permissions: Vec<String>,
    pub content_scripts: Vec<ResolvedContentScript>,
    pub background_js: Option<String>,
    pub action_popup: Option<String>,
}

/// Serializable extension summary for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub permissions: Vec<String>,
    pub path: String,
    pub action_popup: Option<String>,
}

/// Registry persisted to disk (enabled flags).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ExtensionRegistry {
    #[serde(default)]
    enabled: HashMap<String, bool>,
}

/// Manages installed Web Extensions.
pub struct ExtensionManager {
    extensions: HashMap<String, LoadedExtension>,
    web_extensions_dir: PathBuf,
    registry_path: PathBuf,
    storage: ExtensionStorage,
    /// Allama HTTP port exposed to extension shims (`window.exodus.allama`).
    allama_http_port: u16,
}

#[allow(dead_code)]
impl ExtensionManager {
    /// Create manager and ensure plugin directories exist.
    pub fn new(app_data_dir: &Path) -> Result<Self, PluginError> {
        let web_extensions_dir = app_data_dir.join("plugins").join("web-extensions");
        std::fs::create_dir_all(&web_extensions_dir)?;
        let registry_path = app_data_dir.join("extensions").join("registry.json");
        if let Some(parent) = registry_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let storage = ExtensionStorage::new(app_data_dir)?;
        Ok(Self {
            extensions: HashMap::new(),
            web_extensions_dir,
            registry_path,
            storage,
            allama_http_port: 11435,
        })
    }

    /// Set Allama HTTP port for injected `window.exodus.allama` (Ollama-compatible API).
    pub fn set_allama_http_port(&mut self, port: u16) {
        self.allama_http_port = port;
    }

    /// Current Allama HTTP port for extension prelude injection.
    pub fn allama_http_port(&self) -> u16 {
        self.allama_http_port
    }

    fn load_registry(&self) -> ExtensionRegistry {
        if !self.registry_path.exists() {
            return ExtensionRegistry::default();
        }
        std::fs::read_to_string(&self.registry_path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    fn save_registry(&self, registry: &ExtensionRegistry) -> Result<(), PluginError> {
        let json = serde_json::to_string_pretty(registry)
            .map_err(|e| PluginError::Parse(format!("Registry serialize: {e}")))?;
        std::fs::write(&self.registry_path, json)?;
        Ok(())
    }

    /// Scan installed extensions and optional dev directory.
    pub fn scan_and_load(&mut self, dev_extensions_dir: Option<&Path>) -> Result<usize, PluginError> {
        self.extensions.clear();
        let registry = self.load_registry();
        let mut count = 0;

        if let Some(dev_dir) = dev_extensions_dir {
            count += self.scan_directory(dev_dir, &registry)?;
        }
        let web_dir = self.web_extensions_dir.clone();
        count += self.scan_directory(&web_dir, &registry)?;
        tracing::info!(
            "[{}] Extension scan complete: {} loaded",
            Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
            self.extensions.len()
        );
        Ok(count)
    }

    /// Scan a directory of extension folders.
    fn scan_directory(
        &mut self,
        root: &Path,
        registry: &ExtensionRegistry,
    ) -> Result<usize, PluginError> {
        if !root.exists() {
            return Ok(0);
        }
        let mut loaded = 0;
        let entries = std::fs::read_dir(root)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let id = extension_id_from_dir(
                path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
            );
            if self.extensions.contains_key(&id) {
                continue;
            }
            match self.load_extension_from_dir(&path, registry) {
                Ok(()) => loaded += 1,
                Err(e) => tracing::warn!("Skip extension {}: {e}", path.display()),
            }
        }
        Ok(loaded)
    }

    /// Install extension from an unpacked folder (copy or register in place if under web_extensions_dir).
    pub fn install_from_dir(&mut self, source: &Path) -> Result<ExtensionInfo, PluginError> {
        if !source.is_dir() {
            return Err(PluginError::InvalidManifest(format!(
                "Not a directory: {}",
                source.display()
            )));
        }
        let id = extension_id_from_dir(
            source
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("extension"),
        );
        let dest = self.web_extensions_dir.join(&id);
        if dest.exists() {
            return Err(PluginError::AlreadyInstalled(id));
        }
        copy_dir_recursive(source, &dest)?;
        let registry = self.load_registry();
        self.load_extension_from_dir(&dest, &registry)?;
        let mut registry = self.load_registry();
        registry.enabled.insert(id.clone(), true);
        self.save_registry(&registry)?;
        self.info_for(&id)
            .ok_or(PluginError::NotFound(id))
    }

    fn load_extension_from_dir(
        &mut self,
        path: &Path,
        registry: &ExtensionRegistry,
    ) -> Result<(), PluginError> {
        let manifest = load_manifest(path)?;
        let id = extension_id_from_dir(
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&manifest.name),
        );
        let enabled = registry.enabled.get(&id).copied().unwrap_or(true);
        let permissions = effective_permissions(&manifest);
        let content_scripts = resolve_content_scripts(&id, path, &manifest)?;
        let background_js = resolve_background_script(path, &manifest)?;
        let action_popup = manifest
            .action
            .as_ref()
            .and_then(|a| a.default_popup.clone())
            .filter(|p| !p.is_empty());
        let ext = LoadedExtension {
            id: id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            path: path.to_path_buf(),
            enabled,
            permissions,
            host_permissions: manifest.host_permissions.clone(),
            content_scripts,
            background_js,
            action_popup,
        };
        self.extensions.insert(id, ext);
        Ok(())
    }

    /// Human-readable extension name for UI prompts (falls back to id).
    pub fn display_name_for(&self, extension_id: &str) -> String {
        self.extensions
            .get(extension_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| extension_id.to_string())
    }

    /// Manifest host_permissions for an extension.
    pub fn host_permissions_for(&self, extension_id: &str) -> Vec<String> {
        self.extensions
            .get(extension_id)
            .map(|e| e.host_permissions.clone())
            .unwrap_or_default()
    }

    /// List installed extensions.
    pub fn list(&self) -> Vec<ExtensionInfo> {
        let mut items: Vec<ExtensionInfo> = self
            .extensions
            .values()
            .map(|e| self.to_info(e))
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    fn to_info(&self, ext: &LoadedExtension) -> ExtensionInfo {
        ExtensionInfo {
            id: ext.id.clone(),
            name: ext.name.clone(),
            version: ext.version.clone(),
            description: ext.description.clone(),
            enabled: ext.enabled,
            permissions: ext
                .permissions
                .iter()
                .map(|p| p.as_str().to_string())
                .collect(),
            path: ext.path.display().to_string(),
            action_popup: ext.action_popup.clone(),
        }
    }

    fn info_for(&self, id: &str) -> Option<ExtensionInfo> {
        self.extensions.get(id).map(|e| self.to_info(e))
    }

    /// Enable or disable an extension.
    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), PluginError> {
        let ext = self
            .extensions
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        ext.enabled = enabled;
        let mut registry = self.load_registry();
        registry.enabled.insert(id.to_string(), enabled);
        self.save_registry(&registry)
    }

    /// Uninstall extension by id.
    /// Returns true if extension may access `url` (granted host patterns).
    pub fn host_access_allowed(
        &self,
        extension_id: &str,
        url: &str,
        site: &super::site_permissions::SitePermissionStore,
    ) -> bool {
        site.url_allowed(extension_id, url)
    }

    pub fn uninstall(&mut self, id: &str) -> Result<(), PluginError> {
        let ext = self
            .extensions
            .remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        if ext.path.starts_with(&self.web_extensions_dir) {
            std::fs::remove_dir_all(&ext.path)?;
        }
        self.storage.remove_extension(id)?;
        let mut registry = self.load_registry();
        registry.enabled.remove(id);
        self.save_registry(&registry)
    }

    /// Get extension storage backend.
    pub fn storage(&self) -> &ExtensionStorage {
        &self.storage
    }

    /// Permissions for extension.
    pub fn permissions_for(&self, id: &str) -> Result<Vec<Permission>, PluginError> {
        self.extensions
            .get(id)
            .map(|e| e.permissions.clone())
            .ok_or_else(|| PluginError::NotFound(id.to_string()))
    }

    /// Build document_start injection JS for a page URL (Chrome prelude + content scripts).
    /// On-disk root for an extension id.
    pub fn extension_root(&self, extension_id: &str) -> Result<PathBuf, PluginError> {
        let ext = self
            .extensions
            .get(extension_id)
            .ok_or_else(|| PluginError::NotFound(extension_id.to_string()))?;
        Ok(ext.path.clone())
    }

    /// Resolve `extension://` to a `file://` URL for webview navigation.
    pub fn resolve_extension_navigation_url(&self, url: &str) -> Result<String, PluginError> {
        let parsed = parse_extension_url(url)
            .ok_or_else(|| PluginError::InvalidManifest(format!("Invalid extension URL: {url}")))?;
        let path = resolve_extension_file(self, &parsed)?;
        file_url_for_path(&path)
    }

    /// Popup page path from manifest action (if any).
    pub fn popup_path_for(&self, extension_id: &str) -> Option<String> {
        let ext = self.extensions.get(extension_id)?;
        if !ext.enabled {
            return None;
        }
        ext.action_popup.clone()
    }

    /// Install extension from a `.crx` or `.zip` package file.
    pub fn install_from_crx(
        &mut self,
        package_path: &Path,
        require_crx_signature: bool,
    ) -> Result<ExtensionInfo, PluginError> {
        let stem = package_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("extension");
        let id = extension_id_from_dir(stem);
        let dest = self.web_extensions_dir.join(&id);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        extract_extension_package(package_path, &dest, require_crx_signature)?;
        let registry = self.load_registry();
        self.load_extension_from_dir(&dest, &registry)?;
        let mut registry = self.load_registry();
        registry.enabled.insert(id.clone(), true);
        self.save_registry(&registry)?;
        self.info_for(&id)
            .ok_or(PluginError::NotFound(id))
    }

    /// Build document_start injection JS for a page URL (Chrome prelude + content scripts).
    pub fn document_start_script(
        &self,
        page_url: &str,
        tabs: &TabRegistry,
        webview_label: &str,
    ) -> String {
        let prelude = build_extension_prelude(self, tabs, webview_label);
        let scripts = self.active_content_scripts();
        let body = build_document_start_bundle(&scripts, page_url);
        if body.is_empty() {
            prelude
        } else {
            format!("{prelude}\n{body}")
        }
    }

    /// Persist in-page `window.__exodusStorage` into extension storage files.
    pub fn persist_storage_flush(&mut self, json: &str) -> Result<(), PluginError> {
        apply_storage_flush_to_disk(self, json)
    }

    /// Parse combined page flush (storage + runtime outbox) and persist storage.
    pub fn persist_page_flush(
        &mut self,
        json: &str,
    ) -> Result<super::chrome_bridge::PageFlushResult, PluginError> {
        let flush = parse_page_flush(json)?;
        if !flush.storage_json.trim().is_empty() && flush.storage_json != "{}" {
            apply_storage_flush_to_disk(self, &flush.storage_json)?;
        }
        Ok(flush)
    }

    /// Storage seed object for one extension (when storage permission granted).
    pub fn storage_seed_for(&self, extension_id: &str) -> Option<Map<String, Value>> {
        let ext = self.extensions.get(extension_id)?;
        if !ext.enabled || !ext.permissions.contains(&Permission::Storage) {
            return None;
        }
        self.storage()
            .get(extension_id, &ext.permissions, None)
            .ok()
    }

    /// Boot script for a background service worker host.
    pub fn background_boot_script(
        &self,
        extension_id: &str,
        tabs: &TabRegistry,
    ) -> Option<String> {
        let ext = self.extensions.get(extension_id)?;
        if !ext.enabled {
            return None;
        }
        let user = ext.background_js.as_ref()?;
        Some(build_background_boot(self, tabs, extension_id, user))
    }

    /// Build eval script to deliver a runtime message to a background host.
    pub fn background_deliver_script(
        &self,
        request_id: &str,
        message: &serde_json::Value,
    ) -> Result<String, PluginError> {
        let json = serde_json::to_string(message)
            .map_err(|e| PluginError::Parse(format!("Message JSON: {e}")))?;
        Ok(runtime::deliver_background_message_script(request_id, &json))
    }

    /// Build document_end/idle injection JS for eval after navigation.
    pub fn document_end_script(&self, page_url: &str) -> String {
        let scripts = self.active_content_scripts();
        build_document_end_bundle(&scripts, page_url)
    }

    fn active_content_scripts(&self) -> Vec<ResolvedContentScript> {
        self.extensions
            .values()
            .filter(|e| e.enabled)
            .flat_map(|e| e.content_scripts.clone())
            .collect()
    }

    /// Grant additional manifest permissions after user approval.
    pub fn grant_permissions(&mut self, extension_id: &str, raw: &[String]) -> Result<(), PluginError> {
        let extra = super::permissions::parse_permissions(raw);
        let ext = self
            .extensions
            .get_mut(extension_id)
            .ok_or_else(|| PluginError::NotFound(extension_id.to_string()))?;
        for perm in extra {
            if !ext.permissions.contains(&perm) {
                ext.permissions.push(perm);
            }
        }
        Ok(())
    }

    /// Build eval script for `chrome.scripting.executeScript` (func or extension files).
    pub fn build_scripting_injection(
        &self,
        extension_id: &str,
        req: &runtime::ScriptingExecuteRequest,
    ) -> Result<String, PluginError> {
        let ext = self
            .extensions
            .get(extension_id)
            .ok_or_else(|| PluginError::NotFound(extension_id.to_string()))?;
        super::permissions::require_permission(
            extension_id,
            &ext.permissions,
            Permission::Scripting,
        )?;
        if let Some(func) = &req.func {
            let args = req.args.clone().unwrap_or_default();
            let args_json = serde_json::to_string(&args)
                .map_err(|e| PluginError::Parse(format!("Script args JSON: {e}")))?;
            let escaped_func = func.replace('\\', "\\\\").replace('\'', "\\'");
            return Ok(format!(
                r#"(function() {{
  try {{
    var fn = {escaped_func};
    return fn.apply(null, {args_json});
  }} catch (e) {{
    console.error('[Exodus scripting]', e);
  }}
}})();"#
            ));
        }
        if let Some(files) = &req.files {
            let mut joined = String::new();
            for rel in files {
                let path = ext.path.join(rel.trim_start_matches('/'));
                let body = std::fs::read_to_string(&path)?;
                joined.push_str(&body);
                joined.push('\n');
            }
            if joined.is_empty() {
                return Ok(String::new());
            }
            return Ok(format!(
                r#"(function() {{
  try {{
{joined}
  }} catch (e) {{
    console.error('[Exodus scripting]', e);
  }}
}})();"#
            ));
        }
        Ok(String::new())
    }
}

fn resolve_background_script(
    root: &Path,
    manifest: &WebExtensionManifest,
) -> Result<Option<String>, PluginError> {
    let Some(bg) = &manifest.background else {
        return Ok(None);
    };
    let path = root.join(&bg.service_worker);
    let body = std::fs::read_to_string(&path)?;
    Ok(Some(body))
}

fn resolve_content_scripts(
    extension_id: &str,
    root: &Path,
    manifest: &WebExtensionManifest,
) -> Result<Vec<ResolvedContentScript>, PluginError> {
    let mut resolved = Vec::new();
    for cs in &manifest.content_scripts {
        let mut js_bodies = Vec::new();
        for rel in &cs.js {
            let path = root.join(rel);
            let body = std::fs::read_to_string(&path)?;
            js_bodies.push(body);
        }
        let mut css_bodies = Vec::new();
        for rel in &cs.css {
            let path = root.join(rel);
            let body = std::fs::read_to_string(&path)?;
            css_bodies.push(body);
        }
        if js_bodies.is_empty() && css_bodies.is_empty() {
            continue;
        }
        resolved.push(ResolvedContentScript {
            extension_id: extension_id.to_string(),
            matches: cs.matches.clone(),
            exclude_matches: cs.exclude_matches.clone(),
            js_bodies,
            css_bodies,
            all_frames: cs.all_frames.unwrap_or(false),
            run_at: RunAt::from_manifest(cs.run_at.as_deref()),
        });
    }
    Ok(resolved)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), PluginError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_sample_extension(dir: &Path) {
        fs::create_dir_all(dir).expect("dir");
        fs::write(
            dir.join("manifest.json"),
            r#"{
              "manifest_version": 3,
              "name": "Sample",
              "version": "1.0.0",
              "permissions": ["storage"],
              "content_scripts": [{
                "matches": ["<all_urls>"],
                "js": ["content.js"],
                "run_at": "document_start"
              }]
            }"#,
        )
        .expect("manifest");
        fs::write(dir.join("content.js"), "window.__sample = true;").expect("js");
    }

    #[test]
    fn scan_loads_unpacked_extension() {
        let base = std::env::temp_dir().join(format!("exodus_mgr_{}", uuid::Uuid::new_v4()));
        let ext_dir = base.join("plugins").join("web-extensions").join("sample");
        write_sample_extension(&ext_dir);
        let mut mgr = ExtensionManager::new(&base).expect("mgr");
        mgr.scan_and_load(None).expect("scan");
        let list = mgr.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "sample");
        let tabs = TabRegistry::default();
        assert!(mgr
            .document_start_script("https://example.com/", &tabs, "exodus-tab-test")
            .contains("__sample"));
    }
}
