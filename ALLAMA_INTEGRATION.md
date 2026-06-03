# Allama integration (Ollama replacement)

**Status:** Microservice module wired — HTTP on **port 11435**, UDS control plane, Tauri lifecycle. Native GGUF inference requires building the `allama` C++ binary; until then Exodus runs an **embedded Ollama-compatible gateway** backed by the Rust `InferenceEngine` stub.

## Role in Exodus

| Concern | Ollama (legacy) | Allama (Exodus default) |
|--------|------------------|-------------------------|
| Default port | 11434 | **11435** |
| Chat API | `/api/*` | Same + OpenAI `/v1/*` |
| Auto-start | `spawn_sidecar` (exodus-core) | **`spawn_allama`** (default `true`) |
| Browser `ai_port` | was 11434 | **11435** |

Exodus chat, embeddings health checks, and CSP `connect-src` target `localhost:11435` when Allama is enabled.

## Architecture

```
┌─────────────────┐     HTTP :11435      ┌──────────────────────────────┐
│  Browser / AI   │ ───────────────────► │ Native `allama serve`        │
│  (ai.rs)        │   /api/*  /v1/*      │  OR embedded allama_gateway  │
└─────────────────┘                      └──────────────┬───────────────┘
                                                          │
┌─────────────────┐     UDS JSON-RPC     ┌──────────────▼───────────────┐
│ ServiceRegistry │ ◄────────────────────► │ allama_service (control)   │
└─────────────────┘                        └────────────────────────────┘
                                                          │
                                               InferenceEngine (model registry)
```

### Rust modules

| Path | Purpose |
|------|---------|
| `src-tauri/src/microservice/allama_process.rs` | Binary discovery, `spawn_allama_serve`, HTTP probe |
| `src-tauri/src/microservice/allama_gateway.rs` | Embedded Axum server (Ollama + OpenAI shapes) |
| `src-tauri/src/microservice/allama_service.rs` | UDS control: `health`, `get_status`, `get_port` |
| `src-tauri/src/microservice/allama_commands.rs` | Tauri: start/stop/restart/status/health/models |
| `src-tauri/src/allama_manager.rs` | Native-first, gateway fallback, auto-start |

### Startup

1. `InferenceEngine` scans `allama/models/` (GGUF).
2. If `spawn_allama` is true, `AllamaManager` starts:
   - **Native** when `allama/target/release/allama` (or `ALLAMA_BINARY`) exists.
   - **Embedded gateway** otherwise on configured `ai_port` (default 11435).
3. Allama is registered in `ServiceRegistry` as `allama-service`.

### Settings UI

`AllamaServiceSettings.svelte` — toggle auto-start, port, start/stop/restart, **Test HTTP** (`GET /api/tags`), live status.

## Tauri commands

```typescript
await invoke('allama_service_start');
await invoke('allama_service_status');
await invoke('allama_http_health');
await invoke('allama_list_models');
await invoke('set_ai_config', {
  aiPort: 11435,
  spawnAllama: true,
  // ... other fields
});
```

## Build native Allama (optional, full GGUF inference)

Exodus ships a **partial** CMake tree under `allama/` (no `ggml`). Use the **Rust** Allama repo (recommended):

```bash
# From Exodus repo root
sh scripts/build-allama.sh
# Builds ../Allama/allama or ~/Allama/allama via cargo, then prints:
#   export ALLAMA_BINARY=.../target/release/allama

export ALLAMA_BINARY=/path/to/allama/target/release/allama
```

Verify HTTP inference (tags + generate):

```bash
sh scripts/verify-allama-native.sh
# Env: ALLAMA_VERIFY_PORT=11436  ALLAMA_VERIFY_MODEL=gemma4-4b
```

**Models:** Rust `allama serve` reads `~/.allama/models`. Exodus copies/symlinks GGUF trees from `allama/models/` into that directory on spawn (see `link_exodus_models_into_allama_home`). Do **not** pass `--models` to the Rust binary (Exodus detects this automatically).

