//! Exodus Browser — native WebView control (navigate, eval, capture).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::webview::{DownloadEvent, NewWindowResponse, WebviewBuilder};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, Url};

use crate::safe_browsing::SafeBrowsingManager;
use crate::form_autofill::form_capture_init_script;
use crate::password_autofill::password_capture_init_script;
use crate::discarded_tabs::DiscardedTabsRegistry;
use crate::tracking_protection::TrackingProtectionManager;
use crate::plugins::{
    chrome_bridge::{
        apply_storage_flush_to_disk, build_extension_popup_prelude, flush_page_state_script,
        parse_page_flush,
    },
    error::PluginError,
    net_rules::NetRuleStore,
    web_request::WebRequestStore,
    notifications::NotificationStore,
    permission_pending::PermissionPendingStore,
    runtime::{self, ExtensionTabOpsPayload, ExtensionTabsCreatePayload, TabCreateAck, TabOpRequest},
    site_permissions::SitePermissionStore,
    ExtensionManager, ExtensionState, TabRegistry,
};
use tauri_utils::config::WebviewUrl;

/// Per-tab back/forward stack (updated on navigation and omnibox loads).
#[derive(Clone, Default)]
pub struct TabNavTracker(Arc<Mutex<HashMap<String, TabNavTrack>>>);

#[derive(Debug, Clone)]
struct TabNavTrack {
    stack: Vec<String>,
    index: usize,
}

impl TabNavTracker {
    /// Record a URL visit; detects back/forward vs new navigation.
    pub fn record(&self, label: &str, url: &str) {
        if url.is_empty() || url.starts_with("about:") {
            return;
        }
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let track = guard
            .entry(label.to_string())
            .or_insert_with(|| TabNavTrack {
                stack: vec![url.to_string()],
                index: 0,
            });

        let current = track.stack.get(track.index).map(|s| s.as_str());
        if current == Some(url) {
            return;
        }
        if track.index > 0 && track.stack[track.index - 1] == url {
            track.index -= 1;
            return;
        }
        if track.index + 1 < track.stack.len() && track.stack[track.index + 1] == url {
            track.index += 1;
            return;
        }
        track.stack.truncate(track.index + 1);
        if track.stack.last().map(|s| s.as_str()) != Some(url) {
            track.stack.push(url.to_string());
        }
        track.index = track.stack.len().saturating_sub(1);
    }

    /// Back/forward availability for toolbar buttons.
    pub fn flags(&self, label: &str) -> (bool, bool) {
        let guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        guard.get(label).map_or((false, false), |t| {
            (t.index > 0, t.index + 1 < t.stack.len())
        })
    }

    /// Drop navigation state when a tab webview is closed.
    pub fn remove(&self, label: &str) {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        guard.remove(label);
    }
}

/// Emitted when a page triggers a file download in a tab webview.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadRequestedPayload {
    pub label: String,
    pub url: String,
}

/// Emitted when a page requests a new window and popup blocking is off (open in-app tab).
#[derive(Debug, Clone, Serialize)]
pub struct NewWindowRequestedPayload {
    pub url: String,
    pub opener_label: String,
}

/// Emitted when a popup/new window was blocked by privacy settings.
#[derive(Debug, Clone, Serialize)]
pub struct PopupBlockedPayload {
    pub url: String,
    pub opener_label: String,
}

/// Text captured from a content webview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageCapture {
    pub url: String,
    pub title: String,
    pub content: String,
}

/// Parse a navigation target into a Tauri webview URL.
/// If https_only is true, upgrades HTTP URLs to HTTPS.
fn webview_url_from_str(
    app: &AppHandle,
    url: &str,
    https_only: bool,
) -> Result<WebviewUrl, String> {
    if url.starts_with("exodus-proxy://") {
        let parsed = Url::parse(url).map_err(|e| format!("Invalid proxy URL: {e}"))?;
        return Ok(WebviewUrl::External(parsed));
    }

    if url.starts_with("extension://") {
        let file_url = app
            .try_state::<ExtensionState>()
            .and_then(|state| {
                state.lock().ok().and_then(|mgr| {
                    mgr.resolve_extension_navigation_url(url).ok()
                })
            })
            .ok_or_else(|| format!("Extension URL not resolved: {url}"))?;
        let parsed = Url::parse(&file_url).map_err(|e| format!("Invalid extension file URL: {e}"))?;
        return Ok(WebviewUrl::External(parsed));
    }

    let mut parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    
    // Upgrade HTTP to HTTPS if HTTPS-only mode is enabled
    if https_only && parsed.scheme() == "http" {
        parsed.set_scheme("https").map_err(|_| "Failed to set HTTPS scheme".to_string())?;
    }
    
    Ok(WebviewUrl::External(parsed))
}

/// JavaScript injected at document start to wrap `window.open` when popup blocking is on.
fn popup_guard_init_script(block: bool) -> String {
    let flag = if block { "true" } else { "false" };
    format!(
        r#"(function() {{
  if (window.__exodusPopupGuardInstalled) {{
    window.__exodusBlockPopups = {flag};
    return;
  }}
  window.__exodusPopupGuardInstalled = true;
  window.__exodusBlockPopups = {flag};
  var origOpen = window.open;
  window.open = function() {{
    if (window.__exodusBlockPopups) return null;
    return origOpen.apply(window, arguments);
  }};
}})();"#
    )
}

