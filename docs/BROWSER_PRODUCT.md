# Exodus Browser — Product Quality Roadmap

Comparison baseline: **Google Chrome** (ecosystem, polish) and **Brave** (privacy shields, speed).

## Current strength (ship-ready)

| Area | Status |
|------|--------|
| Multi-tab WebView shell | Good — pin, groups, vertical tabs, closed-tab restore |
| Omnibox + local RAG `/ask` | Good — differentiator |
| Bookmarks / history managers | Good |
| Allama local AI sidebar | Good |
| HTTPS-only, popup block, site permissions | Good |
| Session restore | Good |
| Find-in-page, reading mode, translate (basic) | Partial |
| MV3 extensions (subset) | Partial |

## Top gaps vs Chrome / Brave

1. **Threat intelligence** — Safe Browsing uses local demo patterns only (`safe_browsing.rs`). Need hash-prefix API or curated offline lists with updates.
2. **Shields at scale** — Tracker list is local JSON + injection; no EasyList subscription or cosmetic filtering.
3. **Tab discarding** — Freezer/sleep managers exist; WebViews are not destroyed/recreated on discard (memory win limited).
4. **Extension platform** — Missing `webRequest`, full `cookies`/`history` APIs, extension popups, store pipeline.
5. **Sync & profiles** — No encrypted sync; private mode is a toggle, not a separate storage partition.
6. **Downloads** — In-session list + progress; no resume DB or malware scan hook.
7. **New tab** — Brave-style wallpaper library (`static/newtab/wallpapers/`), clock, top sites, picker.
8. **DevTools** — Native toggle exists; release builds may gate the feature flag.

## Recently improved (this track)

- **Default new-tab wallpaper:** Nebula; synced via `localStorage` + `new_tab_settings.json`
- **Privacy defaults (new installs):** HTTPS-only and block popups enabled
- **Incognito shortcut:** ⌘⇧N / Ctrl+Shift+N
- **Tab freezer feedback:** status when inactive tabs are frozen
- Expanded tracker blocklist (`src-tauri/assets/tracker-blocklist.json`)
- Tab lifecycle wired from UI (`src/lib/tabLifecycle.ts`)
- Brave-style **Top sites** + **wallpaper library** on new tab (`NewTabPage.svelte`, `docs/NEWTAB_WALLPAPERS.md`)
- **Shields** button in address bar (tracker count → Privacy settings)
- Download panel: open file + reveal in folder per item
- **Profile storage**: `profiles/default` vs `profiles/private` for history + cookies
- **Download persistence** + HTTP Range resume (`list_persisted_downloads`, `downloads.rs`)
- **Per-site shields**: Shift+click shield → allow trackers on current host
- **Blocklist refresh**: Settings → Update tracker blocklist
- Private mode change recreates WebViews (`exodus-private-mode-changed`)
- Product audit doc (this file)

## Recently shipped (advanced track)

- **`chrome.webRequest` v2** — block + redirect at navigation; `requestHeaders` flush; subresource guard via injected fetch/XHR patch (`plugins/web_request.rs`)
- **Extension popup window** — dedicated `WebviewWindow` (`extension_open_popup_window`); `chrome.action` title/badge; `openPopup` via `exodus-extension-open-popup` event
- **True tab discard** — `browser_discard_tab` destroys WebView; restore on focus
- **Safe Browsing online list** — `refresh_safe_browsing_list` + persisted `threats.json`
- **Blocklist subscription** — EasyList/ABP → domains + cosmetic `##` / `#@#` exceptions
- **`webRequest` header proxy** — native `exodus-proxy://` WebView scheme + `onBeforeSendHeaders` request headers + `onHeadersReceived` response headers (`http_response_proxy.rs`, `docs/HTTP_RESPONSE_PROXY.md`)
- **Cosmetic** — CSS, `/regex/`, `:-abp-has` procedural, `#@#` exceptions
- **MITM subresources** — fetch/XHR/src via loopback proxy when header rules active
- **Bookmark bar** — drag-reorder, drag into folders, `localStorage` visibility
- **Encrypted sync vault** — local AES-256-GCM + cloud `PUT/GET /vault/{deviceId}`; dev server `pnpm sync-server` (`docs/ENCRYPTED_SYNC.md`)
- **Extension popup window** — manifest display name as window title
- **Extension commands registered** — permissions resolve, host access, notifications

## Quick wins (next)

- Subresource proxy without document-start rewriter (needs platform resource API)
- `:-abp-has` / `:matches-css` full procedural engine
- Create bookmark folders from drag-and-drop

## Larger projects

- Full extension host parity
- Brave-class filter subscription (regex + exceptions)
- Safe Browsing provider integration
- Tab discard v2 (destroy WebView, restore scroll)
- Encrypted bookmark/history sync

## How to verify locally

```bash
sh scripts/start-exodus-ai.sh   # Allama + Exodus dev
pnpm test
cargo check -p exodus-tauri
```

Manual checklist:

1. New tab shows wallpaper, **Top sites**, quick links.
2. Visit a tracker-heavy site → Shields count increases → click Shields → Privacy settings.
3. Open 15+ tabs, wait ~5 min → background tabs marked frozen/sleeping (Performance settings).
4. Download a file → **Open** / **Show in folder** from Downloads panel.
