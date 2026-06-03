import { ref, computed } from 'vue';

export interface Tag {
  id: string;
  name: string;
  color: string;
  createdAt: number;
}

export interface TaggedBookmark {
  bookmarkId: string;
  tagId: string;
}

const STORAGE_KEY = 'exodus-tags';
const TAGGED_BOOKMARKS_KEY = 'exodus-tagged-bookmarks';

const tags = ref<Tag[]>([]);
const taggedBookmarks = ref<TaggedBookmark[]>([]);

// Initialize tags from localStorage
function loadTags() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      tags.value = JSON.parse(stored);
    }
  } catch (e) {
    console.error('Failed to load tags:', e);
  }
}

// Save tags to localStorage
function saveTags() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tags.value));
  } catch (e) {
    console.error('Failed to save tags:', e);
  }
}

// Load tagged bookmarks
function loadTaggedBookmarks() {
  try {
    const stored = localStorage.getItem(TAGGED_BOOKMARKS_KEY);
    if (stored) {
      taggedBookmarks.value = JSON.parse(stored);
    }
  } catch (e) {
    console.error('Failed to load tagged bookmarks:', e);
  }
}

// Save tagged bookmarks
function saveTaggedBookmarks() {
  try {
    localStorage.setItem(TAGGED_BOOKMARKS_KEY, JSON.stringify(taggedBookmarks.value));
  } catch (e) {
    console.error('Failed to save tagged bookmarks:', e);
  }
}

// Generate a unique color for new tags
function generateColor(): string {
  const colors = [
    '#ea4335', // Red
    '#fbbc04', // Yellow
    '#34a853', // Green
    '#4285f4', // Blue
    '#9334e6', // Purple
    '#ff6d01', // Orange
    '#46bdc6', // Teal
    '#ff6b6b', // Pink
  ];
  return colors[Math.floor(Math.random() * colors.length)];
}

// Create a new tag
export function createTag(name: string, color?: string): Tag {
  const tag: Tag = {
    id: `tag-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    name,
    color: color || generateColor(),
    createdAt: Date.now(),
  };
  tags.value.push(tag);
  saveTags();
  return tag;
}

// Update a tag
export function updateTag(tagId: string, updates: Partial<Tag>): void {
  const index = tags.value.findIndex(t => t.id === tagId);
  if (index !== -1) {
    tags.value[index] = { ...tags.value[index], ...updates };
    saveTags();
  }
}

// Delete a tag
export function deleteTag(tagId: string): void {
  tags.value = tags.value.filter(t => t.id !== tagId);
  // Remove all associations with this tag
  taggedBookmarks.value = taggedBookmarks.value.filter(tb => tb.tagId !== tagId);
  saveTags();
  saveTaggedBookmarks();
}

// Add tag to bookmark
export function addTagToBookmark(bookmarkId: string, tagId: string): void {
  const exists = taggedBookmarks.value.some(
    tb => tb.bookmarkId === bookmarkId && tb.tagId === tagId
  );
  if (!exists) {
    taggedBookmarks.value.push({ bookmarkId, tagId });
    saveTaggedBookmarks();
  }
}

// Remove tag from bookmark
export function removeTagFromBookmark(bookmarkId: string, tagId: string): void {
  taggedBookmarks.value = taggedBookmarks.value.filter(
    tb => !(tb.bookmarkId === bookmarkId && tb.tagId === tagId)
  );
  saveTaggedBookmarks();
}

// Get all tags for a bookmark
export function getTagsForBookmark(bookmarkId: string): Tag[] {
  const tagIds = taggedBookmarks.value
    .filter(tb => tb.bookmarkId === bookmarkId)
    .map(tb => tb.tagId);
  return tags.value.filter(t => tagIds.includes(t.id));
}

// Get all bookmarks with a specific tag
export function getBookmarksWithTag(tagId: string): string[] {
  return taggedBookmarks.value
    .filter(tb => tb.tagId === tagId)
    .map(tb => tb.bookmarkId);
}

// Search tags by name
export function searchTags(query: string): Tag[] {
  const lowerQuery = query.toLowerCase();
  return tags.value.filter(t => t.name.toLowerCase().includes(lowerQuery));
}

// Get all tags
export function getTags(): Tag[] {
  return tags.value;
}

// Get tag by ID
export function getTagById(tagId: string): Tag | undefined {
  return tags.value.find(t => t.id === tagId);
}

/** Reset in-memory tag store (for unit tests). */
export function clearTagsState(): void {
  tags.value = [];
  taggedBookmarks.value = [];
}

// Initialize the tags system
export function useTags() {
  loadTags();
  loadTaggedBookmarks();

  return {
    tags: computed(() => tags.value),
    createTag,
    updateTag,
    deleteTag,
    addTagToBookmark,
    removeTagFromBookmark,
    getTagsForBookmark,
    getBookmarksWithTag,
    searchTags,
    getTags,
    getTagById,
  };
}
