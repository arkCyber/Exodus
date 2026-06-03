# HTTP response / request header proxy

When an extension registers `chrome.webRequest.onBeforeSendHeaders` or `onHeadersReceived` (blocking) rules, Exodus routes matching traffic through a **native WebView proxy** so headers are applied before content loads.

## Native WebView proxy (`exodus-proxy://`)

1. At app build time, `register_native_proxy_protocol("exodus-proxy")` registers a URI scheme handler (WKURLSchemeHandler on macOS, WebView2 resource filter on Windows, webkitgtk on Linux).
2. Extension flush stores `requestHeaders` / `responseHeaders` in `WebRequestStore`.
3. **Main-frame** navigations rewrite to `exodus-proxy://localhost/fetch?url=…&token=…&resourceType=main_frame` (see `browser_navigate` and `on_navigation`).
4. The scheme handler fetches upstream with `reqwest`, merges extension request headers, applies response header mods, returns the body to the WebView.
5. **Subresources** still use a small document-start script to rewrite `fetch`/XHR/`src` to the same `exodus-proxy://` URLs (WebView has no public per-request hook for arbitrary subresources without a scheme).

## Loopback debug server

A `127.0.0.1` axum server also listens (`/exodus-fetch`, `/health`) for debugging:

```bash
curl "http://127.0.0.1:PORT/health"
```

## Security

- **Token required**: `token` query param must match the per-session UUID.
- **SSRF blocked**: only public `http`/`https` targets; loopback, `localhost`, `.local`, private IPs rejected.
- **Size cap**: 20 MB request/response.

## Subresource matching

Rules include `modifyRequest` / `modifyResponse` flags and `resourceTypes` so only matching URLs are proxied (not all http(s) traffic).

## Limits

- Subresource rewriting still needs injection until the platform exposes full resource interception.
- Some sites break when loaded via custom-scheme origin (relative URLs, strict CSP).
- Rules also merge with `NetworkInterceptor` `ModifyHeaders` when configured.

## Debug

Log line: `HTTP response proxy listening on 127.0.0.1:PORT` (loopback helper; main path is `exodus-proxy://`).
