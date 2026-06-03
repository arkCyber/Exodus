<!--
  Exodus Browser — tab strip with pin, groups, close, and context menu.
-->
<template>
  <div
    class="tab-bar exodus-chrome-tabstrip chrome-drag-surface"
    :class="{
      vertical,
      'vertical-right': vertical && verticalRight,
      'vertical-in-sidebar': vertical && verticalInSidebar,
      'has-macos-titlebar-spacer': macTitlebarSpacer,
    }"
    :style="vertical ? `--vt-width: ${verticalWidth}px` : undefined"
    data-tauri-drag-region
    @mousedown="onTabBarMouseDown"
  >
    <div
      v-if="macTitlebarSpacer"
      class="mac-window-controls-spacer"
      data-tauri-drag-region
      aria-hidden="true"
    />
    <button
      v-for="tab in displayTabs"
      :key="tab.id"
      type="button"
      class="tab-item"
      :class="{
        active: tab.id === activeTabId,
        pinned: tab.pinned,
        'tab-is-new-tab': isNewTabPageTab(tab),
        'has-group': !!groupForTab(tabGroups, tab.id),
        'tab-dragging': dragTabId === tab.id,
        'tab-drag-over': dragOverTabId === tab.id,
      }"
      :style="groupStyle(tab.id)"
      :title="tabAccessibilityTitle(tab)"
      :aria-label="tabAccessibilityTitle(tab)"
      draggable="true"
      @click="emit('switchTab', tab.id)"
      @mousedown="(e) => emit('tabMouseDown', e, tab.id)"
      @contextmenu="(e) => emit('tabContextMenu', e, tab.id)"
      @dragstart="(e) => onTabDragStart(e, tab.id)"
      @dragover="(e) => onTabDragOver(e, tab.id)"
      @dragleave="() => onTabDragLeave(tab.id)"
      @drop="(e) => onTabDrop(e, tab.id)"
      @dragend="onTabDragEnd"
    >
      <img
        class="tab-favicon"
        :src="tabFaviconSrc(tab)"
        alt=""
        width="16"
        height="16"
        @error="(e) => onTabFaviconError(e, tab)"
      />
      <span v-if="showTabTitle(tab)" class="tab-title">{{ tabStripTitle(tab) }}</span>
      <span
        v-if="canShowTabClose(tab)"
        class="tab-close"
        role="button"
        tabindex="0"
        aria-label="Close tab"
        @click.stop="emit('closeTab', tab.id)"
        @keydown.enter.stop="emit('closeTab', tab.id)"
      >
        ×
      </span>
    </button>
    <button type="button" class="tab-new" title="New tab (⌘T)" @click="emit('newTab')">+</button>
    <div
      v-if="!vertical"
      class="tab-strip-drag-fill"
      data-tauri-drag-region
      aria-hidden="true"
      title="Drag window"
      @mousedown="onTabBarMouseDown"
    />
  </div>

  <template v-if="tabContextMenu">
    <button type="button" class="menu-backdrop" aria-label="Close" @click="emit('closeContextMenu')" />
    <div
      class="tab-context-menu"
      :style="{ left: `${tabContextMenu.x}px`, top: `${tabContextMenu.y}px` }"
    >
      <button type="button" class="menu-item" @click="emit('togglePin', tabContextMenu.tabId)">
        {{ tabs.find((t) => t.id === tabContextMenu?.tabId)?.pinned ? 'Unpin tab' : 'Pin tab' }}
      </button>
      <button type="button" class="menu-item" @click="emit('newTabGroup', tabContextMenu.tabId)">
        New tab group
      </button>
      <template v-if="tabGroups.length > 0">
        <button
          v-for="g in tabGroups"
          :key="g.id"
          type="button"
          class="menu-item menu-sub"
          @click="emit('addTabToGroup', tabContextMenu.tabId, g.id)"
        >
          Add to · {{ g.title }}
        </button>
      </template>
      <button
        v-if="groupForTab(tabGroups, tabContextMenu.tabId)"
        type="button"
        class="menu-item"
        @click="emit('removeTabFromGroup', tabContextMenu.tabId)"
      >
        Remove from group
      </button>
      <template v-if="groupForTab(tabGroups, tabContextMenu.tabId)">
        <button
          type="button"
          class="menu-item"
          @click="
            emit('toggleGroupCollapse', groupForTab(tabGroups, tabContextMenu.tabId)!.id, !groupForTab(tabGroups, tabContextMenu.tabId)!.collapsed)
          "
        >
          {{ groupForTab(tabGroups, tabContextMenu.tabId)!.collapsed ? 'Expand group' : 'Collapse group' }}
        </button>
        <button type="button" class="menu-item" @click="emit('renameTabGroup', groupForTab(tabGroups, tabContextMenu.tabId)!.id)">
          Rename group
        </button>
        <button type="button" class="menu-item" @click="emit('cycleTabGroupColor', groupForTab(tabGroups, tabContextMenu.tabId)!.id)">
          Change group color
        </button>
        <button type="button" class="menu-item" @click="emit('deleteTabGroup', groupForTab(tabGroups, tabContextMenu.tabId)!.id)">
          Delete group
        </button>
      </template>
      <button type="button" class="menu-item" @click="emit('duplicateTab', tabContextMenu.tabId)">
        Duplicate tab
      </button>
      <button type="button" class="menu-item" @click="emit('closeTab', tabContextMenu.tabId, true)">
        Close tab
      </button>
    </div>
  </template>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — tab strip with drag-reorder, pin, groups, and context menu.
 */
