<!--
  Exodus Browser — right sidebar (AI, memory, bookmarks, Pocket, P2P, agent).
-->
<template>
  <aside
    v-if="open"
    class="ai-sidebar exodus-sidebar exodus-sidebar--firefox"
    :class="{
      'exodus-sidebar--collapsed': contentCollapsed,
      'exodus-sidebar--left': sidebarPosition === 'left',
    }"
    :style="sidebarStyle"
    aria-label="Exodus sidebar"
  >
    <div
      v-if="!contentCollapsed"
      class="sidebar-resize-handle"
      :class="{ 'resize-handle-left': sidebarPosition === 'left' }"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize sidebar"
      @mousedown.prevent="onResizeStart"
    />

    <div class="sidebar-icon-bar" aria-label="Sidebar tools">
      <div class="icon-list">
        <button
          v-for="item in iconItems"
          :key="item.panel"
          type="button"
          class="sidebar-icon-btn"
          :class="{ active: sidebarPanel === item.panel && !agentPanelOpen }"
          :title="item.title"
          :aria-label="item.title"
          :aria-current="sidebarPanel === item.panel && !agentPanelOpen ? 'page' : undefined"
          @click="onIconPanelClick(item.panel)"
        >
          <svg class="icon-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              :d="sidebarIconPath(item.icon)"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </div>
      <div class="icon-footer">
        <button
          type="button"
          class="sidebar-icon-btn"
          title="Customize sidebar"
          aria-label="Customize sidebar"
          :class="{ active: sidebarPanel === 'customize' && !agentPanelOpen }"
          @click="onIconPanelClick('customize')"
        >
          <svg class="icon-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              :d="sidebarIconPath('customize')"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="sidebar-icon-btn"
          :title="contentCollapsed ? 'Expand panel' : 'Collapse panel'"
          :aria-label="contentCollapsed ? 'Expand sidebar panel' : 'Collapse sidebar panel'"
          @click="toggleContentCollapsed"
        >
          <svg class="icon-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              :d="sidebarIconPath(contentCollapsed ? 'expand' : 'collapse')"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="sidebar-icon-btn close-btn"
          aria-label="Close sidebar"
          title="Close sidebar"
          @click="emit('close')"
        >
          <svg class="icon-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              :d="sidebarIconPath('close')"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      </div>
    </div>

    <div v-show="!contentCollapsed" class="sidebar-content">
      <div class="sidebar-header">
        <h3>{{ panelTitle }}</h3>
      </div>
      <div class="sidebar-body">
        <SidebarVerticalTabsPanel
          v-if="sidebarPanel === 'tabs'"
          :tabs="browserTabs"
          :active-tab-id="activeTabId"
          :sorted-tabs="sortedTabs"
          :strip-width="verticalTabStripWidth"
          :tab-context-menu="tabContextMenu"
          :tab-groups="tabGroups"
          :tab-bar-handlers="tabBarHandlers"
        />

        <SidebarMemoryPanel
          v-else-if="sidebarPanel === 'memory'"
          :indexed-memory-groups="indexedMemoryGroups"
          :history-groups="historyGroups"
          :indexed-count="indexedCount"
          :history-count="historyCount"
          :memory-search-query="memorySearchQuery"
          @navigate="(url) => emit('navigate', url)"
          @load-memory="emit('load-memory')"
          @remove-indexed="(id) => emit('remove-indexed', id)"
          @clear-indexed="emit('clear-indexed')"
          @clear-history="emit('clear-history')"
          @memory-search="(q) => emit('memory-search', q)"
        />

        <SidebarBookmarksPanel
          v-else-if="sidebarPanel === 'bookmarks'"
          :bookmarks="bookmarks"
          :reorder-enabled="true"
          :ui-locale="uiLocale"
          @navigate="(url) => emit('navigate', url)"
          @refresh="emit('load-bookmarks')"
          @add-bookmark="emit('add-bookmark')"
          @remove-bookmark="(id) => emit('remove-bookmark', id)"
          @edit-bookmark="(bookmark) => emit('edit-bookmark', bookmark)"
          @open-in-new-tab="(url, title) => emit('open-in-new-tab', url, title)"
          @reorder="(orderedIds) => emit('reorder-bookmarks', orderedIds)"
        />

        <P2pSidebarPanel
          v-else-if="sidebarPanel === 'p2p'"
          :room-id="p2pRoomId"
          @status="(msg) => emit('p2p-status', msg)"
        />

        <SidebarSyncedTabsPanel
          v-else-if="sidebarPanel === 'synced'"
          :open-tabs="openTabsForSync"
          @navigate="(url) => emit('navigate', url)"
        />

        <SidebarReadingListPanel
          v-else-if="sidebarPanel === 'reading'"
          @navigate="(url) => emit('navigate', url)"
        />

        <PocketPanel v-else-if="sidebarPanel === 'pocket'" @status="(msg) => emit('p2p-status', msg)" />

        <SidebarCustomizePanel
          v-else-if="sidebarPanel === 'customize'"
          :prefs="sidebarPrefs"
          @position-change="(p) => emit('sidebar-position-change', p)"
          @vertical-tabs-change="(v) => emit('vertical-tabs-in-sidebar-change', v)"
          @toggle-tool="(t) => emit('toggle-sidebar-tool', t)"
        />

        <AgentPanel
          v-else-if="agentPanelOpen"
          :command="agentCommand"
          :log="agentLog"
          :executing="isAgentExecuting"
          :dom-summary="agentDomSummary"
          @execute="emit('agent-execute')"
          @compress="emit('agent-compress')"
          @back="emit('agent-back')"
          @preset="(json) => emit('agent-preset', json)"
          @command-change="(v) => emit('agent-command-change', v)"
          @ask-ai="emit('agent-ask-ai')"
          @run-strategy="(id) => emit('agent-run-strategy', id)"
          @strategy-saved="(msg) => emit('p2p-status', msg)"
        />

        <SidebarAiPanel
          v-else
          :ai-chat-history="aiChatHistory"
          :chat-stream-buffer="chatStreamBuffer"
          :ai-stream-mode="aiStreamMode"
          :is-loading="isLoading"
          :ai-online="aiOnline"
          :ai-chat-input="aiChatInput"
          :can-announce-page="canAnnouncePage"
          @navigate="(url) => emit('navigate', url)"
          @send-chat="emit('send-chat')"
          @cancel-chat="emit('cancel-chat')"
          @toggle-agent="emit('toggle-agent')"
          @open-p2p="emit('open-panel', 'p2p')"
          @chat-input="(v) => emit('chat-input', v)"
        />
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue';
import type { AiChatMessage, BookmarkItem, SidebarPanel } from '$lib/browserTypes';
import type { HistoryGroup } from '$lib/historyGroups';
import { sidebarIconPath, type SidebarIconItem } from '$lib/sidebarIcons';
import type { SidebarPreferences } from '$lib/sidebarPreferences';
import type { BrowserTab } from '$lib/browserTypes';
import type { TabGroup } from '$lib/tabGroups';
import type { OpenTabSnapshot } from '$lib/syncedTabs';
import {
  loadSidebarContentCollapsed,
  loadSidebarContentWidth,
  saveSidebarContentCollapsed,
  saveSidebarContentWidth,
  SIDEBAR_CONTENT_MIN_PX,
  sidebarContentMaxPx,
} from '$lib/sidebarLayout';
import AgentPanel from '@/components/AgentPanel.vue';
import PocketPanel from '@/components/PocketPanel.vue';
import P2pSidebarPanel from '@/components/P2pSidebarPanel.vue';
import SidebarAiPanel from '@/components/sidebar/SidebarAiPanel.vue';
import SidebarMemoryPanel from '@/components/sidebar/SidebarMemoryPanel.vue';
import SidebarBookmarksPanel from '@/components/sidebar/SidebarBookmarksPanel.vue';
import SidebarVerticalTabsPanel from '@/components/sidebar/SidebarVerticalTabsPanel.vue';
import SidebarSyncedTabsPanel from '@/components/sidebar/SidebarSyncedTabsPanel.vue';
import SidebarReadingListPanel from '@/components/sidebar/SidebarReadingListPanel.vue';
import SidebarCustomizePanel from '@/components/sidebar/SidebarCustomizePanel.vue';

