<!--
  Exodus Browser — Chrome-style bookmarks sidebar panel.
  Features: folder grouping, search, drag-reorder, context menu, favicon display.
-->
<template>
  <div class="list-panel bookmarks-panel">
    <!-- Search input -->
    <div class="panel-header">
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="ui.searchPlaceholder"
        @input="onSearchInput"
      />
      <button
        type="button"
        class="nav-button secondary"
        :title="ui.refresh"
        @click="emit('refresh')"
      >
        <svg class="nav-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M13 8A5 5 0 1 0 8 13" stroke-linecap="round" />
          <path d="M13 8l-3-3M13 8l-3 3" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <!-- Empty state -->
    <div v-if="filteredBookmarks.length === 0" class="empty-state">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <p class="empty-text">{{ ui.emptyBookmarks }}</p>
      <button
        type="button"
        class="nav-button primary"
        @click="emit('add-bookmark')"
      >
        {{ ui.addBookmark }}
      </button>
    </div>

    <!-- Bookmarks grouped by folder -->
    <template v-else>
      <!-- Bookmarks without folder (bar bookmarks) -->
      <template v-if="bookmarksByFolder[''] && bookmarksByFolder[''].length > 0">
        <div class="folder-section">
          <div class="folder-header">
            <svg class="folder-icon" viewBox="0 0 16 16" fill="currentColor">
              <path d="M2 4h5l1-1h6a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z" />
            </svg>
            <span class="folder-name">{{ ui.bookmarksBar }}</span>
          </div>
          <div
            v-for="bookmark in bookmarksByFolder['']"
            :key="bookmark.id"
            class="bookmark-item"
            :class="{ 'bookmark-item--drag-over': dragOverId === bookmark.id }"
            :draggable="reorderEnabled"
            @click="onBookmarkClick(bookmark)"
            @contextmenu.prevent="onContextMenu(bookmark, $event)"
            @dragstart="(e) => onDragStart(bookmark.id, e)"
            @dragenter="onDragEnter"
            @dragover="onDragOver"
            @dragleave="onDragLeave"
            @drop="(e) => onDrop(bookmark.id, e)"
            @dragend="onDragEnd"
          >
            <img
              v-if="faviconFor(bookmark)"
              class="bookmark-favicon"
              :src="faviconFor(bookmark)"
              alt=""
              width="16"
              height="16"
            />
            <span v-else class="bookmark-favicon bookmark-favicon--fallback">
              <svg viewBox="0 0 16 16" fill="none">
                <circle cx="8" cy="8" r="6.5" stroke="currentColor" stroke-width="1.2" />
                <path d="M2.5 8h11M8 2.5v11" stroke="currentColor" stroke-width="1" opacity="0.55" />
              </svg>
            </span>
            <div class="bookmark-info">
              <div class="bookmark-title">{{ bookmark.title || bookmark.url }}</div>
              <div class="bookmark-url">{{ bookmark.url }}</div>
            </div>
            <button
              type="button"
              class="bookmark-action"
              :title="ui.remove"
              @click.stop="onRemoveBookmark(bookmark.id)"
            >
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M12 4L4 12M4 4l8 8" stroke-linecap="round" />
              </svg>
            </button>
          </div>
        </div>
      </template>

      <!-- Bookmarks in folders -->
      <template v-for="folder in sortedFolderNames" :key="folder">
        <div class="folder-section">
          <div class="folder-header" @click="toggleFolder(folder)">
            <svg class="folder-icon" viewBox="0 0 16 16" fill="currentColor">
              <path d="M2 4h5l1-1h6a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z" />
            </svg>
            <span class="folder-name">{{ folder }}</span>
            <svg
              class="folder-chevron"
              :class="{ 'folder-chevron--open': openFolders.has(folder) }"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
            >
              <path d="M4 6l4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
          <div v-if="openFolders.has(folder)" class="folder-content">
            <div
              v-for="bookmark in bookmarksByFolder[folder]"
              :key="bookmark.id"
              class="bookmark-item bookmark-item--nested"
              :class="{ 'bookmark-item--drag-over': dragOverId === bookmark.id }"
              :draggable="reorderEnabled"
              @click="onBookmarkClick(bookmark)"
              @contextmenu.prevent="onContextMenu(bookmark, $event)"
              @dragstart="(e) => onDragStart(bookmark.id, e)"
              @dragenter="onDragEnter"
              @dragover="onDragOver"
              @dragleave="onDragLeave"
              @drop="(e) => onDrop(bookmark.id, e)"
              @dragend="onDragEnd"
            >
              <img
                v-if="faviconFor(bookmark)"
                class="bookmark-favicon"
                :src="faviconFor(bookmark)"
                alt=""
                width="16"
                height="16"
              />
              <span v-else class="bookmark-favicon bookmark-favicon--fallback">
                <svg viewBox="0 0 16 16" fill="none">
                  <circle cx="8" cy="8" r="6.5" stroke="currentColor" stroke-width="1.2" />
                  <path d="M2.5 8h11M8 2.5v11" stroke="currentColor" stroke-width="1" opacity="0.55" />
                </svg>
              </span>
              <div class="bookmark-info">
                <div class="bookmark-title">{{ bookmark.title || bookmark.url }}</div>
                <div class="bookmark-url">{{ bookmark.url }}</div>
              </div>
              <button
                type="button"
                class="bookmark-action"
                :title="ui.remove"
                @click.stop="onRemoveBookmark(bookmark.id)"
              >
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M12 4L4 12M4 4l8 8" stroke-linecap="round" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </template>
    </template>

    <!-- Context menu -->
    <Teleport to="body">
      <template v-if="contextMenuBookmark">
        <button
          type="button"
          class="menu-backdrop"
          @click="closeContextMenu"
        />
        <div
          class="context-menu bookmark-dropdown"
          role="menu"
          :style="contextMenuStyle"
        >
          <button type="button" class="context-menu-item" role="menuitem" @click="contextOpen">
            {{ ui.contextOpen }}
          </button>
          <button type="button" class="context-menu-item" role="menuitem" @click="contextOpenInNewTab">
            {{ ui.contextOpenInNewTab }}
          </button>
          <button type="button" class="context-menu-item" role="menuitem" @click="contextEdit">
            {{ ui.contextEdit }}
          </button>
          <div class="context-menu-divider" role="separator" />
          <button type="button" class="context-menu-item context-menu-item--danger" role="menuitem" @click="contextDelete">
            {{ ui.contextDelete }}
          </button>
        </div>
      </template>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { BookmarkItem } from '@/lib/browserTypes';
