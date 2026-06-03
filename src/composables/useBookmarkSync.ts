/**
 * Exodus Browser — Bookmark Sync Composable
 * 
 * This composable provides functionality for bookmark synchronization
 * using the Tauri backend.
 */

import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface SyncSettings {
  enabled: boolean;
  sync_interval: number;
  auto_sync: boolean;
  sync_across_devices: boolean;
  conflict_resolution: string;
  last_sync: number;
}

export interface SyncBookmark {
  id: string;
  url: string;
  title: string;
  parent_id?: string;
  position: number;
  date_added: number;
  last_modified: number;
  sync_status: 'NotSynced' | 'Syncing' | 'Synced' | 'Error';
  device_id: string;
}

export interface SyncFolder {
  id: string;
  title: string;
  parent_id?: string;
  position: number;
  date_added: number;
  last_modified: number;
  sync_status: 'NotSynced' | 'Syncing' | 'Synced' | 'Error';
  device_id: string;
}

export interface SyncLogEntry {
  timestamp: number;
  action: string;
  item_type: string;
  item_id: string;
  success: boolean;
  error_message?: string;
}

export function useBookmarkSync() {
  const syncSettings = ref<SyncSettings>({
    enabled: false,
    sync_interval: 300,
    auto_sync: true,
    sync_across_devices: true,
    conflict_resolution: 'local_wins',
    last_sync: 0,
  });

  const syncBookmarks = ref<SyncBookmark[]>([]);
  const syncFolders = ref<SyncFolder[]>([]);
  const syncLog = ref<SyncLogEntry[]>([]);
  const deviceId = ref<string>('');
  const isSyncing = ref(false);
  const syncError = ref<string | null>(null);

  const isSyncEnabled = computed(() => syncSettings.value.enabled);
  const lastSyncTime = computed(() => {
    if (syncSettings.value.last_sync === 0) return null;
    return new Date(syncSettings.value.last_sync * 1000).toLocaleString();
  });

  const syncStats = computed(() => {
    const totalBookmarks = syncBookmarks.value.length;
    const syncedBookmarks = syncBookmarks.value.filter(b => b.sync_status === 'Synced').length;
    const totalFolders = syncFolders.value.length;
    const syncedFolders = syncFolders.value.filter(f => f.sync_status === 'Synced').length;

    return {
      totalBookmarks,
      syncedBookmarks,
      totalFolders,
      syncedFolders,
      syncProgress: totalBookmarks > 0 ? (syncedBookmarks / totalBookmarks) * 100 : 0,
    };
  });

  async function loadSyncSettings(): Promise<void> {
    try {
      const settings = await invoke<SyncSettings>('get_sync_settings');
      syncSettings.value = settings;
    } catch (error) {
      console.error('Failed to load sync settings:', error);
      syncError.value = 'Failed to load sync settings';
    }
  }

  async function updateSyncSettings(settings: SyncSettings): Promise<void> {
    try {
      await invoke('update_sync_settings', { settings });
      syncSettings.value = settings;
    } catch (error) {
      console.error('Failed to update sync settings:', error);
      syncError.value = 'Failed to update sync settings';
      throw error;
    }
  }

  async function loadSyncBookmarks(): Promise<void> {
    try {
      const bookmarks = await invoke<SyncBookmark[]>('get_all_sync_bookmarks');
      syncBookmarks.value = bookmarks;
    } catch (error) {
      console.error('Failed to load sync bookmarks:', error);
      syncError.value = 'Failed to load sync bookmarks';
    }
  }

  async function loadSyncFolders(): Promise<void> {
    try {
      const folders = await invoke<SyncFolder[]>('get_all_sync_folders');
      syncFolders.value = folders;
    } catch (error) {
      console.error('Failed to load sync folders:', error);
      syncError.value = 'Failed to load sync folders';
    }
  }

  async function loadDeviceId(): Promise<void> {
    try {
      const id = await invoke<string>('get_device_id');
      deviceId.value = id;
    } catch (error) {
      console.error('Failed to load device ID:', error);
    }
  }

  async function performSync(): Promise<void> {
    if (isSyncing.value) {
      console.warn('Sync already in progress, ignoring request');
      return;
    }

    isSyncing.value = true;
    syncError.value = null;

    try {
      await invoke('sync_bookmarks');
      await loadSyncBookmarks();
      await loadSyncFolders();
      await loadSyncSettings();
    } catch (error) {
      console.error('Sync failed:', error);
      syncError.value = 'Sync failed';
      throw error;
    } finally {
      isSyncing.value = false;
    }
  }

  async function addSyncBookmark(url: string, title: string): Promise<string> {
    // Validate inputs
    if (!url || url.trim().length === 0) {
      const error = 'URL cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }
    if (!title || title.trim().length === 0) {
      const error = 'Title cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      const error = 'URL must start with http:// or https://';
      syncError.value = error;
      throw new Error(error);
    }

    try {
      const id = await invoke<string>('add_sync_bookmark', { url, title });
      await loadSyncBookmarks();
      return id;
    } catch (error) {
      console.error('Failed to add sync bookmark:', error);
      syncError.value = 'Failed to add bookmark to sync';
      throw error;
    }
  }

  async function updateSyncBookmark(
    id: string,
    url?: string,
    title?: string
  ): Promise<void> {
    // Validate inputs
    if (!id || id.trim().length === 0) {
      const error = 'Bookmark ID cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }
    if (url !== undefined) {
      if (!url || url.trim().length === 0) {
        const error = 'URL cannot be empty';
        syncError.value = error;
        throw new Error(error);
      }
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        const error = 'URL must start with http:// or https://';
        syncError.value = error;
        throw new Error(error);
      }
    }
    if (title !== undefined && (!title || title.trim().length === 0)) {
      const error = 'Title cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }

    try {
      await invoke('update_sync_bookmark', { id, url, title });
      await loadSyncBookmarks();
    } catch (error) {
      console.error('Failed to update sync bookmark:', error);
      syncError.value = 'Failed to update bookmark in sync';
      throw error;
    }
  }

  async function removeSyncBookmark(id: string): Promise<void> {
    // Validate input
    if (!id || id.trim().length === 0) {
      const error = 'Bookmark ID cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }

    try {
      await invoke('remove_sync_bookmark', { id });
      await loadSyncBookmarks();
    } catch (error) {
      console.error('Failed to remove sync bookmark:', error);
      syncError.value = 'Failed to remove bookmark from sync';
      throw error;
    }
  }

  async function addSyncFolder(title: string): Promise<string> {
    // Validate input
    if (!title || title.trim().length === 0) {
      const error = 'Folder title cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }

    try {
      const id = await invoke<string>('add_sync_folder', { title });
      await loadSyncFolders();
      return id;
    } catch (error) {
      console.error('Failed to add sync folder:', error);
      syncError.value = 'Failed to add folder to sync';
      throw error;
    }
  }

  async function updateSyncFolder(id: string, title?: string): Promise<void> {
    // Validate inputs
    if (!id || id.trim().length === 0) {
      const error = 'Folder ID cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }
    if (title !== undefined && (!title || title.trim().length === 0)) {
      const error = 'Folder title cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }

    try {
      await invoke('update_sync_folder', { id, title });
      await loadSyncFolders();
    } catch (error) {
      console.error('Failed to update sync folder:', error);
      syncError.value = 'Failed to update folder in sync';
      throw error;
    }
  }

  async function removeSyncFolder(id: string): Promise<void> {
    // Validate input
    if (!id || id.trim().length === 0) {
      const error = 'Folder ID cannot be empty';
      syncError.value = error;
      throw new Error(error);
    }

    try {
      await invoke('remove_sync_folder', { id });
      await loadSyncFolders();
    } catch (error) {
      console.error('Failed to remove sync folder:', error);
      syncError.value = 'Failed to remove folder from sync';
      throw error;
    }
  }

  async function loadSyncStats(): Promise<Record<string, number>> {
    try {
      const stats = await invoke<Record<string, number>>('get_sync_stats');
      return stats;
    } catch (error) {
      console.error('Failed to load sync stats:', error);
      return {};
    }
  }

  async function loadSyncLog(): Promise<void> {
    try {
      const log = await invoke<SyncLogEntry[]>('get_sync_log');
      syncLog.value = log;
    } catch (error) {
      console.error('Failed to load sync log:', error);
    }
  }

  async function clearSyncLog(): Promise<void> {
    try {
      await invoke('clear_sync_log');
      await loadSyncLog();
    } catch (error) {
      console.error('Failed to clear sync log:', error);
      syncError.value = 'Failed to clear sync log';
      throw error;
    }
  }

  async function initialize(): Promise<void> {
    await Promise.all([
      loadSyncSettings(),
      loadDeviceId(),
      loadSyncLog(),
    ]);
    
    if (syncSettings.value.enabled) {
      await Promise.all([
        loadSyncBookmarks(),
        loadSyncFolders(),
      ]);
    }
  }

  return {
    // State
    syncSettings,
    syncBookmarks,
    syncFolders,
    syncLog,
    deviceId,
    isSyncing,
    syncError,
    
    // Computed
    isSyncEnabled,
    lastSyncTime,
    syncStats,
    
    // Methods
    loadSyncSettings,
    updateSyncSettings,
    loadSyncBookmarks,
    loadSyncFolders,
    loadDeviceId,
    performSync,
    addSyncBookmark,
    updateSyncBookmark,
    removeSyncBookmark,
    addSyncFolder,
    updateSyncFolder,
    removeSyncFolder,
    loadSyncStats,
    loadSyncLog,
    clearSyncLog,
    initialize,
  };
}
