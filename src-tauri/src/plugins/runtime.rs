//! Exodus Browser — Web Extension runtime messaging (sendResponse, cross-context delivery).

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use super::background::background_webview_label;
use super::tabs::TabRegistry;

/// Outbound message from a content-script page flush.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeOutboundMessage {
    pub extension_id: String,
    pub request_id: String,
    pub message: Value,
    #[serde(default)]
    pub source_webview_label: Option<String>,
}

/// Background → content message flushed from a background host.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeBackgroundOutbound {
    pub extension_id: String,
    pub target_tab_id: i64,
    pub message: Value,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub source_webview_label: Option<String>,
}

/// Content-script `chrome.tabs.sendMessage` flushed from a tab webview.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTabMessageOutbound {
    pub extension_id: String,
    pub target_tab_id: i64,
    pub message: Value,
    pub request_id: String,
    #[serde(default)]
    pub source_webview_label: Option<String>,
}

/// Tab open request from `chrome.tabs.create`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabCreateRequest {
    pub request_id: String,
    pub url: String,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub source_webview_label: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Ack for tab create (UI → Rust → source webview).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabCreateAck {
    pub request_id: String,
    pub source_webview_label: String,
    pub chrome_tab_id: i64,
    pub tab_id: String,
    pub url: String,
    pub title: String,
}

/// Emitted when extensions request new browser tabs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionTabsCreatePayload {
    pub requests: Vec<TabCreateRequest>,
}

/// Tab control op from `chrome.tabs.update` / `remove` / `reload` shims.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabOpRequest {
    pub op: String,
    #[serde(default)]
    pub extension_id: Option<String>,
    #[serde(default)]
    pub chrome_tab_id: i64,
    #[serde(default)]
    pub tab_ids: Vec<i64>,
    #[serde(default)]
    pub update_properties: Option<Value>,
}

/// Emitted when extensions request tab updates/removals/reloads.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionTabOpsPayload {
    pub ops: Vec<TabOpRequest>,
}

/// `chrome.scripting.executeScript` flush entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptingExecuteRequest {
    pub extension_id: String,
    pub target_tab_id: i64,
    #[serde(default)]
    pub func: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<Value>>,
    #[serde(default)]
    pub files: Option<Vec<String>>,
}

/// JavaScript evaluated in a background webview to dispatch an inbound message.
pub fn deliver_background_message_script(request_id: &str, message_json: &str) -> String {
    let escaped_id = request_id.replace('\\', "\\\\").replace('\'', "\\'");
    let payload = message_json.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  var reqId = '{escaped_id}';
  var msg = JSON.parse('{payload}');
  window.__exodusPendingReplies = window.__exodusPendingReplies || {{}};
  var listeners = window.__exodusMessageListeners || [];
  for (var i = 0; i < listeners.length; i++) {{
    try {{
      var replied = false;
      var rv = listeners[i](msg, {{}}, function sendResponse(r) {{
        if (replied) return;
        replied = true;
        window.__exodusPendingReplies[reqId] = r;
      }});
      if (rv && typeof rv.then === 'function') {{
        rv.then(function(r) {{ window.__exodusPendingReplies[reqId] = r; }});
      }}
    }} catch (e) {{
      console.error('[Exodus background] onMessage', e);
    }}
  }}
}})();"#
    )
}

/// Collect and clear background pending replies.
pub fn collect_background_replies_script() -> &'static str {
    r#"(function() {
  var out = JSON.stringify(window.__exodusPendingReplies || {});
  window.__exodusPendingReplies = {};
  return out;
})()"#
}

/// Deliver a runtime reply to a content-script webview.
pub fn deliver_content_reply_script(request_id: &str, response_json: &str) -> String {
    let escaped_id = request_id.replace('\\', "\\\\").replace('\'', "\\'");
    let payload = response_json.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  var reqId = '{escaped_id}';
  var resp = JSON.parse('{payload}');
  if (window.__exodusResolveReply) window.__exodusResolveReply(reqId, resp);
}})();"#
    )
}