const props = withDefaults(
  defineProps<{
  open: boolean;
  sidebarPanel: SidebarPanel;
  sidebarPosition?: 'left' | 'right';
  iconItems: SidebarIconItem[];
  sidebarPrefs: SidebarPreferences;
  browserTabs?: BrowserTab[];
  activeTabId?: string | null;
  sortedTabs?: BrowserTab[];
  verticalTabStripWidth?: number;
  tabContextMenu?: { tabId: string; x: number; y: number } | null;
  tabGroups?: TabGroup[];
  tabBarHandlers?: import('@/composables/useBrowserTabBarHandlers').TabBarHandlerMap;
  openTabsForSync?: OpenTabSnapshot[];
  agentPanelOpen: boolean;
  aiChatHistory: AiChatMessage[];
  chatStreamBuffer: string;
  aiStreamMode: 'none' | 'chat' | 'summary';
  isLoading: boolean;
  aiOnline: boolean;
  aiChatInput: string;
  agentCommand: string;
  agentLog: string[];
  agentDomSummary: string;
  isAgentExecuting: boolean;
  indexedMemoryGroups: HistoryGroup[];
  historyGroups: HistoryGroup[];
  indexedCount: number;
  historyCount: number;
  bookmarks: BookmarkItem[];
  p2pRoomId: string;
  canAnnouncePage: boolean;
  memorySearchQuery?: string;
  bookmarkSearchQuery?: string;
}>(),
  {
    sidebarPosition: 'right',
    browserTabs: () => [],
    activeTabId: null,
    sortedTabs: () => [],
    verticalTabStripWidth: 280,
    tabContextMenu: null,
    tabGroups: () => [],
    tabBarHandlers: undefined,
    openTabsForSync: () => [],
  },
);

