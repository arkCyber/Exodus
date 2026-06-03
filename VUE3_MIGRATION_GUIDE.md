# Vue 3 Migration Guide

## Overview
This guide documents the migration from Svelte 5 to Vue 3 to resolve the spinning cursor issue on macOS.

## Migration Status

### ✅ Completed
- Updated package.json (removed Svelte, added Vue 3 + Vue Router + lodash-es)
- Updated vite.config.js (using @vitejs/plugin-vue, port changed to 1421)
- Created src/main.ts (Vue entry point)
- Created src/App.vue (root component)
- Created src/views/HomePage.vue (test page)
- Created src/views/BrowserPage.vue (simplified browser page)
- Created src/components/AddressBar.vue (address bar component)
- Created src/components/BrowserTabBar.vue (tab bar component)
- Created src/components/BookmarkBar.vue (bookmark bar component)
- Created src/components/FindBar.vue (find bar component)
- Created src/components/StatusBar.vue (status bar component)
- Created src/components/ContextMenu.vue (context menu component)
- Created index.html (HTML entry point)
- Updated tsconfig.json for Vue 3
- Created tsconfig.node.json
- Created src/app.css (global styles with Chrome/Brave design system)
- Created src/composables/useTauri.ts (Tauri integration)
- Created src/composables/useMicroservice.ts (microservice integration with throttling)
- Created src/vite-env.d.ts (Vue type declarations)
- Cleared svelte.config.js
- Created VUE3_MIGRATION_GUIDE.md (migration guide)
- Installed dependencies (pnpm install)
- Updated Tauri config (port 1421)
- Updated BrowserPage to use AddressBar, BrowserTabBar, BookmarkBar, FindBar, StatusBar, and ContextMenu
- Updated all component styles to match Chrome/Brave design system
- Added keyboard shortcuts (Cmd/Ctrl+F for find, Escape to close, Cmd+D for bookmark, Cmd+T for new tab, Cmd+R for reload)
- Added status messages and privacy stats UI
- Added context menu with browser actions
- Added developer tools integration
- Prepared Tauri command integration for navigation

### 🚧 In Progress
- Final migration guide update

### ✅ Extension host (Vue 3)
- `useExtensions` composable — tab sync, runtime pump, permission/host-install prompts
- `ExtensionActionBar`, `ExtensionPermissionPrompt`, `ExtensionHostInstallPrompt`
- `BrowserPage.vue` — native per-tab webviews + extension event wiring
- `ExtensionsSettings.vue` — install/rescan/enable/uninstall/site access
- `SettingsModal.vue` — browser, privacy, AI, extensions sections
- `useBrowserConfig` — load/save `get_ai_config` + privacy settings

### ✅ Settings panels (Vue 3)
- Password, Cookie, History manager, Vertical tabs, New tab wallpaper
- Allama service, Inference engine, P2P CDN, Group chat
- Full `SettingsModal.vue` (replaces Svelte migration note)

### ✅ Extension host UX
- `chrome.contextMenus` flush → registry + merged host context menu
- Extension `omnibox` keyword routing in `AddressBar` + `useOmnibox`
- Rust: `extension_context_menus_list_host`, `extension_context_menu_clicked`, `extension_omnibox_list_keywords`, `extension_omnibox_dispatch`

### ✅ Sidebar & downloads (Vue 3)
- `BrowserSidebar.vue` — AI chat, memory/history, bookmarks, Pocket, P2P hub, Agent panel
- `DownloadPanel.vue` + `useBrowserDownloads` — Tauri `exodus-download-*` events
- `useBrowserSidebar` — AI (Allama HTTP), RAG memory, Hermes agent actions
- `ImMessenger.vue`, `P2pSidebarPanel.vue`, `PocketPanel.vue`, `AgentPanel.vue`
- `BrowserPage.vue` — `browser-body-row` layout with right sidebar column
- Vitest: `BrowserSidebar.test.ts`, `DownloadPanel.test.ts`, `useBrowserDownloads.test.ts`, `AgentPanel.test.ts`

### ✅ P2P panels (Vue 3)
- `FileTransfer.vue` — ExodusWorkSpace transfers + workspace files
- `VideoCall.vue` / `MeetingRoom.vue` — WebRTC 1:1 and mesh meetings
- `CollaborativeEditing.vue` + `$lib/collaborativeDocs.ts` — local collab docs (localStorage)
- `ContactDirectoryPanel.vue` — contacts list, IM/call shortcuts
- `P2pSidebarPanel.vue` — IM, Contacts, Group, CDN, WorkSpace, Collab, Call, Meeting tabs

### ✅ Chrome shell parity (Vue 3)
- `PasswordSavePrompt.vue` + `usePasswordSaveOffer` — autofill hooks and save offer after navigation
- `TabGroupEditPrompt.vue` / `TabGroupDeletePrompt.vue` + `useBrowserTabGroups`
- `BrowserTabBar.vue` — group colors, tab context menu (pin, duplicate, groups)
- `ConfirmPrompt.vue` + `useConfirmDialog` + `$lib/confirm.ts` — replaces `window.confirm` for memory/history/collab delete
- `useBrowserSession` — `save_session` / `load_session` with privacy gating
- `useVerticalTabLayout` — vertical tab strip in `BrowserPage.vue` (`browser-shell` / `browser-main`)
- `SettingsModal` → `@vertical-layout-change` updates layout live
- Wired in `BrowserPage.vue` (active entry: `index.html` → `src/main.ts`)

