<!--
  Exodus Browser — Chrome-aligned bookmark bar (side panel, apps, overflow, all bookmarks).
-->
<template>
  <div
    v-if="visible"
    class="bookmark-bar exodus-chrome-bookmarks exodus-bookmark-bar"
    role="navigation"
    :aria-label="ui.barAriaLabel"
    :class="{ 'bookmark-bar--drag-over': dragOverBar }"
    @dragenter="onDragEnterZone"
    @dragover="onDragOverBarZone"
    @dragleave="onDragLeaveZone"
    @drop="onDropBar"
  >
    <!-- Chrome left rail: side panel + apps -->
    <div class="bookmark-bar__lead">
      <button
        type="button"
        class="bookmark-lead-btn"
        :class="{ 'bookmark-lead-btn--active': sidePanelOpen }"
        :title="ui.sidePanel"
        :aria-label="ui.sidePanel"
        @click="emit('toggleSidePanel')"
      >
        <svg class="bookmark-lead-icon" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" stroke="currentColor" stroke-width="1.2" />
          <path d="M5.5 2.5v11" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
      <button
        type="button"
        class="bookmark-lead-btn"
        :title="ui.apps"
        :aria-label="ui.apps"
        @click="emit('openApps')"
      >
        <svg class="bookmark-lead-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
          <rect x="2" y="2" width="4.5" height="4.5" rx="0.8" />
          <rect x="9.5" y="2" width="4.5" height="4.5" rx="0.8" />
          <rect x="2" y="9.5" width="4.5" height="4.5" rx="0.8" />
          <rect x="9.5" y="9.5" width="4.5" height="4.5" rx="0.8" />
        </svg>
      </button>
      <div class="bookmark-groups-wrap">
        <button
          type="button"
          class="bookmark-lead-btn"
          data-testid="bookmark-groups-btn"
          :class="{ 'bookmark-lead-btn--active': showGroupsMenu }"
          :title="ui.bookmarkGroups"
          :aria-label="ui.bookmarkGroups"
          :aria-expanded="showGroupsMenu"
          aria-haspopup="menu"
          @click="toggleGroupsMenu"
        >
          <svg class="bookmark-lead-icon" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <rect x="2" y="2" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.2" />
            <rect x="9" y="2" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.2" />
            <rect x="2" y="9" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.2" />
            <rect x="9" y="9" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.2" />
            <path d="M11.5 11.5h2.5v2.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
        <template v-if="showGroupsMenu">
          <button
            type="button"
            class="bookmark-menu-backdrop"
            :aria-label="ui.closeGroupsMenu"
            @click="closeGroupsMenu"
          />
          <div class="bookmark-groups-menu bookmark-dropdown" role="menu" data-testid="bookmark-groups-menu">
            <button
              type="button"
              class="bookmark-groups-menu__create"
              role="menuitem"
              data-testid="bookmark-group-create"
              @click="openCreateGroupDialog"
            >
              <svg class="bookmark-groups-menu__create-icon" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <rect x="2.5" y="2.5" width="11" height="11" rx="1.5" stroke="currentColor" stroke-width="1.2" />
                <path d="M8 5.5v5M5.5 8h5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              </svg>
              <span class="bookmark-groups-menu__create-label">{{ ui.createNewGroup }}</span>
              <span class="bookmark-groups-menu__shortcut">{{ ui.createGroupShortcut }}</span>
            </button>
            <div v-if="groupMenuEntries.length > 0" class="bookmark-groups-menu__divider" role="separator" />
            <button
              v-for="group in groupMenuEntries"
              :key="group.name"
              type="button"
              class="bookmark-groups-menu__group"
              role="menuitem"
              @click="selectGroupFromMenu(group.name)"
            >
              <span
                class="bookmark-group-dot"
                :style="{ '--bookmark-group-color': bookmarkGroupColorCss(group.color) }"
                aria-hidden="true"
              />
              <span class="bookmark-groups-menu__group-name">{{ group.name }}</span>
              <span class="bookmark-groups-menu__group-count">{{ ui.groupBookmarkCount(group.count) }}</span>
            </button>
          </div>
        </template>
      </div>
    </div>

    <span class="bookmark-bar-separator" aria-hidden="true" />

    <!-- Scrollable bookmark chips + user folders -->
    <div class="bookmark-bar__scroll" @contextmenu="onScrollContextMenu">
      <button
        v-for="bookmark in visibleBarBookmarks"
        :key="bookmark.id"
        type="button"
        class="bookmark-chip"
        :class="{ 'bookmark-chip--drag-over': dragOverId === bookmark.id }"
        :draggable="canDrag"
        :title="bookmark.url"
        @click="() => bookmark.url ? emit('navigate', bookmark.url) : null"
        @auxclick.prevent="onChipAuxClick(bookmark, $event)"
        @contextmenu.prevent="onChipContextMenu(bookmark, $event)"
        @dragstart="(event) => onDragStart(bookmark.id, event)"
        @dragenter="onDragEnterZone"
        @dragover="(event) => onDragOverChip(bookmark.id, event)"
        @dragleave="onDragLeaveZone"
        @drop="(event) => void onDropChip(bookmark.id, event)"
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
        <span v-else class="bookmark-favicon bookmark-favicon--fallback" aria-hidden="true">
          <svg viewBox="0 0 16 16" fill="none">
            <circle cx="8" cy="8" r="6.5" stroke="currentColor" stroke-width="1.2" />
            <path d="M2.5 8h11M8 2.5v11" stroke="currentColor" stroke-width="1" opacity="0.55" />
          </svg>
        </span>
        <span class="bookmark-label">{{ bookmark.title || bookmark.url }}</span>
      </button>

      <div
        v-for="folder in folderNames"
        :key="folder"
        class="bookmark-folder-wrap"
      >
        <button
          type="button"
          class="bookmark-chip bookmark-chip--folder"
          :class="{ 'bookmark-chip--drag-over': dragOverFolder === folder }"
          :title="ui.folderTitle(folder)"
          :style="{ '--bookmark-group-color': bookmarkGroupColorCss(folderColor(folder)) }"
          @click="toggleFolder(folder)"
          @dragenter="onDragEnterZone"
          @dragover="(event) => onDragOverFolderName(folder, event)"
          @dragleave="onDragLeaveZone"
          @drop="(event) => void onDropFolder(folder, event)"
        >
          <span class="bookmark-group-dot bookmark-group-dot--chip" aria-hidden="true" />
          <svg class="bookmark-folder-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2z" />
          </svg>
          <span class="bookmark-label">{{ folder }}</span>
        </button>

        <template v-if="openFolder === folder">
          <button type="button" class="bookmark-menu-backdrop" :aria-label="ui.closeFolderMenu" @click="closeFolder" />
          <div class="bookmark-dropdown" role="menu">
            <p
              v-if="bookmarksInOpenFolder(folder).length === 0"
              class="bookmark-dropdown-empty"
            >
              {{ ui.emptyFolder }}
            </p>
            <button
              v-for="bookmark in bookmarksInOpenFolder(folder)"
              :key="bookmark.id"
              type="button"
              class="bookmark-dropdown-item"
              role="menuitem"
              :title="bookmark.url"
              @click="() => bookmark.url ? navigateFromFolder(bookmark.url) : null"
            >
              <img
                v-if="faviconFor(bookmark)"
                class="bookmark-favicon"
                :src="faviconFor(bookmark)"
                alt=""
                width="16"
                height="16"
              />
              <span class="bookmark-label">{{ bookmark.title || bookmark.url }}</span>
            </button>
          </div>
        </template>
      </div>

      <div v-if="showOverflow" class="bookmark-overflow-wrap">
        <button
          type="button"
          class="bookmark-chip bookmark-chip--overflow"
          :title="ui.moreBookmarks"
          :aria-label="ui.moreBookmarks"
          @click="toggleOverflow"
        >
          <svg class="bookmark-overflow-icon" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M3 4l3.5 4L3 12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M7 4l3.5 4L7 12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>

        <template v-if="showOverflowMenu">
          <button type="button" class="bookmark-menu-backdrop" :aria-label="ui.closeOverflowMenu" @click="closeOverflow" />
          <div class="bookmark-dropdown bookmark-dropdown--overflow" role="menu">
            <button
              v-for="bookmark in overflowBookmarks"
              :key="bookmark.id"
              type="button"
              class="bookmark-dropdown-item"
              role="menuitem"
              :title="bookmark.url"
              @click="() => bookmark.url ? navigateFromOverflow(bookmark.url) : null"
            >
              <img
                v-if="faviconFor(bookmark)"
                class="bookmark-favicon"
                :src="faviconFor(bookmark)"
                alt=""
                width="16"
                height="16"
              />
              <span class="bookmark-label">{{ bookmark.title || bookmark.url }}</span>
            </button>
          </div>
        </template>
      </div>
      <div class="bookmark-bar__spacer" aria-hidden="true" />
    </div>

    <!-- Chrome right rail: all bookmarks folder -->
    <div class="bookmark-bar__end">
      <span class="bookmark-bar-separator" aria-hidden="true" />
      <div class="bookmark-folder-wrap bookmark-folder-wrap--all">
        <button
          type="button"
          class="bookmark-chip bookmark-chip--folder bookmark-chip--all"
          :title="displayAllBookmarksLabel"
          :aria-label="displayAllBookmarksLabel"
          @click="toggleAllBookmarks"
        >
          <svg class="bookmark-folder-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2z" />
          </svg>
          <span class="bookmark-label">{{ displayAllBookmarksLabel }}</span>
        </button>

        <template v-if="showAllBookmarksMenu">
          <button
            type="button"
            class="bookmark-menu-backdrop"
            :aria-label="ui.closeAllBookmarksMenu"
            @click="closeAllBookmarks"
          />
          <div class="bookmark-dropdown bookmark-dropdown--all" role="menu">
            <!-- Empty state when no bookmarks -->
            <template v-if="totalBookmarkCount === 0">
              <p class="bookmark-dropdown-empty">
                {{ ui.emptyAllBookmarks }}
              </p>
              <button
                type="button"
                class="bookmark-dropdown-item bookmark-dropdown-item--action"
                @click="openBookmarksSidebar"
              >
                {{ ui.openBookmarksSidebar }}
              </button>
            </template>
            
            <!-- Bookmarks without folder (bar bookmarks) -->
            <template v-if="bookmarksByFolder[''] && bookmarksByFolder[''].length > 0">
              <button
                v-for="bookmark in bookmarksByFolder['']"
                :key="bookmark.id"
                type="button"
                class="bookmark-dropdown-item"
                role="menuitem"
                :title="bookmark.url"
                @click="() => bookmark.url ? navigateFromAllBookmarks(bookmark.url) : null"
              >
                <img
                  v-if="faviconFor(bookmark)"
                  class="bookmark-favicon"
                  :src="faviconFor(bookmark)"
                  alt=""
                  width="16"
                  height="16"
                />
                <span class="bookmark-label">{{ bookmark.title || bookmark.url }}</span>
              </button>
            </template>
            
            <!-- Folder sections -->
            <template v-for="folder in sortedFolderNames" :key="folder">
              <div class="bookmark-dropdown-folder-header">{{ folder }}</div>
              <button
                v-for="bookmark in bookmarksByFolder[folder]"
                :key="bookmark.id"
                type="button"
                class="bookmark-dropdown-item bookmark-dropdown-item--nested"
                role="menuitem"
                :title="bookmark.url"
                @click="() => bookmark.url ? navigateFromAllBookmarks(bookmark.url) : null"
              >
                <img
                  v-if="faviconFor(bookmark)"
                  class="bookmark-favicon"
                  :src="faviconFor(bookmark)"
                  alt=""
                  width="16"
                  height="16"
                />
                <span class="bookmark-label">{{ bookmark.title || bookmark.url }}</span>
              </button>
            </template>
            
            <button
              type="button"
              class="bookmark-dropdown-item bookmark-dropdown-item--action"
              @click="openAllBookmarksManager"
            >
              {{ ui.openBookmarksManager }}
            </button>
          </div>
        </template>
      </div>
    </div>

    <Teleport to="body">
      <template v-if="contextMenuBookmark">
        <button
          type="button"
          class="bookmark-menu-backdrop"
          aria-label="Close bookmark menu"
          @click="closeContextMenu"
        />
        <div
          class="bookmark-context-menu bookmark-dropdown"
          role="menu"
          :style="contextMenuStyle"
        >
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="contextOpen">
            {{ ui.contextOpen }}
          </button>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="contextOpenInNewTab">
            {{ ui.contextOpenInNewTab }}
          </button>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="contextOpenInNewWindow">
            {{ ui.contextOpenInNewWindow }}
          </button>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="contextOpenInIncognito">
            {{ ui.contextOpenInIncognito }}
          </button>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="contextEdit">
            {{ ui.contextEdit }}
          </button>
          <template v-if="groupMenuEntries.length > 0">
            <div class="bookmark-dropdown-divider" role="separator" />
            <p class="bookmark-dropdown-submenu-label">{{ ui.contextMoveToGroup }}</p>
            <button
              v-for="group in groupMenuEntries"
              :key="`move-${group.name}`"
              type="button"
              class="bookmark-dropdown-item bookmark-dropdown-item--nested"
              role="menuitem"
              @click="contextMoveToGroup(group.name)"
            >
              <span
                class="bookmark-group-dot"
                :style="{ '--bookmark-group-color': bookmarkGroupColorCss(group.color) }"
                aria-hidden="true"
              />
              <span class="bookmark-label">{{ group.name }}</span>
            </button>
          </template>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="contextCopyUrl">
            {{ ui.contextCopyUrl }}
          </button>
          <button
            type="button"
            class="bookmark-dropdown-item bookmark-dropdown-item--danger"
            role="menuitem"
            @click="contextDelete"
          >
            {{ ui.contextDelete }}
          </button>
        </div>
      </template>
    </Teleport>

    <Teleport to="body">
      <template v-if="showCreateGroupDialog">
        <button
          type="button"
          class="bookmark-group-prompt-backdrop"
          :aria-label="ui.newGroupCancel"
          @click="closeCreateGroupDialog"
        />
        <div
          class="bookmark-group-prompt"
          role="dialog"
          aria-modal="true"
          data-testid="bookmark-group-prompt"
          aria-labelledby="bookmark-group-prompt-title"
        >
          <h3 id="bookmark-group-prompt-title">{{ ui.newGroupDialogTitle }}</h3>
          <label class="bookmark-group-prompt__field">
            <span>{{ ui.newGroupNameLabel }}</span>
            <input
              ref="newGroupNameInputRef"
              v-model="newGroupName"
              type="text"
              class="bookmark-group-prompt__input"
              data-testid="bookmark-group-name-input"
              :maxlength="maxGroupNameLength"
              :aria-invalid="!!createGroupNameError"
              autocomplete="off"
              @input="validateNewGroupName"
              @keydown.enter.prevent="confirmCreateGroup"
            />
            <p v-if="createGroupNameError" class="bookmark-group-prompt__error" role="alert">
              {{ createGroupNameError }}
            </p>
          </label>
          <p class="bookmark-group-prompt__color-label">{{ ui.newGroupColorLabel }}</p>
          <div class="bookmark-group-prompt__colors">
            <button
              v-for="color in bookmarkGroupColors"
              :key="color"
              type="button"
              class="bookmark-group-prompt__swatch"
              :class="{ 'bookmark-group-prompt__swatch--selected': newGroupColor === color }"
              :style="{ '--swatch': bookmarkGroupColorCss(color) }"
              :title="color"
              @click="newGroupColor = color"
            />
          </div>
          <div class="bookmark-group-prompt__actions">
            <button type="button" class="bookmark-group-prompt__btn secondary" @click="closeCreateGroupDialog">
              {{ ui.newGroupCancel }}
            </button>
            <button
              type="button"
              class="bookmark-group-prompt__btn primary"
              :disabled="createGroupSaveDisabled"
              @click="confirmCreateGroup"
            >
              {{ ui.newGroupSave }}
            </button>
          </div>
        </div>
      </template>
    </Teleport>

    <Teleport to="body">
      <template v-if="showBarContextMenu">
        <button
          type="button"
          class="bookmark-menu-backdrop"
          aria-label="Close bookmark bar menu"
          @click="closeBarContextMenu"
        />
        <div
          class="bookmark-context-menu bookmark-dropdown"
          role="menu"
          :style="contextMenuStyle"
        >
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="barContextAddBookmark">
            {{ ui.contextAddBookmark }}
          </button>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="barContextToggleBookmarkBar">
            {{ props.visible ? ui.contextHideBookmarkBar : ui.contextShowBookmarkBar }}
          </button>
          <button type="button" class="bookmark-dropdown-item" role="menuitem" @click="barContextOpenManager">
            {{ ui.contextBookmarkManager }}
          </button>
        </div>
      </template>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — Chrome-style bookmark bar with side panel, apps, and all-bookmarks folder.
 */
