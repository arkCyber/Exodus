<!--
  Exodus Browser — toolbar: navigation, omnibox, shields, sidebar shortcuts, chrome menu.
-->
<template>
  <div
    class="address-bar exodus-address-bar exodus-chrome-toolbar chrome-drag-surface"
    data-tauri-drag-region
    @mousedown="onChromeMouseDown"
  >
    <button type="button" class="nav-icon-btn tab-icon-btn" title="Tabs" aria-label="Tabs" @click="emit('showTabs')">
      <LayoutGrid :size="18" class="nav-svg" aria-hidden="true" />
    </button>

    <div class="nav-controls">
      <button
        type="button"
        class="nav-icon-btn"
        :disabled="!canGoBack"
        title="Back"
        aria-label="Back"
        @click="emit('goBack')"
      >
        <ChevronLeft :size="18" class="nav-svg" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="nav-icon-btn"
        :disabled="!canGoForward"
        title="Forward"
        aria-label="Forward"
        @click="emit('goForward')"
      >
        <ChevronRight :size="18" class="nav-svg" aria-hidden="true" />
      </button>
      <button type="button" class="nav-icon-btn" title="Reload (⌘R)" aria-label="Reload" @click="emit('reload')">
        <RefreshCw :size="18" class="nav-svg" aria-hidden="true" />
      </button>
      <button type="button" class="nav-icon-btn" title="Home" aria-label="Home" @click="emit('home')">
        <Home :size="18" class="nav-svg" aria-hidden="true" />
      </button>
    </div>

    <form class="url-input-wrapper omnibox" @submit.prevent="() => void submitNavigate()">
      <span
        v-if="currentUrl && !isNewTabUrl(currentUrl)"
        class="site-indicator"
        :class="{ secure: isSecureUrl(currentUrl) }"
        :title="isSecureUrl(currentUrl) ? 'Connection is secure' : 'Not secure'"
      >
        <Lock
          v-if="isSecureUrl(currentUrl)"
          :size="14"
          class="site-indicator-icon"
          aria-hidden="true"
        />
        <LockOpen
          v-else
          :size="14"
          class="site-indicator-icon site-indicator-icon--insecure"
          aria-hidden="true"
        />
      </span>
      <input
        :id="omniboxInputId"
        v-model="urlInput"
        type="text"
        class="url-input"
        :class="{
          'has-site-indicator': currentUrl && !isNewTabUrl(currentUrl),
          'has-cdn-badge': !!cdnStatusLabel,
        }"
        :placeholder="omniboxPlaceholder"
        autocomplete="off"
        @input="handleInput"
        @focus="onOmniboxFocus"
        @keydown="handleKeydown"
      />
      <button
        v-if="cdnStatusLabel"
        type="button"
        class="cdn-omnibox-badge"
        title="P2P CDN status for this page"
        @click="emit('cdnBadgeClick')"
      >
        {{ cdnStatusLabel }}
      </button>
      <div v-if="showSearchResults" class="search-results-dropdown">
        <div v-if="isSearching" class="search-loading">
          <span class="spinner-small" />
          <span>Searching local memory…</span>
        </div>
        <template v-else-if="searchResults.length > 0">
          <div
            v-for="(result, idx) in searchResults"
            :key="`${result.page.url}-${idx}`"
            class="search-result-item"
            role="link"
            tabindex="0"
            @mousedown.prevent="selectSearchResult(result.page.url)"
            @keydown.enter="selectSearchResult(result.page.url)"
          >
            <div class="result-title">{{ result.page.title }}</div>
            <div class="result-url">{{ result.page.url }}</div>
            <div class="result-meta">
              <span class="result-score">{{ (result.score * 100).toFixed(0) }}%</span>
              <button
                v-if="announceSearchEnabled"
                type="button"
                class="search-p2p-btn"
                title="Announce to P2P CDN lobby"
                @mousedown.stop
                @click.stop="emit('announceSearchResult', result.page.url, result.page.title)"
              >
                P2P
              </button>
            </div>
          </div>
        </template>
        <div v-else class="search-empty">No results in local memory</div>
      </div>
      <ul v-else-if="showSuggestions && suggestions.length" class="omnibox-suggestions" role="listbox">
        <li
          v-for="row in suggestions"
          :key="row.id"
          role="option"
          class="suggestion-row"
          @mousedown.prevent="selectSuggestion(row.url)"
        >
          <span class="suggestion-text">{{ row.text }}</span>
          <span class="suggestion-type">{{ suggestionTypeLabel(row.suggestion_type) }}</span>
        </li>
      </ul>
      <p v-if="activeExtensionKeyword" class="ext-keyword-hint">
        Extension: {{ activeExtensionKeyword.extensionName }} ({{ activeExtensionKeyword.keyword }})
      </p>
    </form>

    <button
      type="button"
      class="toolbar-icon-btn bookmark-star-btn"
      :class="{ bookmarked: isBookmarked }"
      :title="isBookmarked ? 'Remove bookmark' : 'Add bookmark'"
      @click="emit('toggleBookmark')"
    >
      {{ isBookmarked ? '★' : '☆' }}
    </button>

    <button
      v-if="currentUrl && !isNewTabUrl(currentUrl)"
      type="button"
      class="toolbar-icon-btn shields-btn"
      :class="{ 'shields-off': !shieldsEnabled }"
      :title="shieldsTitle"
      @click="onShieldsClick"
    >
      🛡
      <span v-if="shieldsCount > 0" class="toolbar-badge shields-badge">
        {{ shieldsCount > 99 ? '99+' : shieldsCount }}
      </span>
    </button>

    <div class="toolbar-actions">
      <button
        type="button"
        class="toolbar-icon-btn sidebar-toggle-btn"
        :class="{ active: sidebarOpen }"
        title="Sidebar (⌘⇧B)"
        aria-label="Toggle sidebar"
        @click="emit('toggleSidebar')"
      >
        <PanelLeftOpen :size="18" class="toolbar-svg" aria-hidden="true" />
      </button>
      <button
        type="button"
        class="toolbar-icon-btn webchat-toggle-btn"
        :class="{ active: showWebChatView }"
        title="WebChat (⌘⇧W)"
        aria-label="Toggle WebChat"
        @click="handleWebChatClick"
      >
        <MessageCircle :size="18" class="toolbar-svg" aria-hidden="true" />
        <span v-if="webchatUnread > 0" class="toolbar-badge webchat-badge">{{ webchatUnread > 99 ? '99+' : webchatUnread }}</span>
      </button>
      <button type="button" class="toolbar-icon-btn" title="Downloads" @click="emit('openDownloads')">
        <Download :size="18" class="toolbar-svg" aria-hidden="true" />
        <span v-if="downloadsBadge > 0" class="toolbar-badge">{{ downloadsBadge }}</span>
      </button>
    </div>

    <div class="toolbar-end">
      <ExtensionActionBar
        inline
        :refresh-key="extensionsRefreshKey"
        @open-extensions-manager="emit('openExtensions')"
        @popup-closed="emit('extensionPopupClosed')"
      />
      <button type="button" class="chrome-menu-btn" title="Menu" aria-label="Menu" @click="emit('toggleMenu')">
        <Menu :size="18" class="toolbar-svg" aria-hidden="true" />
      </button>
    </div>
    
    <!-- Menu teleported to body to avoid z-index issues with webview -->
    <Teleport to="body">
      <template v-if="showMenu">
        <!-- Webview overlay -->
        <div class="webview-block-overlay" @click="emit('closeMenu')" />
        <!-- Menu backdrop -->
        <button type="button" class="menu-backdrop" aria-label="Close menu" @click="emit('closeMenu')" />
        <!-- Menu dropdown -->
        <div class="chrome-menu-dropdown" role="menu">
        <button type="button" class="menu-item" @click="menuAction('newTab')">
          <Plus :size="16" class="menu-icon" aria-hidden="true" />
          <span>New tab</span>
          <span class="menu-shortcut">⌘T</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('newWindow')">
          <Square :size="16" class="menu-icon" aria-hidden="true" />
          <span>New window</span>
          <span class="menu-shortcut">⌘N</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('newIncognitoWindow')">
          <Shield :size="16" class="menu-icon" aria-hidden="true" />
          <span>New incognito window</span>
          <span class="menu-shortcut">⌘⇧N</span>
        </button>
        <div class="menu-divider" />
        <button type="button" class="menu-item" @click="menuAction('toggleBookmark')">
          <Star :size="16" class="menu-icon" aria-hidden="true" />
          <span>{{ bookmarked ? 'Edit bookmark' : 'Bookmark this page' }}</span>
          <span class="menu-shortcut">⌘D</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('openBookmarksPanel')" @mouseenter="showBookmarksSubmenu = true" @mouseleave="showBookmarksSubmenu = false">
          <Bookmark :size="16" class="menu-icon" aria-hidden="true" />
          <span>Bookmarks</span>
          <span class="menu-shortcut">⌘⇧O</span>
          <ChevronRight :size="14" class="menu-submenu-icon" aria-hidden="true" />
        </button>
        <button type="button" class="menu-item" @click="menuAction('openHistoryPanel')" @mouseenter="showHistorySubmenu = true" @mouseleave="showHistorySubmenu = false">
          <History :size="16" class="menu-icon" aria-hidden="true" />
          <span>History</span>
          <span class="menu-shortcut">⌘Y</span>
          <ChevronRight :size="14" class="menu-submenu-icon" aria-hidden="true" />
        </button>
        <!-- Bookmarks Submenu -->
        <div v-if="showBookmarksSubmenu" class="menu-submenu" @mouseenter="showBookmarksSubmenu = true" @mouseleave="showBookmarksSubmenu = false">
          <button type="button" class="menu-item" @click="menuAction('openBookmarksPanel')">
            <Bookmark :size="16" class="menu-icon" aria-hidden="true" />
            <span>Bookmark manager</span>
          </button>
          <div v-if="props.bookmarkFolders.length > 0" class="menu-divider" />
          <button
            v-for="folder in props.bookmarkFolders"
            :key="folder.name"
            type="button"
            class="menu-item"
            @click="emit('openBookmarksPanel')"
          >
            <Folder :size="16" class="menu-icon" aria-hidden="true" />
            <span>{{ folder.name }}</span>
            <span class="menu-item-count">{{ folder.count }}</span>
          </button>
        </div>
        <!-- History Submenu -->
        <div v-if="showHistorySubmenu" class="menu-submenu" @mouseenter="showHistorySubmenu = true" @mouseleave="showHistorySubmenu = false">
          <button type="button" class="menu-item" @click="menuAction('openHistoryPanel')">
            <History :size="16" class="menu-icon" aria-hidden="true" />
            <span>History manager</span>
          </button>
          <div v-if="props.recentHistory.length > 0" class="menu-divider" />
          <button
            v-for="item in props.recentHistory.slice(0, 8)"
            :key="item.url"
            type="button"
            class="menu-item"
            @click="emit('navigate', item.url)"
          >
            <Clock :size="16" class="menu-icon" aria-hidden="true" />
            <span class="menu-item-text">{{ item.title }}</span>
          </button>
        </div>
        <button type="button" class="menu-item" @click="menuAction('openDownloads')">
          <Download :size="16" class="menu-icon" aria-hidden="true" />
          <span>Downloads</span>
          <span class="menu-shortcut">⌘⇧J</span>
        </button>
        <div class="menu-divider" />
        <button type="button" class="menu-item" @click="menuAction('zoomIn')">
          <ZoomIn :size="16" class="menu-icon" aria-hidden="true" />
          <span>Zoom in</span>
          <span class="menu-shortcut">⌘+</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('zoomOut')">
          <ZoomOut :size="16" class="menu-icon" aria-hidden="true" />
          <span>Zoom out</span>
          <span class="menu-shortcut">⌘-</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('zoomReset')">
          <RotateCcw :size="16" class="menu-icon" aria-hidden="true" />
          <span>Reset zoom</span>
          <span class="menu-shortcut">⌘0</span>
        </button>
        <div class="menu-divider" />
        <button type="button" class="menu-item" @click="menuAction('print')">
          <Printer :size="16" class="menu-icon" aria-hidden="true" />
          <span>Print</span>
          <span class="menu-shortcut">⌘P</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('cast')">
          <Cast :size="16" class="menu-icon" aria-hidden="true" />
          <span>Cast</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('find')">
          <Search :size="16" class="menu-icon" aria-hidden="true" />
          <span>Find</span>
          <span class="menu-shortcut">⌘F</span>
        </button>
        <div class="menu-divider" />
        <button type="button" class="menu-item" @click="menuAction('moreTools')" @mouseenter="showMoreToolsSubmenu = true" @mouseleave="showMoreToolsSubmenu = false">
          <Wrench :size="16" class="menu-icon" aria-hidden="true" />
          <span>More tools</span>
          <ChevronRight :size="14" class="menu-submenu-icon" aria-hidden="true" />
        </button>
        <!-- More Tools Submenu -->
        <div v-if="showMoreToolsSubmenu" class="menu-submenu" @mouseenter="showMoreToolsSubmenu = true" @mouseleave="showMoreToolsSubmenu = false">
          <button type="button" class="menu-item" @click="menuAction('openSettings')">
            <Settings :size="16" class="menu-icon" aria-hidden="true" />
            <span>Settings</span>
          </button>
          <button type="button" class="menu-item" @click="menuAction('print')">
            <Printer :size="16" class="menu-icon" aria-hidden="true" />
            <span>Print</span>
          </button>
          <button type="button" class="menu-item" @click="menuAction('cast')">
            <Cast :size="16" class="menu-icon" aria-hidden="true" />
            <span>Cast</span>
          </button>
          <button type="button" class="menu-item" @click="menuAction('find')">
            <Search :size="16" class="menu-icon" aria-hidden="true" />
            <span>Find</span>
          </button>
        </div>
        <div class="menu-divider" />
        <button type="button" class="menu-item" @click="menuAction('openSettings')">
          <Settings :size="16" class="menu-icon" aria-hidden="true" />
          <span>Settings</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('help')">
          <HelpCircle :size="16" class="menu-icon" aria-hidden="true" />
          <span>Help</span>
        </button>
        <button type="button" class="menu-item" @click="menuAction('openProfile')" @mouseenter="showProfileSubmenu = true" @mouseleave="showProfileSubmenu = false">
          <User :size="16" class="menu-icon" aria-hidden="true" />
          <span>Profile</span>
          <ChevronRight :size="14" class="menu-submenu-icon" aria-hidden="true" />
        </button>
        <!-- Profile Submenu -->
        <div v-if="showProfileSubmenu" class="menu-submenu" @mouseenter="showProfileSubmenu = true" @mouseleave="showProfileSubmenu = false">
          <button type="button" class="menu-item" @click="menuAction('openSettings')">
            <Settings :size="16" class="menu-icon" aria-hidden="true" />
            <span>Settings</span>
          </button>
          <button type="button" class="menu-item" @click="menuAction('help')">
            <HelpCircle :size="16" class="menu-icon" aria-hidden="true" />
            <span>Help</span>
          </button>
        </div>
        <button type="button" class="menu-item" @click="menuAction('exit')">
          <LogOut :size="16" class="menu-icon" aria-hidden="true" />
          <span>Exit</span>
          <span class="menu-shortcut">⌘Q</span>
        </button>
      </div>
    </template>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — address bar with omnibox, shields, chrome menu, and sidebar shortcuts.
 */
