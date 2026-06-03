<script lang="ts">
  /**
   * Exodus Browser — Local Pocket (本地文章保存) UI panel
   */
  import {
    pocketSaveArticle,
    pocketListArticles,
    pocketGetArticle,
    pocketUpdateArticle,
    pocketMarkAsRead,
    pocketDeleteArticle,
    pocketSearchArticles,
    pocketGetArticlesByTag,
    pocketGetAllTags,
    pocketGetStats,
    type SavedArticle,
    type SaveArticleRequest,
    type UpdateArticleRequest,
    type PocketStats,
  } from '$lib/localPocket';

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let articles = $state<SavedArticle[]>([]);
  let currentArticle = $state<SavedArticle | null>(null);
  let stats = $state<PocketStats | null>(null);
  let tags = $state<string[]>([]);
  let searchQuery = $state('');
  let selectedTag = $state<string | null>(null);
  let loading = $state(true);
  let showSaveDialog = $state(false);
  let showReadView = $state(false);

  // Form state for saving articles
  let saveForm = $state<SaveArticleRequest>({
    url: '',
    title: '',
    content: '',
    author: null,
    tags: [],
  });
  let tagInput = $state('');

  function formatDate(dateStr: string): string {
    try {
      return new Date(dateStr).toLocaleString();
    } catch {
      return dateStr;
    }
  }

  function formatReadingTime(minutes: number): string {
    if (minutes < 1) return '< 1 min';
    if (minutes < 60) return `${Math.round(minutes)} min`;
    const hours = Math.floor(minutes / 60);
    const mins = Math.round(minutes % 60);
    return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
  }

  async function load() {
    loading = true;
    try {
      const [a, s, t] = await Promise.all([
        pocketListArticles(),
        pocketGetStats(),
        pocketGetAllTags(),
      ]);
      articles = a;
      stats = s;
      tags = t;
    } catch (error) {
      console.error('PocketPanel load failed:', error);
      onStatus('Failed to load pocket articles');
    } finally {
      loading = false;
    }
  }

  async function runSearch() {
    const q = searchQuery.trim();
    if (!q) {
      articles = await pocketListArticles();
      return;
    }
    articles = await pocketSearchArticles({ query: q, limit: null, offset: null });
  }

  async function filterByTag(tag: string) {
    selectedTag = tag;
    articles = await pocketGetArticlesByTag(tag);
  }

  async function clearTagFilter() {
    selectedTag = null;
    articles = await pocketListArticles();
  }

  async function saveArticle() {
    try {
      const saved = await pocketSaveArticle(saveForm);
      articles = await pocketListArticles();
      stats = await pocketGetStats();
      tags = await pocketGetAllTags();
      showSaveDialog = false;
      saveForm = { url: '', title: '', content: '', author: null, tags: [] };
      tagInput = '';
      onStatus('Article saved successfully');
    } catch (error) {
      console.error('Save article failed:', error);
      onStatus('Failed to save article');
    }
  }

  async function openArticle(article: SavedArticle) {
    currentArticle = article;
    showReadView = true;
  }

  async function markAsRead(article: SavedArticle) {
    try {
      await pocketMarkAsRead(article.id);
      article.read_at = new Date().toISOString();
      stats = await pocketGetStats();
      onStatus('Article marked as read');
    } catch (error) {
      console.error('Mark as read failed:', error);
      onStatus('Failed to mark article as read');
    }
  }

  async function toggleFavorite(article: SavedArticle) {
    try {
      const update: UpdateArticleRequest = {
        id: article.id,
        title: null,
        tags: null,
        is_favorite: !article.is_favorite,
        is_archived: null,
      };
      const updated = await pocketUpdateArticle(update);
      const index = articles.findIndex((a) => a.id === article.id);
      if (index !== -1) {
        articles[index] = updated;
      }
      if (currentArticle?.id === article.id) {
        currentArticle = updated;
      }
      stats = await pocketGetStats();
      onStatus(updated.is_favorite ? 'Article favorited' : 'Article unfavorited');
    } catch (error) {
      console.error('Toggle favorite failed:', error);
      onStatus('Failed to update favorite status');
    }
  }

  async function deleteArticle(article: SavedArticle) {
    if (!confirm('Are you sure you want to delete this article?')) return;
    try {
      await pocketDeleteArticle(article.id);
      articles = articles.filter((a) => a.id !== article.id);
      stats = await pocketGetStats();
      if (currentArticle?.id === article.id) {
        currentArticle = null;
        showReadView = false;
      }
      onStatus('Article deleted');
    } catch (error) {
      console.error('Delete article failed:', error);
      onStatus('Failed to delete article');
    }
  }

  function addTag() {
    const tag = tagInput.trim();
    if (tag && !saveForm.tags.includes(tag)) {
      saveForm.tags.push(tag);
      tagInput = '';
    }
  }

  function removeTag(tag: string) {
    saveForm.tags = saveForm.tags.filter((t) => t !== tag);
  }

  function handleMarkAsRead() {
    if (currentArticle) {
      markAsRead(currentArticle);
    }
  }

  function handleToggleFavorite() {
    if (currentArticle) {
      toggleFavorite(currentArticle);
    }
  }

  load();