import { computed, ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import type { BookmarkItem } from '@/lib/browserTypes';
import { BOOKMARK_BAR_MAX } from '@/lib/bookmarks';
import { bookmarkBarStrings, type BookmarkBarLocale } from '@/lib/bookmarkBarUi';
import {
  allKnownBookmarkGroupNames,
  bookmarkFolderColor,
  bookmarkGroupColorCss,
  buildBookmarkBarGroupEntries,
  isReservedBookmarkGroupName,
  loadSavedBookmarkBarGroups,
  MAX_BOOKMARK_GROUP_NAME_LENGTH,
  saveBookmarkBarGroup,
  validateBookmarkGroupName,
} from '@/lib/bookmarkGroups';
import { faviconUrlFor } from '@/lib/favicon';
import { TAB_GROUP_COLORS } from '@/lib/tabGroups';

interface Props {
  visible?: boolean;
  barBookmarks?: BookmarkItem[];
  folderNames?: string[];
  bookmarks?: BookmarkItem[];
  /** Side panel active state for lead button highlight. */
  sidePanelOpen?: boolean;
  /** Override right-pinned folder label (defaults to locale string). */
  allBookmarksLabel?: string;
  /** Force UI locale for bookmark bar strings. */
  uiLocale?: BookmarkBarLocale;
  /** Enable drag-reorder / drag-to-folder when handlers are provided. */
  reorderEnabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  visible: true,
  barBookmarks: () => [],
  folderNames: () => [],
  bookmarks: () => [],
  sidePanelOpen: false,
  reorderEnabled: false,
});

