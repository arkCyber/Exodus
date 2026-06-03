/**
 * Exodus Browser — synced tabs from other devices (Firefox sidebar; local store + mobile sync hook).
 */

import { getMobileSyncSettings, isMobileSyncEnabled } from '$lib/mobileSync';
import { isNewTabUrl } from '$lib/newTabPage';

export type SyncedTabEntry = {
  id: string;
  title: string;
  url: string;
  lastActive: string;
};

export type SyncedDevice = {
  deviceId: string;
  deviceName: string;
  tabs: SyncedTabEntry[];
};

const STORAGE_KEY = 'exodus.synced-tabs.devices';

/** Demo devices when store is empty (Firefox-style placeholder). */
function seedSyncedDevices(): SyncedDevice[] {
  const now = new Date().toISOString();
  return [
    {
      deviceId: 'phone',
      deviceName: 'Exodus Mobile',
      tabs: [
        {
          id: 'phone-1',
          title: 'Mozilla — Firefox sidebar',
          url: 'https://www.mozilla.org/firefox/',
          lastActive: now,
        },
        {
          id: 'phone-2',
          title: 'Exodus docs',
          url: 'https://github.com/',
          lastActive: now,
        },
      ],
    },
    {
      deviceId: 'tablet',
      deviceName: 'Exodus Tablet',
      tabs: [
        {
          id: 'tab-1',
          title: 'Reading list article',
          url: 'https://example.com/article',
          lastActive: now,
        },
      ],
    },
  ];
}

/** Load synced tab devices from localStorage. */
export function loadSyncedDevices(): SyncedDevice[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      const seed = seedSyncedDevices();
      saveSyncedDevices(seed);
      return seed;
    }
    const parsed = JSON.parse(raw) as SyncedDevice[];
    return Array.isArray(parsed) ? parsed : seedSyncedDevices();
  } catch (error) {
    console.error('loadSyncedDevices failed:', error);
    return seedSyncedDevices();
  }
}

/** Persist synced devices. */
export function saveSyncedDevices(devices: SyncedDevice[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(devices));
  } catch (error) {
    console.error('saveSyncedDevices failed:', error);
  }
}

export type OpenTabSnapshot = { id: string; title: string; url: string };

/** Build "This computer" device from open browser tabs. */
export function buildThisDeviceSyncedTabs(openTabs: OpenTabSnapshot[]): SyncedDevice {
  const now = new Date().toISOString();
  return {
    deviceId: 'this-device',
    deviceName: 'This computer',
    tabs: openTabs
      .filter((t) => t.url && !t.url.startsWith('about:blank') && !isNewTabUrl(t.url))
      .map((t) => ({
        id: t.id,
        title: t.title || t.url,
        url: t.url,
        lastActive: now,
      })),
  };
}

/**
 * Refresh synced tabs (uses mobile sync flag; merges open tabs as "This computer").
 */
export async function refreshSyncedTabs(openTabs: OpenTabSnapshot[] = []): Promise<SyncedDevice[]> {
  let devices = loadSyncedDevices().filter((d) => d.deviceId !== 'this-device');
  const local = buildThisDeviceSyncedTabs(openTabs);
  if (local.tabs.length > 0) {
    devices = [local, ...devices];
  }
  try {
    const enabled = await isMobileSyncEnabled();
    if (enabled) {
      const settings = await getMobileSyncSettings();
      if (settings.enabled) {
        const stamp = new Date().toISOString();
        const mobile = devices.find((d) => d.deviceId === 'phone');
        if (mobile) {
          mobile.tabs = mobile.tabs.map((t) => ({ ...t, lastActive: stamp }));
        }
        saveSyncedDevices(devices);
      }
    }
  } catch (error) {
    console.error('refreshSyncedTabs mobile sync failed:', error);
  }
  if (local.tabs.length > 0 && !devices.some((d) => d.deviceId === 'this-device')) {
    devices = [local, ...devices];
  }
  return devices;
}