import { ref, computed, watch } from 'vue';
import { useOmnibox } from '@/composables/useOmnibox';
import { OMNIBOX_INPUT_ID } from '$lib/browserShortcuts';
import { isChromeInternalUrl } from '$lib/chromeInternal';
import { isSecureUrl } from '$lib/favicon';
import { isNewTabUrl } from '$lib/newTabPage';
import { omniboxSuggestionTypeLabel } from '$lib/browserIntegrations';
import type { SidebarPanel } from '$lib/browserTypes';
import { startWindowDragFromMouseDown } from '$lib/windowDrag';
import ExtensionActionBar from '@/components/ExtensionActionBar.vue';
import {
  LayoutGrid,
  ChevronLeft,
  ChevronRight,
  RefreshCw,
  Home,
  Lock,
  LockOpen,
  Clock,
  Bookmark,
  Download,
  History,
  Settings,
  BookOpen,
  Globe,
  Printer,
  Shield,
  ChevronDown,
  Menu,
  Plus,
  Square,
  Star,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Cast,
  Search,
  Wrench,
  HelpCircle,
  LogOut,
  X,
  MessageSquare,
  MessageCircle,
  Database,
  PanelLeftOpen,
  User,
  Folder
} from '@lucide/vue';

/** Begin window drag when clicking toolbar blank space (not buttons/inputs). */
function onChromeMouseDown(e: MouseEvent): void {
  startWindowDragFromMouseDown(e);
}

