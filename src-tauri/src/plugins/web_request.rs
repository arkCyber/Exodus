//! Exodus Browser — `chrome.webRequest` (block, redirect, subresource via injected guard).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use url::Url;

/// webRequest rule action at navigation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WebRequestAction {
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "redirect")]
    Redirect {
        #[serde(rename = "redirectUrl")]
        redirect_url: String,
    },
}

impl Default for WebRequestAction {
    fn default() -> Self {
        WebRequestAction::Block
    }
}

/// Registered rule for one extension.
#[derive(Debug, Clone)]
pub struct WebRequestRuleSet {
    pub extension_id: String,
    pub url_patterns: Vec<String>,
    pub action: WebRequestAction,
    /// Resource types from filter (`main_frame`, `sub_frame`, `xmlhttprequest`, etc.).
    pub resource_types: Vec<String>,
    /// Optional request headers to add (name -> value) from onBeforeSendHeaders.
    pub request_headers: Vec<(String, String)>,
    /// Response header mutations from onHeadersReceived (blocking).
    pub response_headers: Vec<ResponseHeaderMod>,
}

/// One response header change from `chrome.webRequest.onHeadersReceived`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseHeaderMod {
    pub name: String,
    #[serde(default)]
    pub value: String,
    /// `set`, `remove`, or `append`.
    #[serde(default = "default_header_op_set")]
    pub operation: String,
}

fn default_header_op_set() -> String {
    "set".to_string()
}

/// Navigation evaluation result.
#[derive(Debug, Clone)]
pub struct WebRequestDecision {
    pub extension_id: String,
    pub blocked: bool,
    pub redirect_url: Option<String>,
}

/// Extension-registered webRequest rules.
pub struct WebRequestStore {
    rules: Arc<Mutex<Vec<WebRequestRuleSet>>>,
}

