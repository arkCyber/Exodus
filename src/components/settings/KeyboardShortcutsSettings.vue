<!--
  Exodus Browser — Keyboard shortcuts settings.
-->
<template>
  <section class="settings-section" data-testid="keyboard-shortcuts-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <div class="shortcuts-list">
        <div v-for="shortcut in defaultShortcuts" :key="shortcut.id" class="shortcut-item">
          <div class="shortcut-info">
            <strong>{{ shortcut.label }}</strong>
            <span class="shortcut-keys">
              <kbd v-for="key in shortcut.keys" :key="key">{{ key }}</kbd>
            </span>
          </div>
          <button
            v-if="shortcut.customizable"
            type="button"
            class="nav-button secondary"
            @click="() => void customizeShortcut()"
          >
            {{ ui.customize }}
          </button>
        </div>
      </div>

      <div class="custom-shortcuts">
        <h4>{{ ui.customShortcuts }}</h4>
        <div v-for="(custom, index) in customShortcuts" :key="index" class="custom-shortcut-item">
          <label>
            {{ ui.action }}
            <input v-model="custom.action" type="text" :placeholder="ui.actionPlaceholder" />
          </label>
          <label>
            {{ ui.keys }}
            <input v-model="custom.keys" type="text" :placeholder="ui.keysPlaceholder" />
          </label>
          <button
            type="button"
            class="nav-button secondary danger"
            @click="() => void removeCustomShortcut(index)"
          >
            {{ ui.remove }}
          </button>
        </div>
        <button
          type="button"
          class="nav-button secondary"
          @click="() => void addCustomShortcut()"
        >
          {{ ui.addCustom }}
        </button>
      </div>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="keyboard-shortcuts-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — keyboard shortcuts settings.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => keyboardShortcutsSettingsStrings(props.uiLocale));

type Shortcut = {
  id: string;
  label: string;
  keys: string[];
  customizable: boolean;
};

type CustomShortcut = {
  action: string;
  keys: string;
};

const STORAGE_KEY = 'exodus-keyboard-shortcuts';

const DEFAULT_SHORTCUTS: Shortcut[] = [
  { id: 'new-tab', label: 'New tab', keys: ['Cmd', 'T'], customizable: false },
  { id: 'close-tab', label: 'Close tab', keys: ['Cmd', 'W'], customizable: false },
  { id: 'reload', label: 'Reload', keys: ['Cmd', 'R'], customizable: false },
  { id: 'find', label: 'Find', keys: ['Cmd', 'F'], customizable: false },
  { id: 'zoom-in', label: 'Zoom in', keys: ['Cmd', '+'], customizable: true },
  { id: 'zoom-out', label: 'Zoom out', keys: ['Cmd', '-'], customizable: true },
  { id: 'back', label: 'Back', keys: ['Cmd', '['], customizable: true },
  { id: 'forward', label: 'Forward', keys: [']'], customizable: true },
];

const loading = ref(true);
const defaultShortcuts = ref<Shortcut[]>([...DEFAULT_SHORTCUTS]);
const customShortcuts = ref<CustomShortcut[]>([]);

function loadShortcuts(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      if (data.custom) {
        customShortcuts.value = data.custom;
      }
    }
  } catch (error) {
    console.error('Failed to load keyboard shortcuts:', error);
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ custom: customShortcuts.value }));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save keyboard shortcuts:', error);
    emit('status', ui.value.saveError);
  }
}

async function customizeShortcut(): Promise<void> {
  emit('status', ui.value.customizeNotImplemented);
}

async function addCustomShortcut(): Promise<void> {
  customShortcuts.value.push({ action: '', keys: '' });
  await persist();
  emit('status', ui.value.added);
}

async function removeCustomShortcut(index: number): Promise<void> {
  if (!confirm('Remove this custom shortcut?')) return;
  customShortcuts.value.splice(index, 1);
  await persist();
  emit('status', ui.value.removed);
}

async function resetToDefaults(): Promise<void> {
  customShortcuts.value = [];
  await persist();
  emit('status', ui.value.reset);
}

onMounted(() => {
  loadShortcuts();
  loading.value = false;
});

function keyboardShortcutsSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '键盘快捷键' : 'Keyboard shortcuts',
    hint: isZh ? '查看和自定义键盘快捷键' : 'View and customize keyboard shortcuts',
    loading: isZh ? '加载中...' : 'Loading...',
    customize: isZh ? '自定义' : 'Customize',
    customShortcuts: isZh ? '自定义快捷键' : 'Custom shortcuts',
    action: isZh ? '操作' : 'Action',
    actionPlaceholder: isZh ? '例如：打开书签' : 'e.g., Open bookmarks',
    keys: isZh ? '按键' : 'Keys',
    keysPlaceholder: isZh ? '例如：Cmd+B' : 'e.g., Cmd+B',
    remove: isZh ? '删除' : 'Remove',
    addCustom: isZh ? '添加自定义快捷键' : 'Add custom shortcut',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '键盘快捷键已保存' : 'Keyboard shortcuts saved',
    saveError: isZh ? '保存键盘快捷键失败' : 'Failed to save keyboard shortcuts',
    customizeNotImplemented: isZh ? '自定义功能尚未实现' : 'Customize not yet implemented',
    added: isZh ? '快捷键已添加' : 'Shortcut added',
    removed: isZh ? '快捷键已删除' : 'Shortcut removed',
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

.shortcuts-list {
  margin: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 8px;
  border: 1px solid var(--color-border, #404040);
}

.shortcut-info {
  flex: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.shortcut-info strong {
  font-size: 14px;
  color: var(--color-text-primary, #e0e0e0);
}

.shortcut-keys {
  display: flex;
  gap: 4px;
}

kbd {
  padding: 4px 8px;
  background: var(--color-bg-tertiary, #404040);
  border: 1px solid var(--color-border, #505050);
  border-radius: 4px;
  font-size: 12px;
  color: var(--color-text-primary, #e0e0e0);
  font-family: monospace;
}

.custom-shortcuts {
  margin: 20px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 8px;
  border: 1px solid var(--color-border, #404040);
}

.custom-shortcuts h4 {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

.custom-shortcut-item {
  padding: 12px;
  background: var(--color-bg-tertiary, #404040);
  border-radius: 6px;
  margin-bottom: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
  color: var(--color-text-primary, #e0e0e0);
}

input {
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

.nav-button {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: var(--color-bg-tertiary, #404040);
  color: #fff;
  font-size: 13px;
}

.nav-button.secondary {
  background: var(--color-bg-quaternary, #505050);
}

.nav-button.danger {
  background: #7f1d1d;
}

.nav-button:hover {
  opacity: 0.9;
}

.settings-section h3 {
  margin: 0 0 8px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}
</style>
