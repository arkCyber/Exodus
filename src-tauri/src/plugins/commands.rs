//! Exodus Browser — Tauri commands for Web Extension management.

use std::path::PathBuf;
use serde_json::{Map, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use super::background::{list_background_specs, ExtensionBackgroundSpec};
use super::error::PluginError;
use super::manager::{ExtensionInfo, ExtensionState};
use super::notifications::{NotificationInfo, NotificationOptions, NotificationStore};
use super::host_install_pending::HostInstallPendingStore;
use super::site_permissions::SitePermissionStore;
use super::tabs::{ExtensionTabInfo, TabRegistry};

/// After install: prompt for host permissions or auto-grant per config.
fn handle_post_install_hosts(
    app: &AppHandle,
    mgr: &super::manager::ExtensionManager,
    site: &SitePermissionStore,
    host_install: &HostInstallPendingStore,
    config: &crate::config::ExodusConfig,
    extension_id: &str,
) {
    let hosts = mgr.host_permissions_for(extension_id);
    if hosts.is_empty() {
        return;
    }
    if config.confirm_host_permissions_on_install {
        let name = mgr
            .list()
            .into_iter()
            .find(|e| e.id == extension_id)
            .map(|e| e.name)
            .unwrap_or_else(|| extension_id.to_string());
        let event = host_install.register_install(&extension_id, &name, hosts);
        let _ = app.emit("exodus-extension-host-install-request", event);
    } else {
        let _ = site.grant_hosts(extension_id, &hosts);
    }
}

/// List installed Web Extensions.
#[tauri::command]
pub fn extension_list(state: State<'_, ExtensionState>) -> Result<Vec<ExtensionInfo>, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    Ok(mgr.list())
}

/// Enable or disable an extension.
#[tauri::command]
pub fn extension_set_enabled(
    extension_id: String,
    enabled: bool,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mut mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    mgr.set_enabled(&extension_id, enabled)
        .map_err(|e: PluginError| e.to_string())
}

/// Install unpacked extension from a directory path.
#[tauri::command]
pub fn extension_install_folder(
    folder_path: String,
    app: AppHandle,
    state: State<'_, ExtensionState>,
    site: State<'_, SitePermissionStore>,
    host_install: State<'_, HostInstallPendingStore>,
    config: State<'_, crate::config::ConfigState>,
) -> Result<ExtensionInfo, String> {
    let cfg = config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| format!("Config lock error: {e}"))?;
    let mut mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let info = mgr
        .install_from_dir(PathBuf::from(folder_path).as_path())
        .map_err(|e: PluginError| e.to_string())?;
    handle_post_install_hosts(&app, &mgr, &site, &host_install, &cfg, &info.id);
    Ok(info)
}

/// Uninstall an extension by id.
#[allow(dead_code)]
#[tauri::command]
pub fn extension_uninstall(
    extension_id: String,
    state: State<'_, ExtensionState>,
    site: State<'_, SitePermissionStore>,
) -> Result<(), String> {
    let mut mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    mgr.uninstall(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    site.remove_extension(&extension_id)?;
    Ok(())
}

/// Rescan extension directories.
#[allow(dead_code)]
#[tauri::command]
pub fn extension_rescan(
    app: AppHandle,
    state: State<'_, ExtensionState>,
    site: State<'_, SitePermissionStore>,
    host_install: State<'_, HostInstallPendingStore>,
    config: State<'_, crate::config::ConfigState>,
) -> Result<usize, String> {
    let cfg = config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| format!("Config lock error: {e}"))?;
    let mut mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let dev_dir = dev_extensions_dir();
    let count = mgr
        .scan_and_load(dev_dir.as_deref())
        .map_err(|e: PluginError| e.to_string())?;
    for ext in mgr.list() {
        handle_post_install_hosts(&app, &mgr, &site, &host_install, &cfg, &ext.id);
    }
    Ok(count)
}

/// chrome.storage.local.get
#[tauri::command]
pub fn extension_storage_get(
    extension_id: String,
    keys: Option<Vec<String>>,
    state: State<'_, ExtensionState>,
) -> Result<Map<String, Value>, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr
        .permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    mgr.storage()
        .get(&extension_id, &perms, keys)
        .map_err(|e: PluginError| e.to_string())
}

/// chrome.storage.local.set
#[tauri::command]
pub fn extension_storage_set(
    extension_id: String,
    items: Map<String, Value>,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr
        .permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    mgr.storage()
        .set(&extension_id, &perms, items)
        .map_err(|e: PluginError| e.to_string())
}