/// Deliver fire-and-forget message to tab content listeners (no response).
pub fn deliver_content_message_script(extension_id: &str, message_json: &str) -> String {
    let escaped_ext = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    let payload = message_json.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  var extId = '{escaped_ext}';
  var msg = JSON.parse('{payload}');
  var listeners = window.__exodusContentMessageListeners || [];
  for (var i = 0; i < listeners.length; i++) {{
    try {{
      if (listeners[i].extId !== extId) continue;
      listeners[i].fn(msg, {{}}, function() {{}});
    }} catch (e) {{
      console.error('[Exodus content] onMessage', e);
    }}
  }}
}})();"#
    )
}

/// Deliver `chrome.tabs.sendMessage` to a tab and collect `sendResponse` by request id.
pub fn deliver_content_tab_message_script(
    request_id: &str,
    extension_id: &str,
    message_json: &str,
) -> String {
    let escaped_id = request_id.replace('\\', "\\\\").replace('\'', "\\'");
    let escaped_ext = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    let payload = message_json.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  var reqId = '{escaped_id}';
  var extId = '{escaped_ext}';
  var msg = JSON.parse('{payload}');
  window.__exodusTabPendingReplies = window.__exodusTabPendingReplies || {{}};
  var listeners = window.__exodusContentMessageListeners || [];
  for (var i = 0; i < listeners.length; i++) {{
    try {{
      if (listeners[i].extId !== extId) continue;
      var replied = false;
      var rv = listeners[i].fn(msg, {{}}, function sendResponse(r) {{
        if (replied) return;
        replied = true;
        window.__exodusTabPendingReplies[reqId] = r;
      }});
      if (rv && typeof rv.then === 'function') {{
        rv.then(function(r) {{ window.__exodusTabPendingReplies[reqId] = r; }});
      }}
    }} catch (e) {{
      console.error('[Exodus content] onMessage', e);
    }}
  }}
}})();"#
    )
}

/// Collect and clear tab content `sendResponse` payloads.
pub fn collect_tab_replies_script() -> &'static str {
    r#"(function() {
  var out = JSON.stringify(window.__exodusTabPendingReplies || {});
  window.__exodusTabPendingReplies = {};
  return out;
})()"#
}

/// Deliver tab-create ack to the requesting webview.
pub fn deliver_tab_create_ack_script(ack_json: &str) -> String {
    let payload = ack_json.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  var ack = JSON.parse('{payload}');
  if (window.__exodusResolveTabCreate) window.__exodusResolveTabCreate(ack);
}})();"#
    )
}

/// Script to read background host outbox (tabs.sendMessage from SW).
pub fn flush_background_outbox_script() -> &'static str {
    r#"JSON.stringify(window.__exodusBgOutbox || [])"#
}

fn eval_webview_string(webview: &tauri::Webview, script: &str) -> Result<String, String> {
    let (tx, rx) = mpsc::sync_channel::<String>(1);
    webview
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .map_err(|e| format!("Extension eval failed: {e}"))?;
    let raw = rx
        .recv_timeout(Duration::from_secs(3))
        .map_err(|_| "Extension eval timed out".to_string())?;
    Ok(serde_json::from_str::<String>(&raw).unwrap_or(raw))
}

fn get_webview(app: &AppHandle, label: &str) -> Option<tauri::Webview> {
    app.get_webview(label)
}

