//! Exodus Browser — injected Chrome API shims for content scripts (storage, tabs, runtime).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::permission_pending::PermissionRequestOutbound;
use super::runtime::{
    RuntimeOutboundMessage, RuntimeTabMessageOutbound, ScriptingExecuteRequest, TabCreateRequest,
    TabOpRequest,
};
use super::notifications::NotificationOptions;
use super::manager::ExtensionManager;
use super::permissions::Permission;
use super::tabs::TabRegistry;

/// Combined flush parse result.
pub struct PageFlushResult {
    pub storage_json: String,
    pub outbox: Vec<RuntimeOutboundMessage>,
    pub tab_requests: Vec<TabCreateRequest>,
    pub tab_messages: Vec<RuntimeTabMessageOutbound>,
    pub tab_ops: Vec<TabOpRequest>,
    pub scripting: Vec<ScriptingExecuteRequest>,
    pub notifications: Vec<NotificationFlushRequest>,
    pub permission_requests: Vec<PermissionRequestOutbound>,
    pub dnr_updates: Vec<DnrFlushUpdate>,
    pub web_request_rules: Vec<super::web_request::WebRequestFlushRule>,
    pub action_ops: Vec<ActionFlushOp>,
}

/// `chrome.action` flush operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionFlushOp {
    pub extension_id: String,
    pub op: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Notification create request from content/background flush.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFlushRequest {
    pub extension_id: String,
    #[serde(default)]
    pub notification_id: Option<String>,
    pub options: NotificationOptions,
}

/// declarativeNetRequest dynamic rule update from flush.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnrFlushUpdate {
    pub extension_id: String,
    #[serde(default)]
    pub add_rules: Vec<Value>,
    #[serde(default)]
    pub remove_rule_ids: Vec<String>,
}

/// Build global prelude: storage seeds + tab list + per-extension chrome shims.
pub fn build_extension_prelude(
    mgr: &ExtensionManager,
    tabs: &TabRegistry,
    webview_label: &str,
) -> String {
    let storage_seed = build_storage_seed(mgr);
    let tabs_json = tabs.inject_json();
    let escaped_label = webview_label.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  window.__exodusWebviewLabel = '{escaped_label}';
  {storage_seed}
  window.__exodusTabs = {tabs_json};
  window.__exodusRuntimeOutbox = window.__exodusRuntimeOutbox || [];
  window.__exodusTabRequests = window.__exodusTabRequests || [];
  window.__exodusTabMessageOutbox = window.__exodusTabMessageOutbox || [];
  window.__exodusTabOpsOutbox = window.__exodusTabOpsOutbox || [];
  window.__exodusScriptingOutbox = window.__exodusScriptingOutbox || [];
  window.__exodusNotificationOutbox = window.__exodusNotificationOutbox || [];
  window.__exodusPermRequests = window.__exodusPermRequests || [];
  window.__exodusDnrOutbox = window.__exodusDnrOutbox || [];
  window.__exodusPendingPromises = window.__exodusPendingPromises || {{}};
  window.__exodusContentMessageListeners = window.__exodusContentMessageListeners || [];
  if (!window.__exodusResolveReply) {{
    window.__exodusResolveReply = function(reqId, resp) {{
      var e = window.__exodusPendingPromises[reqId];
      if (e) {{ e.resolve(resp); delete window.__exodusPendingPromises[reqId]; }}
    }};
  }}
  if (!window.__exodusResolveTabCreate) {{
    window.__exodusResolveTabCreate = function(ack) {{
      var e = window.__exodusPendingPromises[ack.requestId];
      if (e) {{
        e.resolve({{ id: ack.chromeTabId, url: ack.url, title: ack.title, active: true }});
        delete window.__exodusPendingPromises[ack.requestId];
      }}
    }};
  }}
  {allama_shim}
  {chrome_shims}
}})();"#,
        allama_shim = build_exodus_allama_shim(mgr.allama_http_port()),
        chrome_shims = build_chrome_api_shims(mgr, false),
    )
}

/// Chrome API prelude for an extension action popup webview (`exodus-ext-popup-{id}`).
pub fn build_extension_popup_prelude(
    mgr: &ExtensionManager,
    tabs: &TabRegistry,
    extension_id: &str,
) -> String {
    build_extension_prelude_for_background(mgr, tabs, extension_id)
}

