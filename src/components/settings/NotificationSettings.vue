<!--
  Exodus Browser — Notification settings.
-->
<template>
  <section class="settings-section" data-testid="notification-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label class="checkbox-row">
        <input v-model="settings.notificationsEnabled" type="checkbox" @change="() => void persist()" />
        <span>{{ ui.notificationsEnabled }}</span>
      </label>

      <template v-if="settings.notificationsEnabled">
        <h4>{{ ui.permissionSection }}</h4>
        <label>
          {{ ui.defaultBehavior }}
          <select v-model="settings.defaultBehavior" data-testid="default-behavior" @change="() => void persist()">
            <option value="ask">{{ ui.ask }}</option>
            <option value="allow">{{ ui.allow }}</option>
            <option value="block">{{ ui.block }}</option>
          </select>
        </label>

        <h4>{{ ui.appearanceSection }}</h4>
        <label class="checkbox-row">
          <input v-model="settings.soundEnabled" type="checkbox" @change="() => void persist()" />
          <span>{{ ui.soundEnabled }}</span>
        </label>
        <label class="checkbox-row">
          <input v-model="settings.badgeEnabled" type="checkbox" @change="() => void persist()" />
          <span>{{ ui.badgeEnabled }}</span>
        </label>

        <h4>{{ ui.quietSection }}</h4>
        <label class="checkbox-row">
          <input v-model="settings.quietMode" type="checkbox" @change="() => void persist()" />
          <span>{{ ui.quietMode }}</span>
        </label>
        <p class="settings-hint">{{ ui.quietModeHint }}</p>

        <label v-if="settings.quietMode">
          {{ ui.quietHours }}
          <div class="time-range">
            <input v-model="settings.quietStart" type="time" @change="() => void persist()" />
            <span>{{ ui.to }}</span>
            <input v-model="settings.quietEnd" type="time" @change="() => void persist()" />
          </div>
        </label>
      </template>

      <h4>{{ ui.historySection }}</h4>
      <div class="stats">
        <p>{{ ui.totalNotifications }}: {{ stats.total }}</p>
        <p>{{ ui.recentNotifications }}: {{ stats.recent }}</p>
      </div>
      <button type="button" class="nav-button secondary" @click="() => void clearHistory()" data-testid="notification-clear-history">
        {{ ui.clearHistory }}
      </button>

      <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="notification-reset">
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — notification settings.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => notificationSettingsStrings(props.uiLocale));

type NotificationSettings = {
  notificationsEnabled: boolean;
  defaultBehavior: 'ask' | 'allow' | 'block';
  soundEnabled: boolean;
  badgeEnabled: boolean;
  quietMode: boolean;
  quietStart: string;
  quietEnd: string;
};

const STORAGE_KEY = 'exodus-notification-settings';
const STATS_KEY = 'exodus-notification-stats';

const DEFAULT_SETTINGS: NotificationSettings = {
  notificationsEnabled: true,
  defaultBehavior: 'ask',
  soundEnabled: true,
  badgeEnabled: true,
  quietMode: false,
  quietStart: '22:00',
  quietEnd: '08:00',
};

const loading = ref(true);
const settings = ref<NotificationSettings>({ ...DEFAULT_SETTINGS });
const stats = ref({ total: 0, recent: 0 });

function loadSettings(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
    const statsData = localStorage.getItem(STATS_KEY);
    if (statsData) {
      stats.value = JSON.parse(statsData);
    }
  } catch (error) {
    console.error('Failed to load notification settings:', error);
    settings.value = { ...DEFAULT_SETTINGS };
  }
}

async function persist(): Promise<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('Failed to save notification settings:', error);
    emit('status', ui.value.saveError);
  }
}

async function resetToDefaults(): Promise<void> {
  settings.value = { ...DEFAULT_SETTINGS };
  await persist();
  emit('status', ui.value.reset);
}

async function clearHistory(): Promise<void> {
  stats.value = { total: 0, recent: 0 };
  localStorage.setItem(STATS_KEY, JSON.stringify(stats.value));
  emit('status', ui.value.historyCleared);
}

onMounted(() => {
  loadSettings();
  loading.value = false;
});

function notificationSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '通知设置' : 'Notifications',
    hint: isZh ? '配置网站通知权限和显示方式' : 'Configure website notification permissions and display',
    loading: isZh ? '加载中...' : 'Loading...',
    notificationsEnabled: isZh ? '启用通知' : 'Enable notifications',
    permissionSection: isZh ? '权限' : 'Permissions',
    defaultBehavior: isZh ? '默认行为' : 'Default behavior',
    ask: isZh ? '询问' : 'Ask',
    allow: isZh ? '允许' : 'Allow',
    block: isZh ? '阻止' : 'Block',
    appearanceSection: isZh ? '外观' : 'Appearance',
    soundEnabled: isZh ? '播放声音' : 'Play sound',
    badgeEnabled: isZh ? '显示徽章' : 'Show badge',
    quietSection: isZh ? '免打扰' : 'Quiet hours',
    quietMode: isZh ? '启用免打扰模式' : 'Enable quiet mode',
    quietModeHint: isZh ? '在指定时间段内不显示通知' : 'Suppress notifications during specified hours',
    quietHours: isZh ? '时间段' : 'Time range',
    to: isZh ? '至' : 'to',
    historySection: isZh ? '历史记录' : 'History',
    totalNotifications: isZh ? '总通知数' : 'Total notifications',
    recentNotifications: isZh ? '最近通知' : 'Recent notifications',
    clearHistory: isZh ? '清除历史记录' : 'Clear history',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    saved: isZh ? '通知设置已保存' : 'Notification settings saved',
    saveError: isZh ? '保存通知设置失败' : 'Failed to save notification settings',
    historyCleared: isZh ? '通知历史记录已清除' : 'Notification history cleared',
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

select {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

.time-range {
  display: flex;
  align-items: center;
  gap: 8px;
}

.time-range input {
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #404040);
  background: var(--color-bg-primary, #1a1a1a);
  color: var(--color-text-primary, #e0e0e0);
  font-size: 13px;
}

.stats {
  padding: 12px;
  background: var(--color-bg-secondary, #2a2a2a);
  border-radius: 6px;
  margin-bottom: 12px;
}

.stats p {
  margin: 0 0 4px;
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
}

.stats p:last-child {
  margin-bottom: 0;
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
