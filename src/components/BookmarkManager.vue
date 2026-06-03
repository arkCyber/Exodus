<template>
  <div class="bookmark-manager">
    <div class="bookmark-manager-header">
      <h2>Bookmarks</h2>
      <div class="bookmark-manager-tabs">
        <button
          @click="activeTab = 'bookmarks'"
          class="tab-btn"
          :class="{ active: activeTab === 'bookmarks' }"
        >
          Bookmarks
        </button>
        <button
          @click="activeTab = 'stats'"
          class="tab-btn"
          :class="{ active: activeTab === 'stats' }"
        >
          Stats
        </button>
        <button
          @click="activeTab = 'sync'"
          class="tab-btn"
          :class="{ active: activeTab === 'sync' }"
        >
          Sync
        </button>
      </div>
      <div class="bookmark-manager-actions" v-if="activeTab === 'bookmarks'">
        <button @click="showImportDialog = true" class="action-btn" title="Import bookmarks">
          <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Import
        </button>
        <button @click="exportBookmarks" class="action-btn" title="Export bookmarks">
          <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Export
        </button>
        <button @click="showAddDialog = true" class="action-btn primary" title="Add bookmark">
          <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M10 5v10M5 10h10" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Add
        </button>
      </div>
    </div>

    <div class="bookmark-manager-search" v-if="activeTab === 'bookmarks'">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search bookmarks..."
        class="search-input"
      />
      <div class="search-filters">
        <select v-model="filterFolder" class="filter-select">
          <option value="">All folders</option>
          <option v-for="folder in folders" :key="folder" :value="folder">
            {{ folder }}
          </option>
        </select>
      </div>
    </div>

    <div class="bookmark-manager-content" v-if="activeTab === 'bookmarks'">
      <div class="bookmark-manager-sidebar">
        <div
          v-for="folder in allFolders"
          :key="folder"
          class="folder-item"
          :class="{ active: selectedFolder === folder }"
          @click="selectedFolder = folder"
        >
          <svg class="folder-icon" viewBox="0 0 20 20" fill="currentColor">
            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
          </svg>
          <span>{{ folder || 'No folder' }}</span>
          <span class="folder-count">{{ getBookmarkCount(folder) }}</span>
        </div>
      </div>

      <div class="bookmark-manager-list">
        <div v-if="filteredBookmarks.length === 0" class="empty-state">
          <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <p>No bookmarks found</p>
        </div>
        <div
          v-for="bookmark in filteredBookmarks"
          :key="bookmark.id"
          class="bookmark-item"
          @click="navigateToBookmark(bookmark.url)"
        >
          <img
            v-if="bookmark.favicon"
            :src="bookmark.favicon"
            class="bookmark-favicon"
            alt=""
          />
          <div v-else class="bookmark-favicon-placeholder">
            <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1">
              <circle cx="10" cy="10" r="8"/>
              <path d="M2 10h16M10 2v16"/>
            </svg>
          </div>
          <div class="bookmark-info">
            <div class="bookmark-title">{{ bookmark.title || bookmark.url }}</div>
            <div class="bookmark-url">{{ bookmark.url }}</div>
            <div v-if="bookmark.folder" class="bookmark-folder">{{ bookmark.folder }}</div>
          </div>
          <div class="bookmark-actions">
            <button @click.stop="editBookmark(bookmark)" class="icon-btn" title="Edit">
              <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
            <button @click.stop="deleteBookmark(bookmark.id)" class="icon-btn danger" title="Delete">
              <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Import Dialog -->
    <div v-if="showImportDialog" class="dialog-overlay" @click="showImportDialog = false">
      <div class="dialog" @click.stop>
        <h3>Import Bookmarks</h3>
        <div class="dialog-form">
          <div class="form-group">
            <label>Select file</label>
            <input
              ref="fileInput"
              type="file"
              accept=".html,.json"
              @change="handleFileSelect"
              class="form-input"
            />
          </div>
          <div class="form-group">
            <label>Format</label>
            <select v-model="importFormat" class="form-input">
              <option value="html">HTML (Netscape Bookmark File)</option>
              <option value="json">JSON</option>
            </select>
          </div>
        </div>
        <div class="dialog-actions">
          <button @click="showImportDialog = false" class="dialog-btn cancel">Cancel</button>
          <button @click="importBookmarks" class="dialog-btn primary">Import</button>
        </div>
      </div>
    </div>

    <!-- Add Bookmark Dialog -->
    <BookmarkEditor
      :visible="showAddDialog"
      :bookmark="null"
      :folders="folders"
      @close="showAddDialog = false"
      @save="onBookmarkSaved"
    />

    <!-- Edit Bookmark Dialog -->
    <BookmarkEditor
      :visible="showEditDialog"
      :bookmark="editingBookmark"
      :folders="folders"
      @close="showEditDialog = false"
      @save="onBookmarkSaved"
    />

    <!-- Sync Settings Tab -->
    <BookmarkSyncSettings v-if="activeTab === 'sync'" />

    <!-- Statistics Tab -->
    <BookmarkStats v-if="activeTab === 'stats'" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useBookmarks } from '@/composables/useBookmarks';