/// Build prelude for a single extension background service worker host.
pub fn build_extension_prelude_for_background(
    mgr: &ExtensionManager,
    tabs: &TabRegistry,
    extension_id: &str,
) -> String {
    let storage_seed = build_storage_seed_for(mgr, extension_id);
    let tabs_json = tabs.inject_json();
    let bg_label = super::background::background_webview_label(extension_id);
    let escaped_label = bg_label.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"{storage_seed}
  window.__exodusWebviewLabel = '{escaped_label}';
  window.__exodusTabs = {tabs_json};
  window.__exodusMessageListeners = window.__exodusMessageListeners || [];
  window.__exodusPendingPromises = window.__exodusPendingPromises || {{}};
  if (!window.__exodusResolveReply) {{
    window.__exodusResolveReply = function(reqId, resp) {{
      var e = window.__exodusPendingPromises[reqId];
      if (e) {{ e.resolve(resp); delete window.__exodusPendingPromises[reqId]; }}
    }};
  }}
  {allama_shim}
  {chrome_shims}"#,
        allama_shim = build_exodus_allama_shim(mgr.allama_http_port()),
        chrome_shims = build_chrome_api_shims_for_extension(mgr, extension_id, true),
    )
}

/// Injected Allama HTTP helper for extensions (`window.exodus.allama`).
fn build_exodus_allama_shim(port: u16) -> String {
    format!(
        r#"
  window.exodus = window.exodus || {{}};
  window.exodus.allama = {{
    port: {port},
    baseUrl: 'http://127.0.0.1:{port}',
    health: function() {{
      return fetch('http://127.0.0.1:{port}/api/tags').then(function(r) {{ return r.ok; }}).catch(function() {{ return false; }});
    }},
    chat: function(messages, model) {{
      return fetch('http://127.0.0.1:{port}/api/chat', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify({{ model: model || 'exodus-default', messages: messages, stream: false }})
      }}).then(function(r) {{ return r.json(); }}).then(function(j) {{
        return (j.message && j.message.content) || '';
      }});
    }},
    generate: function(prompt, model) {{
      return fetch('http://127.0.0.1:{port}/api/generate', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify({{ model: model || 'exodus-default', prompt: prompt, stream: false }})
      }}).then(function(r) {{ return r.json(); }}).then(function(j) {{ return j.response || ''; }});
    }},
    embed: function(text, model) {{
      return fetch('http://127.0.0.1:{port}/v1/embeddings', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify({{ model: model || 'nomic-embed-text', input: String(text).slice(0, 8000) }})
      }}).then(function(r) {{ return r.json(); }}).then(function(j) {{
        var row = j.data && j.data[0];
        return (row && row.embedding) || [];
      }});
    }},
    streamChat: function(messages, model, callbacks) {{
      var cbs = callbacks || {{}};
      return fetch('http://127.0.0.1:{port}/v1/chat/completions', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify({{ model: model || 'exodus-default', messages: messages, stream: true }})
      }}).then(function(res) {{
        if (!res.ok) {{
          return res.text().then(function(t) {{
            if (cbs.onError) cbs.onError('HTTP ' + res.status + ': ' + t);
          }});
        }}
        var reader = res.body && res.body.getReader();
        if (!reader) {{
          if (cbs.onError) cbs.onError('No response body');
          return;
        }}
        var decoder = new TextDecoder();
        var buffer = '';
        function pump() {{
          return reader.read().then(function(result) {{
            if (result.done) {{
              if (cbs.onDone) cbs.onDone();
              return;
            }}
            buffer += decoder.decode(result.value, {{ stream: true }});
            for (;;) {{
              var lineEnd = buffer.indexOf('\\n');
              if (lineEnd === -1) break;
              var line = buffer.slice(0, lineEnd).trim();
              buffer = buffer.slice(lineEnd + 1);
              if (!line.startsWith('data: ')) continue;
              var data = line.slice(6).trim();
              if (data === '[DONE]') {{
                if (cbs.onDone) cbs.onDone();
                return;
              }}
              try {{
                var parsed = JSON.parse(data);
                var content = parsed.choices && parsed.choices[0] && parsed.choices[0].delta && parsed.choices[0].delta.content;
                if (content && cbs.onChunk) cbs.onChunk(content);
              }} catch (e) {{ /* ignore malformed SSE */ }}
            }}
            return pump();
          }});
        }}
        return pump();
      }}).catch(function(err) {{
        if (cbs.onError) cbs.onError(String(err));
      }});
    }}
  }};"#,
    )
}

fn build_storage_seed(mgr: &ExtensionManager) -> String {
    let mut root = Map::new();
    for ext in mgr.list() {
        if !ext.enabled {
            continue;
        }
        if let Some(data) = mgr.storage_seed_for(&ext.id) {
            root.insert(ext.id.clone(), Value::Object(data));
        }
    }
    let json = serde_json::to_string(&root).unwrap_or_else(|_| "{}".to_string());
    format!("window.__exodusStorage = {json};")
}

fn build_storage_seed_for(mgr: &ExtensionManager, extension_id: &str) -> String {
    let data = mgr
        .storage_seed_for(extension_id)
        .map(Value::Object)
        .unwrap_or(Value::Object(Map::new()));
    let mut root = Map::new();
    root.insert(extension_id.to_string(), data);
    let json = serde_json::to_string(&root).unwrap_or_else(|_| "{}".to_string());
    format!("window.__exodusStorage = {json};")
}