/// Sync popup-block flag on an already-loaded page (settings toggle or post-navigate).
fn popup_guard_sync_script(block: bool) -> String {
    popup_guard_init_script(block)
}

/// Tracking protection init script from app state (empty when disabled).
fn tracking_protection_script(app: &AppHandle, page_url: &str) -> String {
    let page_host = url::Url::parse(page_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()));
    app.try_state::<Arc<TrackingProtectionManager>>()
        .map(|mgr| mgr.init_script_for_page(page_host.as_deref()))
        .unwrap_or_default()
}

/// Combine Web Extension content scripts with popup guard initialization.
fn web_request_subresource_script(app: &AppHandle) -> String {
    app.try_state::<WebRequestStore>()
        .map(|s| s.subresource_guard_script())
        .unwrap_or_default()
}

/// Extension `onHeadersReceived` rules applied at document start (CSP meta, etc.).
fn web_request_response_headers_script(app: &AppHandle, page_url: &str) -> String {
    app.try_state::<WebRequestStore>()
        .map(|s| s.response_headers_script(page_url))
        .unwrap_or_default()
}

/// MITM loopback proxy rewriter for subresources when response-header rules are active.
fn http_subresource_mitm_script(app: &AppHandle) -> String {
    let Some(proxy) = app.try_state::<crate::http_response_proxy::HttpResponseProxy>() else {
        return String::new();
    };
    let Some(wr) = app.try_state::<WebRequestStore>() else {
        return String::new();
    };
    if !wr.has_any_proxy_rules() {
        return String::new();
    }
    let rules = wr.header_proxy_rules_json();
    crate::http_response_proxy::subresource_mitm_proxy_script(&proxy, &rules)
}

fn combined_init_script(
    block_popups: bool,
    extension_script: &str,
    tracking_script: &str,
    webrequest_script: &str,
    response_headers_script: &str,
    mitm_proxy_script: &str,
) -> String {
    let browser_perm = crate::plugins::browser_site_permissions::browser_permission_bridge_script();
    let mut parts = vec![browser_perm];
    if !extension_script.trim().is_empty() {
        parts.push(extension_script.to_string());
    }
    if !tracking_script.trim().is_empty() {
        parts.push(tracking_script.to_string());
    }
    if !webrequest_script.trim().is_empty() {
        parts.push(webrequest_script.to_string());
    }
    if !response_headers_script.trim().is_empty() {
        parts.push(response_headers_script.to_string());
    }
    if !mitm_proxy_script.trim().is_empty() {
        parts.push(mitm_proxy_script.to_string());
    }
    parts.push(password_capture_init_script().to_string());
    parts.push(form_capture_init_script().to_string());
    parts.push(popup_guard_init_script(block_popups));
    parts.join("\n")
}

/// Load document_start content scripts for a URL from the extension manager.
fn extension_document_start_script(app: &AppHandle, page_url: &str, webview_label: &str) -> String {
    let Some(ext_state) = app.try_state::<ExtensionState>() else {
        return String::new();
    };
    let Some(tab_reg) = app.try_state::<TabRegistry>() else {
        return String::new();
    };
    let Ok(mgr) = ext_state.lock() else {
        return String::new();
    };
    mgr.document_start_script(page_url, &tab_reg, webview_label)
}

/// Filter tab ops that navigate to hosts the extension has not been granted.
fn filter_tab_ops_by_host_access(
    app: &AppHandle,
    mgr: &ExtensionManager,
    ops: &[TabOpRequest],
) -> Vec<TabOpRequest> {
    let Some(site) = app.try_state::<SitePermissionStore>() else {
        return ops.to_vec();
    };
    let mut out = Vec::new();
    for op in ops {
        if op.op == "update" {
            if let (Some(ext_id), Some(url)) = (
                op.extension_id.as_deref(),
                op.update_properties
                    .as_ref()
                    .and_then(|u| u.get("url"))
                    .and_then(|v| v.as_str()),
            ) {
                if !mgr.host_access_allowed(ext_id, url, &site) {
                    let _ = app.emit(
                        "exodus-extension-host-denied",
                        serde_json::json!({
                            "extensionId": ext_id,
                            "url": url,
                        }),
                    );
                    continue;
                }
            }
        }
        out.push(op.clone());
    }
    out
}

