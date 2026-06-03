<!--
  Exodus Browser — Sync settings (bookmark synchronization across devices).
-->
<template>
  <section class="settings-section" data-testid="sync-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <!-- Sync Toggle -->
      <label>
        {{ ui.enableLabel }}
        <input
          type="checkbox"
          v-model="settings.enabled"
          data-testid="sync-enabled"
          @change="onToggleSync"
          :disabled="isSyncing"
        />
      </label>

      <!-- Device ID -->
      <div v-if="deviceId" class="setting-row">
        <label>{{ ui.deviceIdLabel }}</label>
        <div class="setting-value-with-action">
          <span data-testid="device-id">{{ deviceId }}</span>
          <button
            type="button"
            class="nav-button secondary"
            @click="copyDeviceId"
            :title="ui.copyDeviceId"
          >
            {{ ui.copy }}
          </button>
        </div>
      </div>

      <!-- Last Sync -->
      <div v-if="settings.enabled" class="setting-row">
        <label>{{ ui.lastSyncLabel }}</label>
        <div class="setting-value-with-action">
          <span data-testid="last-sync">{{ lastSyncTime || ui.neverSynced }}</span>
          <button
            type="button"
            class="nav-button secondary"
            @click="onPerformSync"
            :disabled="isSyncing"
            data-testid="sync-now"
          >
            {{ isSyncing ? ui.syncing : ui.syncNow }}
          </button>
        </div>
      </div>

      <!-- Sync Statistics -->
      <div v-if="settings.enabled" class="settings-stats">
        <h4>{{ ui.statsLabel }}</h4>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">{{ ui.totalBookmarks }}</span>
            <span class="stat-value" data-testid="total-bookmarks">{{ stats.totalBookmarks }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">{{ ui.syncedBookmarks }}</span>
            <span class="stat-value" data-testid="synced-bookmarks">{{ stats.syncedBookmarks }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">{{ ui.totalFolders }}</span>
            <span class="stat-value" data-testid="total-folders">{{ stats.totalFolders }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">{{ ui.syncedFolders }}</span>
            <span class="stat-value" data-testid="synced-folders">{{ stats.syncedFolders }}</span>
          </div>
        </div>
        <div class="sync-progress">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${stats.syncProgress}%` }"></div>
          </div>
          <span class="progress-text">{{ Math.round(stats.syncProgress) }}% {{ ui.synced }}</span>
        </div>
      </div>

      <!-- Advanced Settings -->
      <div v-if="settings.enabled" class="settings-advanced">
        <h4>{{ ui.advancedLabel }}</h4>

        <label>
          {{ ui.autoSyncLabel }}
          <p class="settings-hint">{{ ui.autoSyncHint }}</p>
          <input
            type="checkbox"
            v-model="settings.auto_sync"
            data-testid="auto-sync"
            @change="onSettingChange"
          />
        </label>

        <label>
          {{ ui.syncAcrossDevicesLabel }}
          <p class="settings-hint">{{ ui.syncAcrossDevicesHint }}</p>
          <input
            type="checkbox"
            v-model="settings.sync_across_devices"
            data-testid="sync-across-devices"
            @change="onSettingChange"
          />
        </label>

        <label>
          {{ ui.syncIntervalLabel }}
          <p class="settings-hint">{{ ui.syncIntervalHint }}</p>
          <select
            v-model.number="settings.sync_interval"
            data-testid="sync-interval"
            @change="onSettingChange"
          >
            <option :value="60">{{ ui.oneMinute }}</option>
            <option :value="300">{{ ui.fiveMinutes }}</option>
            <option :value="900">{{ ui.fifteenMinutes }}</option>
            <option :value="1800">{{ ui.thirtyMinutes }}</option>
            <option :value="3600">{{ ui.oneHour }}</option>
          </select>
        </label>

        <label>
          {{ ui.conflictResolutionLabel }}
          <p class="settings-hint">{{ ui.conflictResolutionHint }}</p>
          <select
            v-model="settings.conflict_resolution"
            data-testid="conflict-resolution"
            @change="onSettingChange"
          >
            <option value="server">{{ ui.serverWins }}</option>
            <option value="client">{{ ui.clientWins }}</option>
            <option value="manual">{{ ui.manualResolve }}</option>
          </select>
        </label>
      </div>

      <!-- Sync Log -->
      <div v-if="settings.enabled" class="settings-log">
        <h4>{{ ui.logLabel }}</h4>
        <button
          type="button"
          class="nav-button secondary"
          @click="clearLog"
          data-testid="clear-log"
        >
          {{ ui.clearLog }}
        </button>
        <div v-if="syncLog.length > 0" class="log-list">
          <div v-for="(entry, index) in syncLog.slice(-10).reverse()" :key="index" class="log-entry" :class="{ error: !entry.success }">
            <span class="log-time">{{ formatLogTime(entry.timestamp) }}</span>
            <span class="log-action">{{ entry.action }}</span>
            <span class="log-item">{{ entry.item_type }}: {{ entry.item_id }}</span>
            <span v-if="entry.error_message" class="log-error">{{ entry.error_message }}</span>
          </div>
        </div>
        <div v-else class="empty-state">{{ ui.noLogActivity }}</div>
      </div>

      <!-- Error Message -->
      <div v-if="syncError" class="error-message" data-testid="sync-error">
        {{ syncError }}
      </div>

      <!-- Reset Button -->
      <button
        type="button"
        class="nav-button secondary"
        @click="resetToDefaults"
        data-testid="reset-sync"
      >
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — Sync settings for bookmark synchronization across devices.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';
import { useBookmarkSync, type SyncSettings } from '@/composables/useBookmarkSync';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => syncSettingsStrings(props.uiLocale));

const {
  syncSettings,
  deviceId,
  isSyncing,
  syncError,
  lastSyncTime,
  syncStats,
  syncLog,
  updateSyncSettings,
  performSync,
  clearSyncLog,
  initialize,
} = useBookmarkSync();

const settings = ref<SyncSettings>({ ...syncSettings.value });
const loading = ref(true);

const DEFAULT_SETTINGS: SyncSettings = {
  enabled: false,
  auto_sync: true,
  sync_across_devices: true,
  sync_interval: 300,
  conflict_resolution: 'local_wins',
  last_sync: 0,
};

const stats = computed(() => syncStats.value);

async function onToggleSync(): Promise<void> {
  try {
    await updateSyncSettings(settings.value);
    if (settings.value.enabled) {
      await onPerformSync();
    }
    emit('status', ui.value.syncToggled);
  } catch (error) {
    console.error('Failed to toggle sync:', error);
    settings.value.enabled = !settings.value.enabled;
    emit('status', ui.value.toggleError);
  }
}

async function onSettingChange(): Promise<void> {
  const previousSettings = { ...settings.value };
  try {
    await updateSyncSettings(settings.value);
    emit('status', ui.value.settingsSaved);
  } catch (error) {
    console.error('Failed to update settings:', error);
    settings.value = previousSettings;
    emit('status', ui.value.saveError);
  }
}

async function onPerformSync(): Promise<void> {
  try {
    await performSync();
    emit('status', ui.value.syncCompleted);
  } catch (error) {
    console.error('Sync failed:', error);
    emit('status', ui.value.syncFailed);
  }
}

function copyDeviceId(): void {
  if (deviceId.value) {
    navigator.clipboard.writeText(deviceId.value).catch(err => {
      console.error('Failed to copy device ID:', err);
    });
    emit('status', ui.value.deviceIdCopied);
  }
}

async function clearLog(): Promise<void> {
  try {
    await clearSyncLog();
    emit('status', ui.value.logCleared);
  } catch (error) {
    console.error('Failed to clear log:', error);
    emit('status', ui.value.clearLogError);
  }
}

async function resetToDefaults(): Promise<void> {
  try {
    settings.value = { ...DEFAULT_SETTINGS };
    await updateSyncSettings(settings.value);
    emit('status', ui.value.reset);
  } catch (error) {
    console.error('Failed to reset settings:', error);
    emit('status', ui.value.resetError);
  }
}

function formatLogTime(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

onMounted(async () => {
  await initialize();
  settings.value = { ...syncSettings.value };
  loading.value = false;
});

function syncSettingsStrings(locale: AppLocale | undefined) {
  const isZh = locale === 'zh';
  return {
    title: isZh ? '同步' : 'Sync',
    hint: isZh ? '在设备间同步书签和设置' : 'Synchronize bookmarks and settings across devices',
    loading: isZh ? '加载中...' : 'Loading...',
    enableLabel: isZh ? '启用同步' : 'Enable sync',
    deviceIdLabel: isZh ? '设备 ID' : 'Device ID',
    copyDeviceId: isZh ? '复制设备 ID' : 'Copy device ID',
    copy: isZh ? '复制' : 'Copy',
    lastSyncLabel: isZh ? '上次同步' : 'Last sync',
    neverSynced: isZh ? '从未同步' : 'Never synced',
    syncNow: isZh ? '立即同步' : 'Sync now',
    syncing: isZh ? '同步中...' : 'Syncing...',
    statsLabel: isZh ? '同步统计' : 'Sync statistics',
    totalBookmarks: isZh ? '总书签数' : 'Total bookmarks',
    syncedBookmarks: isZh ? '已同步书签' : 'Synced bookmarks',
    totalFolders: isZh ? '总文件夹数' : 'Total folders',
    syncedFolders: isZh ? '已同步文件夹' : 'Synced folders',
    synced: isZh ? '已同步' : 'synced',
    advancedLabel: isZh ? '高级设置' : 'Advanced settings',
    autoSyncLabel: isZh ? '自动同步' : 'Auto-sync on changes',
    autoSyncHint: isZh ? '书签修改时自动同步' : 'Automatically sync when bookmarks are modified',
    syncAcrossDevicesLabel: isZh ? '跨设备同步' : 'Sync across devices',
    syncAcrossDevicesHint: isZh ? '启用跨设备同步' : 'Enable cross-device synchronization',
    syncIntervalLabel: isZh ? '同步间隔' : 'Sync interval',
    syncIntervalHint: isZh ? '自动同步频率（分钟）' : 'How often to auto-sync (in minutes)',
    oneMinute: isZh ? '1 分钟' : '1 minute',
    fiveMinutes: isZh ? '5 分钟' : '5 minutes',
    fifteenMinutes: isZh ? '15 分钟' : '15 minutes',
    thirtyMinutes: isZh ? '30 分钟' : '30 minutes',
    oneHour: isZh ? '1 小时' : '1 hour',
    conflictResolutionLabel: isZh ? '冲突解决' : 'Conflict resolution',
    conflictResolutionHint: isZh ? '如何处理同步冲突' : 'How to handle sync conflicts',
    serverWins: isZh ? '服务器优先' : 'Server wins',
    clientWins: isZh ? '客户端优先' : 'Client wins',
    manualResolve: isZh ? '手动解决' : 'Manual resolve',
    logLabel: isZh ? '同步日志' : 'Sync log',
    clearLog: isZh ? '清除日志' : 'Clear log',
    noLogActivity: isZh ? '暂无同步活动' : 'No sync activity yet',
    reset: isZh ? '重置为默认值' : 'Reset to defaults',
    syncToggled: isZh ? '同步设置已更新' : 'Sync settings updated',
    toggleError: isZh ? '更新同步设置失败' : 'Failed to update sync settings',
    settingsSaved: isZh ? '同步设置已保存' : 'Sync settings saved',
    saveError: isZh ? '保存同步设置失败' : 'Failed to save sync settings',
    syncCompleted: isZh ? '同步完成' : 'Sync completed',
    syncFailed: isZh ? '同步失败' : 'Sync failed',
    deviceIdCopied: isZh ? '设备 ID 已复制' : 'Device ID copied',
    logCleared: isZh ? '日志已清除' : 'Log cleared',
    clearLogError: isZh ? '清除日志失败' : 'Failed to clear log',
    resetError: isZh ? '重置同步设置失败' : 'Failed to reset sync settings',
  };
}
</script>

<style scoped>
.setting-row {
  margin: 16px 0;
}

.setting-value-with-action {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.setting-value-with-action span {
  flex: 1;
  font-family: monospace;
  font-size: 13px;
  color: var(--color-text-secondary, #6b7280);
}

.settings-stats {
  margin: 24px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #f3f4f6);
  border-radius: 8px;
}

.settings-stats h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  font-size: 13px;
  color: var(--color-text-secondary, #6b7280);
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
}

.sync-progress {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar {
  flex: 1;
  height: 6px;
  background: var(--color-bg-tertiary, #e5e7eb);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary, #3b82f6);
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
  white-space: nowrap;
}

.settings-advanced {
  margin: 24px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #f3f4f6);
  border-radius: 8px;
}

.settings-advanced h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
}

.settings-log {
  margin: 24px 0;
  padding: 16px;
  background: var(--color-bg-secondary, #f3f4f6);
  border-radius: 8px;
}

.settings-log h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
}

.log-list {
  margin-top: 12px;
  max-height: 200px;
  overflow-y: auto;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-bg-primary, #ffffff);
  border-radius: 4px;
  font-size: 12px;
  border-left: 3px solid var(--color-primary, #3b82f6);
  margin-bottom: 4px;
}

.log-entry.error {
  border-left-color: var(--color-error, #ef4444);
}

.log-time {
  color: var(--color-text-secondary, #6b7280);
  white-space: nowrap;
}

.log-action {
  color: var(--color-text-primary, #111827);
  font-weight: 500;
}

.log-item {
  color: var(--color-text-secondary, #6b7280);
  flex: 1;
}

.log-error {
  color: var(--color-error, #ef4444);
  font-size: 11px;
}

.empty-state {
  margin-top: 12px;
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary, #6b7280);
  font-size: 13px;
}

.error-message {
  margin: 16px 0;
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--color-error, #ef4444);
  border-radius: 4px;
  color: var(--color-error, #ef4444);
  font-size: 13px;
}

@media (prefers-color-scheme: dark) {
  .settings-stats,
  .settings-advanced,
  .settings-log {
    background: #2d2d2d;
  }

  .log-entry {
    background: #3d3d3d;
  }

  .stat-label,
  .log-item,
  .progress-text,
  .empty-state {
    color: #9ca3af;
  }

  .log-action {
    color: #e5e7eb;
  }

  .stat-value {
    color: #f3f4f6;
  }

  .progress-bar {
    background: #4d4d4d;
  }
}
</style>
