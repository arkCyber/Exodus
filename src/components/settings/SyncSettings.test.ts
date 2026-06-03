/**
 * Tests for SyncSettings component
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { ref } from 'vue';
import SyncSettings from './SyncSettings.vue';
import { useBookmarkSync, type SyncSettings as SyncSettingsType } from '@/composables/useBookmarkSync';

// Mock the useBookmarkSync composable
vi.mock('@/composables/useBookmarkSync', () => ({
  useBookmarkSync: vi.fn(),
}));

describe('SyncSettings', () => {
  const mockSyncSettings: SyncSettingsType = {
    enabled: false,
    auto_sync: true,
    sync_across_devices: true,
    sync_interval: 300,
    conflict_resolution: 'local_wins',
    last_sync: 0,
  };

  const mockSyncStats = {
    totalBookmarks: 10,
    syncedBookmarks: 8,
    totalFolders: 3,
    syncedFolders: 2,
    syncProgress: 80,
  };

  const mockSyncLog = [
    {
      timestamp: 1234567890,
      action: 'sync',
      item_type: 'bookmark',
      item_id: '1',
      success: true,
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useBookmarkSync).mockReturnValue({
      syncSettings: ref(mockSyncSettings),
      deviceId: ref('test-device-id'),
      isSyncing: ref(false),
      syncError: ref(null),
      lastSyncTime: ref(null),
      syncStats: ref(mockSyncStats),
      syncLog: ref(mockSyncLog),
      updateSyncSettings: vi.fn(async () => {}),
      performSync: vi.fn(async () => {}),
      clearSyncLog: vi.fn(async () => {}),
      initialize: vi.fn(async () => {}),
    });
  });

  it('renders title and sync toggle', async () => {
    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.text()).toContain('Sync');
    expect(wrapper.find('[data-testid="sync-enabled"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    const checkbox = wrapper.find('[data-testid="sync-enabled"]');
    expect(checkbox.element).toBeInstanceOf(HTMLInputElement);
    const element = checkbox.element as HTMLInputElement;
    expect(element.checked).toBe(false);
  });

  it('emits status when sync toggle changes', async () => {
    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    const checkbox = wrapper.find('[data-testid="sync-enabled"]');
    await checkbox.setValue(true);
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('displays device ID when available', async () => {
    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.text()).toContain('test-device-id');
  });

  it('displays sync statistics when enabled', async () => {
    vi.mocked(useBookmarkSync).mockReturnValue({
      syncSettings: ref({ ...mockSyncSettings, enabled: true }),
      deviceId: ref('test-device-id'),
      isSyncing: ref(false),
      syncError: ref(null),
      lastSyncTime: ref(null),
      syncStats: ref(mockSyncStats),
      syncLog: ref(mockSyncLog),
      updateSyncSettings: vi.fn(async () => {}),
      performSync: vi.fn(async () => {}),
      clearSyncLog: vi.fn(async () => {}),
      initialize: vi.fn(async () => {}),
    });

    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.text()).toContain('Sync statistics');
    expect(wrapper.text()).toContain('10');
    expect(wrapper.text()).toContain('8');
  });

  it('displays Chinese strings when locale is zh', async () => {
    const wrapper = mount(SyncSettings, { props: { uiLocale: 'zh' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.text()).toContain('同步');
  });

  it('shows loading state initially', async () => {
    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    expect(wrapper.text()).toContain('Loading...');
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.text()).not.toContain('Loading...');
  });

  it('displays sync error when present', async () => {
    vi.mocked(useBookmarkSync).mockReturnValue({
      syncSettings: ref(mockSyncSettings),
      deviceId: ref('test-device-id'),
      isSyncing: ref(false),
      syncError: ref('Sync failed'),
      lastSyncTime: ref(null),
      syncStats: ref(mockSyncStats),
      syncLog: ref(mockSyncLog),
      updateSyncSettings: vi.fn(async () => {}),
      performSync: vi.fn(async () => {}),
      clearSyncLog: vi.fn(async () => {}),
      initialize: vi.fn(async () => {}),
    });

    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(wrapper.text()).toContain('Sync failed');
  });

  it('disables sync button when syncing', async () => {
    vi.mocked(useBookmarkSync).mockReturnValue({
      syncSettings: ref({ ...mockSyncSettings, enabled: true }),
      deviceId: ref('test-device-id'),
      isSyncing: ref(true),
      syncError: ref(null),
      lastSyncTime: ref(null),
      syncStats: ref(mockSyncStats),
      syncLog: ref(mockSyncLog),
      updateSyncSettings: vi.fn(async () => {}),
      performSync: vi.fn(async () => {}),
      clearSyncLog: vi.fn(async () => {}),
      initialize: vi.fn(async () => {}),
    });

    const wrapper = mount(SyncSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 0));
    const syncButton = wrapper.find('[data-testid="sync-now"]');
    expect(syncButton.attributes('disabled')).toBeDefined();
  });
});
