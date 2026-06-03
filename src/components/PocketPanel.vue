<!--
  Exodus Browser — Local Pocket (saved articles) sidebar panel.
-->
<template>
  <div class="pocket-panel">
    <div class="pocket-toolbar">
      <input v-model="searchQuery" type="search" class="search-input" placeholder="Search saved articles…" />
      <button type="button" class="nav-button secondary" @click="() => void refresh()">Refresh</button>
    </div>
    <p v-if="loading" class="muted">Loading…</p>
    <ul v-else class="article-list">
      <li
        v-for="article in displayedArticles"
        :key="article.id"
        class="article-item"
        @click="selectArticle(article)"
      >
        <span class="article-title">{{ article.title || article.url }}</span>
        <span class="article-meta">{{ formatDate(article.saved_at) }}</span>
      </li>
      <li v-if="displayedArticles.length === 0" class="muted">No saved articles</li>
    </ul>
    <div v-if="currentArticle" class="article-detail">
      <h4>{{ currentArticle.title }}</h4>
      <a :href="currentArticle.url" target="_blank" rel="noopener">{{ currentArticle.url }}</a>
      <button type="button" class="nav-button secondary" @click="currentArticle = null">Close</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  pocketListArticles,
  pocketSearchArticles,
  type SavedArticle,
} from '$lib/localPocket';

const emit = defineEmits<{ status: [message: string] }>();

const articles = ref<SavedArticle[]>([]);
const currentArticle = ref<SavedArticle | null>(null);
const searchQuery = ref('');
const loading = ref(true);

const displayedArticles = computed(() => articles.value);

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleString();
  } catch {
    return dateStr;
  }
}

function selectArticle(article: SavedArticle): void {
  currentArticle.value = article;
}

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    const q = searchQuery.value.trim();
    articles.value = q
      ? await pocketSearchArticles({ query: q, limit: 50, offset: null })
      : await pocketListArticles();
  } catch (e) {
    emit('status', String(e));
    articles.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<style scoped>
.pocket-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
}

.pocket-toolbar {
  display: flex;
  gap: 6px;
}

.search-input {
  flex: 1;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid #404040;
  background: #1a1a1a;
  color: #e0e0e0;
  font-size: 12px;
}

.article-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
}

.article-item {
  padding: 8px;
  cursor: pointer;
  border-bottom: 1px solid #333;
  font-size: 12px;
}

.article-item:hover {
  background: rgba(99, 102, 241, 0.15);
}

.article-title {
  display: block;
}

.article-meta {
  font-size: 10px;
  color: #888;
}

.article-detail {
  padding: 8px;
  border-top: 1px solid #404040;
  font-size: 12px;
}

.muted {
  color: #888;
  font-size: 12px;
}

.nav-button.secondary {
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid #555;
  background: #333;
  color: #e0e0e0;
  cursor: pointer;
}
</style>