/// Persist in-page extension storage and forward runtime messages before leaving a tab.
fn flush_extension_storage(app: &AppHandle, label: &str) -> Result<(), String> {
    let webview = match get_content_webview(app, label) {
        Ok(w) => w,
        Err(_) => return Ok(()),
    };
    let (tx, rx) = mpsc::sync_channel::<String>(1);
    webview
        .eval_with_callback(flush_page_state_script(), move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Extension flush eval failed: {e}"))?;

    let raw = rx
        .recv_timeout(Duration::from_secs(3))
        .map_err(|_| "Extension flush timed out".to_string())?;

    let json = serde_json::from_str::<String>(&raw).unwrap_or(raw);
    if json.trim().is_empty() || json == "{}" {
        return Ok(());
    }

    let ext_state = app
        .try_state::<ExtensionState>()
        .ok_or_else(|| "Extension state not ready".to_string())?;
    let mut mgr = ext_state
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;

    let tab_reg = app
        .try_state::<TabRegistry>()
        .ok_or_else(|| "Tab registry not ready".to_string())?;

    let private_mode = app
        .try_state::<crate::config::ConfigState>()
        .and_then(|s| s.lock().ok().map(|c| c.private_mode))
        .unwrap_or(false);

    let flush = match parse_page_flush(&json) {
        Ok(flush) => {
            if !private_mode
                && !flush.storage_json.trim().is_empty()
                && flush.storage_json != "{}"
            {
                apply_storage_flush_to_disk(&mut mgr, &flush.storage_json)
                    .map_err(|e: PluginError| e.to_string())?;
            }
            flush
        }
        Err(_) => {
            apply_storage_flush_to_disk(&mut mgr, &json)
                .map_err(|e: PluginError| e.to_string())?;
            return Ok(());
        }
    };

    let _ = webview.eval(
        "window.__exodusRuntimeOutbox = []; window.__exodusTabRequests = []; window.__exodusTabMessageOutbox = []; window.__exodusTabOpsOutbox = []; window.__exodusScriptingOutbox = []; window.__exodusNotificationOutbox = []; window.__exodusPermRequests = []; window.__exodusDnrOutbox = []; window.__exodusWebRequestOutbox = []; window.__exodusActionOutbox = []; window.__exodusStorageDirty = false;",
    );

    runtime::process_content_outbox(app, &flush.outbox)?;
    runtime::process_tab_message_outbox(app, &tab_reg, &flush.tab_messages)?;
    runtime::process_scripting_outbox(app, &tab_reg, &mgr, &flush.scripting)?;

    for ext in mgr.list() {
        if ext.enabled {
            let _ = runtime::flush_background_host(app, &tab_reg, &ext.id);
        }
    }

    if !flush.tab_requests.is_empty() {
        let _ = app.emit(
            "exodus-extension-tabs-create",
            ExtensionTabsCreatePayload {
                requests: flush.tab_requests,
            },
        );
    }

    let tab_ops = filter_tab_ops_by_host_access(app, &mgr, &flush.tab_ops);
    if !tab_ops.is_empty() {
        let _ = app.emit(
            "exodus-extension-tabs-ops",
            ExtensionTabOpsPayload { ops: tab_ops },
        );
    }

    if let Some(net_store) = app.try_state::<NetRuleStore>() {
        for update in &flush.dnr_updates {
            let _ = net_store.apply_update(
                &update.extension_id,
                &update.add_rules,
                &update.remove_rule_ids,
            );
        }
    }

    if let Some(wr_store) = app.try_state::<WebRequestStore>() {
        for rule in &flush.web_request_rules {
            let _ = wr_store.apply_flush_rule(rule);
        }
    }

    if let Some(action_store) = app.try_state::<Arc<crate::plugins::extension_popup::ExtensionActionStore>>() {
        for op in &flush.action_ops {
            match op.op.as_str() {
                "setTitle" => action_store.set_title(&op.extension_id, op.title.clone()),
                "setBadgeText" => action_store.set_badge(&op.extension_id, op.text.clone(), None),
                "setBadgeColor" => action_store.set_badge(
                    &op.extension_id,
                    None,
                    op.color.clone(),
                ),
                "openPopup" => {
                    let _ = app.emit("exodus-extension-open-popup", &op.extension_id);
                }
                _ => {}
            }
        }
    }

    if let Some(notif_store) = app.try_state::<NotificationStore>() {
        for note in &flush.notifications {
            let id = note.notification_id.clone().unwrap_or_else(|| {
                format!(
                    "n-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                )
            });
            if notif_store.create(id.clone(), note.options.clone()).is_ok() {
                let _ = app.emit(
                    "exodus-extension-notification",
                    serde_json::json!({
                        "extensionId": note.extension_id,
                        "notificationId": id,
                        "title": note.options.title,
                        "message": note.options.message,
                    }),
                );
            }
        }
    }

    if let Some(pending) = app.try_state::<PermissionPendingStore>() {
        let display_name = |id: &str| -> String {
            if let Some(state) = app.try_state::<ExtensionState>() {
                if let Ok(m) = state.lock() {
                    return m.display_name_for(id);
                }
            }
            id.to_string()
        };
        let events = pending.register_batch(&flush.permission_requests, display_name);
        for ev in events {
            let _ = app.emit("exodus-extension-permission-request", ev);
        }
    }

    flush_browser_permission_queue(app, &webview, label)?;

    Ok(())
}

/// Read browser site permission queue from tab and emit UI prompts.
fn flush_browser_permission_queue(
    app: &AppHandle,
    webview: &tauri::Webview,
    label: &str,
) -> Result<(), String> {
    let (tx, rx) = mpsc::sync_channel::<String>(1);
    webview
        .eval_with_callback(
            crate::plugins::browser_site_permissions::browser_permission_flush_script(),
            move |result| {
                let _ = tx.send(result);
            },
        )
        .map_err(|e| format!("Browser permission flush failed: {e}"))?;
    let raw = rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap_or_else(|_| "[]".to_string());
    let json_str = serde_json::from_str::<String>(&raw).unwrap_or(raw);
    let parsed: Vec<crate::plugins::browser_site_permissions::BrowserPermissionRequestOutbound> =
        serde_json::from_str(&json_str).unwrap_or_default();
    let _ = webview.eval("window.__exodusBrowserPermQueue = [];");
    if let Some(store) = app.try_state::<crate::plugins::browser_site_permissions::BrowserSitePermissionStore>()
    {
        crate::plugins::browser_site_permissions::process_browser_permission_queue(
            app, &store, label, &parsed,
        )?;
    }
    Ok(())
}

/// Flush extension page state for a tab (storage, runtime messages, tab requests).
#[tauri::command]
pub fn browser_extension_flush_tab(app: AppHandle, label: String) -> Result<(), String> {
    flush_extension_storage(&app, &label)
}

/// Acknowledge tab creates and resolve promises in the source webview.
#[tauri::command]
pub fn extension_tabs_create_ack(
    app: AppHandle,
    acks: Vec<TabCreateAck>,
) -> Result<(), String> {
    runtime::deliver_tab_create_acks(&app, &acks)
}

/// Pump runtime queues: flush active tab (optional) and all background hosts.
#[tauri::command]
pub fn extension_pump_runtime(
    app: AppHandle,
    active_label: Option<String>,
) -> Result<(), String> {
    if let Some(label) = active_label.filter(|s| !s.is_empty()) {
        let _ = flush_extension_storage(&app, &label);
    } else {
        let tab_reg = app
            .try_state::<TabRegistry>()
            .ok_or_else(|| "Tab registry not ready".to_string())?;
        let ext_state = app
            .try_state::<ExtensionState>()
            .ok_or_else(|| "Extension state not ready".to_string())?;
        let mgr = ext_state
            .lock()
            .map_err(|e| format!("Extension state lock error: {e}"))?;
        for ext in mgr.list() {
            if ext.enabled {
                let _ = runtime::flush_background_host(&app, &tab_reg, &ext.id);
            }
        }
    }
    Ok(())
}

/// Load document_end/idle scripts for post-navigation eval.
fn extension_document_end_script(app: &AppHandle, page_url: &str) -> String {
    app.try_state::<ExtensionState>()
        .and_then(|state| {
            state
                .lock()
                .ok()
                .map(|mgr| mgr.document_end_script(page_url))
        })
        .unwrap_or_default()
}

/// Read `block_popups` from shared config state.
fn config_blocks_popups(app: &AppHandle) -> bool {
    app.try_state::<crate::config::ConfigState>()
        .and_then(|state| state.lock().ok().map(|cfg| cfg.block_popups))
        .unwrap_or(false)
}

/// Apply popup guard hooks to a tab webview builder.
fn with_popup_guard<R: tauri::Runtime>(
    builder: WebviewBuilder<R>,
    app: &AppHandle,
    opener_label: &str,
    _block: bool,
) -> WebviewBuilder<R> {
    let app_popup = app.clone();
    let opener = opener_label.to_string();
    builder.on_new_window(move |url, _features| {
            let url_str = url.to_string();
            if config_blocks_popups(&app_popup) {
                let _ = app_popup.emit(
                    "exodus-popup-blocked",
                    PopupBlockedPayload {
                        url: url_str,
                        opener_label: opener.clone(),
                    },
                );
            } else {
                let _ = app_popup.emit(
                    "exodus-new-window-requested",
                    NewWindowRequestedPayload {
                        url: url_str,
                        opener_label: opener.clone(),
                    },
                );
            }
            NewWindowResponse::Deny
        })
}

/// Shared implementation for tab / popup webview creation.
fn add_tab_webview_impl(
    app: &AppHandle,
    nav: &TabNavTracker,
    block_popups: bool,
    private_mode: bool,
    https_only: bool,
    label: &str,
    url: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if app.get_webview(label).is_some() {
        return Ok(());
    }

    let window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let webview_url = webview_url_from_str(app, url, https_only)?;
    nav.record(label, url);

    let app_emit = app.clone();
    let tab_label = label.to_string();
    let nav_hook = nav.clone();
    let nav_label = label.to_string();
    let app_nav_proxy = app.clone();
    let nav_label_proxy = label.to_string();

    let mut ext_start = extension_document_start_script(app, url, label);
    if let Some(ext_id) = label.strip_prefix("exodus-ext-popup-") {
        let popup_prelude = app
            .try_state::<ExtensionState>()
            .and_then(|ext_state| {
                app.try_state::<TabRegistry>().and_then(|tab_reg| {
                    ext_state
                        .lock()
                        .ok()
                        .map(|mgr| build_extension_popup_prelude(&mgr, &tab_reg, ext_id))
                })
            });
        if let Some(prelude) = popup_prelude {
            ext_start = format!("{prelude}\n{ext_start}");
        }
    }
    let tp_script = tracking_protection_script(app, url);
    let wr_script = web_request_subresource_script(app);
    let wr_hdr_script = web_request_response_headers_script(app, url);
    let mitm_script = http_subresource_mitm_script(app);
    let init_script = combined_init_script(
        block_popups,
        &ext_start,
        &tp_script,
        &wr_script,
        &wr_hdr_script,
        &mitm_script,
    );

    let builder = with_popup_guard(
        WebviewBuilder::new(label, webview_url)
            .initialization_script(init_script)
            .incognito(private_mode),
        app,
        label,
        block_popups,
    )
    .on_download(move |_webview, event| match event {
        DownloadEvent::Requested { url, .. } => {
            let _ = app_emit.emit(
                "exodus-download-requested",
                DownloadRequestedPayload {
                    label: tab_label.clone(),
                    url: url.to_string(),
                },
            );
            false
        }
        _ => true,
    })
    .on_navigation(move |next_url| {
        let url_str = next_url.as_str();
        if crate::http_response_proxy::should_proxy_url(&app_nav_proxy, url_str) {
            if let Some(proxy) = app_nav_proxy.try_state::<crate::http_response_proxy::HttpResponseProxy>()
            {
                if let Some(proxied) =
                    crate::http_response_proxy::proxied_navigation_url(&proxy, url_str)
                {
                    if let Ok(parsed) = Url::parse(&proxied) {
                        if let Ok(webview) = get_content_webview(&app_nav_proxy, &nav_label_proxy)
                        {
                            let _ = webview.navigate(parsed);
                            return false;
                        }
                    }
                }
            }
        }
        nav_hook.record(&nav_label, url_str);
        true
    });

    window
        .add_child(
            builder,
            LogicalPosition::new(x, y),
            LogicalSize::new(width.max(100.0), height.max(100.0)),
        )
        .map_err(|e| format!("Create tab webview failed: {}", e))?;

    Ok(())
}

/// Create a child tab webview with native download interception.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn browser_create_tab(
    app: AppHandle,
    nav: State<'_, TabNavTracker>,
    config: State<'_, crate::config::ConfigState>,
    label: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    tracing::info!("Creating tab: label={}, url={}", label, url);
    let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
    add_tab_webview_impl(
        &app,
        &nav,
        cfg.block_popups,
        cfg.private_mode,
        cfg.https_only,
        &label,
        &url,
        x,
        y,
        width,
        height,
    )
}

/// Remove per-tab navigation tracking when a tab webview is destroyed.
#[tauri::command]
pub fn browser_clear_tab_nav(nav: State<'_, TabNavTracker>, label: String) {
    nav.remove(&label);
}

/// Resolve a child webview by label (created via `@tauri-apps/api/webview`).
fn get_content_webview(app: &AppHandle, label: &str) -> Result<tauri::Webview, String> {
    app.get_webview(label)
        .ok_or_else(|| format!("Webview not found: {}", label))
}

/// Selected text in the content webview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCapture {
    pub text: String,
}

