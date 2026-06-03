<template>
  <div class="bookmark-sync-settings">
    <div class="sync-header">
      <h3>Bookmark Sync</h3>
      <div class="sync-status" :class="syncStatusClass">
        <span class="status-indicator"></span>
        {{ syncStatusText }}
      </div>
    </div>

    <div class="sync-content">
      <!-- Sync Toggle -->
      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label">Enable Sync</label>
          <p class="setting-description">Synchronize bookmarks across your devices</p>
        </div>
        <label class="toggle-switch">
          <input
            type="checkbox"
            v-model="settings.enabled"
            @change="onToggleSync"
            :disabled="isSyncing"
          />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <!-- Device ID -->
      <div class="setting-row" v-if="deviceId">
        <div class="setting-info">
          <label class="setting-label">Device ID</label>
          <p class="setting-description">{{ deviceId }}</p>
        </div>
        <button @click="copyDeviceId" class="icon-btn" title="Copy device ID">
          <svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>

      <!-- Last Sync -->
      <div class="setting-row" v-if="settings.enabled">
        <div class="setting-info">
          <label class="setting-label">Last Sync</label>
          <p class="setting-description">{{ lastSyncTime || 'Never synced' }}</p>
        </div>
        <button
          @click="onPerformSync"
          class="action-btn"
          :disabled="isSyncing"
          title="Sync now"
        >
          <svg v-if="!isSyncing" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <svg v-else class="spin" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          {{ isSyncing ? 'Syncing...' : 'Sync Now' }}
        </button>
      </div>

      <!-- Sync Statistics -->
      <div class="sync-stats" v-if="settings.enabled">
        <h4>Sync Statistics</h4>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">Total Bookmarks</span>
            <span class="stat-value">{{ stats.totalBookmarks }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Synced Bookmarks</span>
            <span class="stat-value">{{ stats.syncedBookmarks }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Total Folders</span>
            <span class="stat-value">{{ stats.totalFolders }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Synced Folders</span>
            <span class="stat-value">{{ stats.syncedFolders }}</span>
          </div>
        </div>
        <div class="sync-progress">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${stats.syncProgress}%` }"></div>
          </div>
          <span class="progress-text">{{ Math.round(stats.syncProgress) }}% synced</span>
        </div>
      </div>

      <!-- Advanced Settings -->
      <div class="advanced-settings" v-if="settings.enabled">
        <h4>Advanced Settings</h4>
        
        <div class="setting-row">
          <div class="setting-info">
            <label class="setting-label">Auto-sync on changes</label>
            <p class="setting-description">Automatically sync when bookmarks are modified</p>
          </div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              v-model="settings.auto_sync"
              @change="onSettingChange"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <label class="setting-label">Sync across devices</label>
            <p class="setting-description">Enable cross-device synchronization</p>
          </div>
          <label class="toggle-switch">
            <input
              type="checkbox"
              v-model="settings.sync_across_devices"
              @change="onSettingChange"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <label class="setting-label">Sync Interval</label>
            <p class="setting-description">How often to auto-sync (in minutes)</p>
          </div>
          <select
            v-model.number="settings.sync_interval"
            @change="onSettingChange"
            class="setting-select"
          >
            <option :value="60">1 minute</option>
            <option :value="300">5 minutes</option>
            <option :value="900">15 minutes</option>
            <option :value="1800">30 minutes</option>
            <option :value="3600">1 hour</option>
          </select>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <label class="setting-label">Conflict Resolution</label>
            <p class="setting-description">How to handle sync conflicts</p>
          </div>
          <select
            v-model="settings.conflict_resolution"
            @change="onSettingChange"
            class="setting-select"
          >
            <option value="local_wins">Local wins</option>
            <option value="remote_wins">Remote wins</option>
            <option value="manual">Manual resolution</option>
          </select>
        </div>
      </div>

      <!-- Sync Log -->
      <div class="sync-log-section" v-if="settings.enabled">
        <div class="sync-log-header">
          <h4>Sync Log</h4>
          <button @click="clearLog" class="text-btn" title="Clear log">
            Clear
          </button>
        </div>
        <div v-if="syncLog.length > 0" class="sync-log-list">
          <div
            v-for="(entry, index) in syncLog.slice().reverse()"
            :key="index"
            class="log-entry"
            :class="{ error: !entry.success }"
          >
            <span class="log-time">{{ formatLogTime(entry.timestamp) }}</span>
            <span class="log-action">{{ entry.action }}</span>
            <span class="log-item">{{ entry.item_type }}: {{ entry.item_id }}</span>
            <span v-if="entry.error_message" class="log-error">{{ entry.error_message }}</span>
          </div>
        </div>
        <div v-else class="empty-state">No sync activity yet</div>
      </div>

      <!-- Error Message -->
      <div v-if="syncError" class="error-message">
        <svg viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
        {{ syncError }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useBookmarkSync, type SyncSettings } from '@/composables/useBookmarkSync';

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

const syncStatusClass = computed(() => {
  if (isSyncing.value) return 'syncing';
  if (syncError.value) return 'error';
  if (settings.value.enabled) return 'synced';
  return 'disabled';
});

const syncStatusText = computed(() => {
  if (isSyncing.value) return 'Syncing...';
  if (syncError.value) return 'Sync Error';
  if (settings.value.enabled) return 'Sync Enabled';
  return 'Sync Disabled';
});

const stats = computed(() => syncStats.value);

async function onToggleSync(): Promise<void> {
  try {
    await updateSyncSettings(settings.value);
    if (settings.value.enabled) {
      await onPerformSync();
    }
  } catch (error) {
    console.error('Failed to toggle sync:', error);
    // Revert the toggle if it failed
    settings.value.enabled = !settings.value.enabled;
  }
}

async function onSettingChange(): Promise<void> {
  const previousSettings = { ...settings.value };
  try {
    await updateSyncSettings(settings.value);
  } catch (error) {
    console.error('Failed to update settings:', error);
    // Revert to previous settings on error
    settings.value = previousSettings;
  }
}

async function onPerformSync(): Promise<void> {
  try {
    await performSync();
  } catch (error) {
    console.error('Sync failed:', error);
  }
}

function copyDeviceId(): void {
  if (deviceId.value) {
    navigator.clipboard.writeText(deviceId.value).catch(err => {
      console.error('Failed to copy device ID:', err);
    });
  }
}

async function clearLog(): Promise<void> {
  if (confirm('Are you sure you want to clear the sync log?')) {
    try {
      await clearSyncLog();
    } catch (error) {
      console.error('Failed to clear log:', error);
    }
  }
}

function formatLogTime(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

onMounted(async () => {
  await initialize();
  settings.value = { ...syncSettings.value };
});
</script>

<style scoped>
.bookmark-sync-settings {
  padding: 20px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.sync-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid var(--border-color);
}

.sync-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.sync-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: 20px;
}

.sync-status.synced {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.sync-status.syncing {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.sync-status.error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.sync-status.disabled {
  background: rgba(156, 163, 175, 0.1);
  color: #9ca3af;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.sync-status.syncing .status-indicator {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid var(--border-color);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
}

.setting-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.setting-description {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

.toggle-switch {
  position: relative;
  width: 44px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border-color);
  transition: 0.3s;
  border-radius: 24px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch input:checked + .toggle-slider {
  background-color: #3b82f6;
}

.toggle-switch input:checked + .toggle-slider:before {
  transform: translateX(20px);
}

.toggle-switch input:disabled + .toggle-slider {
  opacity: 0.5;
  cursor: not-allowed;
}

.setting-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
}

.setting-select:focus {
  outline: none;
  border-color: #3b82f6;
}

.sync-stats {
  margin: 20px 0;
  padding: 16px;
  background: var(--bg-primary);
  border-radius: 8px;
}

.sync-stats h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.sync-progress {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar {
  flex: 1;
  height: 6px;
  background: var(--border-color);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #3b82f6;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.advanced-settings {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}

.advanced-settings h4 {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover:not(:disabled) {
  background: var(--bg-secondary);
  border-color: #3b82f6;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.icon-btn:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.icon-btn svg {
  width: 16px;
  height: 16px;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.error-message {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 6px;
  color: #ef4444;
  font-size: 14px;
}

.error-message svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.sync-log-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}

.sync-log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.sync-log-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.text-btn {
  padding: 4px 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.text-btn:hover {
  background: var(--bg-primary);
  color: var(--text-primary);
}

.sync-log-list {
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-primary);
  border-radius: 4px;
  font-size: 12px;
  border-left: 3px solid #3b82f6;
}

.log-entry.error {
  border-left-color: #ef4444;
}

.log-time {
  color: var(--text-secondary);
  white-space: nowrap;
}

.log-action {
  color: var(--text-primary);
  font-weight: 500;
}

.log-item {
  color: var(--text-secondary);
  flex: 1;
}

.log-error {
  color: #ef4444;
  font-size: 11px;
}
</style>