const props = withDefaults(
  defineProps<{
    canGoBack?: boolean;
    canGoForward?: boolean;
    currentUrl?: string;
    isBookmarked?: boolean;
    downloadsBadge?: number;
    closedTabsCount?: number;
    showMenu?: boolean;
    sidebarOpen?: boolean;
    sidebarPanel?: SidebarPanel;
    shieldsCount?: number;
    shieldsEnabled?: boolean;
    siteAllowTrackers?: boolean;
    cdnStatusLabel?: string | null;
    announceSearchEnabled?: boolean;
    webchatUnread?: number;
    showWebChatView?: boolean;
    /** Bump to refresh extension toolbar icons after install/uninstall. */
    extensionsRefreshKey?: number;
    recentHistory?: Array<{ url: string; title: string }>;
    bookmarkFolders?: Array<{ name: string; count: number }>;
  }>(),
  {
    canGoBack: false,
    canGoForward: false,
    currentUrl: '',
    isBookmarked: false,
    downloadsBadge: 0,
    closedTabsCount: 0,
    showMenu: false,
    sidebarOpen: true,
    webchatUnread: 0,
    sidebarPanel: 'ai',
    shieldsCount: 0,
    shieldsEnabled: true,
    siteAllowTrackers: false,
    cdnStatusLabel: null,
    announceSearchEnabled: true,
    showWebChatView: false,
    extensionsRefreshKey: 0,
    recentHistory: () => [],
    bookmarkFolders: () => [],
  },
);

