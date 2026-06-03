<!--
  Exodus Browser — full history manager settings section.
-->
<template>
  <section id="settings-section-history" class="settings-section" data-testid="history-manager-panel">
    <h3>{{ ui.title }}</h3>
    <p class="hint">{{ ui.hint }}</p>
    <template v-if="settings">
      <label class="checkbox-row">
        <input v-model="settings.enabled" type="checkbox" data-testid="history-manager-enabled" @change="() => void persist()" />
        <span>{{ ui.enable }}</span>
      </label>
      <label class="checkbox-row">
        <input v-model="settings.remember_browsing" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.remember }}</span>
      </label>
      <label>
        {{ ui.retention }}
        <input v-model.number="settings.retention_days" type="number" min="0" max="3650" @change="() => void persist()" />
      </label>
    </template>
    <p v-if="stats?.total_entries != null" class="hint">
      {{ stats.total_entries }} entries · {{ stats.unique_domains ?? 0 }} domains
    </p>
    <div class="toolbar">
      <input
        v-model="searchQuery"
        type="search"
        data-testid="history-manager-search"
        :placeholder="ui.searchPlaceholder"
        class="field"
        @keydown.enter="() => void runSearch()"
      />
      <button type="button" class="nav-button secondary" @click="() => void runSearch()">{{ ui.search }}</button>
      <button type="button" class="nav-button secondary" @click="() => void load()">{{ ui.refresh }}</button>
    </div>
    <ul v-if="entries?.length" class="list" data-testid="history-manager-list">
      <li v-for="e in entries" :key="e.id" class="row">
        <div>
          <strong>{{ e.title || e.url }}</strong>
          <span class="muted">{{ e.url }}</span>
        </div>
        <button type="button" class="nav-button secondary" @click="() => void deleteEntry(e.id)">{{ ui.remove }}</button>
      </li>
    </ul>
    <p v-else-if="!loading" class="hint">{{ ui.empty }}</p>
    <button
      type="button"
      class="nav-button secondary danger full"
      data-testid="history-manager-clear-all"
      @click="() => void clearAll()"
    >
      {{ confirmClear ? ui.confirmClear : ui.clearAll }}
    </button>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — managed browsing history store (settings panel).
 */
import { ref, computed, onMounted } from 'vue';
import {
  clearAllManagedHistory,
  getManagedHistoryStats,
  getRecentManagedHistory,
  loadHistoryManagerSettings,
  removeManagedHistoryEntry,
  saveHistoryManagerSettings,
  searchManagedHistory,
  type HistoryManagerSettings,
  type ManagedHistoryEntry,
} from '$lib/historyManager';
import { type AppLocale } from '@/lib/appLocale';
import { historyManagerSettingsStrings } from '@/lib/historyManagerSettingsUi';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{ status: [message: string] }>();

const ui = computed(() => historyManagerSettingsStrings(props.uiLocale));

const entries = ref<ManagedHistoryEntry[]>([]);
const settings = ref<HistoryManagerSettings | null>(null);
const stats = ref<Record<string, number>>({});
const searchQuery = ref('');
const loading = ref(true);
const confirmClear = ref(false);

/** Load history entries, settings, and stats. */
async function load(): Promise<void> {
  loading.value = true;
  try {
    const [recent, s, st] = await Promise.all([
      getRecentManagedHistory(80),
      loadHistoryManagerSettings(),
      getManagedHistoryStats(),
    ]);
    entries.value = recent ?? [];
    settings.value = s;
    stats.value = st ?? {};
  } catch (error) {
    console.error('HistoryManagerSettings load failed:', error);
    emit('status', 'Failed to load history');
  } finally {
    loading.value = false;
  }
}

/** Search managed history by query. */
async function runSearch(): Promise<void> {
  const q = searchQuery.value.trim();
  entries.value = q ? await searchManagedHistory(q) : await getRecentManagedHistory(80);
}

/** Persist history manager settings. */
async function persist(): Promise<void> {
  if (!settings.value) return;
  try {
    await saveHistoryManagerSettings(settings.value);
    emit('status', 'History settings saved');
  } catch (error) {
    console.error('saveHistoryManagerSettings failed:', error);
  }
}

/** Remove one history entry. */
async function deleteEntry(id: string): Promise<void> {
  try {
    await removeManagedHistoryEntry(id);
    entries.value = entries.value.filter((e) => e.id !== id);
    emit('status', 'Entry removed');
  } catch (error) {
    console.error('removeManagedHistoryEntry failed:', error);
  }
}

/** Clear all managed history (double-click confirm). */
async function clearAll(): Promise<void> {
  if (!confirmClear.value) {
    confirmClear.value = true;
    return;
  }
  try {
    await clearAllManagedHistory();
    entries.value = [];
    confirmClear.value = false;
    stats.value = await getManagedHistoryStats();
    emit('status', 'History cleared');
  } catch (error) {
    console.error('clearAllManagedHistory failed:', error);
  }
}

onMounted(() => void load());
</script>

<style scoped>
.hint { font-size: 12px; color: var(--color-text-secondary, #888); margin-bottom: 8px; }
.toolbar { display: flex; gap: 8px; margin-bottom: 8px; flex-wrap: wrap; }
.field { flex: 1; min-width: 140px; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--color-border, #404040); }
.list { list-style: none; padding: 0; margin: 0; max-height: 200px; overflow-y: auto; display: flex; flex-direction: column; gap: 6px; }
.row { display: flex; justify-content: space-between; gap: 8px; padding: 6px; background: var(--color-bg-secondary, #2a2a2a); border-radius: 6px; font-size: 12px; }
.muted { display: block; color: var(--color-text-secondary, #888); word-break: break-all; }
.checkbox-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.nav-button { padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-accent, #2563eb); color: #fff; }
.nav-button.secondary { background: var(--color-bg-tertiary, #404040); }
.nav-button.danger { background: #7f1d1d; }
.nav-button.full { width: 100%; margin-top: 8px; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }
</style>