const emit = defineEmits<{
  navigate: [url: string];
  reorder: [orderedIds: string[]];
  moveToFolder: [bookmarkId: string, folder: string];
  toggleSidePanel: [];
  openApps: [];
  openAllBookmarks: [];
  openBookmarksSidebar: [];
  openInNewTab: [url: string, title?: string];
  openInNewWindow: [url: string, title?: string];
  openInIncognito: [url: string, title?: string];
  editBookmark: [bookmark: BookmarkItem];
  removeBookmark: [bookmarkId: string];
  copyUrl: [url: string];
  addBookmark: [];
  groupCreated: [name: string, color: string];
  toggleBookmarkBar: [];
}>();

const openFolder = ref<string | null>(null);
const showGroupsMenu = ref(false);
const showCreateGroupDialog = ref(false);
const newGroupName = ref('');
const newGroupColor = ref('blue');
const savedBarGroups = ref(loadSavedBookmarkBarGroups());
const bookmarkGroupColors = TAB_GROUP_COLORS.filter((c) => c !== 'grey');
const maxGroupNameLength = MAX_BOOKMARK_GROUP_NAME_LENGTH;
const newGroupNameInputRef = ref<HTMLInputElement | null>(null);
const createGroupSaveFailed = ref(false);
const showOverflowMenu = ref(false);
const showAllBookmarksMenu = ref(false);
const contextMenuBookmark = ref<BookmarkItem | null>(null);
const showBarContextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const dragId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);
const dragOverFolder = ref<string | null>(null);
const dragOverBar = ref(false);
const dragDepth = ref(0);