import { computed, ref } from 'vue';
import { isNewTabUrl } from '$lib/newTabPage';
import {
  TAB_FAVICON_FALLBACKS,
  tabAccessibilityLabel,
  tabFaviconDisplayUrl,
  tabStripLabel,
} from '$lib/favicon';
import { isMacTauriOverlayTitlebar } from '$lib/platformChrome';
import { canShowTabClose as chromeCanShowTabClose } from '$lib/tabStripChrome';
import { groupForTab, tabGroupColorCss, type TabGroup } from '$lib/tabGroups';
import { startWindowDragFromMouseDown } from '$lib/windowDrag';

/** Drag window from tab strip blank area (not tabs / new-tab button). */
function onTabBarMouseDown(e: MouseEvent): void {
  startWindowDragFromMouseDown(e);
}

interface BrowserTab {
  id: string;
  title: string;
  url: string;
  pinned?: boolean;
  favicon?: string | null;
}

const props = withDefaults(
  defineProps<{
    tabs: BrowserTab[];
    activeTabId: string | null;
    sortedTabs?: BrowserTab[];
    tabContextMenu?: { tabId: string; x: number; y: number } | null;
    tabGroups?: TabGroup[];
    vertical?: boolean;
    /** Vertical strip inside Firefox sidebar (full-width rows). */
    verticalInSidebar?: boolean;
    verticalWidth?: number;
    verticalRight?: boolean;
  }>(),
  {
    sortedTabs: () => [],
    tabContextMenu: null,
    tabGroups: () => [],
    vertical: false,
    verticalInSidebar: false,
    verticalWidth: 220,
    verticalRight: false,
  },
);

const emit = defineEmits<{
  switchTab: [id: string];
  newTab: [];
  closeTab: [id: string, force?: boolean];
  tabMouseDown: [e: MouseEvent, id: string];
  tabContextMenu: [e: MouseEvent, id: string];
  closeContextMenu: [];
  togglePin: [id: string];
  duplicateTab: [id: string];
  reorderTabs: [fromId: string, toId: string];
  newTabGroup: [tabId: string];
  addTabToGroup: [tabId: string, groupId: string];
  removeTabFromGroup: [tabId: string];
  toggleGroupCollapse: [groupId: string, collapsed: boolean];
  renameTabGroup: [groupId: string];
  cycleTabGroupColor: [groupId: string];
  deleteTabGroup: [groupId: string];
}>();

const dragTabId = ref<string | null>(null);
const dragOverTabId = ref<string | null>(null);

/** Reserve space left of tabs for macOS traffic lights (Tauri overlay title bar). */
const macTitlebarSpacer = !props.vertical && isMacTauriOverlayTitlebar();

/** Start dragging a tab (Chrome-style reorder). */
function onTabDragStart(e: DragEvent, tabId: string): void {
  dragTabId.value = tabId;
  dragOverTabId.value = null;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', tabId);
  }
}