import { isReservedBookmarkGroupName } from '@/lib/bookmarkGroups';
import { faviconUrlFor } from '@/lib/favicon';
import { resolveBookmarkBarLocale, type BookmarkBarLocale } from '@/lib/bookmarkBarUi';

interface Props {
  bookmarks?: BookmarkItem[];
  reorderEnabled?: boolean;
  uiLocale?: BookmarkBarLocale;
}

const props = withDefaults(defineProps<Props>(), {
  bookmarks: () => [],
  reorderEnabled: false,
  uiLocale: undefined,
});

const emit = defineEmits<{
  navigate: [url: string];
  refresh: [];
  addBookmark: [];
  removeBookmark: [id: string];
  editBookmark: [bookmark: BookmarkItem];
  openInNewTab: [url: string, title?: string];
  reorder: [orderedIds: string[]];
}>();

const searchQuery = ref('');
const openFolders = ref<Set<string>>(new Set());
const contextMenuBookmark = ref<BookmarkItem | null>(null);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const dragId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);
const dragDepth = ref(0);

const ui = computed(() => {
  const locale = resolveBookmarkBarLocale(props.uiLocale);
  return {
    searchPlaceholder: locale === 'zh-CN' ? '搜索书签...' : 'Search bookmarks...',
    refresh: locale === 'zh-CN' ? '刷新' : 'Refresh',
    emptyBookmarks: locale === 'zh-CN' ? '暂无书签' : 'No bookmarks yet',
    addBookmark: locale === 'zh-CN' ? '添加书签' : 'Add bookmark',
    bookmarksBar: locale === 'zh-CN' ? '书签栏' : 'Bookmarks bar',
    remove: locale === 'zh-CN' ? '删除' : 'Remove',
    contextOpen: locale === 'zh-CN' ? '打开' : 'Open',
    contextOpenInNewTab: locale === 'zh-CN' ? '在新标签页中打开' : 'Open in new tab',
    contextEdit: locale === 'zh-CN' ? '编辑' : 'Edit',
    contextDelete: locale === 'zh-CN' ? '删除' : 'Delete',
  };
});