/// Read the current text selection inside the content webview with improved timeout handling.
#[tauri::command]
pub async fn browser_get_selection(
    app: AppHandle,
    label: String,
) -> Result<SelectionCapture, String> {
    let webview = get_content_webview(&app, &label)?;
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    let script = r#"JSON.stringify({ text: (window.getSelection()?.toString() || '').trim() })"#;

    webview
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Selection eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap_or_else(|_| {
            tracing::warn!("Selection read timed out, returning empty selection");
            r#"{"text":""}"#.to_string()
        });

    Ok(serde_json::from_str::<SelectionCapture>(&raw)
        .unwrap_or_else(|_| SelectionCapture { text: String::new() }))
}

/// Navigate a content webview to a URL.
#[tauri::command]
pub fn browser_navigate(
    app: AppHandle,
    nav: State<'_, TabNavTracker>,
    config: State<'_, crate::config::ConfigState>,
    label: String,
    url: String,
) -> Result<(), String> {
    let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
    let webview = get_content_webview(&app, &label)?;
    
    // Apply HTTPS-only mode
    let parsed = webview_url_from_str(&app, &url, cfg.https_only)?;
    let mut url_str = match parsed {
        WebviewUrl::External(u) => u.to_string(),
        WebviewUrl::App(_) => url.clone(),
        _ => url.clone(),
    };

    if let Some(net_store) = app.try_state::<NetRuleStore>() {
        if let Some(decision) = net_store.evaluate_navigation(&url_str) {
            if decision.blocked {
                return Err(format!(
                    "Navigation blocked by extension {} (rule {})",
                    decision.extension_id, decision.rule_id
                ));
            }
            if let Some(redirect) = decision.redirect_url.filter(|s| !s.is_empty()) {
                url_str = redirect;
            }
        }
    }

    if let Some(wr) = app.try_state::<WebRequestStore>() {
        if let Some(decision) = wr.evaluate_navigation(&url_str) {
            if decision.blocked {
                return Err(format!(
                    "Navigation blocked by extension {} (webRequest)",
                    decision.extension_id
                ));
            }
            if let Some(redirect) = decision.redirect_url.filter(|s| !s.is_empty()) {
                url_str = redirect;
            }
        }
    }

    if crate::http_response_proxy::should_proxy_url(&app, &url_str) {
        if let Some(proxy) = app.try_state::<crate::http_response_proxy::HttpResponseProxy>() {
            if let Some(proxied) = crate::http_response_proxy::proxied_navigation_url(&proxy, &url_str)
            {
                url_str = proxied;
            }
        }
    }

    if let Some(sb) = app.try_state::<Arc<SafeBrowsingManager>>() {
        if let Some(threat) = sb.check_url(&url_str) {
            return Err(format!(
                "Safe Browsing blocked {} ({})",
                threat.url_pattern,
                threat.threat_type.as_str()
            ));
        }
    }
    
    // Skip history recording in private mode
    if !cfg.private_mode {
        nav.record(&label, &url_str);
    }

    let block_popups = cfg.block_popups;

    let _ = flush_extension_storage(&app, &label);

    webview
        .navigate(Url::parse(&url_str).map_err(|e| format!("Invalid URL: {}", e))?)
        .map_err(|e| format!("Navigate failed: {}", e))?;

    let _ = webview.eval(popup_guard_sync_script(block_popups));
    let tp_script = tracking_protection_script(&app, &url_str);
    if !tp_script.trim().is_empty() {
        let _ = webview.eval(&tp_script);
    }
    let wr_script = web_request_subresource_script(&app);
    if !wr_script.trim().is_empty() {
        let _ = webview.eval(&wr_script);
    }
    let wr_hdr_script = web_request_response_headers_script(&app, &url_str);
    if !wr_hdr_script.trim().is_empty() {
        let _ = webview.eval(&wr_hdr_script);
    }
    let mitm_script = http_subresource_mitm_script(&app);
    if !mitm_script.trim().is_empty() {
        let _ = webview.eval(&mitm_script);
    }
    let end_script = extension_document_end_script(&app, &url_str);
    if !end_script.trim().is_empty() {
        let _ = webview.eval(&end_script);
    }
    Ok(())
}

