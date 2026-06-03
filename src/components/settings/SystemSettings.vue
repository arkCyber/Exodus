<!--
  Exodus Browser — System settings.
-->
<template>
  <section class="settings-section" data-testid="system-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label class="checkbox-row">
        <input v-model="settings.defaultBrowser" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.defaultBrowser }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.backgroundApps" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.backgroundApps }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.hardwareAcceleration" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.hardwareAcceleration }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.useGPURendering" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.useGPURendering }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.updateAutomatically" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.updateAutomatically }}</span>
      </label>

      <label>
        {{ ui.updateChannel }}
        <select v-model="settings.updateChannel" data-testid="update-channel" @change="() => void persist()">
          <option value="stable">{{ ui.stable }}</option>
          <option value="beta">{{ ui.beta }}</option>
          <option value="dev">{{ ui.dev }}</option>
          <option value="nightly">{{ ui.nightly }}</option>
        </select>
      </label>

      <div class="info-card">
        <h4>{{ ui.systemInfo }}</h4>
        <div class="info-row">
          <span>{{ ui.os }}</span>
          <span>{{ systemInfo.os }}</span>
        </div>
        <div class="info-row">
          <span>{{ ui.version }}</span>
          <span>{{ systemInfo.version }}</span>
        </div>
        <div class="info-row">
          <span>{{ ui.architecture }}</span>
          <span>{{ systemInfo.arch }}</span>
        </div>
      </div>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="system-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — system settings.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => systemSettingsStrings(props.uiLocale));

type SystemSettings = {
  defaultBrowser: boolean;
  backgroundApps: boolean;
  hardwareAcceleration: boolean;
  useGPURendering: boolean;
  updateAutomatically: boolean;
  updateChannel: 'stable' | 'beta' | 'dev' | 'nightly';
};

const STORAGE_KEY = 'exodus-system-settings';

const DEFAULT_SETTINGS: SystemSettings = {
  defaultBrowser: false,
  backgroundApps: true,
  hardwareAcceleration: true,
  useGPURendering: true,
  updateAutomatically: true,
  updateChannel: 'stable',
};

const loading = ref(true);
const settings = ref<SystemSettings>({ ...DEFAULT_SETTINGS });
const systemInfo = ref({
  os: 'Unknown',
  version: 'Unknown',
  arch: 'Unknown',
});

function loadSettings(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error('Failed to load system settings:', error);
    settings.value = { ...DEFAULT_SETTINGS };
  }
}

function loadSystemInfo(): void {
  if (typeof navigator !== 'undefined') {
    systemInfo.value.os = navigator.platform || 'Unknown';
    systemInfo.value.arch = navigator.userAgent.includes('x64') ? 'x64' : 'Unknown';
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save system settings:', error);
    emit('status', ui.value.saveError);
  }
}

async function resetToDefaults(): Promise<void> {
  settings.value = { ...DEFAULT_SETTINGS };
  await persist();
  emit('status', ui.value.reset);
}

onMounted(() => {
  loadSettings();
  loadSystemInfo();
  loading.value = false;
});

function systemSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '系统' : 'System',
    hint: isZh ? '配置默认浏览器、后台应用和系统更新' : 'Configure default browser, background apps, and system updates',
    loading: isZh ? '加载中...' : 'Loading...',
    defaultBrowser: isZh ? '设为默认浏览器' : 'Set as default browser',
    backgroundApps: isZh ? '允许后台应用运行' : 'Continue running background apps when Exodus is closed',
    hardwareAcceleration: isZh ? '使用硬件加速' : 'Use hardware acceleration when available',
    useGPURendering: isZh ? '使用GPU渲染' : 'Use GPU rendering',
    updateAutomatically: isZh ? '自动更新' : 'Update Exodus automatically',
    updateChannel: isZh ? '更新渠道' : 'Update channel',
    stable: isZh ? '稳定版' : 'Stable',
    beta: isZh ? '测试版' : 'Beta',
    dev: isZh ? '开发版' : 'Dev',
    nightly: isZh ? '每日构建版' : 'Nightly',
    systemInfo: isZh ? '系统信息' : 'System information',
    os: isZh ? '操作系统' : 'OS',
    version: isZh ? '版本' : 'Version',
    architecture: isZh ? '架构' : 'Architecture',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '系统设置已保存' : 'System settings saved',
    saveError: isZh ? '保存系统设置失败' : 'Failed to save system settings',
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

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.checkbox-row {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

select {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

.info-card {
  margin: 20px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 8px;
  border: 1px solid var(--color-border, #404040);
}

.info-card h4 {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border, #404040);
  font-size: 12px;
}

.info-row:last-child {
  border-bottom: none;
}

.info-row span:first-child {
  color: var(--color-text-secondary, #9ca3af);
}

.info-row span:last-child {
  color: var(--color-text-primary, #e0e0e0);
  font-family: monospace;
}

.nav-button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-bg-tertiary, #404040);
  color: #fff;
  font-size: 13px;
  margin-top: 8px;
}

.nav-button:hover {
  background: var(--color-bg-quaternary, #505050);
}

.settings-section h3 {
  margin: 0 0 8px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}
</style>