fn build_chrome_api_shims(mgr: &ExtensionManager, background: bool) -> String {
    let mut parts = Vec::new();
    for ext in mgr.list() {
        if !ext.enabled {
            continue;
        }
        parts.push(build_chrome_api_shims_for_extension(mgr, &ext.id, background));
    }
    parts.join("\n")
}

fn build_chrome_api_shims_for_extension(
    mgr: &ExtensionManager,
    extension_id: &str,
    background: bool,
) -> String {
    let perms = match mgr.permissions_for(extension_id) {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    let mut parts = Vec::new();
    if perms.contains(&Permission::Storage) {
        parts.push(build_storage_shim(&escaped));
    }
    if perms.contains(&Permission::Tabs) || perms.contains(&Permission::ActiveTab) {
        parts.push(build_tabs_shim());
        if !background {
            parts.push(build_tabs_create_shim());
            parts.push(build_tabs_send_message_shim(&escaped));
        }
        parts.push(build_tabs_control_shim(&escaped));
    }
    if perms.contains(&Permission::Scripting) {
        parts.push(build_scripting_shim(&escaped));
    }
    if perms.contains(&Permission::Notifications) {
        parts.push(build_notifications_shim(&escaped));
    }
    if perms.contains(&Permission::DeclarativeNetRequest) {
        parts.push(build_dnr_shim(&escaped));
    }
    if perms.contains(&Permission::WebRequest) {
        parts.push(build_web_request_shim(&escaped));
    }
    parts.push(build_action_shim(&escaped));
    parts.push(build_permissions_request_shim(&escaped));
    if background {
        parts.push(build_runtime_background_shim(extension_id));
    } else {
        parts.push(build_runtime_send_shim(extension_id));
    }
    parts.join("\n")
}

fn build_storage_shim(extension_id: &str) -> String {
    format!(
        r#"(function(extId) {{
  if (!window.chrome) window.chrome = {{}};
  if (!window.chrome.storage) window.chrome.storage = {{}};
  if (window.chrome.storage.local && window.chrome.storage.local.__exodusInstalled) return;
  var bag = (window.__exodusStorage && window.__exodusStorage[extId]) ? window.__exodusStorage[extId] : {{}};
  if (!window.__exodusStorage) window.__exodusStorage = {{}};
  window.__exodusStorage[extId] = bag;
  function normalizeKeys(keys) {{
    if (keys == null) return null;
    if (typeof keys === 'string') return [keys];
    if (Array.isArray(keys)) return keys;
    if (typeof keys === 'object') return Object.keys(keys);
    return null;
  }}
  function pick(bag, keys) {{
    var out = {{}};
    if (!keys) {{
      for (var k in bag) if (Object.prototype.hasOwnProperty.call(bag, k)) out[k] = bag[k];
      return out;
    }}
    for (var i = 0; i < keys.length; i++) {{
      var k = keys[i];
      if (Object.prototype.hasOwnProperty.call(bag, k)) out[k] = bag[k];
    }}
    return out;
  }}
  window.chrome.storage.local = {{
    __exodusInstalled: true,
    get: function(keys, cb) {{
      var result = pick(bag, normalizeKeys(keys));
      var p = Promise.resolve(result);
      if (typeof keys === 'function') {{ keys(result); return p; }}
      if (cb) p.then(cb);
      return p;
    }},
    set: function(items, cb) {{
      for (var k in items) if (Object.prototype.hasOwnProperty.call(items, k)) bag[k] = items[k];
      window.__exodusStorageDirty = true;
      var p = Promise.resolve();
      if (typeof items === 'function') {{ items(); return p; }}
      if (cb) p.then(cb);
      return p;
    }},
    remove: function(keys, cb) {{
      var list = normalizeKeys(keys) || [];
      for (var i = 0; i < list.length; i++) delete bag[list[i]];
      window.__exodusStorageDirty = true;
      var p = Promise.resolve();
      if (typeof keys === 'function') {{ keys(); return p; }}
      if (cb) p.then(cb);
      return p;
    }}
  }};
}})('{extension_id}');"#
    )
}