/** Highlight drop target while dragging. */
function onTabDragOver(e: DragEvent, tabId: string): void {
  if (!dragTabId.value || dragTabId.value === tabId) return;
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
  dragOverTabId.value = tabId;
}

/** Clear drop highlight when pointer leaves a tab. */
function onTabDragLeave(tabId: string): void {
  if (dragOverTabId.value === tabId) dragOverTabId.value = null;
}

/** Commit tab reorder on drop. */
function onTabDrop(e: DragEvent, toId: string): void {
  e.preventDefault();
  const fromId = dragTabId.value ?? e.dataTransfer?.getData('text/plain');
  if (fromId && fromId !== toId) {
    emit('reorderTabs', fromId, toId);
  }
  dragTabId.value = null;
  dragOverTabId.value = null;
}

/** Reset drag state after drag ends. */
function onTabDragEnd(): void {
  dragTabId.value = null;
  dragOverTabId.value = null;
}

const displayTabs = computed(() =>
  props.sortedTabs.length > 0 ? props.sortedTabs : props.tabs,
);

/** Chrome: × on unpinned tabs; pinned tabs are icon-only. */
function canShowTabClose(tab: BrowserTab): boolean {
  return chromeCanShowTabClose(displayTabs.value.length, tab);
}

function isNewTabPageTab(tab: BrowserTab): boolean {
  return isNewTabUrl(tab.url);
}

/** Title text beside favicon (hidden for pinned horizontal tabs). */
function showTabTitle(tab: BrowserTab): boolean {
  if (props.vertical) return true;
  return !tab.pinned;
}

function tabStripTitle(tab: BrowserTab): string {
  return tabStripLabel(tab.title, tab.url);
}

/** Tooltip / aria text for a tab. */
function tabAccessibilityTitle(tab: BrowserTab): string {
  return `${tabAccessibilityLabel(tab.title, tab.url, tab.pinned)} · Right-click for menu`;
}

/** Resolved favicon for tab strip (always returns a URL). */
function tabFaviconSrc(tab: BrowserTab): string {
  return tabFaviconDisplayUrl(tab.url, tab.favicon);
}

/** Fallback when remote favicon fails to load. */
function onTabFaviconError(e: Event, tab: BrowserTab): void {
  const img = e.target as HTMLImageElement | null;
  if (!img || img.dataset.fallbackApplied === '1') return;
  img.dataset.fallbackApplied = '1';
  img.src = isNewTabPageTab(tab) ? TAB_FAVICON_FALLBACKS.newTab : TAB_FAVICON_FALLBACKS.generic;
}

function groupStyle(tabId: string): Record<string, string> | undefined {
  const grp = groupForTab(props.tabGroups, tabId);
  if (!grp) return undefined;
  return { '--tab-group-color': tabGroupColorCss(grp.color) };
}
</script>

<style scoped>
.tab-strip-drag-fill {
  flex: 1 1 48px;
  min-width: 24px;
  min-height: var(--chrome-tab-height, 34px);
  align-self: stretch;
  -webkit-app-region: drag;
  app-region: drag;
  cursor: default;
}