impl Default for WebRequestStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WebRequestStore {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Apply flushed rules from extension background/content.
    pub fn apply_flush_rule(&self, flush: &WebRequestFlushRule) -> Result<(), String> {
        let mut rules = self.rules.lock().map_err(|e| format!("Lock: {}", e))?;
        rules.retain(|r| r.extension_id != flush.extension_id);
        if flush.url_patterns.is_empty() && flush.rules.is_empty() {
            return Ok(());
        }
        let patterns: Vec<String> = if !flush.rules.is_empty() {
            flush.rules.iter().map(|r| r.url_pattern.clone()).collect()
        } else {
            flush.url_patterns.clone()
        };
        let action = flush
            .rules
            .first()
            .map(|r| r.action.clone())
            .unwrap_or(flush.action.clone());
        let resource_types = flush
            .rules
            .first()
            .map(|r| r.resource_types.clone())
            .unwrap_or(flush.resource_types.clone());
        let request_headers = flush.request_headers.clone();
        let response_headers = if !flush.response_headers.is_empty() {
            flush.response_headers.clone()
        } else {
            flush
                .rules
                .iter()
                .find_map(|r| {
                    if r.response_headers.is_empty() {
                        None
                    } else {
                        Some(r.response_headers.clone())
                    }
                })
                .unwrap_or_default()
        };
        if !patterns.is_empty() {
            rules.push(WebRequestRuleSet {
                extension_id: flush.extension_id.clone(),
                url_patterns: patterns,
                action,
                resource_types,
                request_headers,
                response_headers,
            });
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_rules_for_extension(
        &self,
        extension_id: &str,
        patterns: Vec<String>,
    ) -> Result<(), String> {
        self.apply_flush_rule(&WebRequestFlushRule {
            extension_id: extension_id.to_string(),
            url_patterns: patterns,
            action: WebRequestAction::Block,
            resource_types: vec!["main_frame".to_string()],
            request_headers: vec![],
            response_headers: vec![],
            rules: vec![],
        })
    }

    /// True when any extension registered response-header mutations.
    pub fn has_any_response_header_rules(&self) -> bool {
        self.rules
            .lock()
            .ok()
            .map(|rules| rules.iter().any(|r| !r.response_headers.is_empty()))
            .unwrap_or(false)
    }

    /// True when any extension registered request- or response-header mutations.
    pub fn has_any_proxy_rules(&self) -> bool {
        self.rules
            .lock()
            .ok()
            .map(|rules| {
                rules
                    .iter()
                    .any(|r| !r.response_headers.is_empty() || !r.request_headers.is_empty())
            })
            .unwrap_or(false)
    }

    /// Whether response-header rules apply to this URL and resource type.
    pub fn should_apply_response_headers(&self, url: &str, resource_type: &str) -> bool {
        !self
            .response_headers_for_resource(url, resource_type)
            .is_empty()
    }

    /// Whether request-header rules apply to this URL and resource type.
    pub fn should_apply_request_headers(&self, url: &str, resource_type: &str) -> bool {
        !self
            .request_headers_for_resource(url, resource_type)
            .is_empty()
    }

    /// Whether this URL should use the native `exodus-proxy` / loopback fetch proxy.
    pub fn should_proxy_resource(&self, url: &str, resource_type: &str) -> bool {
        self.should_apply_response_headers(url, resource_type)
            || self.should_apply_request_headers(url, resource_type)
    }

    /// JSON rules for injected subresource proxy (`urlPatterns` + `resourceTypes`).
    pub fn response_header_proxy_rules_json(&self) -> String {
        self.header_proxy_rules_json()
    }

    /// JSON rules for subresource/native proxy (request + response header mods).
    pub fn header_proxy_rules_json(&self) -> String {
        #[derive(Serialize)]
        struct ProxyRule {
            #[serde(rename = "urlPatterns")]
            url_patterns: Vec<String>,
            #[serde(rename = "resourceTypes")]
            resource_types: Vec<String>,
            #[serde(rename = "modifyRequest")]
            modify_request: bool,
            #[serde(rename = "modifyResponse")]
            modify_response: bool,
        }
        let rules = match self.rules.lock() {
            Ok(r) => r,
            Err(_) => return "[]".to_string(),
        };
        let out: Vec<ProxyRule> = rules
            .iter()
            .filter(|r| {
                !r.url_patterns.is_empty()
                    && (!r.response_headers.is_empty() || !r.request_headers.is_empty())
            })
            .map(|r| ProxyRule {
                url_patterns: r.url_patterns.clone(),
                resource_types: r.resource_types.clone(),
                modify_request: !r.request_headers.is_empty(),
                modify_response: !r.response_headers.is_empty(),
            })
            .collect();
        serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string())
    }

    /// Collect request headers to add for a URL and resource type.
    pub fn request_headers_for_resource(
        &self,
        url: &str,
        resource_type: &str,
    ) -> Vec<(String, String)> {
        let rules = match self.rules.lock() {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let mut out = Vec::new();
        for rule in rules.iter() {
            if rule.request_headers.is_empty() {
                continue;
            }
            if !rule_applies_to_resource(&rule.resource_types, resource_type) {
                continue;
            }
            for pat in &rule.url_patterns {
                if pattern_matches(pat, url) {
                    out.extend(rule.request_headers.clone());
                    break;
                }
            }
        }
        out
    }

    /// Collect response-header mutations for a URL and resource type.
    pub fn response_headers_for_resource(&self, url: &str, resource_type: &str) -> Vec<ResponseHeaderMod> {
        let rules = match self.rules.lock() {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let mut out = Vec::new();
        for rule in rules.iter() {
            if rule.response_headers.is_empty() {
                continue;
            }
            if !rule_applies_to_resource(&rule.resource_types, resource_type) {
                continue;
            }
            for pat in &rule.url_patterns {
                if pattern_matches(pat, url) {
                    out.extend(rule.response_headers.clone());
                    break;
                }
            }
        }
        out
    }

    /// Collect response-header mutations for a main-frame URL.
    pub fn response_headers_for_url(&self, url: &str) -> Vec<ResponseHeaderMod> {
        self.response_headers_for_resource(url, "main_frame")
    }

    /// Document-start script applying extension response-header rules (CSP meta strip, etc.).
    pub fn response_headers_script(&self, url: &str) -> String {
        let mods = self.response_headers_for_url(url);
        if mods.is_empty() {
            return String::new();
        }
        let json = match serde_json::to_string(&mods) {
            Ok(j) => j,
            Err(_) => return String::new(),
        };
        format!(
            r#"(function() {{
  if (window.__exodusResponseHeaders) return;
  window.__exodusResponseHeaders = true;
  const mods = {json};
  try {{
    mods.forEach(function(m) {{
      var n = (m.name || '').toLowerCase();
      var op = (m.operation || 'set').toLowerCase();
      if (op === 'remove' && (n === 'content-security-policy' || n === 'content-security-policy-report-only')) {{
        document.querySelectorAll('meta[http-equiv="Content-Security-Policy" i], meta[http-equiv="Content-Security-Policy-Report-Only" i]').forEach(function(el) {{ el.remove(); }});
      }}
      if (op === 'set' && n === 'content-security-policy' && m.value) {{
        document.querySelectorAll('meta[http-equiv="Content-Security-Policy" i]').forEach(function(el) {{ el.remove(); }});
        var meta = document.createElement('meta');
        meta.httpEquiv = 'Content-Security-Policy';
        meta.content = m.value;
        (document.head || document.documentElement).appendChild(meta);
      }}
    }});
  }} catch (_) {{}}
}})();"#
        )
    }

