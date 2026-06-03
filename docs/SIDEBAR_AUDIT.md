# Sidebar audit (Firefox-style)

Last review: 2026-05-23.

## Fixed in this pass

| Issue | Fix |
|-------|-----|
| Shortcuts `Ctrl+H` / bookmarks bypassed `resolvePanel` | `openHistory` / `openBookmarksPanel` use `openSidebarPanel()` |
| Toolbar sidebar toggle ignored vertical tabs | `toggleSidebarSmart()` opens **Tabs** when vertical mode on |
| Disabled tool could stay active after reload | `ensureValidSidebarPanel()` after `loadPrefs()` |
| Unchecking **Tabs** tool left vertical mode on | `toggleTool('tabs')` clears `verticalTabsInSidebar` |
| Bookmark search lost on re-open | `bookmarkSearchQuery` prop + watch in `BrowserSidebar` |
| `BrowserPage.test` missing mocks | `useSidebarPreferences` + filtered history refs |
| E2E sidebar selector stale | `aria-label="Toggle sidebar"` |

## Architecture

```
AddressBar / shortcuts
    → openSidebarPanel() / toggleSidebarSmart()
        → useSidebarPreferences.resolvePanel()
        → useBrowserSidebar.openPanel()
BrowserSidebar
    → panels: tabs | memory | bookmarks | synced | reading | pocket | p2p | customize
    → prefs: sidebarPreferences.ts (localStorage)
```

## Fixed in pass 2 (2026-05-23)

| Issue | Fix |
|-------|-----|
| Tab bar missing pin/duplicate/groups | `buildTabBarHandlers` + top & sidebar `BrowserTabBar` |
| Tab group edit/delete prompts missing | `TabGroupEditPrompt` / `TabGroupDeletePrompt` on `BrowserPage` |
| Synced tabs no “This computer” | `buildThisDeviceSyncedTabs` in `refreshSyncedTabs` |
| Reading list = all Pocket items | `pocketListReadingList()` (unread, not archived) |
| Save to reading list | Menu + `pocketSaveArticle` |
| Settings → Customize sidebar | Button + `openSidebarCustomize` emit |
| WebView layout on sidebar move | `watchWebviewLayout` when sidebar opens/moves |

## Fixed in pass 3 (audit + tests, 2026-05-23)

| Issue | Fix |
|-------|-----|
| Reading list showed all Pocket unread | `pocketListReadingList()` filters `reading-list` tag |
| Synced panel stale when tabs change | `watch(openTabs)` on `SidebarSyncedTabsPanel` |
| NTP URLs in “This computer” | `isNewTabUrl` filter in `buildThisDeviceSyncedTabs` |
| Hint text CSS wrong class | `.sidebar-vtabs-hint` in vertical tabs panel |
| Save reading list UX | Opens **Reading list** sidebar after save |
| No dedicated test script | `pnpm test:sidebar` → `scripts/test-sidebar.sh` |

## Remaining gaps (backlog)

1. **Synced tabs** — remote devices still demo data; needs real sync protocol.
2. **Reading list migration** — older Pocket saves without `reading-list` tag won’t appear until re-saved.
3. **Vertical tabs** — context menu uses `position: fixed`; overflow on narrow sidebar may still clip tall menus.

## Fixed in pass 4 (automated test expansion)

| Item | Change |
|------|--------|
| E2E sidebar flows | `e2e/sidebar-firefox.spec.ts` (toggle, customize, reading list menu, settings) |
| BrowserPage wiring tests | `src/views/BrowserPage.sidebar.test.ts` |
| Reading list panel tests | `SidebarReadingListPanel.test.ts` |
| Shortcut coverage | `toggleSidebar` / `openBookmarksPanel` in `browserShortcuts.test.ts` |
| CI auto pipeline | `test-auto.sh` runs `test:sidebar`; `--e2e` includes sidebar spec |
| Mobile sync noise in unit tests | `syncedTabs.test.ts` mocks `mobileSync` |

## Test commands

```bash
pnpm test:sidebar          # dedicated Vitest bundle (~95 tests)
pnpm test:auto             # vue-shell + sidebar unit
pnpm test:auto:e2e         # above + Playwright (vue-shell-qa + sidebar-firefox)
```

```bash
pnpm exec vitest run src/views/BrowserPage.test.ts src/views/BrowserPage.sidebar.test.ts
pnpm exec playwright test e2e/sidebar-firefox.spec.ts
```
