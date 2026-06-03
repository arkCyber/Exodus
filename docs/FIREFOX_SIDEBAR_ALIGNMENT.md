# Firefox Sidebar Alignment (Exodus)

Reference: [Firefox sidebar help](https://support.mozilla.org/en-US/kb/use-sidebar-access-tools-and-vertical-tabs) (Firefox 136+).

## Firefox model

| Element | Firefox | Exodus |
|--------|---------|--------|
| Icon rail | ~48px, tool icons | `--chrome-sidebar-icon-rail: 48px`, SVG icons in `sidebarIcons.ts` |
| Content panel | ~320px, resizable | `--chrome-sidebar-content-width: 320px`, drag left edge |
| Collapse | Icon rail only | Footer « collapse on `BrowserSidebar` |
| History | Clock icon, **Ctrl+Shift+H** | Memory panel, **Ctrl+H** / **Ctrl+Shift+H** |
| Bookmarks | Star icon, **Ctrl+B** | Bookmarks panel; **Ctrl+Alt+B** (Chrome keeps **Ctrl+B** for bar) |
| Toggle sidebar | Toolbar button | **Ctrl+Shift+B** |
| Close | × | Calls `closeSidebar()` (does not toggle) |

## Panels

1. **AI Chat** — Exodus-specific (Firefox optional AI slot).
2. **Memory & History** — Indexed memory + browsing history (Firefox splits History / synced tabs).
3. **Bookmarks** — Search + folder edit.
4. **Pocket** — Saved articles.
5. **P2P** — Group chat / CDN (Exodus-specific).

## Code map

- UI: `src/components/BrowserSidebar.vue`
- State: `src/composables/useBrowserSidebar.ts`
- Icons: `src/lib/sidebarIcons.ts`
- Layout persistence: `src/lib/sidebarLayout.ts`
- Tokens: `src/styles/chrome-layout.css`, `src/lib/chromeLayout.ts`
- Prototype (Svelte): `src/lib/components/FirefoxStyleSidebar.svelte`

## Tests

```bash
pnpm exec vitest run src/lib/sidebarIcons.test.ts src/lib/sidebarLayout.test.ts \
  src/components/BrowserSidebar.test.ts src/composables/useBrowserSidebar.test.ts \
  src/lib/browserShortcuts.test.ts src/components/sidebar/SidebarMemoryPanel.test.vue
```

## UI (Firefox 136)

Shared styles: `src/styles/sidebar-ui.css` (imported from `main.ts`).

- Icon rail: 40px rounded buttons, accent strip on active tool
- Content panel: card-style Customize sections, refined list/search inputs
- Vertical tabs in sidebar: full-width rows with left accent (not top tab strip)
- Toolbar: sidebar + Pocket SVG buttons with active highlight
- Sidebar on **left**: correct border + resize handle on inner edge

## Implemented (Firefox 136 parity)

| Feature | Status |
|--------|--------|
| Vertical tabs in sidebar | `SidebarVerticalTabsPanel` + Customize toggle |
| Synced tabs panel | `SidebarSyncedTabsPanel` + `syncedTabs.ts` |
| Reading list panel | `SidebarReadingListPanel` (local Pocket) |
| Sidebar left/right | Customize → `sidebarPreferences.position` |
| Customize tools | `SidebarCustomizePanel` + gear icon |
| Toolbar Pocket button | `AddressBar` Pocket SVG button |
| Toolbar sidebar toggle | `AddressBar` sidebar button (⌘⇧B) |

## Not yet ported

- Auto-hide on mouse leave
- Hide horizontal tab strip without sidebar (edge case)
- Real Mozilla Account synced tabs (uses local demo + mobile sync hook)

Set `WALLPAPER_FEATURE_ENABLED` separately in `newTabWallpaper.ts` (unrelated).
