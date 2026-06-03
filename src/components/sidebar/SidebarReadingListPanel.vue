<!--
  Exodus Browser — reading list (Firefox sidebar; backed by local Pocket saves).
-->
<template>
  <div class="list-panel reading-panel exodus-sidebar-panel">
    <div class="reading-list-header">
      <select
        v-model="selectedCategory"
        class="category-select"
        aria-label="Filter by category"
        @change="() => void refresh()"
      >
        <option value="">All Categories</option>
        <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
      </select>
      <select
        v-model="sortBy"
        class="sort-select"
        aria-label="Sort by"
        @change="() => void refresh()"
      >
        <option value="saved_at">Newest First</option>
        <option value="priority">Priority</option>
        <option value="reading_time">Reading Time</option>
      </select>
    </div>
    <input
      v-model="searchQuery"
      type="search"
      class="search-input"
      placeholder="Search reading list…"
      aria-label="Search reading list"
    />
    <div v-if="stats" class="reading-list-stats">
      <span class="stat-item">{{ stats.total_articles }} total</span>
      <span class="stat-item">{{ stats.unread_articles }} unread</span>
      <span class="stat-item">{{ stats.in_progress_articles }} in progress</span>
    </div>
    <button type="button" class="nav-button secondary full" :disabled="loading" @click="() => void refresh()">
      Refresh
    </button>
    <p v-if="loading" class="muted">Loading…</p>
    <div
      v-for="article in filtered"
      :key="article.id"
      class="list-item"
      role="link"
      tabindex="0"
      @click="() => article.url ? emit('navigate', article.url) : null"
    >
      <div class="list-item-header">
        <div class="list-title">{{ article.title || article.url }}</div>
        <div class="priority-badge" :class="`priority-${article.reading_priority}`">
          P{{ article.reading_priority }}
        </div>
      </div>
      <div class="list-sub">{{ formatDate(article.saved_at) }}</div>
      <div v-if="article.reading_progress > 0" class="progress-bar">
        <div class="progress-fill" :style="{ width: article.reading_progress + '%' }"></div>
      </div>
      <div class="list-meta">
        <span>{{ article.reading_time_minutes }} min read</span>
        <span v-if="article.reading_list_category" class="category-tag">{{ article.reading_list_category }}</span>
      </div>
    </div>
    <p v-if="!loading && filtered.length === 0" class="muted">
      No items yet. Use the menu <strong>Save to reading list</strong> on a page.
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  pocketGetReadingList,
  pocketGetReadingListStats,
  pocketGetReadingListCategories,
  type SavedArticle,
  type ReadingListStats,
} from '$lib/localPocket';

const emit = defineEmits<{ navigate: [url: string] }>();

const articles = ref<SavedArticle[]>([]);
const categories = ref<string[]>([]);
const stats = ref<ReadingListStats | null>(null);
const searchQuery = ref('');
const selectedCategory = ref('');
const sortBy = ref('saved_at');
const loading = ref(true);

const filtered = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  let result = articles.value;
  
  if (q) {
    result = result.filter(
      (a) =>
        (a.title ?? '').toLowerCase().includes(q) ||
        a.url.toLowerCase().includes(q) ||
        (a.reading_list_category ?? '').toLowerCase().includes(q),
    );
  }
  
  return result;
});

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString();
  } catch {
    return dateStr;
  }
}

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    const [articlesData, statsData, categoriesData] = await Promise.all([
      pocketGetReadingList({
        category: selectedCategory.value || undefined,
        sort_by: sortBy.value,
      }),
      pocketGetReadingListStats(),
      pocketGetReadingListCategories(),
    ]);
    articles.value = articlesData;
    stats.value = statsData;
    categories.value = categoriesData;
  } catch (error) {
    console.error('SidebarReadingListPanel refresh failed:', error);
    articles.value = [];
    stats.value = null;
    categories.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<style scoped>
.reading-list-header {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.category-select,
.sort-select {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-color);
  color: var(--text-color);
  font-size: 13px;
}

.reading-list-stats {
  display: flex;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--text-muted);
}

.stat-item {
  display: flex;
  align-items: center;
}

.list-item-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}

.priority-badge {
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.priority-1 {
  background: #e0e0e0;
  color: #666;
}

.priority-2 {
  background: #b3d9ff;
  color: #0066cc;
}

.priority-3 {
  background: #d4edda;
  color: #155724;
}

.priority-4 {
  background: #fff3cd;
  color: #856404;
}

.priority-5 {
  background: #f8d7da;
  color: #721c24;
}

.progress-bar {
  height: 4px;
  background: var(--border-color);
  border-radius: 2px;
  margin: 6px 0;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-color);
  transition: width 0.3s ease;
}

.list-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

.category-tag {
  background: var(--border-color);
  padding: 2px 6px;
  border-radius: 3px;
}
</style>