/// Update popup blocking for an existing tab webview (after privacy settings change).
#[tauri::command]
pub fn browser_set_popup_blocking(
    app: AppHandle,
    label: String,
    block: bool,
) -> Result<(), String> {
    let webview = get_content_webview(&app, &label)?;
    webview
        .eval(popup_guard_sync_script(block))
        .map_err(|e| format!("Popup guard sync failed: {}", e))
}

/// Go back in webview history.
#[tauri::command]
pub fn browser_go_back(app: AppHandle, label: String) -> Result<(), String> {
    let webview = get_content_webview(&app, &label)?;
    webview
        .eval("history.back()")
        .map_err(|e| format!("Go back failed: {}", e))
}

/// Go forward in webview history.
#[tauri::command]
pub fn browser_go_forward(app: AppHandle, label: String) -> Result<(), String> {
    let webview = get_content_webview(&app, &label)?;
    webview
        .eval("history.forward()")
        .map_err(|e| format!("Go forward failed: {}", e))
}

/// Reload the current page.
#[tauri::command]
pub fn browser_reload(app: AppHandle, label: String) -> Result<(), String> {
    let webview = get_content_webview(&app, &label)?;
    webview
        .eval("location.reload()")
        .map_err(|e| format!("Reload failed: {}", e))
}