const emit = defineEmits<{
  goBack: [];
  goForward: [];
  reload: [];
  home: [];
  navigate: [url: string];
  urlInput: [value: string];
  toggleBookmark: [];
  toggleMenu: [];
  closeMenu: [];
  openDownloads: [];
  openPanel: [panel: SidebarPanel];
  toggleSidebar: [];
  toggleWebChat: [];
  openPocketPanel: [];
  openBookmarksPanel: [];
  openHistoryPanel: [];
  print: [];
  openSettings: [];
  openShields: [];
  toggleSiteShields: [];
  cdnBadgeClick: [];
  announceSearchResult: [url: string, title: string];
  openExtensions: [];
  extensionPopupClosed: [];
  newTab: [];
  newWindow: [];
  newIncognitoWindow: [];
  zoomIn: [];
  zoomOut: [];
  zoomReset: [];
  cast: [];
  find: [];
  menuOpened: [];
  menuClosed: [];
  moreTools: [];
  help: [];
  openProfile: [];
  exit: [];
}>();

const urlInput = ref(props.currentUrl);
const omniboxInputId = OMNIBOX_INPUT_ID;
const showBookmarksSubmenu = ref(false);
const showHistorySubmenu = ref(false);
const showMoreToolsSubmenu = ref(false);
const showProfileSubmenu = ref(false);