const emit = defineEmits<{
  close: [];
  navigate: [url: string];
  'open-panel': [panel: SidebarPanel];
  'send-chat': [];
  'cancel-chat': [];
  'toggle-agent': [];
  'chat-input': [value: string];
  'load-memory': [];
  'remove-indexed': [id: string];
  'clear-indexed': [];
  'clear-history': [];
  'load-bookmarks': [];
  'remove-bookmark': [id: string];
  'update-bookmark-folder': [id: string, folder: string];
  'bookmark-search': [query: string];
  'memory-search': [query: string];
  'sidebar-position-change': [position: 'left' | 'right'];
  'vertical-tabs-in-sidebar-change': [enabled: boolean];
  'toggle-sidebar-tool': [tool: import('$lib/sidebarPreferences').SidebarToolId];
  'p2p-status': [message: string];
  'agent-execute': [];
  'agent-compress': [];
  'agent-back': [];
  'agent-preset': [actionJson: string];
  'agent-command-change': [value: string];
  'agent-ask-ai': [];
  'agent-run-strategy': [templateId: string];
}>();

const bookmarkSearchLocal = ref(props.bookmarkSearchQuery ?? '');

watch(
  () => props.bookmarkSearchQuery,
  (q) => {
    if (q !== undefined && bookmarkSearchLocal.value !== q) bookmarkSearchLocal.value = q;
  },
);
const contentWidthPx = ref(loadSidebarContentWidth());
const contentCollapsed = ref(loadSidebarContentCollapsed());

const iconItems = computed(() => props.iconItems);

const sidebarStyle = computed(() => {
  if (contentCollapsed.value) {
    return { width: 'var(--chrome-sidebar-icon-rail, 48px)' };
  }
  return {
    width: `calc(var(--chrome-sidebar-icon-rail, 48px) + ${contentWidthPx.value}px)`,
    '--sidebar-content-width': `${contentWidthPx.value}px`,
  };
});

/** Open panel; expand content if collapsed (Firefox icon rail). */
function onIconPanelClick(panel: SidebarPanel): void {
  if (contentCollapsed.value) {
    contentCollapsed.value = false;
    saveSidebarContentCollapsed(false);
  }
  emit('open-panel', panel);
}

/** Firefox-style collapse: icon rail only. */
function toggleContentCollapsed(): void {
  contentCollapsed.value = !contentCollapsed.value;
  saveSidebarContentCollapsed(contentCollapsed.value);
}

let resizeActive = false;

