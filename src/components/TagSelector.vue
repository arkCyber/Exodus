<template>
  <div class="tag-selector">
    <div class="selected-tags">
      <span
        v-for="tag in selectedTags"
        :key="tag.id"
        class="tag-chip"
        :style="{ backgroundColor: tag.color + '20', color: tag.color, borderColor: tag.color }"
      >
        {{ tag.name }}
        <button @click="removeTag(tag)" class="tag-chip-remove">×</button>
      </span>
      <button
        @click="showDropdown = !showDropdown"
        class="add-tag-btn"
        :class="{ active: showDropdown }"
      >
        + Add Tag
      </button>
    </div>

    <div v-if="showDropdown" class="tag-dropdown">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search or create tag..."
        class="tag-search"
        @click.stop
      />
      <div class="tag-options">
        <div
          v-for="tag in filteredTags"
          :key="tag.id"
          class="tag-option"
          :style="{ borderLeftColor: tag.color }"
          @click="addTag(tag)"
        >
          <span class="tag-option-name" :style="{ color: tag.color }">{{ tag.name }}</span>
          <span v-if="isTagSelected(tag)" class="tag-option-check">✓</span>
        </div>
        <div
          v-if="searchQuery && !tagExists(searchQuery)"
          class="tag-option create-new"
          @click="createNewTag"
        >
          <span class="tag-option-name">Create "{{ searchQuery }}"</span>
          <span class="tag-option-icon">+</span>
        </div>
        <div v-if="filteredTags.length === 0 && !searchQuery" class="no-tags">
          No tags available
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useTags, type Tag } from '@/composables/useTags';

interface Props {
  bookmarkId?: string;
}

const props = defineProps<Props>();

const {
  tags,
  getTagsForBookmark,
  addTagToBookmark,
  removeTagFromBookmark,
  createTag,
  searchTags,
} = useTags();

const showDropdown = ref(false);
const searchQuery = ref('');
const selectedTags = ref<Tag[]>([]);

const filteredTags = computed(() => {
  if (searchQuery.value) {
    return searchTags(searchQuery.value);
  }
  return tags.value;
});

function isTagSelected(tag: Tag): boolean {
  return selectedTags.value.some(t => t.id === tag.id);
}

function tagExists(name: string): boolean {
  return tags.value.some(t => t.name.toLowerCase() === name.toLowerCase());
}

function addTag(tag: Tag) {
  if (!isTagSelected(tag)) {
    selectedTags.value.push(tag);
    if (props.bookmarkId) {
      addTagToBookmark(props.bookmarkId, tag.id);
    }
  }
  searchQuery.value = '';
}

function removeTag(tag: Tag) {
  selectedTags.value = selectedTags.value.filter(t => t.id !== tag.id);
  if (props.bookmarkId) {
    removeTagFromBookmark(props.bookmarkId, tag.id);
  }
}

function createNewTag() {
  if (searchQuery.value.trim()) {
    const newTag = createTag(searchQuery.value.trim());
    addTag(newTag);
    searchQuery.value = '';
  }
}

function loadTagsForBookmark() {
  if (props.bookmarkId) {
    selectedTags.value = getTagsForBookmark(props.bookmarkId);
  }
}

function handleClickOutside(_e: MouseEvent) {
  if (showDropdown.value) {
    showDropdown.value = false;
  }
}

onMounted(() => {
  loadTagsForBookmark();
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.tag-selector {
  position: relative;
}

.selected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border-radius: 12px;
  border: 1px solid;
  font-size: 12px;
  font-weight: 500;
  gap: 4px;
  transition: all 0.2s ease;
}

.tag-chip-remove {
  padding: 0 4px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.tag-chip-remove:hover {
  opacity: 1;
}

.add-tag-btn {
  padding: 4px 10px;
  border: 1px dashed var(--color-border-primary);
  background: transparent;
  border-radius: 12px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.add-tag-btn:hover,
.add-tag-btn.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.tag-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border-primary);
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  z-index: 1000;
  overflow: hidden;
}

.tag-search {
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-bottom: 1px solid var(--color-border-tertiary);
  font-size: 13px;
  color: var(--color-text-primary);
  background: var(--color-bg-primary);
  outline: none;
}

.tag-search::placeholder {
  color: var(--color-text-tertiary);
}

.tag-options {
  max-height: 200px;
  overflow-y: auto;
}

.tag-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-left: 3px solid;
  cursor: pointer;
  transition: background 0.2s ease;
}

.tag-option:hover {
  background: var(--color-bg-hover);
}

.tag-option.create-new {
  border-left: none;
  color: var(--color-primary);
}

.tag-option-name {
  font-size: 13px;
  color: var(--color-text-primary);
}

.tag-option-check {
  font-size: 14px;
  color: var(--color-primary);
}

.tag-option-icon {
  font-size: 16px;
  color: var(--color-primary);
}

.no-tags {
  padding: 16px;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 13px;
}
</style>