/// Content-script `chrome.runtime.sendMessage` shim.
pub fn build_runtime_send_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.runtime) window.chrome.runtime = {{}};
  if (window.chrome.runtime.__exodusSendInstalled) return;
  window.chrome.runtime.__exodusSendInstalled = true;
  window.chrome.runtime.id = extId;
  window.chrome.runtime.sendMessage = function(message, options, cb) {{
    if (typeof options === 'function') {{ cb = options; options = {{}}; }}
    var reqId = 'rt' + Math.random().toString(36).slice(2);
    window.__exodusRuntimeOutbox.push({{
      extensionId: extId,
      requestId: reqId,
      message: message,
      sourceWebviewLabel: window.__exodusWebviewLabel || ''
    }});
    window.__exodusStorageDirty = true;
    var p = new Promise(function(resolve) {{
      window.__exodusPendingPromises[reqId] = {{ resolve: resolve }};
      setTimeout(function() {{ resolve(undefined); }}, 30000);
    }});
    if (cb) p.then(cb);
    return p;
  }};
  window.chrome.runtime.getManifest = function() {{
    return window.__exodusManifest || {{}};
  }};
  window.chrome.runtime.onInstalled = {{
    addListener: function(fn) {{
      if (!window.__exodusInstalledListeners) window.__exodusInstalledListeners = [];
      window.__exodusInstalledListeners.push(fn);
    }}
  }};
  if (!window.chrome.runtime.onMessage) {{
    window.chrome.runtime.onMessage = {{
      addListener: function(fn) {{
        window.__exodusContentMessageListeners = window.__exodusContentMessageListeners || [];
        window.__exodusContentMessageListeners.push({{ extId: extId, fn: fn }});
      }}
    }};
  }}
  window.chrome.runtime.getURL = function(path) {{
    return 'extension://' + extId + '/' + path.replace(/^\//, '');
  }};
}})('{escaped}');"#
    )
}

/// Background `chrome.runtime.onMessage` shim.
pub fn build_runtime_background_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome) window.chrome = {{}};
  if (!window.chrome.runtime) window.chrome.runtime = {{}};
  window.chrome.runtime.id = extId;
  window.__exodusMessageListeners = window.__exodusMessageListeners || [];
  if (!window.chrome.runtime.onMessage) {{
    window.chrome.runtime.onMessage = {{
      addListener: function(fn) {{
        window.__exodusMessageListeners.push(fn);
      }}
    }};
  }}
  window.__exodusBgOutbox = window.__exodusBgOutbox || [];
  if (!window.chrome.tabs) window.chrome.tabs = {{}};
  window.chrome.tabs.sendMessage = function(tabId, message, options, cb) {{
    if (typeof options === 'function') {{ cb = options; options = {{}}; }}
    var reqId = 'bg' + Math.random().toString(36).slice(2);
    window.__exodusBgOutbox.push({{
      extensionId: extId,
      targetTabId: tabId,
      message: message,
      requestId: reqId,
      sourceWebviewLabel: window.__exodusWebviewLabel || ''
    }});
    window.__exodusStorageDirty = true;
    var p = new Promise(function(resolve) {{
      window.__exodusPendingPromises[reqId] = {{ resolve: resolve }};
      setTimeout(function() {{ resolve(undefined); }}, 30000);
    }});
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

fn build_tabs_shim() -> String {
    r#"(function() {
  if (!window.chrome) window.chrome = {};
  if (window.chrome.tabs && window.chrome.tabs.query && window.chrome.tabs.__exodusInstalled) return;
  window.chrome.tabs = window.chrome.tabs || {};
  window.chrome.tabs.__exodusInstalled = true;
  window.chrome.tabs.query = function(query, cb) {
    var tabs = (window.__exodusTabs || []).map(function(t) {
      return {
        id: t.chromeTabId,
        index: t.index,
        active: !!t.active,
        url: t.url,
        title: t.title,
        windowId: 1
      };
    });
    var q = query || {};
    var out = tabs.filter(function(t) {
      if (q.active === true && !t.active) return false;
      if (q.currentWindow === true && !t.active) return false;
      return true;
    });
    var p = Promise.resolve(out);
    if (typeof query === 'function') { query(out); return p; }
    if (cb) p.then(cb);
    return p;
  };
})();"#
    .to_string()
}

fn build_tabs_create_shim() -> String {
    r#"(function() {
  if (!window.chrome.tabs || window.chrome.tabs.__exodusCreateInstalled) return;
  window.chrome.tabs.__exodusCreateInstalled = true;
  window.chrome.tabs.create = function(props, cb) {
    props = props || {};
    var reqId = 'tab' + Math.random().toString(36).slice(2);
    window.__exodusTabRequests.push({
      requestId: reqId,
      url: props.url || 'about:blank',
      active: props.active !== false,
      sourceWebviewLabel: window.__exodusWebviewLabel || ''
    });
    window.__exodusStorageDirty = true;
    var p = new Promise(function(resolve) {
      window.__exodusPendingPromises[reqId] = { resolve: resolve };
      setTimeout(function() { resolve({ id: -1, url: props.url || '' }); }, 30000);
    });
    if (cb) p.then(cb);
    return p;
  };
})();"#
    .to_string()
}