import BookmarkEditor from './BookmarkEditor.vue';
import BookmarkSyncSettings from './BookmarkSyncSettings.vue';
import BookmarkStats from './BookmarkStats.vue';
import type { BookmarkItem } from '@/lib/browserTypes';

const emit = defineEmits<{
  navigate: [url: string];
  close: [];
}>();

const {
  bookmarks,
  folders,
  addBookmark,
  removeBookmark,
  updateBookmark,
  importBookmarks: importBookmarksData,
  exportBookmarks: exportBookmarksData,
} = useBookmarks();

const searchQuery = ref('');
const filterFolder = ref('');
const selectedFolder = ref('');
const showImportDialog = ref(false);
const showAddDialog = ref(false);
const showEditDialog = ref(false);
const editingBookmark = ref<BookmarkItem | null>(null);
const importFormat = ref('html');
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFile = ref<File | null>(null);
const activeTab = ref<'bookmarks' | 'sync'>('bookmarks');

const allFolders = computed(() => ['', ...folders.value]);

const filteredBookmarks = computed(() => {
  let result = bookmarks.value;

  // Filter by folder
  if (selectedFolder.value !== '') {
    result = result.filter(b => b.folder === selectedFolder.value);
  } else if (filterFolder.value) {
    result = result.filter(b => b.folder === filterFolder.value);
  }

  // Filter by search query
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(b =>
      (b.title?.toLowerCase().includes(query) || b.url?.toLowerCase().includes(query))
    );
  }

  return result;
});

function getBookmarkCount(folder: string): number {
  if (folder === '') {
    return bookmarks.value.filter(b => !b.folder).length;
  }
  return bookmarks.value.filter(b => b.folder === folder).length;
}

function navigateToBookmark(url: string): void {
  if (!url) return;
  emit('navigate', url);
}

function editBookmark(bookmark: BookmarkItem): void {
  editingBookmark.value = bookmark;
  showEditDialog.value = true;
}

async function deleteBookmark(id: string): Promise<void> {
  if (confirm('Are you sure you want to delete this bookmark?')) {
    await removeBookmark(id);
  }
}

function handleFileSelect(event: Event): void {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files[0]) {
    selectedFile.value = target.files[0];
  }
}

async function importBookmarks(): Promise<void> {
  if (!selectedFile.value) {
    alert('Please select a file');
    return;
  }

  try {
    const content = await selectedFile.value.text();
    
    if (importFormat.value === 'json') {
      const data = JSON.parse(content);
      if (Array.isArray(data)) {
        importBookmarksData(data);
      }
    } else {
      // HTML import - parse Netscape bookmark format
      const parser = new DOMParser();
      const doc = parser.parseFromString(content, 'text/html');
      const links = doc.querySelectorAll('a');
      const importedBookmarks: BookmarkItem[] = [];
      
      links.forEach((link, index) => {
        const href = link.getAttribute('href');
        const title = link.textContent || '';
        if (href) {
          importedBookmarks.push({
            id: `imported-${Date.now()}-${index}`,
            title: title.trim(),
            url: href,
            created_at: new Date().toISOString(),
            createdAt: Date.now(),
          });
        }
      });
      
      if (importedBookmarks.length > 0) {
        importBookmarksData(importedBookmarks);
      }
    }
    
    showImportDialog.value = false;
    selectedFile.value = null;
    if (fileInput.value) {
      fileInput.value.value = '';
    }
    alert(`Imported ${importedBookmarksData ? 'bookmarks' : '0 bookmarks'}`);
  } catch (error) {
    console.error('Import failed:', error);
    alert('Failed to import bookmarks. Please check the file format.');
  }
}