const ui = computed(() => bookmarkBarStrings(props.uiLocale));

const displayAllBookmarksLabel = computed(
  () => props.allBookmarksLabel?.trim() || ui.value.allBookmarks,
);

const contextMenuStyle = computed(() => {
  const windowWidth = typeof window !== 'undefined' ? window.innerWidth : 0;
  const windowHeight = typeof window !== 'undefined' ? window.innerHeight : 0;
  const x = Math.max(0, Math.min(contextMenuX.value, windowWidth - 240));
  const y = Math.max(0, Math.min(contextMenuY.value, windowHeight - 200));
  return {
    left: `${x}px`,
    top: `${y}px`,
  };
});

const createGroupNameError = ref('');

const groupMenuEntries = computed(() =>
  buildBookmarkBarGroupEntries(props.bookmarks, props.folderNames, savedBarGroups.value),
);

const createGroupSaveDisabled = computed(
  () => !newGroupName.value.trim() || !!createGroupNameError.value,
);

const visibleBarBookmarks = computed(() => props.barBookmarks.slice(0, BOOKMARK_BAR_MAX));

const showOverflow = computed(() => props.barBookmarks.length > BOOKMARK_BAR_MAX);

const overflowBookmarks = computed(() => props.barBookmarks.slice(BOOKMARK_BAR_MAX));

const sortedAllBookmarks = computed(() =>
  [...props.bookmarks]
    .filter((b) => b && (b.title || b.url))
    .sort((a, b) => {
      const aLabel = (a.title || a.url || '').trim();
      const bLabel = (b.title || b.url || '').trim();
      return aLabel.localeCompare(bLabel);
    }),
);

