# Exodus — Web platform compatibility

## WebAssembly (WASM) in web pages

Exodus tab pages use the **system WebView** (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux). **WebAssembly is provided by that engine** — there is no separate Exodus WASM runtime for ordinary websites.

### What we guarantee

- Standard **`WebAssembly.instantiate` / `compile`** for same-origin and cross-origin `.wasm` resources (subject to CSP).
- Local regression page: **`/test/wasm-smoke.html`** (served from `public/test/`).

### What varies by OS / WebView version

| Feature | Notes |
|---------|--------|
| **Threads + `SharedArrayBuffer`** | Requires page `Cross-Origin-Opener-Policy` + `Cross-Origin-Embedder-Policy`. May show as unavailable on the smoke page — not an Exodus bug. |
| **WebGPU** | Engine-dependent; not polyfilled by Exodus. |
| **Unity / heavy WebGL games** | Test per title; compare with Safari/Edge on the same OS if issues appear. |

### Automated tests

```bash
# Smoke HTML only (Playwright + Vite E2E server)
./scripts/test-wasm.sh

# Native tab webview (requires running app)
# Terminal 1: pnpm tauri dev
# Terminal 2:
./scripts/test-wasm.sh --tauri
```

Playwright spec: `e2e/webassembly.spec.ts`.

### Manual QA (native tab webview — full proof)

Playwright drives a **separate Chromium** by default; it cannot see inside Tauri **content** webviews unless IPC is available on the page under test.

Inside the running **Exodus** app:

1. Omnibox: `http://localhost:1421/test/wasm-smoke.html`
2. DevTools → Console:

```js
await __EXODUS_E2E__.runWasmSmokeCheck()
// → { pass: true, raw: "1" }
```

```bash
./scripts/test-wasm.sh --manual
```

### Host pitfalls (Exodus-specific)

- **Extensions** altering CSP via `webRequest` can break some WASM sites — retest with extensions disabled.
- **HTTP response proxy** (`exodus-proxy`) applies only when extension rules request it; large `.wasm` payloads are capped at 20MB in the proxy path.
- **HTTPS-only mode** upgrades `http://` navigations; wasm subresources should still load over HTTPS on modern sites.

### Not in scope

- **WASM plugins in Tauri** (wasmtime/WasmEdge) — different from page WASM; see architecture notes in project discussions.
- **Native Rust `.dylib` plugins** — separate subsystem, not wired to the extension store UI yet.