/** Drag left edge to resize content panel (Firefox sidebar). */
function onResizeStart(e: MouseEvent): void {
  resizeActive = true;
  const startX = e.clientX;
  const startWidth = contentWidthPx.value;
  const maxW = sidebarContentMaxPx();

  const onMove = (ev: MouseEvent): void => {
    if (!resizeActive) return;
    const delta =
      props.sidebarPosition === 'left' ? ev.clientX - startX : startX - ev.clientX;
    const next = Math.min(maxW, Math.max(SIDEBAR_CONTENT_MIN_PX, startWidth + delta));
    contentWidthPx.value = next;
  };

  const onUp = (): void => {
    resizeActive = false;
    saveSidebarContentWidth(contentWidthPx.value);
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };

  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

onUnmounted(() => {
  resizeActive = false;
});

const panelTitle = computed(() => {
  if (props.agentPanelOpen) return 'Agent';
  switch (props.sidebarPanel) {
    case 'tabs':
      return 'Tabs';
    case 'ai':
      return 'AI Chat';
    case 'memory':
      return 'History';
    case 'bookmarks':
      return 'Bookmarks';
    case 'synced':
      return 'Synced tabs';
    case 'reading':
      return 'Reading list';
    case 'pocket':
      return 'Pocket';
    case 'p2p':
      return 'P2P CDN';
    case 'customize':
      return 'Customize sidebar';
    default:
      return 'Sidebar';
  }
});
</script>

<style scoped>
.ai-sidebar {
  position: relative;
  display: flex;
  flex-shrink: 0;
  border-left: 1px solid var(--chrome-divider, #dadce0);
  background: var(--chrome-toolbar-bg, #dee1e6);
  height: 100%;
  transition: width 0.2s ease;
}

.exodus-sidebar--collapsed .sidebar-content {
  display: none;
}

.sidebar-resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 5px;
  cursor: col-resize;
  z-index: 2;
}

.sidebar-resize-handle:hover {
  background: rgba(26, 115, 232, 0.25);
}

.exodus-sidebar--left .sidebar-resize-handle {
  left: auto;
  right: 0;
}

.resize-handle-left {
  left: auto;
  right: 0;
}

@media (prefers-color-scheme: dark) {
  .ai-sidebar {
    background: #2d2e30;
    border-color: #5f6368;
  }
}

.sidebar-icon-bar {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  width: var(--chrome-sidebar-icon-rail, 48px);
  flex-shrink: 0;
  padding: 8px 4px;
  border-right: 1px solid var(--chrome-divider, #dadce0);
  background: rgba(0, 0, 0, 0.04);
}

@media (prefers-color-scheme: dark) {
  .sidebar-icon-bar {
    background: rgba(0, 0, 0, 0.1);
    border-color: #5f6368;
  }
}

.icon-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-icon-btn {
  width: var(--chrome-sidebar-icon-size, 36px);
  height: var(--chrome-sidebar-icon-size, 36px);
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .sidebar-icon-btn {
    color: #9aa0a6;
  }
}

.sidebar-icon-btn:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .sidebar-icon-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}

/* Active icon styles: see sidebar-ui.css (.exodus-sidebar--firefox) */

.sidebar-content {
  width: var(--sidebar-content-width, var(--chrome-sidebar-content-width, 320px));
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--chrome-tab-bg-active, #ffffff);
}

.icon-svg {
  display: block;
}

@media (prefers-color-scheme: dark) {
  .sidebar-content {
    background: #292a2d;
  }
}

.sidebar-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

@media (prefers-color-scheme: dark) {
  .sidebar-header {
    border-color: #5f6368;
  }
}

.sidebar-header h3 {
  margin: 0;
  font-size: 15px;
  color: var(--chrome-tab-text-active, #202124);
}

@media (prefers-color-scheme: dark) {
  .sidebar-header h3 {
    color: #e8eaed;
  }
}

.sidebar-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  min-height: 0;
}

.list-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.search-input,
.folder-input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  background: var(--chrome-omnibox-bg, #ffffff);
  color: var(--chrome-tab-text-active, #202124);
  font-size: 13px;
  outline: none;
}

@media (prefers-color-scheme: dark) {
  .search-input,
  .folder-input {
    background: #292a2d;
    border-color: #5f6368;
    color: #e8eaed;
  }
}

.search-input:focus,
.folder-input:focus {
  border-color: var(--color-primary, #1a73e8);
  box-shadow: 0 1px 6px rgba(32, 33, 36, 0.28);
}

.list-item.row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  padding: 8px;
  border-radius: 6px;
  transition: background-color 0.15s ease;
}

.list-item.row:hover {
  background: rgba(0, 0, 0, 0.04);
}

@media (prefers-color-scheme: dark) {
  .list-item.row:hover {
    background: rgba(255, 255, 255, 0.06);
  }
}

.list-grow {
  flex: 1;
  cursor: pointer;
  min-width: 0;
}

.list-title {
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
}

@media (prefers-color-scheme: dark) {
  .list-title {
    color: #e8eaed;
  }
}

.list-sub {
  font-size: 11px;
  color: var(--chrome-tab-text, #5f6368);
  word-break: break-all;
}

@media (prefers-color-scheme: dark) {
  .list-sub {
    color: #9aa0a6;
  }
}

.tab-close {
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 18px;
}

.muted {
  color: #888;
  font-size: 12px;
}

.nav-button.secondary {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid #555;
  background: #333;
  color: #e0e0e0;
  cursor: pointer;
}

.full {
  width: 100%;
}
</style>