.tab-bar {
  display: flex;
  align-items: flex-end;
  gap: 0;
  padding: 0 8px 0 0;
  min-height: var(--chrome-tab-height, 34px);
  background: var(--chrome-tab-bg, #dee1e6);
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
  overflow-x: auto;
  overflow-y: hidden;
  flex-shrink: 0;
}

.tab-bar.has-macos-titlebar-spacer {
  padding-left: 0;
}

.tab-bar::-webkit-scrollbar {
  height: 0;
  width: 0;
}

.tab-bar.vertical {
  flex-direction: column;
  align-items: stretch;
  width: var(--vt-width, 220px);
  min-width: var(--vt-width, 220px);
  max-width: var(--vt-width, 220px);
  height: 100%;
  padding: 8px 6px;
  overflow-x: hidden;
  overflow-y: auto;
  border-bottom: none;
  border-right: 1px solid #404040;
}

/* Horizontal strip: Chrome icon + title + close */
.tab-bar:not(.vertical) .tab-item {
  position: relative;
  min-width: 72px;
  max-width: var(--chrome-tab-max-width, 240px);
  flex-shrink: 1;
  padding: 0 6px 0 10px;
  gap: 8px;
  justify-content: flex-start;
}

.tab-bar:not(.vertical) .tab-item.active {
  max-width: 260px;
  flex-shrink: 0;
}

.tab-bar:not(.vertical) .tab-item .tab-favicon {
  flex-shrink: 0;
}

.tab-bar:not(.vertical) .tab-item .tab-close {
  position: static;
  transform: none;
  margin-left: 2px;
  width: 18px;
  height: 18px;
  font-size: 15px;
  opacity: 0.65;
}

.tab-bar:not(.vertical) .tab-item:hover .tab-close,
.tab-bar:not(.vertical) .tab-item.active .tab-close {
  opacity: 1;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  max-width: var(--chrome-tab-max-width, 240px);
  flex-shrink: 1;
  height: 34px;
  padding: 0 12px;
  background: transparent;
  border: none;
  border-radius: 8px 8px 0 0;
  color: var(--chrome-tab-text, #5f6368);
  cursor: pointer;
  font-size: var(--chrome-tab-font-size, 12px);
  position: relative;
  transition: background-color 0.15s ease;
}

.tab-item:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .tab-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

.tab-item.active {
  background: var(--chrome-tab-bg-active, #ffffff);
  color: var(--chrome-tab-text-active, #202124);
  flex-shrink: 0;
}

.tab-bar.vertical .tab-item.active {
  max-width: 260px;
}

.tab-item.active::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--color-primary, #1a73e8);
  border-radius: 2px 2px 0 0;
}

.tab-item.tab-dragging {
  opacity: 0.55;
}

.tab-item.tab-drag-over {
  box-shadow: inset 0 -2px 0 var(--color-primary, #1a73e8);
}

.tab-item.pinned {
  flex-shrink: 0;
  justify-content: center;
}

.tab-bar:not(.vertical) .tab-item.pinned {
  min-width: 40px;
  max-width: 40px;
  padding: 0 8px;
  gap: 0;
}

.tab-bar.vertical .tab-item.pinned {
  min-width: 36px;
  max-width: 36px;
  padding: 0;
}

.tab-item.has-group {
  padding-left: 14px;
}

.tab-item.has-group::after {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 3px;
  background: var(--tab-group-color, #6b7280);
  border-radius: 0 2px 2px 0;
}

.tab-favicon {
  border-radius: 2px;
  flex-shrink: 0;
  width: 16px;
  height: 16px;
}

.tab-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
  font-weight: 500;
}

.tab-close {
  background: transparent;
  border: none;
  color: var(--chrome-tab-text, #5f6368);
  width: 20px;
  height: 20px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  line-height: 1;
  opacity: 0;
  transition: opacity 0.15s ease, background-color 0.15s ease;
  flex-shrink: 0;
}

.tab-item:hover .tab-close {
  opacity: 1;
}

.tab-close:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #ea4335;
}

@media (prefers-color-scheme: dark) {
  .tab-close:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}

.tab-new {
  background: transparent;
  border: none;
  color: var(--chrome-tab-text, #5f6368);
  width: 32px;
  height: 32px;
  border-radius: 50%;
  cursor: pointer;
  flex-shrink: 0;
  font-size: 20px;
  line-height: 1;
  margin-bottom: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

.tab-new:hover {
  background: rgba(0, 0, 0, 0.08);
}

@media (prefers-color-scheme: dark) {
  .tab-new:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}

.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
  background: transparent;
  border: none;
}

.tab-context-menu {
  position: fixed;
  z-index: 3000;
  min-width: 180px;
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  padding: 4px;
}

@media (prefers-color-scheme: dark) {
  .tab-context-menu {
    background: #2d2e30;
    border-color: #5f6368;
  }
}

.menu-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 12px;
  background: transparent;
  border: none;
  color: var(--chrome-tab-text-active, #202124);
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  transition: background-color 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .menu-item {
    color: #e8eaed;
  }
}

.menu-item:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .menu-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

.menu-sub {
  padding-left: 24px;
  font-size: 12px;
}
</style>
