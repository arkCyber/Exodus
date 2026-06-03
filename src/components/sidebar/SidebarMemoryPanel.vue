<!--
  Exodus Browser — sidebar memory & browsing history panel.
-->
<template>
  <div class="list-panel">
    <input
      :value="memorySearchQuery"
      type="search"
      class="search-input"
      placeholder="Search history and memory…"
      aria-label="Search history and memory"
      @input="onSearchInput"
    />
    <div class="history-panel-actions">
      <button type="button" class="nav-button secondary" @click="emit('load-memory')">Refresh</button>
    </div>

    <h4 class="memory-section-title">Indexed memory ({{ indexedCount }})</h4>
    <p class="muted memory-section-hint">Pages saved for /ask search. Use menu → Index page.</p>
    <template v-if="indexedCount > 0">
      <div v-for="group in indexedMemoryGroups" :key="group.label">
        <p class="history-group-label">{{ group.label }}</p>
        <div v-for="page in group.pages" :key="page.id" class="list-item row">
          <div class="list-grow" role="link" tabindex="0" @click="() => page.url ? emit('navigate', page.url) : null">
            <div class="list-title">{{ page.title || page.url }}</div>
            <div class="list-sub">{{ page.url }}</div>
          </div>
          <button
            type="button"
            class="tab-close"
            aria-label="Remove from indexed memory"
            @click.stop="emit('remove-indexed', page.id)"
          >
            ×
          </button>
        </div>
      </div>
      <button type="button" class="nav-button secondary danger full" @click="emit('clear-indexed')">
        Clear indexed memory
      </button>
    </template>
    <p v-else class="muted">No indexed pages yet.</p>

    <h4 class="memory-section-title">Browsing history ({{ historyCount }})</h4>
    <template v-if="historyCount > 0">
      <div v-for="group in historyGroups" :key="group.label">
        <p class="history-group-label">{{ group.label }}</p>
        <div
          v-for="page in group.pages"
          :key="page.id"
          class="list-item"
          role="link"
          tabindex="0"
          @click="() => page.url ? emit('navigate', page.url) : null"
        >
          <div class="list-title">{{ page.title || page.url }}</div>
          <div class="list-sub">
            {{ page.url }}
            <template v-if="page.visit_count && page.visit_count > 1">
              · {{ page.visit_count }} visits
            </template>
          </div>
        </div>
      </div>
      <button type="button" class="nav-button secondary danger full" @click="emit('clear-history')">
        Clear browsing history
      </button>
    </template>
    <p v-else class="muted">No visits recorded yet.</p>
  </div>
</template>

<script setup lang="ts">
import type { HistoryGroup } from '$lib/historyGroups';

withDefaults(
  defineProps<{
    indexedMemoryGroups: HistoryGroup[];
    historyGroups: HistoryGroup[];
    indexedCount: number;
    historyCount: number;
    memorySearchQuery?: string;
  }>(),
  { memorySearchQuery: '' },
);

const emit = defineEmits<{
  navigate: [url: string];
  'load-memory': [];
  'remove-indexed': [id: string];
  'clear-indexed': [];
  'clear-history': [];
  'memory-search': [query: string];
}>();

function onSearchInput(e: Event): void {
  emit('memory-search', (e.target as HTMLInputElement).value);
}
</script>

<style scoped>
.list-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.memory-section-title {
  margin: 12px 0 4px;
  font-size: 13px;
  color: #e8eaed;
}

.memory-section-hint {
  margin: 0 0 8px;
  font-size: 11px;
}

.history-group-label {
  font-size: 11px;
  color: #9aa0a6;
  margin: 8px 0 4px;
}

.list-item {
  padding: 6px 0;
  cursor: pointer;
  font-size: 12px;
}

.list-item.row {
  display: flex;
  gap: 8px;
}

.list-grow {
  flex: 1;
  min-width: 0;
}

.list-title {
  color: #e8eaed;
}

.list-sub {
  color: #9aa0a6;
  font-size: 11px;
  word-break: break-all;
}

.tab-close {
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
}

.muted {
  color: #888;
  font-size: 12px;
}

.nav-button.secondary {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid #555;
  background: #333;
  color: #e0e0e0;
  cursor: pointer;
}

.nav-button.danger {
  border-color: #7f1d1d;
  color: #fca5a5;
}

.full {
  width: 100%;
  margin-top: 8px;
}
</style>
