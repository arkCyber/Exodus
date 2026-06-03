<!--
  Exodus Browser — Search engine management settings.
-->
<template>
  <section class="settings-section" data-testid="search-engine-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label>
        {{ ui.defaultLabel }}
        <select v-model="defaultEngine" data-testid="search-engine-default" @change="() => void persist()">
          <option v-for="engine in engines" :key="engine.id" :value="engine.id">
            {{ engine.name }}
          </option>
        </select>
      </label>

      <div class="engine-list">
        <div v-for="engine in engines" :key="engine.id" class="engine-item">
          <div class="engine-info">
            <strong>{{ engine.name }}</strong>
            <span class="engine-url">{{ engine.url }}</span>
          </div>
          <div class="engine-actions">
            <button
              v-if="engine.id !== defaultEngine"
              type="button"
              class="nav-button secondary"
              @click="() => void setDefault(engine.id)"
            >
              {{ ui.setDefault }}
            </button>
            <button
              v-if="!engine.builtin"
              type="button"
              class="nav-button secondary danger"
              @click="() => void removeEngine(engine.id)"
            >
              {{ ui.remove }}
            </button>
          </div>
        </div>
      </div>

      <div class="add-engine-form">
        <h4>{{ ui.addCustom }}</h4>
        <label>
          {{ ui.nameLabel }}
          <input v-model="newEngine.name" type="text" :placeholder="ui.namePlaceholder" data-testid="search-engine-name" />
        </label>
        <label>
          {{ ui.urlLabel }}
          <input v-model="newEngine.url" type="text" :placeholder="ui.urlPlaceholder" data-testid="search-engine-url" />
        </label>
        <button
          type="button"
          class="nav-button secondary"
          :disabled="!canAddEngine"
          @click="() => void addEngine()"
          data-testid="search-engine-add"
        >
          {{ ui.add }}
        </button>
      </div>

      <div class="builtin-engines">
        <h4>{{ ui.restoreDefaults }}</h4>
        <button
          type="button"
          class="nav-button secondary"
          @click="() => void restoreBuiltinEngines()"
          data-testid="search-engine-restore"
        >
          {{ ui.restore }}
        </button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — search engine management (add/remove/set default).
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => searchEngineSettingsStrings(props.uiLocale));

type SearchEngine = {
  id: string;
  name: string;
  url: string;
  builtin: boolean;
};

const BUILTIN_ENGINES: SearchEngine[] = [
  { id: 'duckduckgo', name: 'DuckDuckGo', url: 'https://duckduckgo.com/?q={query}', builtin: true },
  { id: 'google', name: 'Google', url: 'https://www.google.com/search?q={query}', builtin: true },
  { id: 'bing', name: 'Bing', url: 'https://www.bing.com/search?q={query}', builtin: true },
  { id: 'brave', name: 'Brave', url: 'https://search.brave.com/search?q={query}', builtin: true },
  { id: 'startpage', name: 'Startpage', url: 'https://www.startpage.com/sp/search?query={query}', builtin: true },
];

const STORAGE_KEY = 'exodus-search-engines';
const DEFAULT_KEY = 'exodus-default-search-engine';

const loading = ref(true);
const engines = ref<SearchEngine[]>([]);
const defaultEngine = ref('');
const newEngine = ref({ name: '', url: '' });

const canAddEngine = computed(() => {
  return newEngine.value.name.trim().length > 0 && newEngine.value.url.trim().length > 0;
});

function loadEngines(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      engines.value = JSON.parse(stored);
    } else {
      engines.value = [...BUILTIN_ENGINES];
    }
    defaultEngine.value = localStorage.getItem(DEFAULT_KEY) || 'duckduckgo';
  } catch (error) {
    console.error('Failed to load search engines:', error);
    engines.value = [...BUILTIN_ENGINES];
    defaultEngine.value = 'duckduckgo';
  }
}

function saveEngines(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(engines.value));
    localStorage.setItem(DEFAULT_KEY, defaultEngine.value);
  } catch (error) {
    console.error('Failed to save search engines:', error);
  }
}