    /// Evaluate main-frame navigation.
    pub fn evaluate_navigation(&self, url: &str) -> Option<WebRequestDecision> {
        self.evaluate(url, "main_frame")
    }

    /// Evaluate subresource URL (fetch/XHR/script).
    pub fn evaluate_subresource(&self, url: &str, resource_type: &str) -> Option<WebRequestDecision> {
        self.evaluate(url, resource_type)
    }

    fn evaluate(&self, url: &str, resource_type: &str) -> Option<WebRequestDecision> {
        let rules = self.rules.lock().ok()?;
        for rule in rules.iter() {
            if !rule_applies_to_resource(&rule.resource_types, resource_type) {
                continue;
            }
            for pat in &rule.url_patterns {
                if pattern_matches(pat, url) {
                    return Some(match &rule.action {
                        WebRequestAction::Block => WebRequestDecision {
                            extension_id: rule.extension_id.clone(),
                            blocked: true,
                            redirect_url: None,
                        },
                        WebRequestAction::Redirect { redirect_url } => WebRequestDecision {
                            extension_id: rule.extension_id.clone(),
                            blocked: false,
                            redirect_url: Some(redirect_url.clone()),
                        },
                    });
                }
            }
        }
        None
    }

    /// Hostnames to block subresources (fetch/XHR) from all extension rules.
    pub fn subresource_block_hosts(&self) -> Vec<String> {
        let rules = self.rules.lock().ok();
        let rules = match rules {
            Some(r) => r,
            None => return Vec::new(),
        };
        let mut hosts = HashSet::new();
        for rule in rules.iter() {
            if !rule_applies_to_resource(&rule.resource_types, "xmlhttprequest")
                && !rule_applies_to_resource(&rule.resource_types, "sub_frame")
                && !rule.resource_types.is_empty()
            {
                continue;
            }
            for pat in &rule.url_patterns {
                if let Some(h) = host_from_pattern(pat) {
                    hosts.insert(h);
                }
            }
        }
        let mut out: Vec<String> = hosts.into_iter().collect();
        out.sort();
        out
    }

    /// Document-start script blocking extension webRequest subresources.
    pub fn subresource_guard_script(&self) -> String {
        let hosts = self.subresource_block_hosts();
        if hosts.is_empty() {
            return String::new();
        }
        let json = serde_json::to_string(&hosts).unwrap_or_else(|_| "[]".to_string());
        format!(
            r#"(function() {{
  if (window.__exodusWebRequestGuard) return;
  window.__exodusWebRequestGuard = true;
  const blocked = {json};
  function hostBlocked(host) {{
    const h = (host || '').toLowerCase();
    return blocked.some(function(d) {{
      const d0 = d.toLowerCase();
      return h === d0 || h.endsWith('.' + d0);
    }});
  }}
  function urlBlocked(url) {{
    try {{ return hostBlocked(new URL(url, location.href).hostname); }}
    catch {{ return false; }}
  }}
  const origFetch = window.fetch;
  if (origFetch) {{
    window.fetch = function(input, init) {{
      const u = typeof input === 'string' ? input : (input && input.url) || '';
      if (urlBlocked(u)) return Promise.reject(new Error('webRequest blocked'));
      return origFetch.apply(this, arguments);
    }};
  }}
  const XO = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function(method, url) {{
    if (urlBlocked(url)) throw new Error('webRequest blocked');
    return XO.apply(this, arguments);
  }};
}})();"#
        )
    }
}

fn rule_applies_to_resource(rule_types: &[String], actual: &str) -> bool {
    if rule_types.is_empty() {
        return true;
    }
    rule_types
        .iter()
        .any(|t| t.eq_ignore_ascii_case(actual) || t == "*")
}

fn host_from_pattern(pat: &str) -> Option<String> {
    if let Ok(u) = Url::parse(pat) {
        return u.host_str().map(|s| s.to_string());
    }
    if pat.contains("://") {
        let rest = pat.split("://").nth(1)?;
        let host = rest
            .trim_start_matches('*')
            .trim_start_matches('.')
            .split('/')
            .next()?
            .trim_end_matches('*');
        if !host.is_empty() && !host.contains('*') {
            return Some(host.to_string());
        }
    }
    if pat.starts_with("||") {
        let rest = pat.strip_prefix("||")?;
        let host = rest.split('^').next()?.split('/').next()?;
        if !host.is_empty() {
            return Some(host.to_string());
        }
    }
    None
}

/// One rule line from extension flush.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebRequestRuleFlush {
    pub url_pattern: String,
    #[serde(default)]
    pub action: WebRequestAction,
    #[serde(default)]
    pub resource_types: Vec<String>,
    #[serde(default)]
    pub response_headers: Vec<ResponseHeaderMod>,
}