/// chrome.storage.local.remove
#[tauri::command]
pub fn extension_storage_remove(
    extension_id: String,
    keys: Vec<String>,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr
        .permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    mgr.storage()
        .remove(&extension_id, &perms, keys)
        .map_err(|e: PluginError| e.to_string())
}

/// chrome.storage.local.clear
#[tauri::command]
pub fn extension_storage_clear(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let perms = mgr
        .permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    mgr.storage()
        .clear(&extension_id, &perms)
        .map_err(|e: PluginError| e.to_string())
}

/// Sync open tabs from the UI (powers chrome.tabs.query in content scripts).
#[tauri::command]
pub fn extension_sync_tabs(
    tabs: Vec<ExtensionTabInfo>,
    registry: State<'_, TabRegistry>,
) -> Result<(), String> {
    registry.sync(tabs);
    Ok(())
}

/// Query tabs from the registry (chrome.tabs.query subset).
#[tauri::command]
pub fn extension_tabs_query(
    query: serde_json::Value,
    registry: State<'_, TabRegistry>,
) -> Result<Vec<ExtensionTabInfo>, String> {
    Ok(registry.query(&query))
}

/// Update tab properties (chrome.tabs.update).
#[tauri::command]
pub fn extension_tabs_update(
    tab_id: i64,
    update_properties: serde_json::Value,
    app: AppHandle,
) -> Result<ExtensionTabInfo, String> {
    // Extract URL if provided
    let url = update_properties
        .get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Extract active if provided
    let active = update_properties.get("active").and_then(|v| v.as_bool());
    
    // Get current tab info
    let tab_reg = app
        .try_state::<TabRegistry>()
        .ok_or_else(|| "Tab registry not ready".to_string())?;
    
    // Find the tab
    let mut tabs = tab_reg.query(&serde_json::json!({"chromeTabId": tab_id}));
    if tabs.is_empty() {
        return Err(format!("Tab with chromeTabId {} not found", tab_id));
    }
    
    let tab = &mut tabs[0];
    
    // Update URL if provided
    if let Some(new_url) = url {
        tab.url = new_url;
    }
    
    // Update active if provided
    if let Some(is_active) = active {
        tab.active = is_active;
    }
    
    // Note: Actual navigation would need to be handled by the browser
    // This is a simplified implementation
    Ok(tab.clone())
}

/// Remove tab (chrome.tabs.remove).
#[tauri::command]
pub fn extension_tabs_remove(tab_ids: Vec<i64>, app: AppHandle) -> Result<(), String> {
    let ops = vec![super::runtime::TabOpRequest {
        op: "remove".to_string(),
        extension_id: None,
        chrome_tab_id: 0,
        tab_ids,
        update_properties: None,
    }];
    let _ = app.emit(
        "exodus-extension-tabs-ops",
        super::runtime::ExtensionTabOpsPayload { ops },
    );
    Ok(())
}

/// Reload tab (chrome.tabs.reload).
#[tauri::command]
pub fn extension_tabs_reload(tab_id: i64, app: AppHandle) -> Result<(), String> {
    let ops = vec![super::runtime::TabOpRequest {
        op: "reload".to_string(),
        extension_id: None,
        chrome_tab_id: tab_id,
        tab_ids: vec![],
        update_properties: None,
    }];
    let _ = app.emit(
        "exodus-extension-tabs-ops",
        super::runtime::ExtensionTabOpsPayload { ops },
    );
    Ok(())
}

/// List background service worker hosts to create (hidden webviews).
#[tauri::command]
pub fn extension_background_specs(
    state: State<'_, ExtensionState>,
    registry: State<'_, TabRegistry>,
) -> Result<Vec<ExtensionBackgroundSpec>, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    Ok(list_background_specs(&mgr, &registry))
}

/// Boot a background service worker in its host webview (after the webview exists).
#[tauri::command]
pub fn extension_background_boot(
    app: AppHandle,
    extension_id: String,
    state: State<'_, ExtensionState>,
    registry: State<'_, TabRegistry>,
) -> Result<(), String> {
    let script = {
        let mgr = state
            .lock()
            .map_err(|e| format!("Extension state lock error: {e}"))?;
        mgr.background_boot_script(&extension_id, &registry)
            .ok_or_else(|| format!("No background worker for extension: {extension_id}"))?
    };
    let label = super::background::background_webview_label(&extension_id);
    let webview = app
        .get_webview(&label)
        .ok_or_else(|| format!("Background webview not found: {label}"))?;
    webview
        .eval(&script)
        .map_err(|e| format!("Background boot eval failed: {e}"))
}

