//! Local HTTP fetch proxy — applies `webRequest` request/response header rules.
//!
//! **Native path:** `exodus-proxy://` WebView URI scheme (WKURLSchemeHandler / WebView2 filter).
//! **Debug path:** loopback `127.0.0.1` axum server for health checks.
//! Secured with auth token, SSRF blocks, per-URL rule matching, method/body passthrough, 20MB cap.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use crate::plugins::web_request::{ResponseHeaderMod, WebRequestStore};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use url::Url;

/// Custom URI scheme served by the WebView (native resource loading).
pub const EXODUS_PROXY_SCHEME: &str = "exodus-proxy";

/// Maximum bytes read from upstream responses (and incoming proxy POST bodies).
pub const MAX_PROXY_BODY_BYTES: usize = 20 * 1024 * 1024;

/// Loopback proxy port, auth token, and lifecycle handle.
#[derive(Debug, Clone)]
pub struct HttpResponseProxy {
    pub port: u16,
    pub auth_token: String,
}

#[derive(Debug, Deserialize)]
struct FetchQuery {
    url: String,
    #[serde(default = "default_resource_type")]
    resource_type: String,
    token: String,
}

fn default_resource_type() -> String {
    "main_frame".to_string()
}

/// Start the loopback proxy (idempotent if already running).
pub async fn start_http_response_proxy(app: AppHandle) -> Result<HttpResponseProxy, String> {
    if let Some(existing) = app.try_state::<HttpResponseProxy>() {
        return Ok((*existing).clone());
    }

    let auth_token = uuid::Uuid::new_v4().to_string();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Bind proxy port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Proxy local_addr: {e}"))?
        .port();

    let proxy_state = HttpResponseProxy {
        port,
        auth_token: auth_token.clone(),
    };

    let app_clone = app.clone();
    let router = Router::new()
        .route(
            "/exodus-fetch",
            any(fetch_handler).layer(axum::extract::DefaultBodyLimit::max(MAX_PROXY_BODY_BYTES)),
        )
        .route("/health", any(health_handler))
        .with_state(app_clone);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!("HTTP response proxy stopped: {e}");
        }
    });

    app.manage(proxy_state.clone());
    tracing::info!("HTTP response proxy listening on 127.0.0.1:{port}");
    Ok(proxy_state)
}

/// Build a native `exodus-proxy://` URL for WebView navigation or subresource load.
pub fn proxied_navigation_url(proxy: &HttpResponseProxy, target_url: &str) -> Option<String> {
    build_proxy_url(proxy, target_url, "main_frame")
}

/// Build proxy URL for any resource type.
pub fn build_proxy_url(
    proxy: &HttpResponseProxy,
    target_url: &str,
    resource_type: &str,
) -> Option<String> {
    if !is_public_http_url(target_url) {
        return None;
    }
    let mut u = Url::parse(&format!("{EXODUS_PROXY_SCHEME}://localhost/fetch")).ok()?;
    u.query_pairs_mut()
        .append_pair("url", target_url)
        .append_pair("token", &proxy.auth_token)
        .append_pair("resourceType", resource_type);
    Some(u.to_string())
}

/// Whether main-frame `url` should load through the native proxy.
pub fn should_proxy_url(app: &AppHandle, url: &str) -> bool {
    if !is_public_http_url(url) {
        return false;
    }
    app.try_state::<WebRequestStore>()
        .map(|s| s.should_proxy_resource(url, "main_frame"))
        .unwrap_or(false)
}

/// Register the `exodus-proxy` URI scheme on the Tauri builder (must run before webviews).
pub fn register_native_proxy_protocol<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
) -> tauri::Builder<R> {
    builder.register_asynchronous_uri_scheme_protocol(EXODUS_PROXY_SCHEME, |ctx, request, responder| {
        let app = ctx.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            let response = serve_scheme_request(&app, request).await;
            responder.respond(response);
        });
    })
}