// Watch for menu open/close to notify parent
watch(() => props.showMenu, (isOpen) => {
  if (isOpen) {
    emit('menuOpened');
  } else {
    emit('menuClosed');
  }
});

function handleWebChatClick(): void {
  emit('toggleWebChat');
}

const {
  suggestions,
  showSuggestions,
  activeExtensionKeyword,
  searchResults,
  isSearching,
  showSearchResults,
  scheduleSuggestions,
  hideSuggestions,
  clearSearchResults,
  handleOmniboxSubmit,
} = useOmnibox();

function suggestionTypeLabel(type: string): string {
  return omniboxSuggestionTypeLabel(type);
}

const omniboxPlaceholder = computed(() => {
  if (activeExtensionKeyword.value) {
    return `Search ${activeExtensionKeyword.value.keyword}…`;
  }
  return 'Search or type a URL (/ask for memory)';
});

const shieldsTitle = computed(() => {
  if (props.siteAllowTrackers) {
    return 'Trackers allowed on this site (Shift+click to block again)';
  }
  if (props.shieldsEnabled) {
    return `Shields up · ${props.shieldsCount} blocked · Shift+click allow on this site`;
  }
  return 'Shields off · click Privacy · Shift+click per-site allow';
});

const bookmarked = computed(() => props.isBookmarked);

watch(
  () => props.currentUrl,
  (newValue) => {
    urlInput.value = newValue;
  },
);

function handleInput(): void {
  emit('urlInput', urlInput.value);
  scheduleSuggestions(urlInput.value);
}

function onOmniboxFocus(): void {
  scheduleSuggestions(urlInput.value);
}

function selectSuggestion(url: string): void {
  if (!url) return;
  urlInput.value = url;
  hideSuggestions();
  emit('navigate', url);
}

function selectSearchResult(url: string): void {
  if (!url) return;
  urlInput.value = url;
  clearSearchResults();
  hideSuggestions();
  emit('navigate', url);
}