/// Get extension manifest (chrome.runtime.getManifest).
#[tauri::command]
pub fn extension_get_manifest(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<serde_json::Value, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    
    let extensions = mgr.list();
    let info = extensions
        .iter()
        .find(|ext| ext.id == extension_id)
        .ok_or_else(|| format!("Extension {} not found", extension_id))?;
    
    use super::manifest::load_manifest;
    use std::path::Path;
    let manifest = load_manifest(Path::new(&info.path))
        .map_err(|e: PluginError| e.to_string())?;
    
    // Convert manifest to JSON manually since it doesn't implement Serialize
    let json = serde_json::json!({
        "manifest_version": manifest.manifest_version,
        "name": manifest.name,
        "version": manifest.version,
        "description": manifest.description,
        "permissions": manifest.permissions,
        "host_permissions": manifest.host_permissions
    });
    
    Ok(json)
}

/// Emit onInstalled event for an extension (chrome.runtime.onInstalled).
#[tauri::command]
pub fn extension_emit_installed_event(
    extension_id: String,
    reason: String,
    app: AppHandle,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "extensionId": extension_id,
        "reason": reason
    });
    app.emit("exodus-extension-installed", payload)
        .map_err(|e| e.to_string())
}

/// Check if extension has a permission (chrome.permissions.contains).
#[tauri::command]
pub fn extension_permissions_contains(
    extension_id: String,
    permission: String,
    state: State<'_, ExtensionState>,
) -> Result<bool, String> {
    use super::permissions::{contains_permission, parse_permissions};
    
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    
    let granted = mgr.permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    
    let perm = parse_permissions(&[permission]).into_iter().next();
    match perm {
        Some(p) => Ok(contains_permission(&granted, p)),
        None => Ok(false),
    }
}

/// Get all granted permissions (chrome.permissions.getAll).
#[tauri::command]
pub fn extension_permissions_get_all(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<Vec<String>, String> {
    use super::permissions::get_all_permissions;
    
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    
    let granted = mgr.permissions_for(&extension_id)
        .map_err(|e: PluginError| e.to_string())?;
    
    Ok(get_all_permissions(&granted))
}

/// Request additional permissions (chrome.permissions.request) via UI prompt.
#[tauri::command]
pub fn extension_permissions_request(
    extension_id: String,
    permissions: Vec<String>,
    app: AppHandle,
    state: State<'_, ExtensionState>,
    pending: State<'_, super::permission_pending::PermissionPendingStore>,
) -> Result<bool, String> {
    let request_id = format!(
        "perm-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let batch = vec![super::permission_pending::PermissionRequestOutbound {
        extension_id: extension_id.clone(),
        request_id: request_id.clone(),
        permissions: permissions.clone(),
        source_webview_label: None,
    }];
    let display_name = |id: &str| -> String {
        state
            .lock()
            .ok()
            .map(|m| m.display_name_for(id))
            .unwrap_or_else(|| id.to_string())
    };
    for ev in pending.register_batch(&batch, display_name) {
        let _ = app.emit("exodus-extension-permission-request", ev);
    }
    Ok(false)
}