Place GGUF files under `allama/models/` or install into `~/.allama/models` via `allama pull`.

**Cleanup + Modelfile/templates:** `sh scripts/cleanup-and-enrich-allama-models.sh` — archives stubs, adds `Modelfile` / `template.jinja` / `system.txt` / `parameters.json` per model (see `docs/ALLAMA_MODELS.md`).

## Clients

- **Browser (Exodus UI):** `src/lib/allamaClient.ts` — HTTP + SSE streaming, `checkEmbeddingsOnline`, `allamaEmbed`
- **Config loader:** `src/lib/aiConfig.ts` — `loadAiConfig()` from Tauri `get_ai_config`
- **Sidebar chat:** `src/lib/sidebarAiChat.ts` — `streamSidebarChat` / `streamSidebarSummarize` (direct HTTP; supports `AbortSignal` for Stop)
- **Agent panel:** `ask: question` in the command box, or **Ask AI** button — sends page DOM context to sidebar chat via Allama
- **Extensions:** injected `window.exodus.allama` — `health`, `chat`, `generate`, `embed`, `streamChat` (`chrome_bridge.rs`); TypeScript helpers in `src/lib/extensions/exodusAllama.ts`
- **Node:** `allama/js-client` — default base URL `http://127.0.0.1:11435`
- **Python:** `allama/python-client`

## Scripts

```bash
sh scripts/test-allama.sh         # Rust + frontend; native verify if binary exists
sh scripts/build-allama.sh        # cargo build (Rust allama) or cmake if ggml present
sh scripts/verify-allama-native.sh # /api/tags + /api/generate smoke test
pnpm verify                       # includes test-allama in full CI locally
```

## Inference engine (Rust)

| Path | Behavior |
|------|----------|
| Native `allama serve` on :11435 | `InferenceEngine` calls **Allama HTTP** (`generate` / `chat` / `embed`) |
| Embedded gateway | Gateway uses `generate_local` (no HTTP loopback); Tauri `inference_*` uses local path |
| External clients | Browser `ai.rs`, Hermes, Python → HTTP on configured port |

Models directory: **`{app_data_dir}/allama/models`** (created on startup). Bundled `allama/models` is scanned if the app dir has no GGUF files yet.

### Hermes

Analysis / custom tasks with `use_allama=true` or `inference_engine=allama` in task metadata call Allama HTTP. Default port **11435** (`HermesConfig.allama_http_port`).

### Python microservice

- `ALLAMA_BASE_URL` / `OLLAMA_HOST` env vars set when spawning the Python process.
- Execute with prefix `ALLAMA_CHAT:your prompt` to route through Rust → Allama HTTP.
- Code referencing `allama_client` / `AllamaClient` uses the bridge when Allama is online.

## Context compression (TQ)

Configured via `inference_update_config` when using the inference API directly. Native Allama adds SIMD/GPU/KV optimizations from the upstream project.

## Notes

- Large models (20GB+) need disk and RAM; first load is slow.
- Embedded mode returns stub text until native binary or FFI is connected.
- Vector search UI uses `checkEmbeddingsOnline` on the same port as chat (no separate `embeddings_health` invoke from the frontend).

---

**Document version:** 2.0  
**Last updated:** 2026-05-19  
## Testing

| Suite | Command | Status |
|-------|---------|--------|
| Allama stack (mock HTTP + gateway + Hermes + Python) | `cargo test -p exodus-tauri --lib allama_stack` | 9 tests |
| Full Rust lib | `cargo test -p exodus-tauri --lib` | 279 passed |
| Frontend | `pnpm test` | 149 passed |
| Extension Allama shim | `cargo test -p exodus-tauri chrome_bridge::tests::allama_shim` | 1 test |

**Build:** `cargo check -p exodus-tauri` OK
