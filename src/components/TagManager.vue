<template>
  <div class="tag-manager">
    <div class="tag-manager-header">
      <h3>Tags</h3>
      <button @click="showCreateDialog = true" class="create-tag-btn">
        + New Tag
      </button>
    </div>

    <div class="tag-list">
      <div
        v-for="tag in tags"
        :key="tag.id"
        class="tag-item"
        :style="{ borderLeftColor: tag.color }"
      >
        <span class="tag-name" :style="{ color: tag.color }">{{ tag.name }}</span>
        <div class="tag-actions">
          <button @click="editTag(tag)" class="tag-action-btn" title="Edit">
            ✎
          </button>
          <button @click="handleDeleteTag(tag.id)" class="tag-action-btn delete" title="Delete">
            ×
          </button>
        </div>
      </div>
      <div v-if="tags.length === 0" class="empty-state">
        No tags yet. Create one to get started.
      </div>
    </div>

    <!-- Create/Edit Tag Dialog -->
    <div v-if="showCreateDialog || showEditDialog" class="dialog-overlay" @click="closeDialog">
      <div class="dialog" @click.stop>
        <h3>{{ showEditDialog ? 'Edit Tag' : 'Create Tag' }}</h3>
        <div class="dialog-form">
          <div class="form-group">
            <label for="tag-name">Name</label>
            <input
              id="tag-name"
              v-model="tagName"
              type="text"
              placeholder="Tag name"
              class="form-input"
            />
          </div>
          <div class="form-group">
            <label for="tag-color">Color</label>
            <div class="color-picker">
              <button
                v-for="color in availableColors"
                :key="color"
                :class="['color-option', { selected: tagColor === color }]"
                :style="{ backgroundColor: color }"
                @click="tagColor = color"
              ></button>
            </div>
          </div>
        </div>
        <div class="dialog-actions">
          <button @click="closeDialog" class="dialog-btn cancel">Cancel</button>
          <button @click="saveTag" class="dialog-btn primary">
            {{ showEditDialog ? 'Save' : 'Create' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useTags, type Tag } from '@/composables/useTags';

const {
  tags,
  createTag,
  updateTag,
  deleteTag: deleteTagFromComposable,
} = useTags();

const showCreateDialog = ref(false);
const showEditDialog = ref(false);
const editingTag = ref<Tag | null>(null);
const tagName = ref('');
const tagColor = ref('');

const availableColors = [
  '#ea4335', // Red
  '#fbbc04', // Yellow
  '#34a853', // Green
  '#4285f4', // Blue
  '#9334e6', // Purple
  '#ff6d01', // Orange
  '#46bdc6', // Teal
  '#ff6b6b', // Pink
];

function editTag(tag: Tag) {
  editingTag.value = tag;
  tagName.value = tag.name;
  tagColor.value = tag.color;
  showEditDialog.value = true;
}

function handleDeleteTag(tagId: string) {
  if (confirm('Are you sure you want to delete this tag?')) {
    deleteTagFromComposable(tagId);
  }
}

function closeDialog() {
  showCreateDialog.value = false;
  showEditDialog.value = false;
  editingTag.value = null;
  tagName.value = '';
  tagColor.value = '';
}

function saveTag() {
  if (!tagName.value.trim()) {
    alert('Please enter a tag name');
    return;
  }

  if (showEditDialog.value && editingTag.value) {
    updateTag(editingTag.value.id, {
      name: tagName.value,
      color: tagColor.value,
    });
  } else {
    createTag(tagName.value, tagColor.value);
  }

  closeDialog();
}
</script>

<style scoped>
.tag-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-primary);
}

.tag-manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--color-border-primary);
}

.tag-manager-header h3 {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0;
}

.create-tag-btn {
  padding: 6px 12px;
  border: none;
  background: var(--color-primary);
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.2s ease;
}

.create-tag-btn:hover {
  background: var(--color-primary-hover);
}

.tag-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.tag-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--color-bg-secondary);
  border-left: 4px solid;
  border-radius: 4px;
  margin-bottom: 6px;
  transition: background 0.2s ease;
}

.tag-item:hover {
  background: var(--color-bg-hover);
}

.tag-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.tag-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.tag-item:hover .tag-actions {
  opacity: 1;
}

.tag-action-btn {
  padding: 4px 8px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  color: var(--color-text-secondary);
  transition: background 0.2s ease;
}

.tag-action-btn:hover {
  background: var(--color-bg-tertiary);
}

.tag-action-btn.delete:hover {
  background: #fee2e2;
  color: #dc2626;
}

.empty-state {
  padding: 32px;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 14px;
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
  background: var(--color-bg-primary);
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  width: 400px;
  max-width: 90vw;
  padding: 24px;
}

.dialog h3 {
  font-size: 18px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0 0 16px 0;
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
  color: var(--color-text-secondary);
  margin-bottom: 6px;
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border-primary);
  border-radius: 4px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-bg-primary);
  transition: border-color 0.2s ease;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.color-picker {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.color-option {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
}

.color-option:hover {
  transform: scale(1.1);
}

.color-option.selected {
  border-color: var(--color-text-primary);
  box-shadow: 0 0 0 2px var(--color-bg-primary), 0 0 0 4px var(--color-primary);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: background 0.2s ease;
}

.dialog-btn.cancel {
  background: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.dialog-btn.cancel:hover {
  background: var(--color-bg-hover);
}

.dialog-btn.primary {
  background: var(--color-primary);
  color: white;
}

.dialog-btn.primary:hover {
  background: var(--color-primary-hover);
}
</style>