async function persist(): Promise<void> {
  saveEngines();
  emit('status', ui.value.saved);
}

async function setDefault(id: string): Promise<void> {
  defaultEngine.value = id;
  await persist();
  emit('status', ui.value.defaultSet);
}

async function addEngine(): Promise<void> {
  const id = `custom-${Date.now()}`;
  engines.value.push({
    id,
    name: newEngine.value.name.trim(),
    url: newEngine.value.url.trim(),
    builtin: false,
  });
  newEngine.value = { name: '', url: '' };
  await persist();
  emit('status', ui.value.added);
}

async function removeEngine(id: string): Promise<void> {
  if (!confirm('Remove this search engine?')) return;
  engines.value = engines.value.filter((e) => e.id !== id);
  if (defaultEngine.value === id) {
    defaultEngine.value = engines.value[0]?.id || 'duckduckgo';
  }
  await persist();
  emit('status', ui.value.removed);
}

async function restoreBuiltinEngines(): Promise<void> {
  engines.value = [...BUILTIN_ENGINES];
  defaultEngine.value = 'duckduckgo';
  await persist();
  emit('status', ui.value.restored);
}

onMounted(() => {
  loadEngines();
  loading.value = false;
});

function searchEngineSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '搜索引擎' : 'Search engine',
    hint: isZh ? '管理默认搜索引擎和自定义搜索引擎' : 'Manage default and custom search engines',
    loading: isZh ? '加载中...' : 'Loading...',
    defaultLabel: isZh ? '默认搜索引擎' : 'Default search engine',
    setDefault: isZh ? '设为默认' : 'Set as default',
    remove: isZh ? '删除' : 'Remove',
    addCustom: isZh ? '添加自定义搜索引擎' : 'Add custom search engine',
    nameLabel: isZh ? '名称' : 'Name',
    namePlaceholder: isZh ? '例如：My Search' : 'e.g., My Search',
    urlLabel: isZh ? '搜索URL（使用 {query} 作为查询占位符）' : 'Search URL (use {query} as placeholder)',
    urlPlaceholder: isZh ? '例如：https://example.com/search?q={query}' : 'e.g., https://example.com/search?q={query}',
    add: isZh ? '添加' : 'Add',
    restoreDefaults: isZh ? '恢复默认搜索引擎' : 'Restore default search engines',
    restore: isZh ? '恢复' : 'Restore',
    saved: isZh ? '搜索引擎设置已保存' : 'Search engine settings saved',
    defaultSet: isZh ? '默认搜索引擎已更新' : 'Default search engine updated',
    added: isZh ? '搜索引擎已添加' : 'Search engine added',
    removed: isZh ? '搜索引擎已删除' : 'Search engine removed',
    restored: isZh ? '默认搜索引擎已恢复' : 'Default search engines restored',
  };
}
</script>

<style scoped>
.settings-hint {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
  margin: 0 0 12px;
}

.loading-state {
  padding: 20px;
  text-align: center;
  color: var(--color-text-secondary, #9ca3af);
}

.engine-list {
  margin: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.engine-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 8px;
  border: 1px solid var(--color-border, #404040);
}

.engine-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.engine-info strong {
  font-size: 14px;
  color: var(--color-text-primary, #e0e0e0);
}

.engine-url {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
  word-break: break-all;
}

.engine-actions {
  display: flex;
  gap: 8px;
}

.add-engine-form {
  margin: 20px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 8px;
  border: 1px solid var(--color-border, #404040);
}

.add-engine-form h4 {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.builtin-engines {
  margin-top: 16px;
}

.builtin-engines h4 {
  margin: 0 0 8px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.nav-button {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-accent, #2563eb);
  color: #fff;
  font-size: 13px;
}

.nav-button.secondary {
  background: var(--color-bg-tertiary, #404040);
}

.nav-button.danger {
  background: #7f1d1d;
}

.nav-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

input,
select {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

input::placeholder {
  color: var(--color-text-secondary, #9ca3af);
}

.settings-section h3 {
  margin: 0 0 8px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}
</style>
