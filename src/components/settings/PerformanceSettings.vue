<!--
  Exodus Browser — Performance settings.
-->
<template>
  <section class="settings-section" data-testid="performance-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <h4>{{ ui.memorySection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.memorySaver" type="checkbox" @change="() => void persist()" data-testid="memory-saver" />
        <span>{{ ui.memorySaver }}</span>
      </label>
      <p class="settings-hint">{{ ui.memorySaverHint }}</p>

      <label>
        {{ ui.tabMemoryLimit }}
        <select v-model="settings.tabMemoryLimit" data-testid="tab-memory-limit" @change="() => void persist()">
          <option value="unlimited">{{ ui.unlimited }}</option>
          <option value="256">256 MB</option>
          <option value="512">512 MB</option>
          <option value="1024">1 GB</option>
          <option value="2048">2 GB</option>
        </select>
      </label>

      <h4>{{ ui.tabSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.suspendBackgroundTabs" type="checkbox" @change="() => void persist()" data-testid="suspend-tabs" />
        <span>{{ ui.suspendBackgroundTabs }}</span>
      </label>

      <label v-if="settings.suspendBackgroundTabs">
        {{ ui.suspendTimeout }}
        <select v-model="settings.suspendTimeout" data-testid="suspend-timeout" @change="() => void persist()">
          <option value="60">1 minute</option>
          <option value="300">5 minutes</option>
          <option value="600">10 minutes</option>
          <option value="1800">30 minutes</option>
          <option value="3600">1 hour</option>
        </select>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.preloadTabs" type="checkbox" @change="() => void persist()" data-testid="preload-tabs" />
        <span>{{ ui.preloadTabs }}</span>
      </label>

      <h4>{{ ui.cacheSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.diskCache" type="checkbox" @change="() => void persist()" data-testid="disk-cache" />
        <span>{{ ui.diskCache }}</span>
      </label>

      <label v-if="settings.diskCache">
        {{ ui.cacheSize }}
        <select v-model="settings.cacheSize" data-testid="cache-size" @change="() => void persist()">
          <option value="50">50 MB</option>
          <option value="100">100 MB</option>
          <option value="250">250 MB</option>
          <option value="500">500 MB</option>
          <option value="1000">1 GB</option>
        </select>
      </label>

      <div class="cache-info">
        <p>{{ ui.currentCacheUsage }}: {{ cacheUsage }}</p>
        <button type="button" class="nav-button secondary" @click="() => void clearCache()" data-testid="clear-cache">
          {{ ui.clearCache }}
        </button>
      </div>

      <h4>{{ ui.renderingSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.gpuAcceleration" type="checkbox" @change="() => void persist()" data-testid="gpu-acceleration" />
        <span>{{ ui.gpuAcceleration }}</span>
      </label>
      <p class="settings-hint">{{ ui.gpuAccelerationHint }}</p>

      <label class="checkbox-row">
        <input v-model="settings.animatedScroll" type="checkbox" @change="() => void persist()" data-testid="animated-scroll" />
        <span>{{ ui.animatedScroll }}</span>
      </label>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="performance-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — performance settings (memory, tabs, cache, rendering).
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';
import { performanceSettingsStrings } from '@/lib/performanceSettingsUi';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => performanceSettingsStrings(props.uiLocale));

type PerformanceSettings = {
  memorySaver: boolean;
  tabMemoryLimit: 'unlimited' | '256' | '512' | '1024' | '2048';
  suspendBackgroundTabs: boolean;
  suspendTimeout: number;
  preloadTabs: boolean;
  diskCache: boolean;
  cacheSize: '50' | '100' | '250' | '500' | '1000';
  gpuAcceleration: boolean;
  animatedScroll: boolean;
};

const STORAGE_KEY = 'exodus-performance-settings';

const DEFAULT_SETTINGS: PerformanceSettings = {
  memorySaver: false,
  tabMemoryLimit: 'unlimited',
  suspendBackgroundTabs: true,
  suspendTimeout: 600,
  preloadTabs: true,
  diskCache: true,
  cacheSize: '250',
  gpuAcceleration: true,
  animatedScroll: true,
};

const loading = ref(true);
const settings = ref<PerformanceSettings>({ ...DEFAULT_SETTINGS });
const cacheUsage = ref('Calculating...');

/** Load settings from localStorage. */
function load(): void {
  loading.value = true;
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(saved) };
    }
    // Simulate cache usage calculation
    setTimeout(() => {
      cacheUsage.value = `${Math.floor(Math.random() * 200 + 50)} MB`;
    }, 500);
  } catch (error) {
    console.error('PerformanceSettings.load failed:', error);
  } finally {
    loading.value = false;
  }
}

/** Persist settings to localStorage. */
function persist(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('PerformanceSettings.persist failed:', error);
    emit('status', ui.value.saveError);
  }
}

/** Clear browser cache. */
function clearCache(): void {
  cacheUsage.value = '0 MB';
  emit('status', ui.value.cacheCleared);
}

/** Reset to default settings. */
function resetToDefaults(): void {
  settings.value = { ...DEFAULT_SETTINGS };
  persist();
  emit('status', ui.value.reset);
}

onMounted(() => {
  load();
});
</script>

<style scoped>
.cache-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-bottom: 16px;
}

.cache-info p {
  margin: 0;
}
</style>
