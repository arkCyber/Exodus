<template>
  <div class="bookmark-stats">
    <div class="stats-header">
      <h3>Bookmark Statistics</h3>
      <button @click="refreshStats" class="refresh-btn" title="Refresh">
        <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <div class="stats-content">
      <!-- Overview Cards -->
      <div class="stats-overview">
        <div class="stat-card">
          <div class="stat-icon bookmarks">
            <svg viewBox="0 0 20 20" fill="currentColor">
              <path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z"/>
            </svg>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.totalBookmarks }}</div>
            <div class="stat-label">Total Bookmarks</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon folders">
            <svg viewBox="0 0 20 20" fill="currentColor">
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
            </svg>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.totalFolders }}</div>
            <div class="stat-label">Folders</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon tags">
            <svg viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M17.707 9.293a1 1 0 010 1.414l-7 7a1 1 0 01-1.414 0l-7-7A.997.997 0 012 10V5a3 3 0 013-3h5c.256 0 .512.098.707.293l7 7zM5 6a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"/>
            </svg>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.totalTags }}</div>
            <div class="stat-label">Tags</div>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon recent">
            <svg viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/>
            </svg>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.recentlyAdded }}</div>
            <div class="stat-label">Added This Week</div>
          </div>
        </div>
      </div>

      <!-- Detailed Stats -->
      <div class="stats-details">
        <!-- Folder Distribution -->
        <div class="stats-section">
          <h4>Folder Distribution</h4>
          <div v-if="stats.largestFolders.length > 0" class="stats-list">
            <div
              v-for="folder in stats.largestFolders"
              :key="folder.name"
              class="stats-item"
            >
              <div class="stats-item-info">
                <span class="stats-item-name">{{ folder.name || 'No folder' }}</span>
                <span class="stats-item-count">{{ folder.count }} bookmarks</span>
              </div>
              <div class="stats-item-bar">
                <div
                  class="stats-item-fill"
                  :style="{ width: `${(folder.count / stats.totalBookmarks) * 100}%` }"
                ></div>
              </div>
            </div>
          </div>
          <div v-else class="empty-state">No folders yet</div>
        </div>

        <!-- Tag Distribution -->
        <div class="stats-section">
          <h4>Tag Usage</h4>
          <div v-if="stats.mostUsedTags.length > 0" class="stats-list">
            <div
              v-for="tag in stats.mostUsedTags"
              :key="tag.name"
              class="stats-item"
            >
              <div class="stats-item-info">
                <span class="stats-item-name">{{ tag.name }}</span>
                <span class="stats-item-count">{{ tag.count }} bookmarks</span>
              </div>
              <div class="stats-item-bar">
                <div
                  class="stats-item-fill"
                  :style="{ width: `${(tag.count / Math.max(...Object.values(stats.bookmarksByTag))) * 100}%` }"
                ></div>
              </div>
            </div>
          </div>
          <div v-else class="empty-state">No tags yet</div>
        </div>

        <!-- Additional Metrics -->
        <div class="stats-section">
          <h4>Additional Metrics</h4>
          <div class="metrics-grid">
            <div class="metric-item">
              <span class="metric-label">Untagged Bookmarks</span>
              <span class="metric-value">{{ stats.untaggedBookmarks }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Bookmarks Without Folder</span>
              <span class="metric-value">{{ bookmarksWithoutFolder }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Bookmarks with Multiple Tags</span>
              <span class="metric-value">{{ bookmarksWithMultipleTags }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Average Tags per Bookmark</span>
              <span class="metric-value">{{ averageTagsPerBookmark }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useBookmarkStats } from '@/composables/useBookmarkStats';

const {
  stats,
  getFolderDistribution,
  getTagDistribution,
  bookmarksWithoutFolder,
  bookmarksWithMultipleTags,
  averageTagsPerBookmark,
} = useBookmarkStats();

function refreshStats(): void {
  // Stats are computed reactively, no explicit refresh needed
  // This function can be used to trigger any future refresh logic
}
</script>

<style scoped>
.bookmark-stats {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-secondary);
  border-radius: 8px;
  padding: 20px;
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid var(--border-color);
}

.stats-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.refresh-btn:hover {
  background: var(--bg-primary);
  color: var(--text-primary);
}

.refresh-btn svg {
  width: 16px;
  height: 16px;
}

.stats-content {
  flex: 1;
  overflow-y: auto;
}

.stats-overview {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: var(--bg-primary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.stat-icon.bookmarks {
  background: #3b82f6;
}

.stat-icon.folders {
  background: #f59e0b;
}

.stat-icon.tags {
  background: #10b981;
}

.stat-icon.recent {
  background: #8b5cf6;
}

.stat-icon svg {
  width: 20px;
  height: 20px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.stats-details {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stats-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.stats-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stats-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.stats-item-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stats-item-name {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}

.stats-item-count {
  font-size: 12px;
  color: var(--text-secondary);
}

.stats-item-bar {
  height: 6px;
  background: var(--border-color);
  border-radius: 3px;
  overflow: hidden;
}

.stats-item-fill {
  height: 100%;
  background: #3b82f6;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.empty-state {
  padding: 24px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
  background: var(--bg-primary);
  border-radius: 6px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.metric-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--bg-primary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.metric-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.metric-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

@media (max-width: 768px) {
  .stats-overview {
    grid-template-columns: repeat(2, 1fr);
  }

  .metrics-grid {
    grid-template-columns: 1fr;
  }
}
</style>