/// Document-start script: proxy only URLs matching extension response-header rules.
pub fn subresource_mitm_proxy_script(proxy: &HttpResponseProxy, rules_json: &str) -> String {
    let token = serde_json::to_string(&proxy.auth_token).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"(function() {{
  if (window.__exodusMitmProxy) return;
  window.__exodusMitmProxy = true;
  const BASE = 'exodus-proxy://localhost/fetch';
  const TOKEN = {token};
  const RULES = {rules_json};
  function patternMatch(pat, url) {{
    try {{
      var p = (pat || '').toLowerCase();
      var u = url.toLowerCase();
      if (p === '<all_urls>' || p === '*://*/*') return u.startsWith('http://') || u.startsWith('https://');
      if (p.indexOf('://') >= 0) {{
        var parts = p.split('://');
        if (parts.length !== 2) return false;
        var scheme = parts[0];
        var rest = parts[1].replace(/^\*+/, '').replace(/^\./, '').replace(/\/\*$/, '').replace(/\*$/, '');
        if (scheme !== '*' && !u.startsWith(scheme + '://')) return false;
        if (!rest || rest === '*') return true;
        var host = new URL(url).hostname.toLowerCase();
        return host === rest || host.endsWith('.' + rest);
      }}
      return u.indexOf(p) >= 0;
    }} catch (_) {{ return false; }}
  }}
  function shouldProxy(url, rt) {{
    var type = rt || 'other';
    return RULES.some(function(rule) {{
      if (!rule.modifyRequest && !rule.modifyResponse) return false;
      var types = rule.resourceTypes || rule.resource_types || [];
      if (types.length && types.indexOf(type) < 0 && types.indexOf('*') < 0) return false;
      var patterns = rule.urlPatterns || rule.url_patterns || [];
      return patterns.some(function(pat) {{ return patternMatch(pat, url); }});
    }});
  }}
  function proxify(url, rt, method) {{
    try {{
      const u = new URL(url, location.href);
      if (u.protocol !== 'http:' && u.protocol !== 'https:') return url;
      if (u.protocol === 'exodus-proxy:') return url;
      if (!shouldProxy(u.href, rt)) return url;
      var q = BASE + '?url=' + encodeURIComponent(u.href) + '&resourceType=' + encodeURIComponent(rt || 'other') + '&token=' + encodeURIComponent(TOKEN);
      return q;
    }} catch (_) {{ return url; }}
  }}
  function patchSrc(el, attr, rt) {{
    var v = el.getAttribute(attr);
    if (v) el.setAttribute(attr, proxify(v, rt));
  }}
  const origFetch = window.fetch;
  if (origFetch) {{
    window.fetch = function(input, init) {{
      var url = typeof input === 'string' ? input : (input && input.url) || '';
      var method = (init && init.method) || (input && input.method) || 'GET';
      if (!shouldProxy(url, 'fetch')) return origFetch.apply(this, arguments);
      var proxied = proxify(url, 'fetch');
      if (typeof input === 'string') return origFetch.call(this, proxied, init);
      if (input && input.url) {{
        try {{
          return origFetch.call(this, new Request(proxied, input), init);
        }} catch (_) {{}}
      }}
      return origFetch.apply(this, arguments);
    }};
  }}
  const xo = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function(method, url) {{
    var rt = 'xmlhttprequest';
    if (shouldProxy(String(url), rt)) arguments[1] = proxify(String(url), rt, method);
    return xo.apply(this, arguments);
  }};
  const imgSrc = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
  if (imgSrc && imgSrc.set) {{
    Object.defineProperty(HTMLImageElement.prototype, 'src', {{
      set: function(v) {{ imgSrc.set.call(this, proxify(String(v), 'image')); }},
      get: imgSrc.get,
      configurable: true
    }});
  }}
  function patchTree(root) {{
    if (!root || !root.querySelectorAll) return;
    root.querySelectorAll('script[src],link[rel="stylesheet"][href],img[src],iframe[src],video[src],audio[src]').forEach(function(el) {{
      if (el.tagName === 'SCRIPT') patchSrc(el, 'src', 'script');
      else if (el.tagName === 'LINK') patchSrc(el, 'href', 'stylesheet');
      else patchSrc(el, 'src', el.tagName.toLowerCase());
    }});
  }}
  patchTree(document.documentElement);
  var debounce;
  const mo = new MutationObserver(function() {{
    clearTimeout(debounce);
    debounce = setTimeout(function() {{ patchTree(document.documentElement); }}, 50);
  }});
  mo.observe(document.documentElement || document, {{ childList: true, subtree: true }});
}})();"#,
    )
}