/// Resolve a pending permission request from the UI.
#[tauri::command]
pub fn extension_permissions_resolve(
    request_id: String,
    granted: bool,
    app: AppHandle,
    state: State<'_, ExtensionState>,
    site: State<'_, SitePermissionStore>,
    pending: State<'_, super::permission_pending::PermissionPendingStore>,
) -> Result<(), String> {
    let Some(entry) = pending.take(&request_id) else {
        return Err(format!("Unknown permission request: {request_id}"));
    };
    if granted {
        let (api_perms, host_perms) =
            SitePermissionStore::split_permission_strings(&entry.permissions);
        let mut mgr = state
            .lock()
            .map_err(|e| format!("Extension state lock error: {e}"))?;
        if !api_perms.is_empty() {
            mgr.grant_permissions(&entry.extension_id, &api_perms)
                .map_err(|e: PluginError| e.to_string())?;
        }
        if !host_perms.is_empty() {
            site.grant_hosts(&entry.extension_id, &host_perms)?;
        }
    }
    let response = serde_json::json!(granted);
    let response_json = serde_json::to_string(&response).map_err(|e| format!("Reply JSON: {e}"))?;
    if let Some(source) = &entry.source_webview_label {
        if !source.is_empty() {
            let script =
                super::runtime::deliver_content_reply_script(&request_id, &response_json);
            if let Some(wv) = app.get_webview(source) {
                let _ = wv.eval(&script);
            }
        }
    }
    Ok(())
}
/// Create a notification (chrome.notifications.create).
#[allow(dead_code)]
#[tauri::command]
pub fn extension_notifications_create(
    notification_id: Option<String>,
    options: NotificationOptions,
    state: State<'_, NotificationStore>,
) -> Result<String, String> {
    let id = notification_id.unwrap_or_else(|| format!("notif-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()));
    
    state.create(id.clone(), options)?;
    Ok(id)
}

/// Update a notification (chrome.notifications.update).
#[allow(dead_code)]
#[tauri::command]
pub fn extension_notifications_update(
    notification_id: String,
    options: NotificationOptions,
    state: State<'_, NotificationStore>,
) -> Result<bool, String> {
    state.update(&notification_id, options)
}

/// Clear a notification (chrome.notifications.clear).
#[allow(dead_code)]
#[tauri::command]
pub fn extension_notifications_clear(
    notification_id: String,
    state: State<'_, NotificationStore>,
) -> Result<bool, String> {
    state.clear(&notification_id)
}

#[allow(dead_code)]
/// Get all notifications (chrome.notifications.getAll).
#[tauri::command]
pub fn extension_notifications_get_all(
    state: State<'_, NotificationStore>,
) -> Result<Vec<NotificationInfo>, String> {
    Ok(state.get_all())
}


#[tauri::command]
pub fn extension_install_crx(
    package_path: String,
    app: AppHandle,
    state: State<'_, ExtensionState>,
    site: State<'_, SitePermissionStore>,
    host_install: State<'_, HostInstallPendingStore>,
    config: State<'_, crate::config::ConfigState>,
) -> Result<ExtensionInfo, String> {
    let cfg = config
        .lock()
        .map(|c| c.clone())
        .map_err(|e| format!("Config lock error: {e}"))?;
    let mut mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let info = mgr
        .install_from_crx(
            PathBuf::from(package_path).as_path(),
            cfg.require_crx_signature,
        )
        .map_err(|e: PluginError| e.to_string())?;
    handle_post_install_hosts(&app, &mgr, &site, &host_install, &cfg, &info.id);
    Ok(info)
}

/// Resolve install-time host permission prompt from the UI.
#[tauri::command]
pub fn extension_host_install_resolve(
    request_id: String,
    granted: bool,
    site: State<'_, SitePermissionStore>,
    host_install: State<'_, HostInstallPendingStore>,
) -> Result<(), String> {
    let Some(entry) = host_install.take(&request_id) else {
        return Err(format!("Unknown host install request: {request_id}"));
    };
    if granted {
        site.grant_hosts(&entry.extension_id, &entry.host_permissions)?;
    }
    Ok(())
}

/// Update whether install should confirm host_permissions.
#[tauri::command]
pub fn extension_set_confirm_host_permissions(
    confirm: bool,
    app: AppHandle,
    config: State<'_, crate::config::ConfigState>,
) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("App data dir: {e}"))?;
    let mut cfg = config
        .lock()
        .map_err(|e| format!("Config lock error: {e}"))?;
    cfg.confirm_host_permissions_on_install = confirm;
    cfg.save_to(&data_dir)
        .map_err(|e| format!("Save config failed: {e}"))
}

/// List stored per-origin browser permission decisions.
#[tauri::command]
pub fn browser_site_permissions_list(
    store: State<'_, super::browser_site_permissions::BrowserSitePermissionStore>,
) -> Result<Vec<super::browser_site_permissions::BrowserSitePermissionEntry>, String> {
    Ok(store.list_grants())
}

/// Revoke browser site permission(s) for an origin (`kinds` empty = all kinds).
#[tauri::command]
pub fn browser_site_permissions_revoke(
    origin: String,
    kinds: Option<Vec<String>>,
    store: State<'_, super::browser_site_permissions::BrowserSitePermissionStore>,
) -> Result<(), String> {
    store.revoke(&origin, kinds)
}