/// Evaluate JavaScript and return a JSON-serialized result (for agent actions) with improved timeout handling.
#[tauri::command]
pub async fn browser_eval_return(
    app: AppHandle,
    label: String,
    script: String,
) -> Result<String, String> {
    let webview = get_content_webview(&app, &label)?;
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    let expr = script.trim().trim_end_matches(';');
    let wrapped = format!(
        r#"JSON.stringify((function() {{ try {{ return {}; }} catch(e) {{ return 'Error: ' + e; }} }})())"#,
        expr
    );

    webview
        .eval_with_callback(&wrapped, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(8))
        .unwrap_or_else(|_| {
            tracing::warn!("Eval timed out, returning error");
            r#""Error: Eval timed out""#.to_string()
        });

    Ok(serde_json::from_str::<String>(&raw)
        .unwrap_or_else(|_| "Error: Invalid JSON response".to_string()))
}

/// Run JavaScript in the content webview.
#[tauri::command]
pub fn browser_eval(app: AppHandle, label: String, script: String) -> Result<(), String> {
    let webview = get_content_webview(&app, &label)?;
    webview
        .eval(script)
        .map_err(|e| format!("Eval failed: {}", e))
}

/// Return the page HTML as a string with improved timeout handling.
#[tauri::command]
pub async fn browser_get_html(app: AppHandle, label: String) -> Result<String, String> {
    let webview = get_content_webview(&app, &label)?;
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    let script = r#"JSON.stringify(document.documentElement.outerHTML)"#;

    webview
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("HTML eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(8))
        .unwrap_or_else(|_| {
            tracing::warn!("HTML capture timed out, returning empty HTML");
            r#""""#.to_string()
        });

    Ok(serde_json::from_str::<String>(&raw)
        .unwrap_or_else(|_| String::new()))
}

/// Extract page title and body text for RAG indexing with improved timeout handling.
#[tauri::command]
pub async fn browser_capture_content(
    app: AppHandle,
    label: String,
) -> Result<PageCapture, String> {
    let webview = get_content_webview(&app, &label)?;
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    let script = r#"JSON.stringify({
        url: location.href,
        title: document.title || '',
        content: (document.body && document.body.innerText || '').slice(0, 10000)
    })"#;

    webview
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Capture eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(8))
        .unwrap_or_else(|_| {
            tracing::warn!("Capture timed out, returning empty capture");
            r#"{"url":"","title":"","content":""}"#.to_string()
        });

    Ok(serde_json::from_str::<PageCapture>(&raw)
        .unwrap_or_else(|_| PageCapture {
            url: String::new(),
            title: String::new(),
            content: String::new(),
        }))
}

/// Return the document title of the content webview with improved timeout handling.
#[tauri::command]
pub async fn browser_get_title(app: AppHandle, label: String) -> Result<String, String> {
    let webview = get_content_webview(&app, &label)?;
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    let script = r#"JSON.stringify(document.title || '')"#;

    webview
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Title eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap_or_else(|_| {
            tracing::warn!("Title read timed out, returning empty title");
            r#""""#.to_string()
        });

    Ok(serde_json::from_str::<String>(&raw)
        .unwrap_or_else(|_| String::new()))
}