/** Organize bookmarks by folder for hierarchical all-bookmarks menu (Chrome-style). */
const bookmarksByFolder = computed(() => {
  const result: Record<string, BookmarkItem[]> = {};
  
  // Group bookmarks by folder
  for (const bookmark of props.bookmarks) {
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

/** Get sorted folder names for all-bookmarks menu (excluding reserved names). */
const sortedFolderNames = computed(() => {
  return Object.keys(bookmarksByFolder.value)
    .filter((folder) => folder && !isReservedBookmarkGroupName(folder))
    .sort((a, b) => a.localeCompare(b));
});

/** Total bookmark count for empty state check. */
const totalBookmarkCount = computed(() => {
  return props.bookmarks.filter((b) => b && (b.title || b.url)).length;
});

const canDrag = computed(() => props.reorderEnabled);

/** Resolve favicon URL from bookmark row or remote service. */
function faviconFor(bookmark: BookmarkItem): string {
  if (!bookmark || !bookmark.url) return '';
  try {
    return bookmark.favicon || faviconUrlFor(bookmark.url) || '';
  } catch (e) {
    console.error('Failed to resolve favicon for bookmark:', bookmark.id, e);
    return '';
  }
}

/** Bookmarks inside a named folder. */
function bookmarksInOpenFolder(folder: string): BookmarkItem[] {
  return props.bookmarks.filter((bookmark) => bookmark.folder === folder);
}

/** Resolve stored color for a folder chip. */
function folderColor(folderName: string): string {
  return bookmarkFolderColor(folderName, savedBarGroups.value);
}

function closeMenus(): void {
  openFolder.value = null;
  showGroupsMenu.value = false;
  showOverflowMenu.value = false;
  showAllBookmarksMenu.value = false;
  contextMenuBookmark.value = null;
  showBarContextMenu.value = false;
}

function toggleGroupsMenu(): void {
  showAllBookmarksMenu.value = false;
  showOverflowMenu.value = false;
  openFolder.value = null;
  showGroupsMenu.value = !showGroupsMenu.value;
}

function closeGroupsMenu(): void {
  showGroupsMenu.value = false;
}

/** All folder/group names used for create-dialog validation. */
function knownGroupNamesForValidation(): string[] {
  return allKnownBookmarkGroupNames(props.bookmarks, props.folderNames, savedBarGroups.value);
}

function validateNewGroupName(): void {
  createGroupSaveFailed.value = false;
  const err = validateBookmarkGroupName(newGroupName.value, knownGroupNamesForValidation());
  createGroupNameError.value = err ? ui.value.groupNameError(err) : '';
}

function openCreateGroupDialog(): void {
  closeGroupsMenu();
  newGroupName.value = '';
  newGroupColor.value = 'blue';
  createGroupNameError.value = '';
  createGroupSaveFailed.value = false;
  showCreateGroupDialog.value = true;
  void nextTick(() => newGroupNameInputRef.value?.focus());
}

function closeCreateGroupDialog(): void {
  showCreateGroupDialog.value = false;
  createGroupNameError.value = '';
  createGroupSaveFailed.value = false;
}

function confirmCreateGroup(): void {
  validateNewGroupName();
  const name = newGroupName.value.trim();
  if (!name || createGroupNameError.value) return;
  try {
    const ok = saveBookmarkBarGroup(name, newGroupColor.value, knownGroupNamesForValidation());
    if (!ok) {
      createGroupSaveFailed.value = true;
      createGroupNameError.value = ui.value.groupNameError('exists');
      return;
    }
    savedBarGroups.value = loadSavedBookmarkBarGroups();
    emit('groupCreated', name, newGroupColor.value);
    closeCreateGroupDialog();
    openFolder.value = name;
  } catch (error) {
    console.error('confirmCreateGroup failed:', error);
    createGroupSaveFailed.value = true;
    createGroupNameError.value = ui.value.groupNameError('invalid_chars');
  }
}

function contextMoveToGroup(folderName: string): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark?.id || !folderName) return;
  closeContextMenu();
  try {
    emit('moveToFolder', bookmark.id, folderName);
  } catch (error) {
    console.error('Failed to move bookmark to group:', error);
  }
}

function selectGroupFromMenu(folderName: string): void {
  closeGroupsMenu();
  toggleFolder(folderName);
}

function closeContextMenu(): void {
  contextMenuBookmark.value = null;
}

function closeBarContextMenu(): void {
  showBarContextMenu.value = false;
}

function onScrollContextMenu(event: MouseEvent): void {
  const target = event.target as HTMLElement | null;
  if (!target) return;
  if (target.closest('.bookmark-chip, .bookmark-folder-wrap, .bookmark-overflow-wrap, button, a')) {
    return;
  }
  event.preventDefault();
  closeMenus();
  showBarContextMenu.value = true;
  contextMenuX.value = event.clientX ?? 0;
  contextMenuY.value = event.clientY ?? 0;
}

function barContextAddBookmark(): void {
  closeBarContextMenu();
  try {
    emit('addBookmark');
  } catch (error) {
    console.error('Failed to add bookmark:', error);
  }
}

function barContextOpenManager(): void {
  closeBarContextMenu();
  try {
    emit('openAllBookmarks');
  } catch (error) {
    console.error('Failed to open bookmarks manager:', error);
  }
}

function barContextToggleBookmarkBar(): void {
  closeBarContextMenu();
  try {
    emit('toggleBookmarkBar');
  } catch (error) {
    console.error('Failed to toggle bookmark bar:', error);
  }
}

function onChipContextMenu(bookmark: BookmarkItem, event: MouseEvent): void {
  if (!bookmark || !bookmark.url) return;
  if (!event || typeof event.clientX !== 'number' || typeof event.clientY !== 'number') return;
  closeMenus();
  contextMenuBookmark.value = bookmark;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
}

function onChipAuxClick(bookmark: BookmarkItem, event: MouseEvent): void {
  if (!bookmark || !bookmark.url) return;
  if (event.button !== 1) return;
  closeMenus();
  try {
    emit('openInNewTab', bookmark.url, bookmark.title || bookmark.url);
  } catch (error) {
    console.error('Failed to open bookmark in new tab (aux click):', error);
  }
}

function contextOpen(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  closeContextMenu();
  try {
    emit('navigate', bookmark.url);
  } catch (error) {
    console.error('Failed to open bookmark:', error);
  }
}

function contextOpenInNewTab(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  closeContextMenu();
  try {
    emit('openInNewTab', bookmark.url, bookmark.title || bookmark.url);
  } catch (error) {
    console.error('Failed to open bookmark in new tab:', error);
  }
}

function contextOpenInNewWindow(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  closeContextMenu();
  try {
    emit('openInNewWindow', bookmark.url, bookmark.title || bookmark.url);
  } catch (error) {
    console.error('Failed to open bookmark in new window:', error);
  }
}

function contextOpenInIncognito(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  closeContextMenu();
  try {
    emit('openInIncognito', bookmark.url, bookmark.title || bookmark.url);
  } catch (error) {
    console.error('Failed to open bookmark in incognito:', error);
  }
}

function contextEdit(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.id) return;
  closeContextMenu();
  try {
    emit('editBookmark', bookmark);
  } catch (error) {
    console.error('Failed to edit bookmark:', error);
  }
}

function contextDelete(): void {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.id) return;
  closeContextMenu();
  try {
    emit('removeBookmark', bookmark.id);
  } catch (error) {
    console.error('Failed to delete bookmark:', error);
  }
}

async function contextCopyUrl(): Promise<void> {
  const bookmark = contextMenuBookmark.value;
  if (!bookmark || !bookmark.url) return;
  try {
    await navigator.clipboard.writeText(bookmark.url);
    emit('copyUrl', bookmark.url);
  } catch (error) {
    console.error('copy bookmark url failed:', error);
  }
  closeContextMenu();
}

/** Skip shortcut when user is typing in a field or contenteditable. */
function isEditableShortcutTarget(event: KeyboardEvent): boolean {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return target.isContentEditable;
}

function onGroupsShortcut(event: KeyboardEvent): void {
  if (!(event.ctrlKey && event.metaKey && event.key.toLowerCase() === 'p')) return;
  if (!props.visible || isEditableShortcutTarget(event)) return;
  event.preventDefault();
  if (showCreateGroupDialog.value) {
    closeCreateGroupDialog();
    return;
  }
  openCreateGroupDialog();
}

