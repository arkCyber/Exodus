<!--
  Exodus Browser — Font and page zoom settings.
-->
<template>
  <section class="settings-section" data-testid="font-zoom-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label>
        {{ ui.defaultZoom }}
        <select v-model="settings.defaultZoom" data-testid="default-zoom" @change="() => void persist()">
          <option value="50">50%</option>
          <option value="67">67%</option>
          <option value="75">75%</option>
          <option value="80">80%</option>
          <option value="90">90%</option>
          <option value="100">100%</option>
          <option value="110">110%</option>
          <option value="125">125%</option>
          <option value="150">150%</option>
          <option value="175">175%</option>
          <option value="200">200%</option>
        </select>
      </label>

      <label>
        {{ ui.standardFont }}
        <select v-model="settings.standardFont" data-testid="standard-font" @change="() => void persist()">
          <option value="default">{{ ui.systemDefault }}</option>
          <option value="Arial">Arial</option>
          <option value="Times New Roman">Times New Roman</option>
          <option value="Courier New">Courier New</option>
          <option value="Georgia">Georgia</option>
          <option value="Verdana">Verdana</option>
          <option value="Comic Sans MS">Comic Sans MS</option>
          <option value="Impact">Impact</option>
          <option value="Trebuchet MS">Trebuchet MS</option>
        </select>
      </label>

      <label>
        {{ ui.serifFont }}
        <select v-model="settings.serifFont" data-testid="serif-font" @change="() => void persist()">
          <option value="default">{{ ui.systemDefault }}</option>
          <option value="Times New Roman">Times New Roman</option>
          <option value="Georgia">Georgia</option>
          <option value="Palatino">Palatino</option>
        </select>
      </label>

      <label>
        {{ ui.sansSerifFont }}
        <select v-model="settings.sansSerifFont" data-testid="sans-serif-font" @change="() => void persist()">
          <option value="default">{{ ui.systemDefault }}</option>
          <option value="Arial">Arial</option>
          <option value="Helvetica">Helvetica</option>
          <option value="Verdana">Verdana</option>
          <option value="Tahoma">Tahoma</option>
        </select>
      </label>

      <label>
        {{ ui.monospaceFont }}
        <select v-model="settings.monospaceFont" data-testid="monospace-font" @change="() => void persist()">
          <option value="default">{{ ui.systemDefault }}</option>
          <option value="Courier New">Courier New</option>
          <option value="Consolas">Consolas</option>
          <option value="Monaco">Monaco</option>
          <option value="Menlo">Menlo</option>
        </select>
      </label>

      <label>
        {{ ui.fontSize }}
        <input
          v-model.number="settings.fontSize"
          type="number"
          min="9"
          max="72"
          data-testid="font-size"
          @change="() => void persist()"
        />
      </label>

      <label class="checkbox-row">
        <input v-model="settings.smoothScrolling" type="checkbox" data-testid="smooth-scrolling" @change="() => void persist()" />
        <span>{{ ui.smoothScrolling }}</span>
      </label>

      <div class="preview-section">
        <h4>{{ ui.preview }}</h4>
        <div class="font-preview" :style="previewStyle">
          <p>{{ ui.previewText }}</p>
          <p class="serif">{{ ui.previewSerif }}</p>
          <p class="monospace">{{ ui.previewMonospace }}</p>
        </div>
      </div>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="font-zoom-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — font and zoom settings.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => fontAndZoomSettingsStrings(props.uiLocale));

type FontZoomSettings = {
  defaultZoom: number;
  standardFont: string;
  serifFont: string;
  sansSerifFont: string;
  monospaceFont: string;
  fontSize: number;
  smoothScrolling: boolean;
};

const STORAGE_KEY = 'exodus-font-zoom-settings';

const DEFAULT_SETTINGS: FontZoomSettings = {
  defaultZoom: 100,
  standardFont: 'default',
  serifFont: 'default',
  sansSerifFont: 'default',
  monospaceFont: 'default',
  fontSize: 16,
  smoothScrolling: true,
};

const loading = ref(true);
const settings = ref<FontZoomSettings>({ ...DEFAULT_SETTINGS });

const previewStyle = computed(() => ({
  fontSize: `${settings.value.fontSize}px`,
  fontFamily: getFontFamily(settings.value.standardFont),
  zoom: `${settings.value.defaultZoom}%`,
}));

function getFontFamily(font: string): string {
  if (font === 'default') return '';
  return font;
}

function loadSettings(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error('Failed to load font settings:', error);
    settings.value = { ...DEFAULT_SETTINGS };
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save font settings:', error);
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

function fontAndZoomSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '字体和缩放' : 'Fonts and zoom',
    hint: isZh ? '自定义页面字体大小和缩放级别' : 'Customize page font size and zoom level',
    loading: isZh ? '加载中...' : 'Loading...',
    defaultZoom: isZh ? '默认缩放' : 'Default zoom',
    standardFont: isZh ? '标准字体' : 'Standard font',
    serifFont: isZh ? '衬线字体' : 'Serif font',
    sansSerifFont: isZh ? '无衬线字体' : 'Sans-serif font',
    monospaceFont: isZh ? '等宽字体' : 'Monospace font',
    fontSize: isZh ? '字体大小（像素）' : 'Font size (px)',
    smoothScrolling: isZh ? '平滑滚动' : 'Smooth scrolling',
    preview: isZh ? '预览' : 'Preview',
    previewText: isZh ? '这是标准字体预览文本。The quick brown fox jumps over the lazy dog.' : 'This is standard font preview text. The quick brown fox jumps over the lazy dog.',
    previewSerif: isZh ? '这是衬线字体预览文本。' : 'This is serif font preview text.',
    previewMonospace: isZh ? '这是等宽字体预览文本。' : 'This is monospace font preview text.',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '字体设置已保存' : 'Font settings saved',
    saveError: isZh ? '保存字体设置失败' : 'Failed to save font settings',
    systemDefault: isZh ? '系统默认' : 'System default',
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

input,
select {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

input[type="number"] {
  max-width: 120px;
}

.preview-section {
  margin: 20px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 8px;
  border: 1px solid var(--color-border, #404040);
}

.preview-section h4 {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.font-preview {
  padding: 12px;
  background: var(--color-bg-primary, #1a1a1a);
  border-radius: 6px;
  min-height: 100px;
}

.font-preview p {
  margin: 0 0 8px;
  line-height: 1.5;
}

.font-preview p:last-child {
  margin-bottom: 0;
}

.font-preview .serif {
  font-family: 'Times New Roman', Georgia, serif;
}

.font-preview .monospace {
  font-family: 'Courier New', Consolas, monospace;
}

.nav-button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-bg-tertiary, #404040);
  color: #fff;
  font-size: 13px;
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