fn build_tabs_send_message_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.tabs) window.chrome.tabs = {{}};
  if (window.chrome.tabs.__exodusSendInstalled) return;
  window.chrome.tabs.__exodusSendInstalled = true;
  window.chrome.tabs.sendMessage = function(tabId, message, options, cb) {{
    if (typeof options === 'function') {{ cb = options; options = {{}}; }}
    var reqId = 'tm' + Math.random().toString(36).slice(2);
    window.__exodusTabMessageOutbox.push({{
      extensionId: extId,
      targetTabId: tabId,
      message: message,
      requestId: reqId,
      sourceWebviewLabel: window.__exodusWebviewLabel || ''
    }});
    window.__exodusStorageDirty = true;
    var p = new Promise(function(resolve) {{
      window.__exodusPendingPromises[reqId] = {{ resolve: resolve }};
      setTimeout(function() {{ resolve(undefined); }}, 30000);
    }});
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

/// Per-extension `chrome.tabs.update` / `remove` / `reload` (flush via `__exodusTabOpsOutbox`).
fn build_tabs_control_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.tabs) window.chrome.tabs = {{}};
  if (!window.chrome.tabs.__exodusControl) window.chrome.tabs.__exodusControl = {{}};
  if (window.chrome.tabs.__exodusControl[extId]) return;
  window.chrome.tabs.__exodusControl[extId] = true;
  window.chrome.tabs.update = function(tabId, updateProperties, cb) {{
    if (typeof updateProperties === 'function') {{ cb = updateProperties; updateProperties = {{}}; }}
    window.__exodusTabOpsOutbox.push({{
      op: 'update',
      extensionId: extId,
      chromeTabId: tabId,
      updateProperties: updateProperties || {{}}
    }});
    window.__exodusStorageDirty = true;
    var p = Promise.resolve(undefined);
    if (cb) p.then(cb);
    return p;
  }};
  window.chrome.tabs.remove = function(tabIds, cb) {{
    var ids = Array.isArray(tabIds) ? tabIds : [tabIds];
    window.__exodusTabOpsOutbox.push({{ op: 'remove', extensionId: extId, tabIds: ids }});
    window.__exodusStorageDirty = true;
    var p = Promise.resolve(undefined);
    if (cb) p.then(cb);
    return p;
  }};
  window.chrome.tabs.reload = function(tabId, reloadProperties, cb) {{
    if (typeof reloadProperties === 'function') {{ cb = reloadProperties; reloadProperties = {{}}; }}
    window.__exodusTabOpsOutbox.push({{ op: 'reload', extensionId: extId, chromeTabId: tabId }});
    window.__exodusStorageDirty = true;
    var p = Promise.resolve(undefined);
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

/// `chrome.scripting.executeScript` MVP (func string or extension file paths).
fn build_scripting_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.scripting) window.chrome.scripting = {{}};
  if (window.chrome.scripting.__exodusInstalled) return;
  window.chrome.scripting.__exodusInstalled = true;
  window.chrome.scripting.executeScript = function(injection, cb) {{
    injection = injection || {{}};
    var tabId = injection.target && injection.target.tabId;
    if (tabId == null) tabId = (window.__exodusTabs && window.__exodusTabs[0]) ? window.__exodusTabs[0].chromeTabId : 1;
    window.__exodusScriptingOutbox.push({{
      extensionId: extId,
      targetTabId: tabId,
      func: injection.func ? injection.func.toString() : null,
      args: injection.args || [],
      files: injection.files || []
    }});
    window.__exodusStorageDirty = true;
    var p = Promise.resolve([{{}}]);
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

/// `chrome.notifications.create` MVP (flush → host UI).
fn build_notifications_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.notifications) window.chrome.notifications = {{}};
  if (window.chrome.notifications.__exodusInstalled) return;
  window.chrome.notifications.__exodusInstalled = true;
  window.chrome.notifications.create = function(notificationId, options, cb) {{
    if (typeof notificationId === 'object') {{ options = notificationId; notificationId = null; }}
    if (typeof options === 'function') {{ cb = options; options = {{}}; }}
    window.__exodusNotificationOutbox.push({{
      extensionId: extId,
      notificationId: notificationId,
      options: options || {{}}
    }});
    window.__exodusStorageDirty = true;
    var id = notificationId || ('n' + Math.random().toString(36).slice(2));
    var p = Promise.resolve(id);
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

/// `chrome.webRequest` + `chrome.action` shims.
fn build_web_request_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome) window.chrome = {{}};
  if (!window.chrome.webRequest) window.chrome.webRequest = {{}};
  if (window.chrome.webRequest.__exodusInstalled) return;
  window.chrome.webRequest.__exodusInstalled = true;
  window.__exodusWebRequestOutbox = window.__exodusWebRequestOutbox || [];
  function pushRule(filter, action, resourceTypes, redirectUrl) {{
    var patterns = (filter && filter.urls) ? filter.urls : [];
    var types = (filter && filter.types) ? filter.types : (resourceTypes || ['main_frame']);
    patterns.forEach(function(pat) {{
      window.__exodusWebRequestOutbox.push({{
        extensionId: extId,
        urlPatterns: [],
        action: action || {{ type: 'block' }},
        resourceTypes: types,
        requestHeaders: [],
        rules: [{{
          urlPattern: pat,
          action: action || {{ type: 'block' }},
          resourceTypes: types
        }}]
      }});
    }});
    window.__exodusStorageDirty = true;
  }}
  var noopListener = {{ addListener: function() {{}}, removeListener: function() {{}}, hasListener: function() {{ return false; }} }};
  window.chrome.webRequest.onBeforeRequest = {{
    addListener: function(callback, filter, extra) {{
      var blocking = extra && (extra.indexOf('blocking') >= 0 || extra.indexOf('requestBody') >= 0);
      if (!blocking) return;
      pushRule(filter, {{ type: 'block' }}, filter && filter.types, null);
    }},
    removeListener: function() {{}},
    hasListener: function() {{ return false; }}
  }};
  window.chrome.webRequest.onBeforeSendHeaders = {{
    addListener: function(callback, filter, extra) {{
      var patterns = (filter && filter.urls) ? filter.urls : [];
      var hdrs = [];
      try {{
        var details = {{ url: patterns[0] || '', requestHeaders: [] }};
        var out = callback(details);
        if (out && out.requestHeaders) hdrs = out.requestHeaders.map(function(h) {{ return [h.name, h.value]; }});
      }} catch (_) {{}}
      if (hdrs.length) {{
        window.__exodusWebRequestOutbox.push({{
          extensionId: extId,
          urlPatterns: patterns,
          action: {{ type: 'block' }},
          resourceTypes: (filter && filter.types) || [],
          requestHeaders: hdrs,
          rules: []
        }});
        window.__exodusStorageDirty = true;
      }}
    }},
    removeListener: function() {{}},
    hasListener: function() {{ return false; }}
  }};
  window.chrome.webRequest.onHeadersReceived = {{
    addListener: function(callback, filter, extra) {{
      var blocking = extra && extra.indexOf('blocking') >= 0;
      if (!blocking) return;
      var patterns = (filter && filter.urls) ? filter.urls : [];
      var hdrs = [];
      try {{
        var details = {{
          url: patterns[0] || '',
          responseHeaders: [],
          type: (filter && filter.types && filter.types[0]) || 'main_frame'
        }};
        var out = callback(details);
        if (out && out.responseHeaders) {{
          hdrs = out.responseHeaders.map(function(h) {{
            var op = 'set';
            if (h.value === '' || h.value == null) op = 'remove';
            return {{ name: h.name, value: h.value || '', operation: op }};
          }});
        }}
      }} catch (_) {{}}
      if (patterns.length && hdrs.length) {{
        window.__exodusWebRequestOutbox.push({{
          extensionId: extId,
          urlPatterns: patterns,
          action: {{ type: 'block' }},
          resourceTypes: (filter && filter.types) || ['main_frame'],
          requestHeaders: [],
          responseHeaders: hdrs,
          rules: []
        }});
        window.__exodusStorageDirty = true;
      }}
    }},
    removeListener: function() {{}},
    hasListener: function() {{ return false; }}
  }};
  window.chrome.webRequest.onCompleted = noopListener;
}})('{escaped}');"#
    )
}

