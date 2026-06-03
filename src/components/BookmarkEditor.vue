<template>
  <div v-if="visible" class="dialog-overlay" @click="$emit('close')">
    <div class="dialog" @click.stop>
      <h3>{{ bookmark ? 'Edit Bookmark' : 'Add Bookmark' }}</h3>
      <div class="dialog-form">
        <div class="form-group">
          <label for="bookmark-title">Title</label>
          <input
            id="bookmark-title"
            v-model="form.title"
            type="text"
            placeholder="Bookmark title"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label for="bookmark-url">URL</label>
          <input
            id="bookmark-url"
            v-model="form.url"
            type="text"
            placeholder="https://example.com"
            class="form-input"
          />
        </div>
        <div class="form-group">
          <label for="bookmark-folder">Folder</label>
          <select id="bookmark-folder" v-model="form.folder" class="form-input">
            <option value="">No folder</option>
            <option v-for="folder in folders" :key="folder" :value="folder">
              {{ folder }}
            </option>
          </select>
        </div>
        <div class="form-group">
          <label for="bookmark-description">Description (optional)</label>
          <textarea
            id="bookmark-description"
            v-model="form.description"
            placeholder="Add a description for this bookmark..."
            class="form-textarea"
            rows="3"
          />
        </div>
        <div class="form-group">
          <label>Tags</label>
          <TagSelector :bookmark-id="bookmark?.id" />
        </div>
      </div>
      <div class="dialog-actions">
        <button @click="$emit('close')" class="dialog-btn cancel">Cancel</button>
        <button @click="save" class="dialog-btn primary">
          {{ bookmark ? 'Save' : 'Add' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import TagSelector from './TagSelector.vue';
import { useBookmarks } from '@/composables/useBookmarks';
import type { BookmarkItem } from '@/lib/browserTypes';

interface Props {
  visible?: boolean;
  bookmark?: BookmarkItem | null;
  folders?: string[];
  /** Prefill fields when adding a bookmark (e.g. from active tab). */
  draft?: { title?: string; url?: string } | null;
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  bookmark: null,
  folders: () => [],
  draft: null,
});

const emit = defineEmits<{
  close: [];
  save: [bookmark: BookmarkItem];
}>();

const { addBookmark, updateBookmark } = useBookmarks();

const form = ref({
  title: '',
  url: '',
  folder: '',
  description: '',
});

watch(
  () => [props.visible, props.bookmark, props.draft] as const,
  ([visible, bookmark, draft]) => {
    if (!visible) return;
    if (bookmark) {
      form.value = {
        title: bookmark.title,
        url: bookmark.url,
        folder: bookmark.folder || '',
        description: bookmark.description || '',
      };
      return;
    }
    form.value = {
      title: draft?.title?.trim() || '',
      url: draft?.url?.trim() || '',
      folder: '',
      description: '',
    };
  },
  { immediate: true },
);

function save() {
  if (!form.value.title.trim() || !form.value.url.trim()) {
    alert('Please fill in title and URL');
    return;
  }

  if (props.bookmark) {
    updateBookmark(props.bookmark.id, {
      title: form.value.title,
      url: form.value.url,
      folder: form.value.folder || undefined,
      description: form.value.description || undefined,
    });
    emit('save', { ...props.bookmark, ...form.value });
  } else {
    const saved = addBookmark(
      form.value.title,
      form.value.url,
      form.value.folder || undefined,
      undefined,
      form.value.description || undefined,
    );
    if (saved) {
      emit('save', saved);
    }
  }

  emit('close');
}
</script>

<style scoped>
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
  animation: backdropFadeIn 0.2s ease;
}

@keyframes backdropFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.dialog {
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  width: 450px;
  max-width: 90vw;
  padding: 24px;
  animation: modalSlideIn 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .dialog {
    background: #2d2e30;
    border-color: #5f6368;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dialog h3 {
  font-size: 18px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
  margin: 0 0 16px 0;
}

@media (prefers-color-scheme: dark) {
  .dialog h3 {
    color: #e8eaed;
  }
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

@media (prefers-color-scheme: dark) {
  .form-group label {
    color: #9aa0a6;
  }
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  border-radius: 8px;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
  background: var(--chrome-omnibox-bg, #ffffff);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
  outline: none;
}

.form-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  border-radius: 8px;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
  background: var(--chrome-omnibox-bg, #ffffff);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
  outline: none;
  resize: vertical;
  font-family: inherit;
}

.form-input:focus,
.form-textarea:focus {
  border-color: var(--color-primary, #1a73e8);
  box-shadow: 0 1px 6px rgba(32, 33, 36, 0.28);
}

@media (prefers-color-scheme: dark) {
  .form-input,
  .form-textarea {
    background: #292a2d;
    border-color: #5f6368;
    color: #e8eaed;
  }
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
  transition: background-color 0.15s ease;
}

.dialog-btn.cancel {
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  border: 1px solid var(--chrome-divider, #dadce0);
}

@media (prefers-color-scheme: dark) {
  .dialog-btn.cancel {
    color: #9aa0a6;
    border-color: #5f6368;
  }
}

.dialog-btn.cancel:hover {
  background: rgba(0, 0, 0, 0.04);
}

@media (prefers-color-scheme: dark) {
  .dialog-btn.cancel:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

.dialog-btn.primary {
  background: var(--color-primary, #1a73e8);
  color: white;
}

.dialog-btn.primary:hover {
  background: #1557b0;
}
</style>