const contextMenuStyle = computed(() => {
  const windowWidth = typeof window !== 'undefined' ? window.innerWidth : 0;
  const windowHeight = typeof window !== 'undefined' ? window.innerHeight : 0;
  const x = Math.max(0, Math.min(contextMenuX.value, windowWidth - 200));
  const y = Math.max(0, Math.min(contextMenuY.value, windowHeight - 200));
  return {
    left: `${x}px`,
    top: `${y}px`,
  };
});

const filteredBookmarks = computed(() => {
  if (!searchQuery.value.trim()) return props.bookmarks;
  const query = searchQuery.value.toLowerCase();
  return props.bookmarks.filter(
    (b) =>
      (b.title?.toLowerCase().includes(query) || b.url?.toLowerCase().includes(query)),
  );
});

const bookmarksByFolder = computed(() => {
  const result: Record<string, BookmarkItem[]> = {};
  
  for (const bookmark of filteredBookmarks.value) {
    if (!bookmark || !(bookmark.title || bookmark.url)) continue;
    const folder = bookmark.folder || '';
    if (!result[folder]) {
      result[folder] = [];
    }
    result[folder].push(bookmark);
  }
  
  // Sort bookmarks within each folder
  for (const folder in result) {
    result[folder].sort((a, b) => {
      const aLabel = (a.title || a.url || '').trim();
      const bLabel = (b.title || b.url || '').trim();
      return aLabel.localeCompare(bLabel);
    });
  }
  
  return result;
});

const sortedFolderNames = computed(() => {
  return Object.keys(bookmarksByFolder.value)
    .filter((folder) => folder && !isReservedBookmarkGroupName(folder))
    .sort((a, b) => a.localeCompare(b));
});

function faviconFor(bookmark: BookmarkItem): string {
  if (!bookmark || !bookmark.url) return '';
  try {
    return bookmark.favicon || faviconUrlFor(bookmark.url) || '';
  } catch (e) {
    console.error('Failed to resolve favicon for bookmark:', bookmark.id, e);
    return '';
  }
}

function onSearchInput(): void {
  // Search is handled by computed property
}

function onBookmarkClick(bookmark: BookmarkItem): void {
  if (!bookmark?.url) return;
  emit('navigate', bookmark.url);
}

function onRemoveBookmark(id: string): void {
  emit('removeBookmark', id);
}

function toggleFolder(folder: string): void {
  if (openFolders.value.has(folder)) {
    openFolders.value.delete(folder);
  } else {
    openFolders.value.add(folder);
  }
}

function onContextMenu(bookmark: BookmarkItem, event: MouseEvent): void {
  if (!bookmark || !bookmark.url) return;
  if (!event || typeof event.clientX !== 'number' || typeof event.clientY !== 'number') return;
  contextMenuBookmark.value = bookmark;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
}

function closeContextMenu(): void {
  contextMenuBookmark.value = null;
}

function contextOpen(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  closeContextMenu();
  emit('navigate', bookmark.url);
}

function contextOpenInNewTab(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  closeContextMenu();
  emit('openInNewTab', bookmark.url, bookmark.title || bookmark.url);
}

function contextEdit(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.id) return;
  closeContextMenu();
  emit('editBookmark', bookmark);
}

function contextDelete(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.id) return;
  closeContextMenu();
  emit('removeBookmark', bookmark.id);
}

function onDragStart(id: string, event: DragEvent): void {
  if (!props.reorderEnabled) return;
  dragId.value = id;
  dragDepth.value = 0;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', id);
  }
}

function onDragEnter(): void {
  dragDepth.value += 1;
}

function onDragLeave(event: DragEvent): void {
  const related = event.relatedTarget as Node | null;
  if (related && event.currentTarget instanceof Node && event.currentTarget.contains(related)) {
    return;
  }
  dragDepth.value = Math.max(0, dragDepth.value - 1);
  if (dragDepth.value === 0) {
    dragOverId.value = null;
  }
}

function onDragOver(event: DragEvent): void {
  if (!props.reorderEnabled) return;
  event.preventDefault();
}

function onDrop(targetId: string, event: DragEvent): void {
  if (!props.reorderEnabled) return;
  event.preventDefault();
  const fromId = dragId.value ?? event.dataTransfer?.getData('text/plain');
  onDragEnd();
  if (!fromId || !targetId || fromId === targetId) return;

  const ids = filteredBookmarks.value.map((b) => b?.id).filter(Boolean) as string[];
  const fromIdx = ids.indexOf(fromId);
  const toIdx = ids.indexOf(targetId);
  if (toIdx < 0 || fromIdx < 0) return;

  const next = [...ids];
  next.splice(fromIdx, 1);
  next.splice(toIdx, 0, fromId);
  emit('reorder', next);
}