fn build_action_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome) window.chrome = {{}};
  if (!window.chrome.action) window.chrome.action = {{}};
  if (window.chrome.browserAction) return;
  window.chrome.browserAction = window.chrome.action;
  if (window.chrome.action.__exodusInstalled) return;
  window.chrome.action.__exodusInstalled = true;
  window.__exodusActionOutbox = window.__exodusActionOutbox || [];
  window.chrome.action.setTitle = function(d) {{
    var t = (d && d.title) != null ? d.title : d;
    window.__exodusActionOutbox.push({{ extensionId: extId, op: 'setTitle', title: t }});
    window.__exodusStorageDirty = true;
    return Promise.resolve();
  }};
  window.chrome.action.setBadgeText = function(d) {{
    var t = (d && d.text) != null ? d.text : d;
    window.__exodusActionOutbox.push({{ extensionId: extId, op: 'setBadgeText', text: t }});
    window.__exodusStorageDirty = true;
    return Promise.resolve();
  }};
  window.chrome.action.setBadgeBackgroundColor = function(d) {{
    var c = (d && d.color) != null ? d.color : d;
    window.__exodusActionOutbox.push({{ extensionId: extId, op: 'setBadgeColor', color: c }});
    window.__exodusStorageDirty = true;
    return Promise.resolve();
  }};
  window.chrome.action.openPopup = function() {{
    window.__exodusActionOutbox.push({{ extensionId: extId, op: 'openPopup' }});
    window.__exodusStorageDirty = true;
    return Promise.resolve();
  }};
  window.chrome.action.onClicked = {{ addListener: function() {{}}, removeListener: function() {{}} }};
}})('{escaped}');"#
    )
}