</script>

<div class="pocket-panel">
  <div class="pocket-header">
    <h2>📚 Local Pocket</h2>
    <button class="btn-primary" onclick={() => (showSaveDialog = true)}>+ Save Article</button>
  </div>

  {#if stats}
    <div class="pocket-stats">
      <div class="stat-item">
        <span class="stat-value">{stats.total_articles}</span>
        <span class="stat-label">Total</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{stats.unread_articles}</span>
        <span class="stat-label">Unread</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{stats.favorite_articles}</span>
        <span class="stat-label">Favorites</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{formatReadingTime(stats.total_reading_time_minutes)}</span>
        <span class="stat-label">Reading Time</span>
      </div>
    </div>
  {/if}

  <div class="pocket-controls">
    <input
      type="text"
      placeholder="Search articles..."
      bind:value={searchQuery}
      onkeydown={(e) => e.key === 'Enter' && runSearch()}
      class="search-input"
    />
    <button onclick={runSearch}>Search</button>
  </div>

  {#if selectedTag}
    <div class="tag-filter">
      <span>Filtering by: <strong>{selectedTag}</strong></span>
      <button onclick={clearTagFilter}>Clear</button>
    </div>
  {/if}

  <div class="tags-bar">
    {#each tags as tag}
      <button
        class="tag-chip {selectedTag === tag ? 'active' : ''}"
        onclick={() => filterByTag(tag)}
      >
        #{tag}
      </button>
    {/each}
  </div>

  {#if loading}
    <div class="loading">Loading articles...</div>
  {:else if articles.length === 0}
    <div class="empty-state">
      <p>No articles saved yet. Click "Save Article" to get started.</p>
    </div>
  {:else}
    <div class="articles-list">
      {#each articles as article}
        <div class="article-card" role="button" tabindex="0" onclick={() => openArticle(article)} onkeydown={(e) => e.key === 'Enter' && openArticle(article)}>
          <div class="article-header">
            <h3 class="article-title">{article.title}</h3>
            <div class="article-actions">
              <button
                class="icon-btn {article.is_favorite ? 'favorite' : ''}"
                onclick={(e) => { e.stopPropagation(); toggleFavorite(article); }}
                title="Toggle favorite"
              >
                {article.is_favorite ? '★' : '☆'}
              </button>
              <button
                class="icon-btn"
                onclick={(e) => { e.stopPropagation(); deleteArticle(article); }}
                title="Delete"
              >
                ×
              </button>
            </div>
          </div>
          <p class="article-excerpt">{article.excerpt}</p>
          <div class="article-meta">
            <span class="article-date">{formatDate(article.saved_at)}</span>
            <span class="article-tags">
              {#each article.tags as tag}
                <span class="mini-tag">#{tag}</span>
              {/each}
            </span>
            <span class="article-time">{formatReadingTime(article.reading_time_minutes)}</span>
            {#if !article.read_at}
              <span class="unread-badge">Unread</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if showSaveDialog}
    <div 
      class="modal-overlay" 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="save-dialog-title"
      onclick={() => (showSaveDialog = false)}
      onkeydown={(e) => e.key === 'Escape' && (showSaveDialog = false)}
      tabindex="-1"
    >
      <div class="modal" onclick={(e) => e.stopPropagation()} role="document">
        <h3 id="save-dialog-title">Save Article</h3>
        <div class="form-group">
          <label for="pocket-url">URL</label>
          <input id="pocket-url" type="url" bind:value={saveForm.url} placeholder="https://..." />
        </div>
        <div class="form-group">
          <label for="pocket-title">Title</label>
          <input id="pocket-title" type="text" bind:value={saveForm.title} placeholder="Article title" />
        </div>
        <div class="form-group">
          <label for="pocket-content">Content</label>
          <textarea id="pocket-content" bind:value={saveForm.content} placeholder="Article content..." rows="6"></textarea>
        </div>
        <div class="form-group">
          <label for="pocket-author">Author (optional)</label>
          <input id="pocket-author" type="text" bind:value={saveForm.author} placeholder="Author name" />
        </div>
        <div class="form-group">
          <label for="pocket-tags">Tags</label>
          <div class="tag-input-group">
            <input id="pocket-tags" type="text" bind:value={tagInput} placeholder="Add tag..." onkeydown={(e) => e.key === 'Enter' && addTag()} />
            <button onclick={addTag}>Add</button>
          </div>
          <div class="tags-list">
            {#each saveForm.tags as tag}
              <span class="tag-chip">
                #{tag}
                <button onclick={() => removeTag(tag)} aria-label={`Remove tag ${tag}`}>×</button>
              </span>
            {/each}
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn-secondary" onclick={() => (showSaveDialog = false)}>Cancel</button>
          <button class="btn-primary" onclick={saveArticle}>Save</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showReadView && currentArticle}
    <div 
      class="modal-overlay" 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="read-dialog-title"
      onclick={() => (showReadView = false)}
      onkeydown={(e) => e.key === 'Escape' && (showReadView = false)}
      tabindex="-1"
    >
      <div class="modal read-modal" onclick={(e) => e.stopPropagation()} role="document">
        <div class="read-header">
          <h2 id="read-dialog-title">{currentArticle.title}</h2>
          <button class="close-btn" onclick={() => (showReadView = false)} aria-label="Close">×</button>
        </div>
        <div class="read-meta">
          {#if currentArticle.author}
            <span>By {currentArticle.author}</span>
          {/if}
          <span>Saved {formatDate(currentArticle.saved_at)}</span>
          <span>{formatReadingTime(currentArticle.reading_time_minutes)}</span>
        </div>
        <div class="read-content">
          {currentArticle.content}
        </div>
        <div class="read-actions">
          <button class="btn-secondary" onclick={handleToggleFavorite}>
            {currentArticle.is_favorite ? '★ Unfavorite' : '☆ Favorite'}
          </button>
          <button class="btn-secondary" onclick={handleMarkAsRead}>
            ✓ Mark as Read
          </button>
          <button class="btn-secondary" onclick={() => currentArticle && deleteArticle(currentArticle)} aria-label="Delete article">🗑️ Delete</button>
          <button class="btn-primary" onclick={() => (showReadView = false)}>Close</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .pocket-panel {
    padding: 1rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .pocket-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .pocket-header h2 {
    margin: 0;
  }

  .pocket-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.5rem;
    margin-bottom: 1rem;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
  }

  .stat-item {
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: 1.5rem;
    font-weight: bold;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .pocket-controls {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .search-input {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid var(--border);
    border-radius: 4px;
  }

  .tag-filter {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
    margin-bottom: 0.5rem;
  }

  .tags-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-bottom: 1rem;
  }

  .tag-chip {
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .tag-chip.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .tag-chip button {
    background: none;
    border: none;
    cursor: pointer;
    margin-left: 0.25rem;
  }

  .articles-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .article-card {
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
    cursor: pointer;
    transition: transform 0.2s;
  }

  .article-card:hover {
    transform: translateY(-2px);
  }

  .article-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 0.5rem;
  }

  .article-title {
    margin: 0;
    font-size: 1rem;
  }

  .article-actions {
    display: flex;
    gap: 0.25rem;
  }

  .icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1.25rem;
    padding: 0.25rem;
  }

  .icon-btn.favorite {
    color: gold;
  }

  .article-excerpt {
    margin: 0.5rem 0;
    color: var(--text-muted);
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .article-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .mini-tag {
    background: var(--bg-tertiary);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
  }

  .unread-badge {
    background: var(--accent);
    color: white;
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    font-weight: bold;
  }

  .loading,
  .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--bg-primary);
    border-radius: 8px;
    padding: 1.5rem;
    max-width: 600px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
  }

  .modal h3 {
    margin-top: 0;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: bold;
  }

  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    box-sizing: border-box;
  }

  .tag-input-group {
    display: flex;
    gap: 0.5rem;
  }

  .tag-input-group input {
    flex: 1;
  }

  .tags-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-top: 0.5rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .btn-primary,
  .btn-secondary {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border: none;
    cursor: pointer;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .read-modal {
    max-width: 800px;
  }

  .read-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .read-header h2 {
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
  }

  .read-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    margin-bottom: 1rem;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .read-content {
    line-height: 1.6;
    margin-bottom: 1rem;
  }

  .read-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