function exportBookmarks(): void {
  try {
    const data = exportBookmarksData();
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `bookmarks-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error('Export failed:', error);
    alert('Failed to export bookmarks');
  }
}

function onBookmarkSaved(): void {
  showAddDialog.value = false;
  showEditDialog.value = false;
  editingBookmark.value = null;
}

onMounted(() => {
  // Initialize with all folders selected
  selectedFolder.value = '';
});
</script>

<style scoped>
.bookmark-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--chrome-tab-bg-active, #ffffff);
}

.bookmark-manager-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
  gap: 20px;
}

.bookmark-manager-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
}

.bookmark-manager-tabs {
  display: flex;
  gap: 4px;
}

.tab-btn {
  padding: 8px 16px;
  border: none;
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.tab-btn.active {
  background: rgba(26, 115, 232, 0.12);
  color: var(--color-primary, #1a73e8);
}

.bookmark-manager-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 6px;
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.action-btn.primary {
  background: var(--color-primary, #1a73e8);
  color: white;
  border-color: var(--color-primary, #1a73e8);
}

.action-btn.primary:hover {
  background: #1557b0;
}

.action-btn svg {
  width: 16px;
  height: 16px;
}

.bookmark-manager-search {
  display: flex;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

.search-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  border-radius: 6px;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
  background: var(--chrome-omnibox-bg, #ffffff);
  outline: none;
}

.search-input:focus {
  border-color: var(--color-primary, #1a73e8);
  box-shadow: 0 1px 6px rgba(32, 33, 36, 0.28);
}

.search-filters {
  display: flex;
  gap: 8px;
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  border-radius: 6px;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
  background: var(--chrome-omnibox-bg, #ffffff);
  outline: none;
}

.bookmark-manager-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.bookmark-manager-sidebar {
  width: 200px;
  border-right: 1px solid var(--chrome-divider, #dadce0);
  overflow-y: auto;
  padding: 8px 0;
}

.folder-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.folder-item:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.folder-item.active {
  background: rgba(26, 115, 232, 0.12);
  color: var(--color-primary, #1a73e8);
}

.folder-icon {
  width: 16px;
  height: 16px;
  color: var(--chrome-tab-text, #5f6368);
  flex-shrink: 0;
}

.folder-item span {
  flex: 1;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
}

.folder-count {
  font-size: 12px;
  color: var(--chrome-tab-text, #5f6368);
  background: var(--chrome-tab-bg-hover, #e8eaed);
  padding: 2px 6px;
  border-radius: 10px;
}

.bookmark-manager-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--chrome-tab-text, #5f6368);
}

.empty-state svg {
  width: 48px;
  height: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.bookmark-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.bookmark-item:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.bookmark-favicon {
  width: 16px;
  height: 16px;
  border-radius: 2px;
  flex-shrink: 0;
}

.bookmark-favicon-placeholder {
  width: 16px;
  height: 16px;
  border-radius: 2px;
  background: var(--chrome-tab-bg-hover, #e8eaed);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.bookmark-favicon-placeholder svg {
  width: 12px;
  height: 12px;
  color: var(--chrome-tab-text, #5f6368);
}

.bookmark-info {
  flex: 1;
  min-width: 0;
}

.bookmark-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bookmark-url {
  font-size: 12px;
  color: var(--chrome-tab-text, #5f6368);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bookmark-folder {
  font-size: 11px;
  color: var(--color-primary, #1a73e8);
  margin-top: 2px;
}

.bookmark-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.bookmark-item:hover .bookmark-actions {
  opacity: 1;
}

.icon-btn {
  padding: 4px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--chrome-tab-text, #5f6368);
  transition: background 0.15s ease;
}

.icon-btn:hover {
  background: rgba(0, 0, 0, 0.08);
}

.icon-btn.danger {
  color: #d93025;
}

.icon-btn.danger:hover {
  background: rgba(217, 48, 37, 0.12);
}

.icon-btn svg {
  width: 16px;
  height: 16px;
}

.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.dialog {
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  width: 450px;
  max-width: 90vw;
  padding: 24px;
}

.dialog h3 {
  margin: 0 0 16px 0;
  font-size: 18px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
}

.dialog-form {
  margin-bottom: 24px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--chrome-tab-text, #5f6368);
  margin-bottom: 6px;
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  border-radius: 6px;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
  background: var(--chrome-omnibox-bg, #ffffff);
  outline: none;
}

.form-input:focus {
  border-color: var(--color-primary, #1a73e8);
  box-shadow: 0 1px 6px rgba(32, 33, 36, 0.28);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 16px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.15s ease;
}

.dialog-btn.cancel {
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  border: 1px solid var(--chrome-divider, #dadce0);
}

.dialog-btn.cancel:hover {
  background: rgba(0, 0, 0, 0.04);
}

.dialog-btn.primary {
  background: var(--color-primary, #1a73e8);
  color: white;
}

.dialog-btn.primary:hover {
  background: #1557b0;
}

@media (prefers-color-scheme: dark) {
  .bookmark-manager {
    background: #2d2e30;
  }

  .bookmark-manager-header h2,
  .bookmark-title {
    color: #e8eaed;
  }

  .action-btn,
  .filter-select,
  .search-input {
    background: #292a2d;
    border-color: #5f6368;
    color: #e8eaed;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .search-input:focus {
    border-color: #8ab4f8;
  }

  .folder-item span {
    color: #e8eaed;
  }

  .folder-icon,
  .bookmark-favicon-placeholder svg {
    color: #9aa0a6;
  }

  .folder-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .folder-item.active {
    background: rgba(138, 180, 248, 0.12);
    color: #8ab4f8;
  }

  .folder-count {
    background: rgba(255, 255, 255, 0.08);
    color: #9aa0a6;
  }

  .bookmark-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .bookmark-url {
    color: #9aa0a6;
  }

  .icon-btn {
    color: #9aa0a6;
  }

  .icon-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .dialog {
    background: #2d2e30;
    border-color: #5f6368;
  }

  .dialog h3 {
    color: #e8eaed;
  }

  .form-group label {
    color: #9aa0a6;
  }

  .form-input {
    background: #292a2d;
    border-color: #5f6368;
    color: #e8eaed;
  }

  .dialog-btn.cancel {
    color: #9aa0a6;
    border-color: #5f6368;
  }
}
</style>