/// `chrome.declarativeNetRequest.updateDynamicRules` MVP.
fn build_dnr_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.declarativeNetRequest) window.chrome.declarativeNetRequest = {{}};
  if (window.chrome.declarativeNetRequest.__exodusInstalled) return;
  window.chrome.declarativeNetRequest.__exodusInstalled = true;
  window.chrome.declarativeNetRequest.updateDynamicRules = function(opts, cb) {{
    opts = opts || {{}};
    window.__exodusDnrOutbox.push({{
      extensionId: extId,
      addRules: opts.addRules || [],
      removeRuleIds: opts.removeRuleIds || []
    }});
    window.__exodusStorageDirty = true;
    var p = Promise.resolve(undefined);
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

/// `chrome.permissions.request` (flush → UI prompt).
fn build_permissions_request_shim(extension_id: &str) -> String {
    let escaped = extension_id.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function(extId) {{
  if (!window.chrome.permissions) window.chrome.permissions = {{}};
  if (window.chrome.permissions.__exodusRequestInstalled) return;
  window.chrome.permissions.__exodusRequestInstalled = true;
  window.chrome.permissions.request = function(perms, cb) {{
    var reqId = 'perm' + Math.random().toString(36).slice(2);
    var list = perms && perms.permissions ? perms.permissions : (perms || {{}}).permissions || [];
    if (Array.isArray(perms)) list = perms;
    window.__exodusPermRequests.push({{
      extensionId: extId,
      requestId: reqId,
      permissions: list,
      sourceWebviewLabel: window.__exodusWebviewLabel || ''
    }});
    window.__exodusStorageDirty = true;
    var p = new Promise(function(resolve) {{
      window.__exodusPendingPromises[reqId] = {{ resolve: resolve }};
      setTimeout(function() {{ resolve(false); }}, 120000);
    }});
    if (cb) p.then(cb);
    return p;
  }};
}})('{escaped}');"#
    )
}

/// Script evaluated in a tab webview to read storage + runtime outbox for persistence.
pub fn flush_page_state_script() -> &'static str {
    r#"JSON.stringify({
  storage: window.__exodusStorage || {},
  outbox: window.__exodusRuntimeOutbox || [],
  tabRequests: window.__exodusTabRequests || [],
  tabMessages: window.__exodusTabMessageOutbox || [],
  tabOps: window.__exodusTabOpsOutbox || [],
  scripting: window.__exodusScriptingOutbox || [],
  notifications: window.__exodusNotificationOutbox || [],
  permissionRequests: window.__exodusPermRequests || [],
  dnrUpdates: window.__exodusDnrOutbox || [],
  webRequestRules: window.__exodusWebRequestOutbox || [],
  actionOps: window.__exodusActionOutbox || []
})"#
}

/// Parsed flush payload from a content webview.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageFlushPayload {
    #[serde(default)]
    storage: Map<String, Value>,
    #[serde(default)]
    outbox: Vec<RuntimeOutboundMessage>,
    #[serde(default)]
    tab_requests: Vec<TabCreateRequest>,
    #[serde(default)]
    tab_messages: Vec<RuntimeTabMessageOutbound>,
    #[serde(default)]
    tab_ops: Vec<TabOpRequest>,
    #[serde(default)]
    scripting: Vec<ScriptingExecuteRequest>,
    #[serde(default)]
    notifications: Vec<NotificationFlushRequest>,
    #[serde(default)]
    permission_requests: Vec<PermissionRequestOutbound>,
    #[serde(default)]
    dnr_updates: Vec<DnrFlushUpdate>,
    #[serde(default)]
    web_request_rules: Vec<super::web_request::WebRequestFlushRule>,
    #[serde(default)]
    action_ops: Vec<ActionFlushOp>,
}

/// Apply storage flush and return runtime messages for background delivery.
pub fn parse_page_flush(json: &str) -> Result<PageFlushResult, super::error::PluginError> {
    let payload: PageFlushPayload = serde_json::from_str(json)
        .map_err(|e| super::error::PluginError::Parse(format!("Flush JSON: {e}")))?;
    let storage_json = serde_json::to_string(&payload.storage)
        .map_err(|e| super::error::PluginError::Parse(format!("Storage serialize: {e}")))?;
    Ok(PageFlushResult {
        storage_json,
        outbox: payload.outbox,
        tab_requests: payload.tab_requests,
        tab_messages: payload.tab_messages,
        tab_ops: payload.tab_ops,
        scripting: payload.scripting,
        notifications: payload.notifications,
        permission_requests: payload.permission_requests,
        dnr_updates: payload.dnr_updates,
        web_request_rules: payload.web_request_rules,
        action_ops: payload.action_ops,
    })
}

/// Deliver extension manifest to webview for chrome.runtime.getManifest.
#[allow(dead_code)]
pub fn deliver_manifest_script(manifest_json: &str) -> String {
    let escaped = manifest_json.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  window.__exodusManifest = {escaped};
}})();"#
    )
}