/// Returns true when `url` is http(s) and not a blocked loopback/private target.
pub fn is_public_http_url(url_str: &str) -> bool {
    let Ok(parsed) = Url::parse(url_str) else {
        return false;
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }
    !is_blocked_proxy_host(&parsed)
}

fn is_blocked_proxy_host(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return true;
    };
    let host_lower = host.to_lowercase();
    if host_lower == "localhost"
        || host_lower.ends_with(".localhost")
        || host_lower.ends_with(".local")
    {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_blocked_ip(ip);
    }
    false
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified(),
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "{\"ok\":true,\"service\":\"exodus-http-proxy\"}")
}

fn parse_fetch_query(query: Option<&str>) -> Result<FetchQuery, StatusCode> {
    parse_proxy_query(query)
}

fn parse_proxy_query(query: Option<&str>) -> Result<FetchQuery, StatusCode> {
    let mut url = String::new();
    let mut resource_type = default_resource_type();
    let mut token = String::new();
    for (k, v) in url::form_urlencoded::parse(query.unwrap_or("").as_bytes()) {
        match k.as_ref() {
            "url" => url = v.into_owned(),
            "resource_type" | "resourceType" => resource_type = v.into_owned(),
            "token" => token = v.into_owned(),
            _ => {}
        }
    }
    if url.is_empty() || token.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(FetchQuery {
        url,
        resource_type,
        token,
    })
}

fn is_forbidden_request_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "host" | "connection" | "content-length" | "transfer-encoding" | "upgrade" | "te"
    )
}

/// Core proxy fetch: apply webRequest request/response header rules, return HTTP response bytes.
async fn execute_proxy_fetch<R: tauri::Runtime>(
    app: &AppHandle<R>,
    method: Method,
    query: FetchQuery,
    body: Vec<u8>,
    content_type: Option<String>,
) -> Result<http::Response<Vec<u8>>, StatusCode> {
    let proxy = app
        .try_state::<HttpResponseProxy>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if query.token != proxy.auth_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let target = query.url.trim();
    if !is_public_http_url(target) {
        return Err(StatusCode::FORBIDDEN);
    }

    let rt = {
        let rt = query.resource_type.trim();
        if rt.is_empty() {
            "main_frame"
        } else {
            rt
        }
    };

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut rb = client.request(method.clone(), target);
    if !body.is_empty() {
        rb = rb.body(body);
    }
    if let Some(ct) = content_type {
        rb = rb.header("content-type", ct);
    }

    if let Some(wr) = app.try_state::<WebRequestStore>() {
        for (name, value) in wr.request_headers_for_resource(target, rt) {
            if !is_forbidden_request_header(&name) {
                rb = rb.header(name, value);
            }
        }
    }

    let upstream = rb.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let status = upstream.status();
    let mut header_map: HashMap<String, String> = HashMap::new();
    for (name, value) in upstream.headers().iter() {
        if let Ok(v) = value.to_str() {
            header_map.insert(name.as_str().to_string(), v.to_string());
        }
    }

    if let Some(wr) = app.try_state::<WebRequestStore>() {
        for m in wr.response_headers_for_resource(target, rt) {
            apply_response_header_mod(&mut header_map, &m);
        }
    }

    if let Some(net) = app.try_state::<Arc<crate::network_interception::NetworkInterceptor>>() {
        let res_type = crate::network_interception::ResourceType::from_str(rt);
        let net_method = match method {
            Method::POST => crate::network_interception::RequestMethod::POST,
            Method::PUT => crate::network_interception::RequestMethod::PUT,
            Method::DELETE => crate::network_interception::RequestMethod::DELETE,
            Method::PATCH => crate::network_interception::RequestMethod::PATCH,
            Method::HEAD => crate::network_interception::RequestMethod::HEAD,
            Method::OPTIONS => crate::network_interception::RequestMethod::OPTIONS,
            _ => crate::network_interception::RequestMethod::GET,
        };
        let net_req = crate::network_interception::InterceptedRequest::new(
            target.to_string(),
            net_method,
            res_type,
        );
        if let Some(crate::network_interception::InterceptionAction::ModifyHeaders(map)) =
            net.intercept_request(net_req)
        {
            for (k, v) in map {
                header_map.insert(k, v);
            }
        }
    }

    let resp_body = upstream
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if resp_body.len() > MAX_PROXY_BODY_BYTES {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let mut builder = http::Response::builder().status(status.as_u16());
    for (name, value) in header_map {
        if let (Ok(n), Ok(v)) = (
            http::header::HeaderName::from_bytes(name.as_bytes()),
            http::header::HeaderValue::from_str(&value),
        ) {
            builder = builder.header(n, v);
        }
    }
    builder = builder
        .header("x-exodus-proxied", "1")
        .header("access-control-allow-origin", "*");
    builder
        .body(resp_body.to_vec())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handle `exodus-proxy://localhost/fetch?...` WebView scheme requests (native proxy).
pub async fn serve_scheme_request<R: tauri::Runtime>(
    app: &AppHandle<R>,
    request: http::Request<Vec<u8>>,
) -> http::Response<Vec<u8>> {
    let method = match request.method().as_str() {
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        _ => Method::GET,
    };
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let query = match parse_proxy_query(request.uri().query()) {
        Ok(q) => q,
        Err(status) => return proxy_error_response(status),
    };
    let body = request.into_body();
    if body.len() > MAX_PROXY_BODY_BYTES {
        return proxy_error_response(StatusCode::PAYLOAD_TOO_LARGE);
    }
    match execute_proxy_fetch(app, method, query, body, content_type).await {
        Ok(r) => r,
        Err(status) => proxy_error_response(status),
    }
}

fn proxy_error_response(status: StatusCode) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status.as_u16())
        .header(http::header::CONTENT_TYPE, "text/plain")
        .body(format!("{status}").into_bytes())
        .unwrap_or_else(|_| {
            http::Response::builder()
                .status(500)
                .body(Vec::new())
                .expect("Failed to build response")
        })
}

