# Exodus — automated testing

## Quick commands

| Command | Scope |
|---------|--------|
| `pnpm test:auto` | **PR gate** — Vue shell Vitest subset |
| `pnpm test:auto:e2e` | Shell subset + Playwright |
| `pnpm test:auto:ci` | Full Vitest + Playwright (**matches CI PR gate**) |
| `pnpm test:auto:all` | Shell + full Vitest |
| `pnpm test:auto:full` | `--e2e --all` + `verify-quick` |
| `pnpm test:vue-shell` | Vue browser shell regression (~50 tests) |
| `pnpm test` | All Vitest unit tests under `src/**/*.test.ts` |
| `pnpm test:frontend:full` | `vue-tsc` + full Vitest |
| `pnpm test:e2e:tauri` | Playwright vs running `tauri dev` (`TAURI_E2E=1`) |
| `pnpm test:wasm` | WebAssembly smoke page (Playwright) |
| `pnpm test:wasm:tauri` | WASM inside native tab webview (needs `tauri dev`) |
| `pnpm verify` | Frontend + Rust + integration scripts |

CI layout: [`CI.md`](./CI.md).

**Important:** Run each `pnpm` command on its own line. Do not paste shell comments on the same line (e.g. `pnpm test:auto:ci # note`) — `pnpm` forwards `#` and following words as script arguments. Scripts now ignore `#…` and stray words, but separate lines are still clearer.

### Tauri E2E (two terminals)

```bash
# Terminal 1
pnpm tauri dev

# Terminal 2 (after dev server is up at http://localhost:1421)
pnpm test:e2e:tauri
```

Playwright’s default `pnpm exec playwright test` starts **`pnpm dev:e2e`** on **`http://127.0.0.1:1431`** so it does not conflict with `tauri dev` on `:1421`. Use `TAURI_E2E=1` only when targeting an already-running Tauri/Vite shell on `:1421`.

## Vitest setup

- Config: [`vitest.config.ts`](../vitest.config.ts)
- Global setup: [`src/test/setup.ts`](../src/test/setup.ts) — `performance.memory` stub for jsdom
- Tauri mock helper: [`src/test/tauriCoreMock.ts`](../src/test/tauriCoreMock.ts)

Files that mock `@tauri-apps/api/core` must include `isTauri: () => true` when testing invoke paths (or use `createTauriCoreMock()`).

## Manual QA (Vue shell)

See [`MANUAL_QA_VUE_SHELL.md`](./MANUAL_QA_VUE_SHELL.md) for `tauri dev` checklist (Safe Browsing, `/ask`, CDN badge, ⌘⇧T, Shields).

## Coverage thresholds

Vitest coverage gates (lines/functions/statements 80%, branches 75%) are defined in `vitest.config.ts`. Run:

```bash
pnpm vitest run --coverage
```