/// Process content-script runtime outbox (sendMessage → background, sendResponse back).
pub fn process_content_outbox(
    app: &AppHandle,
    messages: &[RuntimeOutboundMessage],
) -> Result<(), String> {
    for msg in messages {
        let bg_label = background_webview_label(&msg.extension_id);
        let Some(bg_wv) = get_webview(app, &bg_label) else {
            continue;
        };

        let _ = bg_wv.eval("window.__exodusPendingReplies = {};");
        let message_json = serde_json::to_string(&msg.message).map_err(|e| format!("Message JSON: {e}"))?;
        let deliver = deliver_background_message_script(&msg.request_id, &message_json);
        let _ = bg_wv.eval(&deliver);

        let replies_raw = eval_webview_string(&bg_wv, collect_background_replies_script())?;
        let replies: HashMap<String, Value> = serde_json::from_str(&replies_raw).unwrap_or_default();

        if let Some(response) = replies.get(&msg.request_id) {
            if let Some(source) = &msg.source_webview_label {
                if let Some(src_wv) = get_webview(app, source) {
                    let response_json =
                        serde_json::to_string(response).map_err(|e| format!("Reply JSON: {e}"))?;
                    let script = deliver_content_reply_script(&msg.request_id, &response_json);
                    let _ = src_wv.eval(&script);
                }
            }
        }
    }
    Ok(())
}

/// Deliver a reply to a webview (content or background host).
pub fn deliver_reply_to_webview(
    app: &AppHandle,
    webview_label: &str,
    request_id: &str,
    response: &Value,
) -> Result<(), String> {
    let Some(wv) = get_webview(app, webview_label) else {
        return Ok(());
    };
    let response_json = serde_json::to_string(response).map_err(|e| format!("Reply JSON: {e}"))?;
    let script = deliver_content_reply_script(request_id, &response_json);
    let _ = wv.eval(&script);
    Ok(())
}

/// Deliver tab message with response collection; returns response value when received.
pub fn deliver_tab_message_with_response(
    app: &AppHandle,
    target_webview_label: &str,
    request_id: &str,
    extension_id: &str,
    message: &Value,
) -> Result<Option<Value>, String> {
    let Some(wv) = get_webview(app, target_webview_label) else {
        return Ok(None);
    };
    let _ = wv.eval("window.__exodusTabPendingReplies = {};");
    let message_json = serde_json::to_string(message).map_err(|e| format!("Message JSON: {e}"))?;
    let deliver = deliver_content_tab_message_script(request_id, extension_id, &message_json);
    let _ = wv.eval(&deliver);
    let replies_raw = eval_webview_string(&wv, collect_tab_replies_script())?;
    let replies: HashMap<String, Value> = serde_json::from_str(&replies_raw).unwrap_or_default();
    Ok(replies.get(request_id).cloned())
}

/// Deliver tab-targeted messages to content scripts (background or content origin).
pub fn process_tab_targeted_messages(
    app: &AppHandle,
    tabs: &TabRegistry,
    extension_id: &str,
    target_tab_id: i64,
    message: &Value,
) -> Result<(), String> {
    let Some(target) = tabs.find_by_chrome_id(target_tab_id) else {
        return Ok(());
    };
    let Some(wv) = get_webview(app, &target.webview_label) else {
        return Ok(());
    };
    let message_json = serde_json::to_string(message).map_err(|e| format!("Message JSON: {e}"))?;
    let script = deliver_content_message_script(extension_id, &message_json);
    let _ = wv.eval(&script);
    Ok(())
}

/// Deliver background outbox messages to target tab content scripts.
pub fn process_background_outbox(
    app: &AppHandle,
    tabs: &TabRegistry,
    messages: &[RuntimeBackgroundOutbound],
) -> Result<(), String> {
    for msg in messages {
        if let (Some(req_id), Some(target)) = (
            msg.request_id.as_deref(),
            tabs.find_by_chrome_id(msg.target_tab_id),
        ) {
            if let Some(response) = deliver_tab_message_with_response(
                app,
                &target.webview_label,
                req_id,
                &msg.extension_id,
                &msg.message,
            )? {
                if let Some(source) = &msg.source_webview_label {
                    deliver_reply_to_webview(app, source, req_id, &response)?;
                }
            }
            continue;
        }
        process_tab_targeted_messages(
            app,
            tabs,
            &msg.extension_id,
            msg.target_tab_id,
            &msg.message,
        )?;
    }
    Ok(())
}