/** Submit omnibox: `/ask` search, extension keyword, or navigation. */
async function submitNavigate(): Promise<void> {
  const raw = urlInput.value.trim();
  if (!raw) return;
  if (isChromeInternalUrl(raw)) {
    hideSuggestions();
    clearSearchResults();
    emit('navigate', raw);
    return;
  }
  const handled = await handleOmniboxSubmit(raw);
  if (handled === 'ask' || handled === 'extension') {
    return;
  }
  emit('navigate', raw);
}

function handleKeydown(e: KeyboardEvent): void {
  if (e.key === 'Enter') {
    e.preventDefault();
    void submitNavigate();
    return;
  }
  if (e.key === 'Escape') {
    hideSuggestions();
    emit('closeMenu');
  }
}

function onShieldsClick(e: MouseEvent): void {
  if (e.shiftKey) {
    emit('toggleSiteShields');
  } else {
    emit('openShields');
  }
}

type MenuAction =
  | 'newTab'
  | 'newWindow'
  | 'newIncognitoWindow'
  | 'toggleBookmark'
  | 'openBookmarksPanel'
  | 'openHistoryPanel'
  | 'openDownloads'
  | 'zoomIn'
  | 'zoomOut'
  | 'zoomReset'
  | 'print'
  | 'cast'
  | 'find'
  | 'moreTools'
  | 'openSettings'
  | 'help'
  | 'openProfile'
  | 'exit';

/** Close menu then emit the matching chrome menu action. */
function menuAction(action: MenuAction): void {
  console.log('=== menuAction called ===', action);
  emit('closeMenu');
  emit('menuClosed');
  switch (action) {
    case 'newTab':
      console.log('Emitting newTab');
      emit('newTab');
      break;
    case 'newWindow':
      console.log('Emitting newWindow');
      emit('newWindow');
      break;
    case 'newIncognitoWindow':
      console.log('Emitting newIncognitoWindow');
      emit('newIncognitoWindow');
      break;
    case 'toggleBookmark':
      emit('toggleBookmark');
      break;
    case 'openBookmarksPanel':
      emit('openBookmarksPanel');
      break;
    case 'openHistoryPanel':
      emit('openHistoryPanel');
      break;
    case 'openDownloads':
      emit('openDownloads');
      break;
    case 'zoomIn':
      emit('zoomIn');
      break;
    case 'zoomOut':
      emit('zoomOut');
      break;
    case 'zoomReset':
      emit('zoomReset');
      break;
    case 'print':
      emit('print');
      break;
    case 'cast':
      emit('cast');
      break;
    case 'find':
      emit('find');
      break;
    case 'moreTools':
      emit('moreTools');
      break;
    case 'openSettings':
      emit('openSettings');
      break;
    case 'help':
      emit('help');
      break;
    case 'openProfile':
      emit('openProfile');
      break;
    case 'exit':
      emit('exit');
      break;
  }
}
</script>

