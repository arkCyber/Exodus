<!--
  Exodus Browser — Accessibility settings.
-->
<template>
  <section class="settings-section" data-testid="accessibility-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label class="checkbox-row">
        <input v-model="settings.forceDarkMode" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.forceDarkMode }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.reduceMotion" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.reduceMotion }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.highContrast" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.highContrast }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.screenReader" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.screenReader }}</span>
      </label>

      <label>
        {{ ui.minimumFontSize }}
        <input
          v-model.number="settings.minimumFontSize"
          type="number"
          min="12"
          max="24"
          data-testid="min-font-size"
          @change="() => void persist()"
        />
      </label>

      <label>
        {{ ui.cursorSize }}
        <select v-model="settings.cursorSize" data-testid="cursor-size" @change="() => void persist()">
          <option value="default">{{ ui.systemDefault }}</option>
          <option value="small">{{ ui.small }}</option>
          <option value="medium">{{ ui.medium }}</option>
          <option value="large">{{ ui.large }}</option>
          <option value="extra-large">{{ ui.extraLarge }}</option>
        </select>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.focusIndicator" type="checkbox" data-testid="focus-indicator" @change="() => void persist()" />
        <span>{{ ui.focusIndicator }}</span>
      </label>

      <label class="checkbox-row">
        <input v-model="settings.textToSpeech" type="checkbox" data-testid="text-to-speech" @change="() => void persist()" />
        <span>{{ ui.textToSpeech }}</span>
      </label>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="accessibility-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — accessibility settings.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => accessibilitySettingsStrings(props.uiLocale));

type AccessibilitySettings = {
  forceDarkMode: boolean;
  reduceMotion: boolean;
  highContrast: boolean;
  screenReader: boolean;
  minimumFontSize: number;
  cursorSize: 'default' | 'small' | 'medium' | 'large' | 'extra-large';
  focusIndicator: boolean;
  textToSpeech: boolean;
};

const STORAGE_KEY = 'exodus-accessibility-settings';

const DEFAULT_SETTINGS: AccessibilitySettings = {
  forceDarkMode: false,
  reduceMotion: false,
  highContrast: false,
  screenReader: false,
  minimumFontSize: 12,
  cursorSize: 'default',
  focusIndicator: true,
  textToSpeech: false,
};

const loading = ref(true);
const settings = ref<AccessibilitySettings>({ ...DEFAULT_SETTINGS });

function loadSettings(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error('Failed to load accessibility settings:', error);
    settings.value = { ...DEFAULT_SETTINGS };
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save accessibility settings:', error);
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

function accessibilitySettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '无障碍' : 'Accessibility',
    hint: isZh ? '配置屏幕阅读器、高对比度和其他辅助功能' : 'Configure screen reader, high contrast, and other accessibility features',
    loading: isZh ? '加载中...' : 'Loading...',
    forceDarkMode: isZh ? '强制深色模式' : 'Force dark mode',
    reduceMotion: isZh ? '减少动画' : 'Reduce motion',
    highContrast: isZh ? '高对比度' : 'High contrast',
    screenReader: isZh ? '屏幕阅读器支持' : 'Screen reader support',
    minimumFontSize: isZh ? '最小字体大小' : 'Minimum font size',
    cursorSize: isZh ? '光标大小' : 'Cursor size',
    systemDefault: isZh ? '系统默认' : 'System default',
    small: isZh ? '小' : 'Small',
    medium: isZh ? '中' : 'Medium',
    large: isZh ? '大' : 'Large',
    extraLarge: isZh ? '特大' : 'Extra large',
    focusIndicator: isZh ? '显示焦点指示器' : 'Show focus indicator',
    textToSpeech: isZh ? '文本转语音' : 'Text-to-speech',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '无障碍设置已保存' : 'Accessibility settings saved',
    saveError: isZh ? '保存无障碍设置失败' : 'Failed to save accessibility settings',
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

select,
input[type="number"] {
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