/// Resolve a browser site permission prompt (camera / mic / geolocation).
#[tauri::command]
pub fn browser_site_permission_resolve(
    request_id: String,
    granted: bool,
    app: AppHandle,
    store: State<'_, super::browser_site_permissions::BrowserSitePermissionStore>,
) -> Result<(), String> {
    let Some(entry) = store.take_pending(&request_id) else {
        return Err(format!("Unknown browser permission request: {request_id}"));
    };
    store.set_decision(&entry.origin, entry.kind, granted)?;
    let script = super::browser_site_permissions::deliver_browser_permission_reply_script(
        &request_id,
        granted,
    );
    if let Some(wv) = app.get_webview(&entry.webview_label) {
        let _ = wv.eval(&script);
    }
    Ok(())
}

/// Check whether an extension has been granted host access to a URL.
#[tauri::command]
pub fn extension_validate_host_access(
    extension_id: String,
    url: String,
    state: State<'_, ExtensionState>,
    site: State<'_, SitePermissionStore>,
) -> Result<bool, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    Ok(mgr.host_access_allowed(&extension_id, &url, &site))
}

/// List granted host patterns for an extension.
#[tauri::command]
pub fn extension_site_permissions_list(
    extension_id: String,
    site: State<'_, SitePermissionStore>,
) -> Result<Vec<String>, String> {
    Ok(site.hosts_for(&extension_id))
}

/// Revoke specific host patterns for an extension.
#[tauri::command]
pub fn extension_site_permissions_revoke(
    extension_id: String,
    patterns: Vec<String>,
    site: State<'_, SitePermissionStore>,
) -> Result<(), String> {
    site.revoke_hosts(&extension_id, &patterns)
}

/// Revoke all granted host patterns for an extension.
#[tauri::command]
pub fn extension_site_permissions_revoke_all(
    extension_id: String,
    site: State<'_, SitePermissionStore>,
) -> Result<(), String> {
    site.revoke_all_hosts(&extension_id)
}
/// Bundled / dev store catalog entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreExtensionEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub path: String,
    pub installed: bool,
}

/// Update remote extension catalog URL in config.
#[tauri::command]
pub fn extension_set_store_url(
    url: String,
    app: AppHandle,
    config: State<'_, crate::config::ConfigState>,
) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("App data dir: {e}"))?;
    let mut cfg = config
        .lock()
        .map_err(|e| format!("Config lock error: {e}"))?;
    cfg.extension_store_url = url.trim().to_string();
    cfg.save_to(&data_dir)
        .map_err(|e| format!("Save config failed: {e}"))
}

/// Fetch remote store catalog JSON from configured URL (optional).
#[tauri::command]
pub async fn extension_store_fetch_remote(
    config: State<'_, crate::config::ConfigState>,
) -> Result<Vec<StoreExtensionEntry>, String> {
    let url = {
        let cfg = config
            .lock()
            .map_err(|e| format!("Config lock error: {e}"))?;
        cfg.extension_store_url.clone()
    };
    if url.trim().is_empty() {
        return Ok(Vec::new());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;
    let raw = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Store fetch failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Store HTTP error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Store body read failed: {e}"))?;
    let entries: Vec<StoreExtensionEntry> =
        serde_json::from_str(&raw).map_err(|e| format!("Store JSON parse: {e}"))?;
    Ok(entries)
}

/// List extensions available in the dev `extensions/` store folder.
#[tauri::command]
pub fn extension_store_list(state: State<'_, ExtensionState>) -> Result<Vec<StoreExtensionEntry>, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    let installed: std::collections::HashSet<String> =
        mgr.list().into_iter().map(|e| e.id).collect();
    let mut out = Vec::new();
    let Some(dev_dir) = dev_extensions_dir() else {
        return Ok(out);
    };
    let entries = std::fs::read_dir(&dev_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = super::manifest::extension_id_from_dir(
            path.file_name().and_then(|s| s.to_str()).unwrap_or("ext"),
        );
        let manifest = super::manifest::load_manifest(&path).map_err(|e| e.to_string())?;
        out.push(StoreExtensionEntry {
            id: id.clone(),
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            path: path.display().to_string(),
            installed: installed.contains(&id),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Resolve popup URL for an extension (`extension://` scheme).
#[tauri::command]
pub fn extension_popup_url(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<Option<String>, String> {
    let mgr = state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    Ok(mgr
        .popup_path_for(&extension_id)
        .map(|p| format!("extension://{extension_id}/{p}")))
}

/// Workspace `extensions/` folder used in development builds.
pub fn dev_extensions_dir() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../extensions");
    if dir.exists() { Some(dir) } else { None }
}
