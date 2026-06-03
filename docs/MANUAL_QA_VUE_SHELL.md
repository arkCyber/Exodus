# Manual QA — Vue browser shell (`tauri dev`)

Run from repo root:

```bash
pnpm tauri dev
```

Dev UI loads at `http://localhost:1421` (`BrowserPage.vue`).

## Safe Browsing navigation guard

1. Enable Safe Browsing in **Settings → Privacy** (or use a known test URL your backend flags).
2. Enter a flagged URL in the address bar and press Enter.
3. **Expect:** `SafeBrowsingPrompt` modal with threat summary.
4. Click **Go back** — navigation should cancel.
5. Enter the same URL again, click **Proceed anyway** — tab should navigate.

## Address bar `/ask` (memory search)

1. Focus the address bar, type `/ask your query`, press Enter.
2. **Expect:** semantic search dropdown (when RAG/microservice is up) or a status hint if offline.

## P2P CDN page badge

1. Open a normal https page with CDN integration enabled.
2. **Expect:** CDN indicator on the address bar when the page is indexed/announced.

## Closed tabs (⌘⇧T)

1. Open several tabs, close two with ⌘W.
2. Press **⌘⇧T** twice.
3. **Expect:** last closed tabs restore in reverse order.

## Site Shields (tracker toggle)

1. Visit `https://example.com`.
2. Open the address bar **Shields** menu; toggle trackers for this site.
3. **Shift+click** the shield control (if wired) — **Expect:** per-site override without changing global defaults.

## Session & chrome smoke

- **⌘T / ⌘W** new/close tab
- **Find in page** (⌘F) bar appears and counts update
- **Settings** opens; Privacy panels render without console errors

## Automated regression (before manual pass)

```bash
pnpm test:auto          # vue-shell + full Vitest (595 tests)
pnpm test:auto:e2e      # + Playwright UI smoke (Vite, no Tauri)
pnpm test:auto:full     # + verify-quick (Rust subset)
```

CI: see [`CI.md`](./CI.md) — PRs run `test:auto:e2e` only; `main` push + nightly run full `verify.sh`.

Tauri backend E2E (Safe Browsing, CDN when indexed):

```bash
# Terminal 1
pnpm tauri dev

# Terminal 2
pnpm test:e2e:tauri
```
