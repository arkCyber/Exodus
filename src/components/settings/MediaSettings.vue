<!--
  Exodus Browser — Media settings (autoplay, picture-in-picture, etc.).
-->
<template>
  <section class="settings-section" data-testid="media-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <h4>{{ ui.autoplaySection }}</h4>
      <label>
        {{ ui.autoplayPolicy }}
        <select v-model="settings.autoplayPolicy" data-testid="autoplay-policy" @change="() => void persist()">
          <option value="allow">{{ ui.autoplayAllow }}</option>
          <option value="block">{{ ui.autoplayBlock }}</option>
          <option value="limit">{{ ui.autoplayLimit }}</option>
        </select>
      </label>
      <p class="settings-hint">{{ ui.autoplayHint }}</p>

      <h4>{{ ui.pictureInPictureSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.pipEnabled" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.pipEnabled }}</span>
      </label>
      <label class="checkbox-row">
        <input v-model="settings.pipAutoEnter" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.pipAutoEnter }}</span>
      </label>

      <h4>{{ ui.mediaSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.hardwareAcceleration" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.hardwareAcceleration }}</span>
      </label>
      <p class="settings-hint">{{ ui.hardwareAccelerationHint }}</p>

      <label class="checkbox-row">
        <input v-model="settings.preloadMedia" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.preloadMedia }}</span>
      </label>

      <label>
        {{ ui.defaultVolume }}
        <input
          v-model.number="settings.defaultVolume"
          type="range"
          min="0"
          max="100"
          step="1"
          data-testid="default-volume"
          @input="() => void persist()"
        />
        <span class="volume-value">{{ settings.defaultVolume }}%</span>
      </label>

      <h4>{{ ui.castingSection }}</h4>
      <label class="checkbox-row">
        <input v-model="settings.castingEnabled" type="checkbox" data-testid="casting-enabled" @change="() => void persist()" />
        <span>{{ ui.castingEnabled }}</span>
      </label>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="media-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — media settings (autoplay, PiP, hardware acceleration).
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => mediaSettingsStrings(props.uiLocale));

type MediaSettings = {
  autoplayPolicy: 'allow' | 'block' | 'limit';
  pipEnabled: boolean;
  pipAutoEnter: boolean;
  hardwareAcceleration: boolean;
  preloadMedia: boolean;
  defaultVolume: number;
  castingEnabled: boolean;
};

const STORAGE_KEY = 'exodus-media-settings';

const DEFAULT_SETTINGS: MediaSettings = {
  autoplayPolicy: 'limit',
  pipEnabled: true,
  pipAutoEnter: false,
  hardwareAcceleration: true,
  preloadMedia: true,
  defaultVolume: 100,
  castingEnabled: true,
};

const loading = ref(true);
const settings = ref<MediaSettings>({ ...DEFAULT_SETTINGS });

function loadSettings(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error('Failed to load media settings:', error);
    settings.value = { ...DEFAULT_SETTINGS };
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save media settings:', error);
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
  loading.value = false;
});

function mediaSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '媒体设置' : 'Media',
    hint: isZh ? '配置自动播放、画中画和硬件加速' : 'Configure autoplay, picture-in-picture, and hardware acceleration',
    loading: isZh ? '加载中...' : 'Loading...',
    autoplaySection: isZh ? '自动播放' : 'Autoplay',
    autoplayPolicy: isZh ? '自动播放策略' : 'Autoplay policy',
    autoplayAllow: isZh ? '允许所有' : 'Allow all',
    autoplayBlock: isZh ? '阻止所有' : 'Block all',
    autoplayLimit: isZh ? '限制（仅静音）' : 'Limit (muted only)',
    autoplayHint: isZh ? '限制策略仅允许静音视频自动播放' : 'Limit policy allows autoplay only for muted videos',
    pictureInPictureSection: isZh ? '画中画' : 'Picture-in-picture',
    pipEnabled: isZh ? '启用画中画' : 'Enable picture-in-picture',
    pipAutoEnter: isZh ? '自动进入画中画' : 'Auto-enter picture-in-picture',
    mediaSection: isZh ? '媒体播放' : 'Media playback',
    hardwareAcceleration: isZh ? '硬件加速' : 'Hardware acceleration',
    hardwareAccelerationHint: isZh ? '使用 GPU 加速视频解码和渲染' : 'Use GPU for video decoding and rendering',
    preloadMedia: isZh ? '预加载媒体' : 'Preload media',
    defaultVolume: isZh ? '默认音量' : 'Default volume',
    castingSection: isZh ? '投屏' : 'Casting',
    castingEnabled: isZh ? '启用投屏' : 'Enable casting',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '媒体设置已保存' : 'Media settings saved',
    saveError: isZh ? '保存媒体设置失败' : 'Failed to save media settings',
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

h4 {
  margin: 20px 0 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
  text-transform: uppercase;
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

input[type="range"] {
  width: 100%;
  max-width: 300px;
  margin-top: 4px;
}

.volume-value {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
  margin-left: 8px;
}

select {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

.nav-button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-bg-tertiary, #404040);
  color: #fff;
  font-size: 13px;
  margin-top: 16px;
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
