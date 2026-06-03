/**
 * Unit tests for synced tabs store.
 */

import { describe, expect, it, beforeEach, vi } from 'vitest';

vi.mock('$lib/mobileSync', () => ({
  isMobileSyncEnabled: vi.fn(async () => false),
  getMobileSyncSettings: vi.fn(async () => ({ enabled: false })),
}));
import { NEWTAB_INTERNAL_URL } from './newTabPage';
import {
  buildThisDeviceSyncedTabs,
  loadSyncedDevices,
  refreshSyncedTabs,
  saveSyncedDevices,
} from './syncedTabs';

describe('syncedTabs', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('seeds demo devices when empty', () => {
    const devices = loadSyncedDevices();
    expect(devices.length).toBeGreaterThanOrEqual(1);
    expect(devices[0].tabs.length).toBeGreaterThan(0);
  });

  it('persists devices', () => {
    saveSyncedDevices([
      { deviceId: 'x', deviceName: 'Test', tabs: [{ id: '1', title: 'T', url: 'https://a.com', lastActive: '' }] },
    ]);
    expect(loadSyncedDevices()[0].deviceName).toBe('Test');
  });

  it('excludes new-tab URLs from this computer', () => {
    const device = buildThisDeviceSyncedTabs([
      { id: '1', title: 'NTP', url: NEWTAB_INTERNAL_URL },
      { id: '2', title: 'Site', url: 'https://example.com' },
    ]);
    expect(device.tabs).toHaveLength(1);
    expect(device.tabs[0]?.url).toContain('example.com');
  });

  it('includes this computer when open tabs passed', async () => {
    const devices = await refreshSyncedTabs([
      { id: 't1', title: 'Example', url: 'https://example.com' },
    ]);
    expect(devices[0]?.deviceId).toBe('this-device');
    expect(devices[0]?.tabs[0]?.url).toContain('example.com');
  });
});