/// Deliver content-script `chrome.tabs.sendMessage` outbox entries.
pub fn process_tab_message_outbox(
    app: &AppHandle,
    tabs: &TabRegistry,
    messages: &[RuntimeTabMessageOutbound],
) -> Result<(), String> {
    for msg in messages {
        let Some(target) = tabs.find_by_chrome_id(msg.target_tab_id) else {
            continue;
        };
        if let Some(response) = deliver_tab_message_with_response(
            app,
            &target.webview_label,
            &msg.request_id,
            &msg.extension_id,
            &msg.message,
        )? {
            if let Some(source) = &msg.source_webview_label {
                deliver_reply_to_webview(app, source, &msg.request_id, &response)?;
            }
        }
    }
    Ok(())
}

/// Execute flushed scripting requests in target tab webviews.
pub fn process_scripting_outbox(
    app: &AppHandle,
    tabs: &TabRegistry,
    mgr: &super::manager::ExtensionManager,
    requests: &[ScriptingExecuteRequest],
) -> Result<(), String> {
    let site = app.try_state::<super::site_permissions::SitePermissionStore>();
    for req in requests {
        let Some(target) = tabs.find_by_chrome_id(req.target_tab_id) else {
            continue;
        };
        if let Some(site_store) = &site {
            if !mgr.host_access_allowed(&req.extension_id, &target.url, site_store) {
                continue;
            }
        }
        let Some(wv) = get_webview(app, &target.webview_label) else {
            continue;
        };
        let script = mgr
            .build_scripting_injection(&req.extension_id, req)
            .map_err(|e| e.to_string())?;
        if !script.is_empty() {
            let _ = wv.eval(&script);
        }
    }
    Ok(())
}

/// Flush a background host outbox and deliver to tabs.
pub fn flush_background_host(
    app: &AppHandle,
    tabs: &TabRegistry,
    extension_id: &str,
) -> Result<(), String> {
    let label = background_webview_label(extension_id);
    let Some(wv) = get_webview(app, &label) else {
        return Ok(());
    };
    let raw = eval_webview_string(&wv, flush_background_outbox_script())?;
    let messages: Vec<RuntimeBackgroundOutbound> = serde_json::from_str(&raw).unwrap_or_default();
    let _ = wv.eval("window.__exodusBgOutbox = [];");
    process_background_outbox(app, tabs, &messages)
}

/// Push tab-create acks into source webviews.
pub fn deliver_tab_create_acks(app: &AppHandle, acks: &[TabCreateAck]) -> Result<(), String> {
    for ack in acks {
        let ack_json = serde_json::to_string(ack).map_err(|e| format!("Ack JSON: {e}"))?;
        let script = deliver_tab_create_ack_script(&ack_json);
        if let Some(wv) = get_webview(app, &ack.source_webview_label) {
            let _ = wv.eval(&script);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deliver_scripts_contain_resolve_hook() {
        let s = deliver_content_reply_script("r1", r#"{"pong":1}"#);
        assert!(s.contains("__exodusResolveReply"));
    }

    #[test]
    fn tab_message_outbound_deserializes() {
        let raw = r#"{"extensionId":"ext","targetTabId":2,"message":{"x":1},"requestId":"tm1","sourceWebviewLabel":"exodus-tab-a"}"#;
        let msg: RuntimeTabMessageOutbound = serde_json::from_str(raw).expect("json");
        assert_eq!(msg.target_tab_id, 2);
        assert_eq!(msg.request_id, "tm1");
    }

    #[test]
    fn background_outbound_with_request_id_deserializes() {
        let raw = r#"{"extensionId":"ext","targetTabId":1,"message":{"t":"tab-ping"},"requestId":"bg1","sourceWebviewLabel":"exodus-ext-bg-ext"}"#;
        let msg: RuntimeBackgroundOutbound = serde_json::from_str(raw).expect("json");
        assert_eq!(msg.request_id.as_deref(), Some("bg1"));
    }

    #[test]
    fn tab_message_script_collects_pending_replies() {
        let s = deliver_content_tab_message_script("tm9", "ext", r#"{"type":"ping"}"#);
        assert!(s.contains("__exodusTabPendingReplies"));
        assert!(collect_tab_replies_script().contains("__exodusTabPendingReplies"));
    }
}
