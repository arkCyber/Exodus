<!--
  Exodus Browser — downloads overlay panel (Chrome-style actions per item).
-->
<template>
  <BrowserPanel :open="showDownloads" title="Downloads" @close="emit('close')">
    <div v-if="downloads.length > 0" class="downloads-list">
      <div v-for="download in downloads" :key="download.id" class="download-item">
        <div class="download-row">
          <span class="download-name">{{ download.filename }}</span>
          <span
            class="download-status"
            :class="{
              done: download.status === 'completed',
              failed: download.status === 'failed',
            }"
          >
            {{ download.status }}
            <template v-if="download.status === 'downloading' && download.total > 0">
              · {{ download.progress.toFixed(0) }}%
            </template>
          </span>
        </div>
        <div
          v-if="download.status === 'downloading' || download.status === 'pending'"
          class="download-progress-track"
        >
          <div
            class="download-progress-bar"
            :style="{ width: `${Math.max(download.progress, 2)}%` }"
          />
        </div>
        <div v-if="download.path && download.status === 'completed'" class="download-item-actions">
          <button type="button" class="download-action" @click="emit('open-file', download.path!)">
            Open
          </button>
          <button
            type="button"
            class="download-action secondary"
            @click="emit('reveal-file', download.path!)"
          >
            Show in folder
          </button>
        </div>
      </div>
      <div class="download-actions">
        <button type="button" class="nav-button secondary" @click="emit('open-folder')">
          Open folder
        </button>
        <button type="button" class="nav-button secondary" @click="emit('clear')">Clear list</button>
      </div>
    </div>
    <div v-else class="empty-state">
      No downloads yet. Use menu → Save page as download.
    </div>
  </BrowserPanel>
</template>

<script setup lang="ts">
import BrowserPanel from '@/components/BrowserPanel.vue';
import type { DownloadRecord } from '$lib/browserTypes';

defineProps<{
  showDownloads: boolean;
  downloads: DownloadRecord[];
}>();

const emit = defineEmits<{
  close: [];
  'open-folder': [];
  clear: [];
  'open-file': [path: string];
  'reveal-file': [path: string];
}>();
</script>

<style scoped>
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 14px;
}

@media (prefers-color-scheme: dark) {
  .empty-state {
    color: #9aa0a6;
  }
}

.downloads-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.download-item {
  padding: 12px 0;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

@media (prefers-color-scheme: dark) {
  .download-item {
    border-color: #5f6368;
  }
}

.download-row {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
}

.download-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--chrome-tab-text-active, #202124);
}

@media (prefers-color-scheme: dark) {
  .download-name {
    color: #e8eaed;
  }
}

.download-progress-track {
  height: 4px;
  background: var(--chrome-divider, #dadce0);
  border-radius: 2px;
  margin-top: 6px;
  overflow: hidden;
}

@media (prefers-color-scheme: dark) {
  .download-progress-track {
    background: #5f6368;
  }
}

.download-progress-bar {
  height: 100%;
  background: var(--color-primary, #1a73e8);
  transition: width 0.2s ease;
}

.download-status {
  color: var(--chrome-tab-text, #5f6368);
  font-size: 12px;
  padding: 4px 8px;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 12px;
  flex-shrink: 0;
}

@media (prefers-color-scheme: dark) {
  .download-status {
    color: #9aa0a6;
    background: rgba(255, 255, 255, 0.08);
  }
}

.download-status.done {
  color: #137333;
}

.download-status.failed {
  color: #d93025;
}

.download-item-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.download-action {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 16px;
  border: 1px solid var(--color-primary, #1a73e8);
  background: rgba(26, 115, 232, 0.1);
  color: var(--color-primary, #1a73e8);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.download-action:hover {
  background: rgba(26, 115, 232, 0.15);
}

.download-action.secondary {
  border-color: var(--chrome-divider, #dadce0);
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .download-action.secondary {
    border-color: #5f6368;
    color: #9aa0a6;
  }
}

.download-action.secondary:hover {
  background: rgba(0, 0, 0, 0.04);
}

@media (prefers-color-scheme: dark) {
  .download-action.secondary:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

.download-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}

.nav-button.secondary {
  padding: 6px 12px;
  border-radius: 16px;
  border: 1px solid var(--chrome-divider, #dadce0);
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .nav-button.secondary {
    border-color: #5f6368;
    color: #9aa0a6;
  }
}

.nav-button.secondary:hover {
  background: rgba(0, 0, 0, 0.04);
}

@media (prefers-color-scheme: dark) {
  .nav-button.secondary:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}
</style>