### ✅ Shell parity (latest)
- `useClosedTabs` + `restoreClosedTab` (⌘⇧T) + context menu entry
- `useFindInPage` — native `findInTab` + iframe `window.find`
- `BrowserSitePermissionPrompt.vue` + `useBrowserSitePermissions`
- `bindLifecycleRecovery()` on mount; `mountBrowserShortcuts()` (replaces ad-hoc keydown)
- **`AddressBar.vue`** — chrome menu (reopen tab count, downloads, index, translate, print), shields badge, sidebar shortcuts
- **`useSiteShields`** — per-site tracker allow/block + global tracking protection
- **`useBrowserTabLifecycle`** — `register_tab` / `tab_sleep_*` on tab open/close/activate
- **`BrowserSitePermissionsSettings.vue`** in Settings modal

### ✅ Omnibox & privacy (latest)
- `useOmnibox` — `/ask` → `semantic_search` dropdown in `AddressBar.vue`
- `useCdnPageStatus` — P2P CDN omnibox badge (`P2P · N` / cached)
- `PrivacyShieldSettings.vue` — Safe Browsing, tracking protection, blocklist, encrypted sync

### ✅ Safe Browsing & Svelte retirement
- `SafeBrowsingPrompt.vue` + `useSafeBrowsingNavigation` — `checkNavigationGuard` before `commitNavigation`
- `navigateToAddress` applies `applyHttpsOnly` then Safe Browsing gate
- **Svelte shell retired**: `+page.svelte` moved to `src/routes/_archive/+page.svelte.legacy` (see `src/routes/RETIRED.md`)

### ⏳ Pending
- End-to-end verification in `tauri dev`

## Svelte 5 to Vue 3 Syntax Mapping

### Reactive State
```svelte
// Svelte 5
let count = $state(0);
let items = $state([1, 2, 3]);
```
```vue
// Vue 3
import { ref, reactive } from 'vue';

const count = ref(0);
const items = ref([1, 2, 3]);

// For objects
const state = reactive({ count: 0, items: [1, 2, 3] });
```

### Effects
```svelte
// Svelte 5
$effect(() => {
  console.log(count);
});
```
```vue
// Vue 3
import { watchEffect } from 'vue';

watchEffect(() => {
  console.log(count.value);
});
```

### Computed Values
```svelte
// Svelte 5
let doubled = $derived(count * 2);
```
```vue
// Vue 3
import { computed } from 'vue';

const doubled = computed(() => count.value * 2);
```

### Lifecycle Hooks
```svelte
// Svelte 5
onMount(() => {
  console.log('mounted');
});

onDestroy(() => {
  console.log('destroyed');
});
```
```vue
// Vue 3
import { onMounted, onUnmounted } from 'vue';

onMounted(() => {
  console.log('mounted');
});

onUnmounted(() => {
  console.log('destroyed');
});
```

### Props
```svelte
// Svelte 5
<script>
  let { title, count = 0 } = $props();
</script>
```
```vue
// Vue 3
<script setup lang="ts">
interface Props {
  title: string;
  count?: number;
}

const props = withDefaults(defineProps<Props>(), {
  count: 0
});
</script>
```

### Events
```svelte
// Svelte 5
<button onclick={handleClick}>Click</button>
```
```vue
// Vue 3
<button @click="handleClick">Click</button>
```

### Two-way Binding
```svelte
// Svelte 5
<input bind:value={text} />
```
```vue
// Vue 3
<input v-model="text" />
```

## Key Vue 3 Features for Spinning Cursor Prevention

### 1. Async Batching
Vue 3 automatically batches state updates in the same event loop tick, preventing excessive DOM updates.

### 2. shallowRef
Use `shallowRef` for large objects (like tab lists) to avoid deep reactivity overhead:
```vue
import { shallowRef } from 'vue';

// ❌ Bad: Deep reactivity for large objects
const tabs = ref([...largeData]);

// ✅ Good: Shallow reactivity
const tabs = shallowRef([...largeData]);
```

### 3. Throttling
Use the `useMicroservice` composable with built-in throttling for high-frequency events:
```vue
import { useMicroservice } from '@/composables/useMicroservice';

const { listenThrottled } = useMicroservice('network-service');

// Automatically throttled to 60 FPS (16ms)
listenThrottled('progress-update', (progress) => {
  loadingProgress.value = progress;
});
```

## Component Migration Order

1. **Core Components** (Priority: High)
   - +page.svelte → HomePage.vue
   - BrowserContent.svelte
   - BrowserPanel.svelte

2. **Browser UI Components** (Priority: High)
   - AddressBar.svelte
   - BrowserTabBar.svelte
   - BookmarkBar.svelte
   - FindBar.svelte

3. **Settings Components** (Priority: Medium)
   - SettingsModal.svelte
   - ExtensionsSettings.svelte
   - AllamaServiceSettings.svelte
   - Other settings panels

4. **Microservice Components** (Priority: Medium)
   - FileTransfer.svelte
   - GroupChatPanel.svelte
   - P2pCdnPanel.svelte
   - Other microservice UI

5. **Utility Components** (Priority: Low)
   - MetricsChart.svelte
   - PerformanceMonitor.svelte
   - Other monitoring components

## Testing Checklist

- [ ] Install dependencies: `pnpm install`
- [ ] Test dev server: `pnpm dev`
- [ ] Test Tauri dev: `cargo tauri dev`
- [ ] Verify no spinning cursor
- [ ] Test all browser features
- [ ] Test all settings
- [ ] Test microservice integrations
- [ ] Test production build: `pnpm build && cargo tauri build`

## Notes

- All Svelte components remain in `src/lib/components/` until migrated
- Migrated Vue components should go to `src/components/`
- Keep the original Svelte files for reference during migration
- Delete Svelte files only after successful migration and testing