/// Deliver onInstalled event to webview.
#[allow(dead_code)]
pub fn deliver_on_installed_event_script(reason: &str) -> String {
    let escaped = reason.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        r#"(function() {{
  if (window.__exodusInstalledListeners) {{
    window.__exodusInstalledListeners.forEach(function(fn) {{
      try {{
        fn({{ reason: '{escaped}' }});
      }} catch (e) {{
        console.error('onInstalled listener error:', e);
      }}
    }});
  }}
}})();"#
    )
}

/// Apply flushed storage data to the extension manager.
#[allow(dead_code)]
pub fn apply_storage_flush(mgr: &mut ExtensionManager, extension_id: &str, json: &str) -> Result<(), super::error::PluginError> {
    if json.trim().is_empty() || json == "{}" {
        return Ok(());
    }
    let value: Value = serde_json::from_str(json)
        .map_err(|e| super::error::PluginError::Parse(format!("Storage flush JSON: {e}")))?;
    if let Value::Object(map) = value {
        let perms = mgr.permissions_for(extension_id)?;
        mgr.storage().set(extension_id, &perms, map)?;
    }
    Ok(())
}

/// Merge flushed JSON into extension storage on disk.
pub fn apply_storage_flush_to_disk(
    mgr: &mut ExtensionManager,
    json: &str,
) -> Result<(), super::error::PluginError> {
    let root: Map<String, Value> = serde_json::from_str(json)
        .map_err(|e| super::error::PluginError::Parse(format!("Storage flush JSON: {e}")))?;
    for (ext_id, value) in root {
        let obj = match value {
            Value::Object(map) => map,
            _ => continue,
        };
        let perms = mgr.permissions_for(&ext_id)?;
        if !perms.contains(&Permission::Storage) {
            continue;
        }
        mgr.storage().set(&ext_id, &perms, obj)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allama_shim_includes_configured_port() {
        let shim = build_exodus_allama_shim(11435);
        assert!(shim.contains("window.exodus.allama"));
        assert!(shim.contains("127.0.0.1:11435"));
        assert!(shim.contains("streamChat"));
        assert!(shim.contains("embed"));
    }

    #[test]
    fn parse_page_flush_roundtrip() {
        let json = r#"{"storage":{"ext-a":{"n":1}},"outbox":[{"extensionId":"ext-a","requestId":"r1","message":{"type":"ping"}}],"tabRequests":[],"tabMessages":[]}"#;
        let flush = parse_page_flush(json).expect("parse");
        assert!(flush.storage_json.contains("ext-a"));
        assert_eq!(flush.outbox.len(), 1);
        assert_eq!(flush.outbox[0].request_id, "r1");
    }

    #[test]
    fn parse_page_flush_includes_tab_messages() {
        let json = r#"{"storage":{},"outbox":[],"tabRequests":[],"tabMessages":[{"extensionId":"e","targetTabId":1,"message":{"t":1},"requestId":"tm1"}]}"#;
        let flush = parse_page_flush(json).expect("parse");
        assert_eq!(flush.tab_messages.len(), 1);
    }

    #[test]
    fn storage_flush_roundtrip_via_disk() {
        let dir = std::env::temp_dir().join(format!("exodus_flush_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&dir).expect("mgr");
        let ext_dir = dir.join("plugins/web-extensions/store");
        std::fs::create_dir_all(&ext_dir).ok();
        std::fs::write(
            ext_dir.join("manifest.json"),
            r#"{"manifest_version":3,"name":"S","version":"1","permissions":["storage"],"content_scripts":[{"matches":["<all_urls>"],"js":["c.js"]}]}"#,
        )
        .ok();
        std::fs::write(ext_dir.join("c.js"), "").ok();
        mgr.scan_and_load(None).ok();
        let seed = r#"{"store":{"counter":1}}"#;
        apply_storage_flush_to_disk(&mut mgr, seed).expect("flush");
        let tabs = TabRegistry::default();
        let script = mgr.document_start_script("https://example.com", &tabs, "exodus-tab-t");
        assert!(script.contains("counter"));
    }

    #[test]
    fn prelude_contains_storage_and_tabs() {
        let dir = std::env::temp_dir().join(format!("exodus_bridge_{}", uuid::Uuid::new_v4()));
        let mut mgr = ExtensionManager::new(&dir).expect("mgr");
        let ext_dir = dir.join("plugins/web-extensions/sample");
        std::fs::create_dir_all(&ext_dir).ok();
        std::fs::write(
            ext_dir.join("manifest.json"),
            r#"{"manifest_version":3,"name":"T","version":"1","permissions":["storage","tabs"],"content_scripts":[{"matches":["<all_urls>"],"js":["c.js"]}]}"#,
        )
        .ok();
        std::fs::write(ext_dir.join("c.js"), "").ok();
        mgr.scan_and_load(None).ok();
        let tabs = TabRegistry::default();
        let prelude = build_extension_prelude(&mgr, &tabs, "exodus-tab-x");
        assert!(prelude.contains("__exodusStorage"));
        assert!(prelude.contains("chrome.storage.local"));
        assert!(prelude.contains("chrome.tabs.query"));
    }
}