function onEscapeKey(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return;
  if (showCreateGroupDialog.value) {
    closeCreateGroupDialog();
    return;
  }
  closeMenus();
  closeContextMenu();
  closeBarContextMenu();
}

watch(
  () => [props.folderNames, props.bookmarks] as const,
  () => {
    savedBarGroups.value = loadSavedBookmarkBarGroups();
  },
  { deep: true },
);

onMounted(() => {
  console.log('[BookmarkBar] Component mounted', {
    visible: props.visible,
    barBookmarksCount: props.barBookmarks.length,
    allBookmarksCount: props.bookmarks.length,
    uiLocale: props.uiLocale,
  });
  document.addEventListener('keydown', onEscapeKey);
  document.addEventListener('keydown', onGroupsShortcut);
});

onUnmounted(() => {
  console.log('[BookmarkBar] Component unmounted');
  document.removeEventListener('keydown', onEscapeKey);
  document.removeEventListener('keydown', onGroupsShortcut);
});

function toggleFolder(folder: string): void {
  showAllBookmarksMenu.value = false;
  showOverflowMenu.value = false;
  showGroupsMenu.value = false;
  openFolder.value = openFolder.value === folder ? null : folder;
}

function closeFolder(): void {
  openFolder.value = null;
}

function toggleOverflow(): void {
  openFolder.value = null;
  showAllBookmarksMenu.value = false;
  showOverflowMenu.value = !showOverflowMenu.value;

  // Position dropdown using JavaScript when opening
  if (showOverflowMenu.value) {
    nextTick(() => {
      const button = document.querySelector('.bookmark-chip--overflow') as HTMLElement;
      const dropdown = document.querySelector('.bookmark-dropdown--overflow') as HTMLElement;
      if (button && dropdown) {
        const buttonRect = button.getBoundingClientRect();
        dropdown.style.position = 'fixed';
        dropdown.style.top = `${buttonRect.bottom + 4}px`;
        dropdown.style.left = `${buttonRect.left}px`;
        // Ensure dropdown doesn't overflow viewport
        const dropdownRect = dropdown.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        if (dropdownRect.right > viewportWidth) {
          dropdown.style.left = `${viewportWidth - dropdownRect.width - 8}px`;
        }
      }
    });
  }
}

function closeOverflow(): void {
  showOverflowMenu.value = false;
}

function toggleAllBookmarks(): void {
  openFolder.value = null;
  showOverflowMenu.value = false;
  showAllBookmarksMenu.value = !showAllBookmarksMenu.value;

  // Position dropdown using JavaScript when opening
  if (showAllBookmarksMenu.value) {
    nextTick(() => {
      const button = document.querySelector('.bookmark-chip--all') as HTMLElement;
      const dropdown = document.querySelector('.bookmark-dropdown--all') as HTMLElement;
      if (button && dropdown) {
        const buttonRect = button.getBoundingClientRect();
        dropdown.style.position = 'fixed';
        dropdown.style.top = `${buttonRect.bottom + 4}px`;
        dropdown.style.left = `${buttonRect.left}px`;
        // Ensure dropdown doesn't overflow viewport
        const dropdownRect = dropdown.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        if (dropdownRect.right > viewportWidth) {
          dropdown.style.left = `${viewportWidth - dropdownRect.width - 8}px`;
        }
      }
    });
  }
}

function closeAllBookmarks(): void {
  showAllBookmarksMenu.value = false;
}

function navigateFromFolder(url: string): void {
  if (!url) return;
  closeFolder();
  try {
    emit('navigate', url);
  } catch (error) {
    console.error('Failed to navigate from folder:', error);
  }
}

function navigateFromOverflow(url: string): void {
  if (!url) return;
  closeOverflow();
  try {
    emit('navigate', url);
  } catch (error) {
    console.error('Failed to navigate from overflow:', error);
  }
}

function navigateFromAllBookmarks(url: string): void {
  if (!url) return;
  closeAllBookmarks();
  try {
    emit('navigate', url);
  } catch (error) {
    console.error('Failed to navigate from all bookmarks:', error);
  }
}

function openAllBookmarksManager(): void {
  closeAllBookmarks();
  try {
    emit('openAllBookmarks');
  } catch (error) {
    console.error('Failed to open bookmarks manager:', error);
  }
}

function openBookmarksSidebar(): void {
  closeAllBookmarks();
  try {
    emit('openBookmarksSidebar');
  } catch (error) {
    console.error('Failed to open bookmarks sidebar:', error);
  }
}

function clearDragHighlights(): void {
  dragOverId.value = null;
  dragOverFolder.value = null;
  dragOverBar.value = false;
}

function onDragEnterZone(): void {
  dragDepth.value += 1;
}

function onDragLeaveZone(event: DragEvent): void {
  const related = event.relatedTarget as Node | null;
  if (related && event.currentTarget instanceof Node && event.currentTarget.contains(related)) {
    return;
  }
  dragDepth.value = Math.max(0, dragDepth.value - 1);
  if (dragDepth.value === 0) clearDragHighlights();
}

function onDragStart(id: string, event: DragEvent): void {
  if (!canDrag.value) return;
  dragId.value = id;
  dragDepth.value = 0;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', id);
  }
}

function onDragOverChip(id: string, event: DragEvent): void {
  if (!canDrag.value) return;
  event.preventDefault();
  if (dragId.value && dragId.value !== id) dragOverId.value = id;
}

function onDragOverFolderName(folderName: string, event: DragEvent): void {
  if (!canDrag.value) return;
  event.preventDefault();
  if (!dragId.value || !folderName) return;
  const dragged = props.bookmarks.find((bookmark) => bookmark && bookmark.id === dragId.value);
  if (!dragged || dragged.folder === folderName) return;
  dragOverFolder.value = folderName;
}

function onDragOverBarZone(event: DragEvent): void {
  if (!canDrag.value) return;
  event.preventDefault();
  if (dragId.value) dragOverBar.value = true;
}

