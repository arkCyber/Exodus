/**
 * Tests for useBookmarkSync composable
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { useBookmarkSync } from './useBookmarkSync';

describe('useBookmarkSync', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Initialization', () => {
    it('initializes with default settings', () => {
      const { syncSettings } = useBookmarkSync();
      
      expect(syncSettings.value).toEqual({
        enabled: false,
        sync_interval: 300,
        auto_sync: true,
        sync_across_devices: true,
        conflict_resolution: 'local_wins',
        last_sync: 0,
      });
    });

    it('initializes with empty state', () => {
      const { syncBookmarks, syncFolders, syncLog, deviceId, isSyncing, syncError } = useBookmarkSync();
      
      expect(syncBookmarks.value).toEqual([]);
      expect(syncFolders.value).toEqual([]);
      expect(syncLog.value).toEqual([]);
      expect(deviceId.value).toBe('');
      expect(isSyncing.value).toBe(false);
      expect(syncError.value).toBe(null);
    });
  });

  describe('Computed Properties', () => {
    it('computes isSyncEnabled correctly', () => {
      const { syncSettings, isSyncEnabled } = useBookmarkSync();
      
      expect(isSyncEnabled.value).toBe(false);
      
      syncSettings.value.enabled = true;
      expect(isSyncEnabled.value).toBe(true);
    });

    it('computes lastSyncTime correctly', () => {
      const { syncSettings, lastSyncTime } = useBookmarkSync();
      
      expect(lastSyncTime.value).toBe(null);
      
      syncSettings.value.last_sync = 1234567890;
      expect(lastSyncTime.value).toBeTruthy();
    });

    it('computes syncStats correctly', () => {
      const { syncBookmarks, syncFolders, syncStats } = useBookmarkSync();
      
      syncBookmarks.value = [
        { id: '1', url: 'https://test.com', title: 'Test', sync_status: 'Synced' as const, device_id: 'dev1', position: 0, date_added: 0, last_modified: 0 },
        { id: '2', url: 'https://test2.com', title: 'Test2', sync_status: 'NotSynced' as const, device_id: 'dev1', position: 0, date_added: 0, last_modified: 0 },
      ];
      
      syncFolders.value = [
        { id: '1', title: 'Folder1', sync_status: 'Synced' as const, device_id: 'dev1', position: 0, date_added: 0, last_modified: 0 },
      ];
      
      expect(syncStats.value).toEqual({
        totalBookmarks: 2,
        syncedBookmarks: 1,
        totalFolders: 1,
        syncedFolders: 1,
        syncProgress: 50,
      });
    });
  });

  describe('Input Validation', () => {
    it('validates URL in addSyncBookmark', async () => {
      const { addSyncBookmark } = useBookmarkSync();
      
      await expect(addSyncBookmark('', 'Test')).rejects.toThrow('URL cannot be empty');
      await expect(addSyncBookmark('invalid-url', 'Test')).rejects.toThrow('URL must start with http:// or https://');
    });

    it('validates title in addSyncBookmark', async () => {
      const { addSyncBookmark } = useBookmarkSync();
      
      await expect(addSyncBookmark('https://test.com', '')).rejects.toThrow('Title cannot be empty');
    });

    it('validates ID in updateSyncBookmark', async () => {
      const { updateSyncBookmark } = useBookmarkSync();
      
      await expect(updateSyncBookmark('', 'https://test.com', 'Test')).rejects.toThrow('Bookmark ID cannot be empty');
    });

    it('validates URL in updateSyncBookmark', async () => {
      const { updateSyncBookmark } = useBookmarkSync();
      
      await expect(updateSyncBookmark('1', '', 'Test')).rejects.toThrow('URL cannot be empty');
      await expect(updateSyncBookmark('1', 'invalid-url', 'Test')).rejects.toThrow('URL must start with http:// or https://');
    });

    it('validates title in updateSyncBookmark', async () => {
      const { updateSyncBookmark } = useBookmarkSync();
      
      await expect(updateSyncBookmark('1', 'https://test.com', '')).rejects.toThrow('Title cannot be empty');
    });

    it('validates ID in removeSyncBookmark', async () => {
      const { removeSyncBookmark } = useBookmarkSync();
      
      await expect(removeSyncBookmark('')).rejects.toThrow('Bookmark ID cannot be empty');
    });

    it('validates title in addSyncFolder', async () => {
      const { addSyncFolder } = useBookmarkSync();
      
      await expect(addSyncFolder('')).rejects.toThrow('Folder title cannot be empty');
    });

    it('validates ID in updateSyncFolder', async () => {
      const { updateSyncFolder } = useBookmarkSync();
      
      await expect(updateSyncFolder('', 'Test')).rejects.toThrow('Folder ID cannot be empty');
    });

    it('validates title in updateSyncFolder', async () => {
      const { updateSyncFolder } = useBookmarkSync();
      
      await expect(updateSyncFolder('1', '')).rejects.toThrow('Folder title cannot be empty');
    });

    it('validates ID in removeSyncFolder', async () => {
      const { removeSyncFolder } = useBookmarkSync();
      
      await expect(removeSyncFolder('')).rejects.toThrow('Folder ID cannot be empty');
    });
  });

  describe('Error Handling', () => {
    it('handles loadSyncSettings errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to load'));
      
      const { loadSyncSettings, syncError } = useBookmarkSync();
      
      await loadSyncSettings();
      
      expect(syncError.value).toBe('Failed to load sync settings');
    });

    it('handles updateSyncSettings errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to update'));
      
      const { updateSyncSettings, syncError } = useBookmarkSync();
      
      await expect(updateSyncSettings({ enabled: true, sync_interval: 300, auto_sync: true, sync_across_devices: true, conflict_resolution: 'local_wins', last_sync: 0 }))
        .rejects.toThrow();
      
      expect(syncError.value).toBe('Failed to update sync settings');
    });

    it('handles performSync errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Sync failed'));
      
      const { performSync, syncError } = useBookmarkSync();
      
      await expect(performSync()).rejects.toThrow();
      
      expect(syncError.value).toBe('Sync failed');
    });

    it('handles clearSyncLog errors', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed to clear'));
      
      const { clearSyncLog, syncError } = useBookmarkSync();
      
      await expect(clearSyncLog()).rejects.toThrow();
      
      expect(syncError.value).toBe('Failed to clear sync log');
    });
  });

  describe('Boundary Conditions', () => {
    it('handles whitespace-only inputs', async () => {
      const { addSyncBookmark } = useBookmarkSync();
      
      await expect(addSyncBookmark('   ', 'Test')).rejects.toThrow('URL cannot be empty');
      await expect(addSyncBookmark('https://test.com', '   ')).rejects.toThrow('Title cannot be empty');
    });

    it('handles sync during sync', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);
      
      const { performSync, isSyncing } = useBookmarkSync();
      
      isSyncing.value = true;
      
      await performSync();
      
      expect(invoke).not.toHaveBeenCalled();
    });

    it('handles empty sync log', async () => {
      vi.mocked(invoke).mockResolvedValue([]);
      
      const { loadSyncLog, syncLog } = useBookmarkSync();
      
      await loadSyncLog();
      
      expect(syncLog.value).toEqual([]);
    });
  });
});