async fn fetch_handler<R: tauri::Runtime>(
    State(app): State<AppHandle<R>>,
    req: Request<Body>,
) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let query = parse_fetch_query(req.uri().query())?;
    let body = axum::body::to_bytes(req.into_body(), MAX_PROXY_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    let http_resp = execute_proxy_fetch(&app, method, query, body.to_vec(), content_type).await?;
    Ok(axum_response_from_http(http_resp))
}

fn axum_response_from_http(resp: http::Response<Vec<u8>>) -> Response {
    let (parts, body) = resp.into_parts();
    let mut out = Response::new(body.into());
    *out.status_mut() = StatusCode::from_u16(parts.status.as_u16()).unwrap_or(StatusCode::OK);
    *out.headers_mut() = parts.headers;
    out
}

/// Apply one `ResponseHeaderMod` to a header map (case-insensitive name).
fn apply_response_header_mod(headers: &mut HashMap<String, String>, m: &ResponseHeaderMod) {
    let name = m.name.trim();
    if name.is_empty() {
        return;
    }
    let op = m.operation.to_lowercase();
    let key = headers
        .keys()
        .find(|k| k.eq_ignore_ascii_case(name))
        .cloned()
        .unwrap_or_else(|| name.to_string());
    match op.as_str() {
        "remove" => {
            headers.remove(&key);
        }
        "append" => {
            let prev = headers.get(&key).cloned().unwrap_or_default();
            let sep = if prev.is_empty() { "" } else { ", " };
            headers.insert(key, format!("{prev}{sep}{}", m.value));
        }
        _ => {
            headers.insert(key, m.value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_remove_and_set() {
        let mut h = HashMap::from([(
            "Content-Security-Policy".to_string(),
            "default-src 'none'".to_string(),
        )]);
        apply_response_header_mod(
            &mut h,
            &ResponseHeaderMod {
                name: "Content-Security-Policy".into(),
                value: String::new(),
                operation: "remove".into(),
            },
        );
        assert!(!h.contains_key("Content-Security-Policy"));
    }

    #[test]
    fn blocks_loopback_targets() {
        assert!(!is_public_http_url("http://127.0.0.1/"));
        assert!(!is_public_http_url("http://localhost/"));
        assert!(!is_public_http_url("http://192.168.1.1/"));
        assert!(is_public_http_url("https://example.com/"));
    }
}