async function onDropChip(targetId: string, event: DragEvent): Promise<void> {
  if (!canDrag.value) return;
  event.preventDefault();
  const fromId = dragId.value ?? event.dataTransfer?.getData('text/plain');
  onDragEnd();
  if (!fromId || !targetId || fromId === targetId) return;

  const ids = props.barBookmarks.map((bookmark) => bookmark?.id).filter(Boolean) as string[];
  const fromIdx = ids.indexOf(fromId);
  const toIdx = ids.indexOf(targetId);
  if (toIdx < 0 || fromIdx < 0) return;

  const next = [...ids];
  next.splice(fromIdx, 1);
  next.splice(toIdx, 0, fromId);
  emit('reorder', next);
}

async function onDropFolder(folderName: string, event: DragEvent): Promise<void> {
  if (!canDrag.value) return;
  event.preventDefault();
  const fromId = dragId.value ?? event.dataTransfer?.getData('text/plain');
  onDragEnd();
  if (!fromId || !folderName) return;
  const dragged = props.bookmarks.find((bookmark) => bookmark && bookmark.id === fromId);
  if (!dragged || dragged.folder === folderName) return;
  emit('moveToFolder', fromId, folderName);
}

async function onDropBar(event: DragEvent): Promise<void> {
  if (!canDrag.value) return;
  event.preventDefault();
  const fromId = dragId.value ?? event.dataTransfer?.getData('text/plain');
  onDragEnd();
  if (!fromId) return;
  emit('moveToFolder', fromId, '');
}

function onDragEnd(): void {
  dragId.value = null;
  dragDepth.value = 0;
  clearDragHighlights();
}

</script>

<style scoped>
.bookmark-bar {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0 4px 0 2px;
  height: var(--chrome-bookmark-bar-height, 32px);
  min-height: var(--chrome-bookmark-bar-height, 32px);
  background: var(--chrome-toolbar-bg, #dee1e6);
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
  overflow: hidden;
}

.bookmark-bar--drag-over {
  outline: 1px dashed var(--color-primary, #1a73e8);
  outline-offset: -2px;
}

.bookmark-bar__lead {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 2px;
  padding: 0 2px;
}

.bookmark-lead-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--chrome-bookmark-lead-btn-size, 28px);
  height: var(--chrome-bookmark-lead-btn-size, 28px);
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--chrome-tab-text, #202124);
  cursor: pointer;
  flex-shrink: 0;
}

.bookmark-lead-btn:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.bookmark-lead-btn--active {
  color: var(--color-primary, #1a73e8);
  background: rgba(26, 115, 232, 0.12);
}

.bookmark-lead-icon {
  width: var(--chrome-bookmark-favicon-size, 16px);
  height: var(--chrome-bookmark-favicon-size, 16px);
  display: block;
}

.bookmark-bar-separator {
  width: 1px;
  height: 18px;
  margin: 0 4px;
  background: var(--chrome-bookmark-separator, rgba(0, 0, 0, 0.15));
  flex-shrink: 0;
}

.bookmark-bar__scroll {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  scrollbar-width: none;
  padding: 0 2px;
}

.bookmark-bar__scroll::-webkit-scrollbar {
  height: 0;
  width: 0;
}

.bookmark-bar__spacer {
  flex: 1 0 48px;
  min-width: 48px;
  align-self: stretch;
}

.bookmark-bar__end {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  margin-left: auto;
}

.bookmark-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--chrome-bookmark-item-gap, 6px);
  min-height: var(--chrome-bookmark-item-height, 22px);
  padding: var(--chrome-bookmark-item-padding-y, 2px) var(--chrome-bookmark-item-padding-x, 8px);
  border: none;
  border-radius: var(--chrome-bookmark-item-radius, 4px);
  background: transparent;
  color: var(--chrome-tab-text-active, #202124);
  font-size: var(--chrome-bookmark-font-size, 12px);
  line-height: 1.333;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: background-color 0.15s ease;
  user-select: none;
}