<style scoped>
.address-bar {
  display: flex;
  align-items: center;
  gap: var(--chrome-toolbar-gap, 4px);
  padding: var(--chrome-toolbar-padding-y, 4px) var(--chrome-toolbar-padding-x, 8px);
  min-height: var(--chrome-toolbar-min-height, 40px);
  background: var(--chrome-toolbar-bg, #dee1e6);
  border-top: 1px solid rgba(218, 220, 224, 0.01);
  border-bottom: 1px solid rgba(218, 220, 224, 0.03);
  position: relative;
}

.nav-controls {
  display: flex;
  gap: 2px;
}

.nav-icon-btn,
.toolbar-icon-btn,
.chrome-menu-btn {
  background: transparent;
  border: none;
  color: var(--chrome-tab-text, #202124);
  min-width: var(--chrome-icon-btn-size, 32px);
  width: var(--chrome-icon-btn-size, 32px);
  height: var(--chrome-icon-btn-size, 32px);
  border-radius: var(--chrome-icon-btn-radius, 50%);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--chrome-icon-btn-font-size, 16px);
  line-height: 1;
  position: relative;
  flex-shrink: 0;
  transition: background-color 0.15s ease;
}

.nav-svg,
.toolbar-svg {
  display: block;
  width: var(--chrome-icon-size, 16px);
  height: var(--chrome-icon-size, 16px);
  flex-shrink: 0;
}

.tab-icon-btn {
  margin-right: 4px;
}

.nav-icon-btn:hover:not(:disabled),
.toolbar-icon-btn:hover,
.chrome-menu-btn:hover {
  background: rgba(0, 0, 0, 0.08);
}

@media (prefers-color-scheme: dark) {
  .nav-icon-btn,
  .toolbar-icon-btn,
  .chrome-menu-btn {
    color: #e8eaed;
  }

  .nav-icon-btn:hover:not(:disabled),
  .toolbar-icon-btn:hover,
  .chrome-menu-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}

.nav-icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toolbar-icon-btn.bookmarked {
  color: #fbbc04;
}

.toolbar-icon-btn.active {
  background: rgba(0, 0, 0, 0.08);
  color: var(--color-primary, #1a73e8);
}

@media (prefers-color-scheme: dark) {
  .toolbar-icon-btn.active {
    background: rgba(255, 255, 255, 0.1);
  }
}

.shields-btn.shields-off {
  opacity: 0.5;
}

.toolbar-badge {
  position: absolute;
  top: -2px;
  right: -2px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  font-size: 10px;
  line-height: 16px;
  border-radius: 8px;
  background: #ea4335;
  color: #fff;
  font-weight: 600;
  border: 2px solid var(--chrome-toolbar-bg, #dee1e6);
}

@media (prefers-color-scheme: dark) {
  .toolbar-badge {
    border-color: var(--chrome-toolbar-bg, #35363a);
  }
}

.shields-badge {
  background: #34a853;
}

.url-input-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  background: var(--chrome-omnibox-bg, #ffffff);
  border: 1px solid rgba(218, 220, 224, 0.15);
  border-radius: var(--chrome-omnibox-radius, 24px);
  padding: 0 var(--chrome-omnibox-padding-x, 12px);
  min-height: var(--chrome-omnibox-height, 34px);
  max-height: var(--chrome-omnibox-height, 34px);
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}

.url-input-wrapper:focus-within {
  background: var(--chrome-omnibox-bg, #ffffff);
  border-color: var(--color-primary, #1a73e8);
  box-shadow: 0 1px 6px rgba(32, 33, 36, 0.28);
}

.site-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--chrome-omnibox-icon-size, 16px);
  height: var(--chrome-omnibox-icon-size, 16px);
  margin-right: 8px;
  flex-shrink: 0;
  color: var(--chrome-tab-text, #5f6368);
}

.site-indicator.secure {
  color: #188038;
}

.site-indicator-icon {
  width: var(--chrome-omnibox-icon-size, 16px);
  height: var(--chrome-omnibox-icon-size, 16px);
  display: block;
}

.site-indicator-icon--insecure {
  color: #e37400;
}

.url-input {
  flex: 1;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.8);
  font-size: var(--chrome-omnibox-font-size, 14px);
  font-weight: 500;
  outline: none;
  line-height: 1.4;
}

.url-input::placeholder {
  color: rgba(255, 255, 255, 0.6);
}

.url-input.has-site-indicator {
  padding-left: 0;
}

.omnibox-suggestions {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(100% + 4px);
  margin: 0;
  padding: 4px 0;
  list-style: none;
  background: var(--chrome-omnibox-bg, #ffffff);
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  max-height: 320px;
  overflow-y: auto;
}

.suggestion-row {
  padding: 6px 12px;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: var(--chrome-omnibox-font-size, 13px);
  color: var(--color-text-primary, #202124);
}

.suggestion-row:hover {
  background: var(--color-bg-hover, #e8eaed);
}

.suggestion-type {
  color: #9aa0a6;
  font-size: 11px;
  text-transform: capitalize;
}

.ext-keyword-hint {
  position: absolute;
  left: 16px;
  bottom: -18px;
  font-size: 11px;
  color: #8ab4f8;
  margin: 0;
}

.cdn-omnibox-badge {
  flex-shrink: 0;
  margin-left: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  border: 1px solid #6366f1;
  background: rgba(99, 102, 241, 0.2);
  color: #c7d2fe;
  font-size: 11px;
  cursor: pointer;
}

.url-input.has-cdn-badge {
  padding-right: 4px;
}

.search-results-dropdown {
  position: absolute;
  left: 0;
  right: 0;
  top: 100%;
  margin: 4px 0 0;
  padding: 4px 0;
  background: var(--color-bg-primary, #fff);
  border: 1px solid var(--color-border-tertiary, #e0e0e0);
  border-radius: var(--radius-md, 8px);
  box-shadow: var(--shadow-md, 0 2px 8px rgba(0, 0, 0, 0.12));
  z-index: 101;
  max-height: 320px;
  overflow-y: auto;
}

.search-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  color: var(--color-text-secondary, #5f6368);
  font-size: var(--chrome-omnibox-font-size, 13px);
}

.spinner-small {
  width: 14px;
  height: 14px;
  border: 2px solid #555;
  border-top-color: #8ab4f8;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.search-result-item {
  padding: 8px 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--color-border-tertiary, #e0e0e0);
}

.search-result-item:hover {
  background: var(--color-bg-hover, #e8eaed);
}

.search-result-item:last-child {
  border-bottom: none;
}

.result-title {
  font-size: var(--chrome-omnibox-font-size, 13px);
  color: var(--color-text-primary, #202124);
  margin-bottom: 2px;
}

.result-url {
  font-size: 11px;
  color: #9aa0a6;
  word-break: break-all;
}

.result-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.result-score {
  font-size: 11px;
  color: #22c55e;
}

.search-p2p-btn {
  padding: 2px 6px;
  font-size: 10px;
  border-radius: 4px;
  border: 1px solid #6366f1;
  background: transparent;
  color: #a5b4fc;
  cursor: pointer;
}

.search-empty {
  padding: 12px 16px;
  color: #888;
  font-size: 13px;
}

.toolbar-actions {
  display: flex;
  gap: 2px;
  align-items: center;
  min-width: 0;
  flex-shrink: 1;
}

.toolbar-end {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  margin-left: auto;
}

.chrome-menu-btn {
  font-size: 18px;
  letter-spacing: -1px;
}

.webchat-toggle-btn {
  position: relative;
  transition: all 0.2s ease;
}

.webchat-toggle-btn:hover {
  background: var(--color-bg-hover, #e8eaed);
}

.webchat-toggle-btn.active {
  background: var(--color-bg-active, #d3e3fd);
  color: #1a73e8;
}

.webchat-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  background: #ff4d4f;
  color: white;
  font-size: 10px;
  font-weight: 600;
  padding: 2px 5px;
  border-radius: 10px;
  min-width: 16px;
  text-align: center;
  line-height: 1;
  animation: badgePop 0.3s ease;
}

@keyframes badgePop {
  0% {
    transform: scale(0);
  }
  50% {
    transform: scale(1.2);
  }
  100% {
    transform: scale(1);
  }
}

@media (prefers-color-scheme: dark) {
  .webchat-toggle-btn:hover {
    background: #3c4043;
  }
  
  .webchat-toggle-btn.active {
    background: #1a73e8;
    color: white;
  }
}

.webview-block-overlay {
  position: fixed;
  inset: 0;
  z-index: 2147483644;
  background: rgba(0, 0, 0, 0.1);
  pointer-events: auto;
  cursor: default;
}

.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 2147483645;
  background: transparent;
  border: none;
  pointer-events: auto;
}

.chrome-menu-dropdown {
  position: fixed;
  right: 8px;
  top: 48px;
  min-width: 260px;
  max-height: calc(100vh - 60px);
  overflow-y: auto;
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  z-index: 2147483647;
  padding: 4px;
  animation: fadeIn 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .chrome-menu-dropdown {
    background: #2d2e30;
    border-color: #5f6368;
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  color: var(--chrome-tab-text-active, #202124);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}

.menu-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
}

.menu-shortcut {
  margin-left: auto;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 12px;
}

.menu-submenu-icon {
  margin-left: auto;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .menu-item {
    color: #e8eaed;
  }
  .menu-shortcut {
    color: #9aa0a6;
  }
  .menu-submenu-icon {
    color: #9aa0a6;
  }
}

.menu-item:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .menu-item:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
  }
}

.menu-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.menu-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--chrome-divider, #dadce0);
}

.menu-submenu {
  position: absolute;
  left: 100%;
  top: 0;
  margin-left: 4px;
  min-width: 240px;
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  z-index: 10000;
  padding: 4px;
  animation: fadeIn 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .menu-submenu {
    background: #2d2e30;
    border-color: #5f6368;
  }
}

.menu-item-count {
  margin-left: auto;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 12px;
  opacity: 0.7;
}

.menu-item-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}
</style>