/// Flush payload from extension background/content.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebRequestFlushRule {
    pub extension_id: String,
    #[serde(default)]
    pub url_patterns: Vec<String>,
    #[serde(default)]
    pub action: WebRequestAction,
    #[serde(default)]
    pub resource_types: Vec<String>,
    #[serde(default)]
    pub request_headers: Vec<(String, String)>,
    #[serde(default)]
    pub response_headers: Vec<ResponseHeaderMod>,
    #[serde(default)]
    pub rules: Vec<WebRequestRuleFlush>,
}

pub fn pattern_matches(pattern: &str, url: &str) -> bool {
    let pat = pattern.trim();
    if pat.is_empty() {
        return false;
    }
    if pat == "<all_urls>" || pat == "*://*/*" {
        return url.starts_with("http://") || url.starts_with("https://");
    }
    let url_lower = url.to_lowercase();
    if let Ok(parsed) = Url::parse(&url_lower) {
        if let Some(host) = parsed.host_str() {
            if pat.contains("://") {
                let parts: Vec<&str> = pat.split("://").collect();
                if parts.len() == 2 {
                    let scheme_part = parts[0];
                    let rest = parts[1];
                    let host_pat = rest
                        .trim_start_matches('*')
                        .trim_start_matches('.')
                        .trim_end_matches("/*")
                        .trim_end_matches('*');
                    if scheme_part != "*" && !url_lower.starts_with(&format!("{}://", scheme_part)) {
                        return false;
                    }
                    if host_pat.is_empty() || host_pat == "*" {
                        return true;
                    }
                    return host == host_pat || host.ends_with(&format!(".{}", host_pat));
                }
            }
        }
    }
    url_lower.contains(&pat.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_and_redirect() {
        let store = WebRequestStore::new();
        store
            .apply_flush_rule(&WebRequestFlushRule {
                extension_id: "e1".into(),
                url_patterns: vec![],
                action: WebRequestAction::Block,
                resource_types: vec!["main_frame".into()],
                request_headers: vec![],
                response_headers: vec![],
                rules: vec![WebRequestRuleFlush {
                    url_pattern: "*://evil.test/*".into(),
                    action: WebRequestAction::Block,
                    resource_types: vec!["main_frame".into()],
                    response_headers: vec![],
                }],
            })
            .unwrap();
        assert!(store.evaluate_navigation("https://evil.test/x").unwrap().blocked);
        store
            .apply_flush_rule(&WebRequestFlushRule {
                extension_id: "e2".into(),
                url_patterns: vec![],
                action: WebRequestAction::default(),
                resource_types: vec!["main_frame".into()],
                request_headers: vec![],
                response_headers: vec![],
                rules: vec![WebRequestRuleFlush {
                    url_pattern: "*://old.example/*".into(),
                    action: WebRequestAction::Redirect {
                        redirect_url: "https://new.example/".into(),
                    },
                    resource_types: vec!["main_frame".into()],
                    response_headers: vec![],
                }],
            })
            .unwrap();
        let d = store.evaluate_navigation("https://old.example/page").unwrap();
        assert_eq!(d.redirect_url.as_deref(), Some("https://new.example/"));
    }

    #[test]
    fn request_headers_for_matching_url() {
        let store = WebRequestStore::new();
        store
            .apply_flush_rule(&WebRequestFlushRule {
                extension_id: "e4".into(),
                url_patterns: vec!["*://api.example/*".into()],
                action: WebRequestAction::Block,
                resource_types: vec!["xmlhttprequest".into()],
                request_headers: vec![("X-Exodus".into(), "1".into())],
                response_headers: vec![],
                rules: vec![],
            })
            .unwrap();
        let hdrs = store.request_headers_for_resource(
            "https://api.example/v1",
            "xmlhttprequest",
        );
        assert_eq!(hdrs.len(), 1);
        assert_eq!(hdrs[0].0, "X-Exodus");
        assert!(store.should_proxy_resource("https://api.example/v1", "xmlhttprequest"));
    }

    #[test]
    fn response_headers_script_for_csp_remove() {
        let store = WebRequestStore::new();
        store
            .apply_flush_rule(&WebRequestFlushRule {
                extension_id: "e3".into(),
                url_patterns: vec!["*://csp.test/*".into()],
                action: WebRequestAction::Block,
                resource_types: vec!["main_frame".into()],
                request_headers: vec![],
                response_headers: vec![ResponseHeaderMod {
                    name: "Content-Security-Policy".into(),
                    value: String::new(),
                    operation: "remove".into(),
                }],
                rules: vec![],
            })
            .unwrap();
        let script = store.response_headers_script("https://csp.test/page");
        assert!(script.contains("__exodusResponseHeaders"));
        assert!(script.contains("Content-Security-Policy"));
    }
}