.bookmark-chip:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.bookmark-chip--drag-over {
  background: rgba(26, 115, 232, 0.12);
  box-shadow: inset 0 0 0 1px var(--color-primary, #1a73e8);
}

.bookmark-chip--overflow {
  min-width: 26px;
  justify-content: center;
  color: var(--chrome-tab-text, #202124);
  padding-inline: 6px;
}

.bookmark-overflow-icon {
  width: 14px;
  height: 14px;
  display: block;
  color: var(--chrome-tab-text, #202124);
}

.bookmark-favicon {
  width: var(--chrome-bookmark-favicon-size, 16px);
  height: var(--chrome-bookmark-favicon-size, 16px);
  flex-shrink: 0;
  border-radius: 2px;
  display: block;
}

.bookmark-favicon--fallback {
  color: var(--chrome-tab-text, #202124);
}

.bookmark-favicon--fallback svg {
  width: 100%;
  height: 100%;
  display: block;
}

.bookmark-label {
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: var(--chrome-bookmark-label-max-width, 148px);
  font-weight: 500;
}

.bookmark-groups-wrap {
  position: relative;
  flex-shrink: 0;
}

.bookmark-groups-menu {
  left: 0;
  top: calc(100% + 4px);
  min-width: 280px;
  padding: 6px 0;
}

.bookmark-groups-menu__create {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--chrome-tab-text-active, #202124);
  font-size: var(--chrome-bookmark-font-size, 12px);
  text-align: left;
  cursor: pointer;
}

.bookmark-groups-menu__create:hover,
.bookmark-groups-menu__group:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.bookmark-groups-menu__create-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-groups-menu__create-label {
  flex: 1;
}

.bookmark-groups-menu__shortcut {
  font-size: 11px;
  color: var(--chrome-tab-text, #5f6368);
  letter-spacing: 0.02em;
}

.bookmark-groups-menu__divider {
  height: 1px;
  margin: 4px 0;
  background: var(--chrome-divider, #dadce0);
}

.bookmark-groups-menu__group {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--chrome-tab-text-active, #202124);
  font-size: var(--chrome-bookmark-font-size, 12px);
  text-align: left;
  cursor: pointer;
}

.bookmark-groups-menu__group-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-groups-menu__group-count {
  font-size: 11px;
  color: var(--chrome-tab-text, #5f6368);
  flex-shrink: 0;
}

.bookmark-group-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--bookmark-group-color, #6b7280);
}

.bookmark-group-dot--chip {
  width: 10px;
  height: 10px;
}

.bookmark-group-prompt-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  border: none;
  background: rgba(0, 0, 0, 0.45);
  cursor: default;
}

.bookmark-group-prompt {
  position: fixed;
  left: 50%;
  top: 50%;
  z-index: 10002;
  transform: translate(-50%, -50%);
  width: min(360px, calc(100vw - 32px));
  padding: 20px;
  border-radius: 12px;
  background: var(--chrome-tab-bg-active, #fff);
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.2);
}

.bookmark-group-prompt h3 {
  margin: 0 0 14px;
  font-size: 16px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
}

.bookmark-group-prompt__field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
  font-size: 12px;
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-group-prompt__input {
  padding: 8px 10px;
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 6px;
  font-size: 14px;
}

.bookmark-group-prompt__input[aria-invalid='true'] {
  border-color: #d93025;
}

.bookmark-group-prompt__error {
  margin: 4px 0 0;
  font-size: 12px;
  color: #d93025;
}

.bookmark-group-prompt__color-label {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-group-prompt__colors {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.bookmark-group-prompt__swatch {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  background: var(--swatch, #6b7280);
  cursor: pointer;
}

.bookmark-group-prompt__swatch--selected {
  border-color: var(--chrome-tab-text-active, #202124);
  box-shadow: 0 0 0 2px var(--chrome-tab-bg-active, #fff);
}

.bookmark-group-prompt__actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.bookmark-group-prompt__btn {
  padding: 8px 16px;
  border-radius: 6px;
  border: none;
  font-size: 13px;
  cursor: pointer;
}

.bookmark-group-prompt__btn.secondary {
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-group-prompt__btn.primary {
  background: var(--color-primary, #1a73e8);
  color: #fff;
}

.bookmark-group-prompt__btn.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.bookmark-folder-wrap {
  position: relative;
  flex-shrink: 0;
}

.bookmark-folder-wrap--all {
  padding-right: 4px;
}

.bookmark-folder-icon {
  width: var(--chrome-bookmark-favicon-size, 16px);
  height: var(--chrome-bookmark-favicon-size, 16px);
  flex-shrink: 0;
  color: var(--chrome-tab-text, #202124);
}

.bookmark-menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
  border: none;
  background: transparent;
  cursor: default;
}

.bookmark-dropdown {
  position: fixed;
  top: calc(100% + 4px);
  left: 0;
  z-index: 1000;
  min-width: 240px;
  max-width: 360px;
  max-height: 420px;
  overflow-y: auto;
  padding: 4px;
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  background: var(--chrome-tab-bg-active, #ffffff);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bookmark-context-menu {
  position: fixed;
  z-index: 10000;
  min-width: 200px;
  max-width: 300px;
  padding: 4px;
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  background: var(--chrome-tab-bg-active, #ffffff);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bookmark-dropdown-empty {
  margin: 0;
  padding: 8px 12px;
  font-size: var(--chrome-bookmark-font-size, 12px);
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-dropdown--overflow,
.bookmark-dropdown--all {
  left: auto;
  right: 0;
}

.bookmark-dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--chrome-bookmark-item-gap, 6px);
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--chrome-tab-text-active, #202124);
  font-size: var(--chrome-bookmark-font-size, 12px);
  text-align: left;
  cursor: pointer;
}

.bookmark-dropdown-item:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.bookmark-dropdown-item--action {
  margin-top: 2px;
  border-top: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 0 0 4px 4px;
  color: var(--color-primary, #1a73e8);
  font-weight: 500;
}

.bookmark-dropdown-item--danger {
  color: #d93025;
}

.bookmark-dropdown-item--danger:hover {
  background: rgba(217, 48, 37, 0.08);
}

.bookmark-dropdown-folder-header {
  margin: 4px 0 2px 0;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--chrome-tab-text, #5f6368);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.bookmark-dropdown-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--chrome-divider, #dadce0);
}

.bookmark-dropdown-submenu-label {
  margin: 2px 0 0;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-dropdown-item--nested {
  padding-left: 28px;
}

@media (prefers-color-scheme: dark) {
  .bookmark-bar-separator {
    background: var(--chrome-bookmark-separator, rgba(255, 255, 255, 0.12));
  }

  .bookmark-chip {
    color: #e8eaed;
  }

  .bookmark-chip:hover,
  .bookmark-lead-btn:hover,
  .bookmark-dropdown-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .bookmark-lead-btn {
    color: #e8eaed;
  }

  .bookmark-folder-icon,
  .bookmark-favicon--fallback {
    color: #e8eaed;
  }

  .bookmark-dropdown {
    background: #292a2d;
    border-color: #5f6368;
  }

  .bookmark-dropdown-item {
    color: #e8eaed;
  }

  .bookmark-dropdown-item--action {
    border-top-color: #5f6368;
    color: #8ab4f8;
  }

  .bookmark-context-menu {
    background: #292a2d;
    border-color: #5f6368;
  }

  .bookmark-dropdown-item--danger {
    color: #f28b82;
  }

  .bookmark-dropdown-item--danger:hover {
    background: rgba(242, 139, 130, 0.12);
  }

  .bookmark-dropdown-folder-header {
    color: #9aa0a6;
  }

  .bookmark-groups-menu__create,
  .bookmark-groups-menu__group {
    color: #e8eaed;
  }

  .bookmark-groups-menu__create:hover,
  .bookmark-groups-menu__group:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .bookmark-groups-menu__divider {
    background: #5f6368;
  }

  .bookmark-group-prompt {
    background: #292a2d;
  }

  .bookmark-group-prompt h3 {
    color: #e8eaed;
  }

  .bookmark-group-prompt__input {
    background: #202124;
    border-color: #5f6368;
    color: #e8eaed;
  }

  .bookmark-group-prompt__btn.secondary {
    color: #9aa0a6;
  }
}
</style>