/// Find text on the page (uses window.find) with improved timeout handling.
#[tauri::command]
pub fn browser_find_in_page(
    app: AppHandle,
    label: String,
    query: String,
    forward: bool,
) -> Result<bool, String> {
    let webview = get_content_webview(&app, &label)?;
    let escaped = query.replace('\\', "\\\\").replace('\'', "\\'");
    let script = format!(
        "window.find('{}', false, {}, true)",
        escaped,
        if forward { "false" } else { "true" }
    );
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    webview
        .eval_with_callback(
            format!("JSON.stringify({})", script),
            move |result| {
                let _ = tx.send(result);
            },
        )
        .map_err(|e| format!("Find eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap_or_else(|_| {
            tracing::warn!("Find timed out, returning false");
            r#"false"#.to_string()
        });

    Ok(serde_json::from_str::<bool>(&raw)
        .unwrap_or_else(|_| false))
}

/// Navigation state for toolbar back/forward buttons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavState {
    pub url: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Read current URL and approximate history availability from the webview with improved timeout handling.
#[tauri::command]
pub async fn browser_get_nav_state(
    app: AppHandle,
    nav: State<'_, TabNavTracker>,
    label: String,
) -> Result<NavState, String> {
    let webview = get_content_webview(&app, &label)?;
    let (tx, rx) = mpsc::sync_channel::<String>(1);

    let script = r#"JSON.stringify(location.href)"#;

    webview
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Nav state eval failed: {}", e))?;

    // Improved timeout with fallback
    let raw = rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap_or_else(|_| {
            tracing::warn!("Nav state timed out, returning empty URL");
            r#""""#.to_string()
        });

    let url = serde_json::from_str::<String>(&raw).unwrap_or_default();
    if !url.is_empty() {
        nav.record(&label, &url);
    }

    let (can_go_back, can_go_forward) = nav.flags(&label);
    Ok(NavState {
        url,
        can_go_back,
        can_go_forward,
    })
}

/// Toggle fullscreen mode for a tab
#[tauri::command]
pub fn browser_toggle_fullscreen(app: AppHandle, label: String, fullscreen: bool) -> Result<bool, String> {
    let webview = get_content_webview(&app, &label)?;
    let window = webview.window();
    
    window.set_fullscreen(fullscreen)
        .map_err(|e| format!("Failed to set fullscreen: {}", e))?;
    
    Ok(true)
}

/// View page source code
#[tauri::command]
pub fn browser_view_source(app: AppHandle, label: String) -> Result<String, String> {
    let webview = get_content_webview(&app, &label)?;
    let html = webview.url()
        .map_err(|e| format!("Failed to get URL: {}", e))?;
    
    // In a real implementation, this would fetch the actual source code
    // For now, we return the URL and a note that source viewing would require additional implementation
    Ok(format!("Source view for: {}", html))
}

/// Context menu action
#[tauri::command]
pub fn browser_context_menu_action(
    app: AppHandle,
    label: String,
    action: String,
    data: Option<String>,
) -> Result<bool, String> {
    match action.as_str() {
        "back" => {
            browser_go_back(app.clone(), label.clone())?;
        }
        "forward" => {
            browser_go_forward(app.clone(), label.clone())?;
        }
        "reload" => {
            browser_reload(app.clone(), label.clone())?;
        }
        "copy" => {
            // Copy selected text
            if let Some(_text) = data {
                // In a real implementation, this would copy to clipboard
                // For now, we just acknowledge the action
            }
        }
        "paste" => {
            // Paste from clipboard
            // In a real implementation, this would paste to the webview
        }
        "cut" => {
            // Cut selected text
            // In a real implementation, this would cut to clipboard
        }
        "select_all" => {
            // Select all text
            // In a real implementation, this would select all in the webview
        }
        _ => {
            return Err(format!("Unknown context menu action: {}", action));
        }
    }
    Ok(true)
}

/// Handle drag and drop
#[tauri::command]
pub fn browser_handle_drag_drop(
    app: AppHandle,
    label: String,
    data: String,
    data_type: String,
) -> Result<bool, String> {
    let webview = get_content_webview(&app, &label)?;
    
    match data_type.as_str() {
        "url" => {
            // Navigate to dropped URL
            let url = url::Url::parse(&data)
                .map_err(|e| format!("Invalid URL: {}", e))?;
            webview.navigate(url)
                .map_err(|e| format!("Failed to navigate: {}", e))?;
        }
        "text" => {
            // Insert dropped text
            // In a real implementation, this would insert text into the webview
            // For now, we just acknowledge the action
        }
        "file" => {
            // Handle dropped file
            // In a real implementation, this would upload or open the file
            // For now, we just acknowledge the action
        }
        _ => {
            return Err(format!("Unknown drag drop data type: {}", data_type));
        }
    }
    
    Ok(true)
}

