<!--
  Exodus Browser — cookie manager settings section.
-->
<template>
  <section class="settings-section" data-testid="cookie-manager-settings">
    <h3>Cookies</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <div class="toolbar">
        <input v-model="searchQuery" type="search" placeholder="Search domain or name…" class="field" data-testid="cookie-search" />
        <button type="button" class="nav-button secondary danger" @click="() => void clearAll()" data-testid="cookie-delete-all">Delete all</button>
      </div>
      <p v-if="filtered.length === 0" class="hint">No cookies stored.</p>
      <ul v-else class="list">
        <li v-for="c in filtered" :key="c.id" class="row">
          <div>
            <strong>{{ c.name }}</strong>
            <span class="muted">{{ c.domain }}{{ c.path }}</span>
          </div>
          <button type="button" class="nav-button secondary danger" @click="() => void remove(c.id)" data-testid="cookie-delete">Delete</button>
        </li>
      </ul>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits<{ status: [message: string] }>();

type CookieEntry = {
  id: string;
  domain: string;
  name: string;
  value: string;
  path: string;
};

const cookies = ref<CookieEntry[]>([]);
const searchQuery = ref('');
const loading = ref(true);

const filtered = computed(() => {
  const list = cookies.value ?? [];
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return list;
  return list.filter(
    (c) => c.domain.toLowerCase().includes(q) || c.name.toLowerCase().includes(q),
  );
});

async function load(): Promise<void> {
  loading.value = true;
  try {
    cookies.value = await invoke<CookieEntry[]>('list_cookies');
  } catch (error) {
    console.error('list_cookies failed:', error);
    emit('status', 'Failed to load cookies');
  } finally {
    loading.value = false;
  }
}

async function remove(id: string): Promise<void> {
  try {
    await invoke('delete_cookie', { id });
    await load();
    emit('status', 'Cookie deleted');
  } catch (error) {
    console.error('delete_cookie failed:', error);
  }
}

async function clearAll(): Promise<void> {
  if (!confirm('Delete all cookies?')) return;
  try {
    await invoke('delete_all_cookies');
    await load();
    emit('status', 'All cookies deleted');
  } catch (error) {
    console.error('delete_all_cookies failed:', error);
  }
}

onMounted(() => void load());
</script>

<style scoped>
.toolbar { display: flex; gap: 8px; margin-bottom: 12px; }
.field { flex: 1; padding: 6px 8px; border-radius: 6px; border: 1px solid var(--color-border, #404040); }
.list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 6px; max-height: 240px; overflow-y: auto; }
.row { display: flex; justify-content: space-between; align-items: center; padding: 6px 8px; background: var(--color-bg-secondary, #2a2a2a); border-radius: 6px; font-size: 12px; }
.muted { display: block; color: var(--color-text-secondary, #888); }
.hint { font-size: 12px; color: var(--color-text-secondary, #888); }
.nav-button { padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-accent, #2563eb); color: #fff; }
.nav-button.secondary { background: var(--color-bg-tertiary, #404040); }
.nav-button.danger { background: #7f1d1d; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
</style>