function onDragEnd(): void {
  dragId.value = null;
  dragDepth.value = 0;
  dragOverId.value = null;
}

onMounted(() => {
  // Auto-expand folders with bookmarks
  for (const folder of sortedFolderNames.value) {
    if (bookmarksByFolder.value[folder]?.length > 0) {
      openFolders.value.add(folder);
    }
  }
});

onUnmounted(() => {
  closeContextMenu();
});
</script>

<style scoped>
.bookmarks-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.panel-header {
  display: flex;
  gap: 8px;
  padding: 12px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

.search-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--chrome-border, #dadce0);
  border-radius: 4px;
  font-size: 13px;
  background: var(--chrome-bg, #ffffff);
  color: var(--chrome-text, #202124);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary, #1a73e8);
}

.nav-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  border: 1px solid var(--chrome-border, #dadce0);
  border-radius: 4px;
  background: var(--chrome-bg, #ffffff);
  color: var(--chrome-text, #202124);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.nav-button:hover {
  background: var(--chrome-bg-hover, #f1f3f4);
}

.nav-button.primary {
  background: var(--color-primary, #1a73e8);
  color: white;
  border-color: var(--color-primary, #1a73e8);
}

.nav-button.primary:hover {
  background: #1557b0;
}

.nav-icon {
  width: 16px;
  height: 16px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
}

.empty-icon {
  width: 48px;
  height: 48px;
  color: var(--chrome-text-secondary, #5f6368);
  margin-bottom: 16px;
}

.empty-text {
  color: var(--chrome-text-secondary, #5f6368);
  font-size: 14px;
  margin-bottom: 16px;
}

.folder-section {
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

.folder-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.15s ease;
}

.folder-header:hover {
  background: var(--chrome-bg-hover, #f1f3f4);
}

.folder-icon {
  width: 16px;
  height: 16px;
  color: var(--chrome-text-secondary, #5f6368);
  flex-shrink: 0;
}

.folder-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--chrome-text, #202124);
}

.folder-chevron {
  width: 12px;
  height: 12px;
  color: var(--chrome-text-secondary, #5f6368);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.folder-chevron--open {
  transform: rotate(180deg);
}

.folder-content {
  padding: 0 12px 12px;
}

.bookmark-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.15s ease;
  user-select: none;
}

.bookmark-item:hover {
  background: var(--chrome-bg-hover, #f1f3f4);
}

.bookmark-item--nested {
  padding-left: 28px;
}

.bookmark-item--drag-over {
  background: rgba(26, 115, 232, 0.12);
  box-shadow: inset 0 0 0 1px var(--color-primary, #1a73e8);
}

.bookmark-favicon {
  width: 16px;
  height: 16px;
  border-radius: 2px;
  flex-shrink: 0;
}

.bookmark-favicon--fallback {
  color: var(--chrome-text-secondary, #5f6368);
  display: flex;
  align-items: center;
  justify-content: center;
}

.bookmark-favicon--fallback svg {
  width: 100%;
  height: 100%;
}

.bookmark-info {
  flex: 1;
  min-width: 0;
}

.bookmark-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--chrome-text, #202124);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bookmark-url {
  font-size: 11px;
  color: var(--chrome-text-secondary, #5f6368);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bookmark-action {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--chrome-text-secondary, #5f6368);
  cursor: pointer;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.15s ease, background-color 0.15s ease;
  flex-shrink: 0;
}

.bookmark-item:hover .bookmark-action {
  opacity: 1;
}

.bookmark-action:hover {
  background: var(--chrome-bg-hover, #f1f3f4);
  color: var(--chrome-text, #202124);
}

.bookmark-action svg {
  width: 14px;
  height: 14px;
}

.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9998;
  background: transparent;
  border: none;
}

.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 180px;
  padding: 4px 0;
  background: var(--chrome-bg, #ffffff);
  border: 1px solid var(--chrome-border, #dadce0);
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  color: var(--chrome-text, #202124);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.context-menu-item:hover {
  background: var(--chrome-bg-hover, #f1f3f4);
}

.context-menu-item--danger {
  color: #d93025;
}

.context-menu-item--danger:hover {
  background: #fce8e6;
}

.context-menu-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--chrome-divider, #dadce0);
}
</style>