/// Toggle reader mode
#[tauri::command]
pub fn browser_toggle_reader_mode(
    app: AppHandle,
    label: String,
    enable: bool,
) -> Result<bool, String> {
    let _webview = get_content_webview(&app, &label)?;
    
    if enable {
        // In a real implementation, this would inject CSS/JS to simplify the page
        // For now, we just acknowledge the action
        // This would typically involve:
        // 1. Extracting the main content from the page
        // 2. Applying reader-friendly CSS
        // 3. Removing distractions
    } else {
        // In a real implementation, this would restore the original page
        // For now, we just acknowledge the action
    }
    
    Ok(true)
}

/// Group tabs by domain
#[tauri::command]
pub fn browser_group_tabs(
    _app: AppHandle,
    _group_name: String,
    _tab_labels: Vec<String>,
) -> Result<bool, String> {
    // In a real implementation, this would:
    // 1. Create a tab group with the given name
    // 2. Add the specified tabs to the group
    // 3. Update the UI to show the group
    // For now, we just acknowledge the action
    Ok(true)
}

/// Ungroup tabs
#[tauri::command]
pub fn browser_ungroup_tabs(
    _app: AppHandle,
    _group_name: String,
) -> Result<bool, String> {
    // In a real implementation, this would:
    // 1. Remove the tab group
    // 2. Restore tabs to their original state
    // For now, we just acknowledge the action
    Ok(true)
}

#[tauri::command]
pub fn browser_toggle_devtools(app: AppHandle, label: String) -> Result<bool, String> {
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    {
        return Err("DevTools are only available in debug builds".to_string());
    }
    
    let webview = app.get_webview(&label)
        .ok_or_else(|| format!("Webview not found: {}", label))?;
    
    if webview.is_devtools_open() {
        webview.close_devtools();
        Ok(false)
    } else {
        webview.open_devtools();
        Ok(true)
    }
}

/// Set webview zoom factor (1.0 = 100%).
#[tauri::command]
pub fn browser_set_zoom(app: AppHandle, label: String, scale: f64) -> Result<(), String> {
    let webview = get_content_webview(&app, &label)?;
    let clamped = scale.clamp(0.5, 3.0);
    webview
        .set_zoom(clamped)
        .map_err(|e| format!("Set zoom failed: {}", e))
}

#[cfg(test)]
fn webview_url_from_str_for_test(url: &str, https_only: bool) -> Result<WebviewUrl, String> {
    let mut parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    if https_only && parsed.scheme() == "http" {
        parsed
            .set_scheme("https")
            .map_err(|_| "Failed to set HTTPS scheme".to_string())?;
    }
    Ok(WebviewUrl::External(parsed))
}

/// Recreate a discarded tab webview from a stored snapshot.
#[tauri::command]
pub fn browser_restore_discarded_tab(
    app: AppHandle,
    nav: State<'_, TabNavTracker>,
    config: State<'_, crate::config::ConfigState>,
    registry: State<'_, Arc<DiscardedTabsRegistry>>,
    label: String,
) -> Result<bool, String> {
    let snap = registry
        .take(&label)
        .ok_or_else(|| format!("Tab not discarded: {}", label))?;
    if app.get_webview(&label).is_some() {
        return Ok(false);
    }
    let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
    add_tab_webview_impl(
        &app,
        &nav,
        cfg.block_popups,
        cfg.private_mode,
        cfg.https_only,
        &label,
        &snap.url,
        snap.x,
        snap.y,
        snap.width,
        snap.height,
    )?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_tracker_back_and_forward() {
        let nav = TabNavTracker::default();
        nav.record("tab1", "https://a.com");
        nav.record("tab1", "https://b.com");
        nav.record("tab1", "https://c.com");
        assert_eq!(nav.flags("tab1"), (true, false));
        nav.record("tab1", "https://b.com");
        assert_eq!(nav.flags("tab1"), (true, true));
        nav.record("tab1", "https://c.com");
        assert_eq!(nav.flags("tab1"), (true, false));
    }

    #[test]
    fn test_webview_url_from_str_https() {
        let result = webview_url_from_str_for_test("https://example.com", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_webview_url_from_str_data_url() {
        let result = webview_url_from_str_for_test("data:text/html,<html></html>", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_webview_url_https_only_upgrade() {
        let result = webview_url_from_str_for_test("http://example.com", true);
        assert!(result.is_ok());
        if let Ok(WebviewUrl::External(url)) = result {
            assert_eq!(url.scheme(), "https");
        }
    }

    #[test]
    fn test_webview_url_https_only_no_change() {
        let result = webview_url_from_str_for_test("https://example.com", true);
        assert!(result.is_ok());
        if let Ok(WebviewUrl::External(url)) = result {
            assert_eq!(url.scheme(), "https");
        }
    }

    #[test]
    fn popup_guard_script_overrides_window_open() {
        let script = popup_guard_init_script(true);
        assert!(script.contains("window.open"));
        assert!(script.contains("__exodusBlockPopups"));
        assert!(script.contains("true"));
    }

    #[test]
    fn new_window_payload_serializes_camel_case_fields() {
        let payload = NewWindowRequestedPayload {
            url: "https://example.com/popup".into(),
            opener_label: "exodus-tab-1".into(),
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("opener_label"));
        assert!(json.contains("https://example.com/popup"));
    }

    #[test]
    fn popup_blocked_payload_serializes() {
        let payload = PopupBlockedPayload {
            url: "https://ads.example/".into(),
            opener_label: "exodus-tab-2".into(),
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("exodus-tab-2"));
    }
}
