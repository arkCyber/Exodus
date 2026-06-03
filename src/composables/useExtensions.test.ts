/**
 * Exodus Browser — useExtensions composable tests.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';
import { ref } from 'vue';

const invokeMock = vi.fn();
const listenMock = vi.fn(async () => () => {});

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: listenMock,
}));

vi.mock('$lib/extensions/syncTabs', () => ({
  syncExtensionTabs: vi.fn(),
}));

vi.mock('$lib/extensions/backgroundHosts', () => ({
  ensureExtensionBackgrounds: vi.fn(async () => {}),
}));

vi.mock('$lib/extensions/tabOps', () => ({
  listenExtensionTabOps: vi.fn(async (cb: (ops: unknown[]) => void) => {
    cb([{ op: 'reload', chromeTabId: 1 }]);
    return () => {};
  }),
}));

vi.mock('$lib/extensions/extensionEvents', () => ({
  flushExtensionTab: vi.fn(),
  pumpExtensionRuntime: vi.fn(),
  listenExtensionTabCreates: vi.fn(async () => () => {}),
  listenExtensionPermissionRequests: vi.fn(async () => () => {}),
  listenExtensionHostInstallRequests: vi.fn(async () => () => {}),
  listenExtensionNotifications: vi.fn(async () => () => {}),
  listenExtensionHostDenied: vi.fn(async () => () => {}),
}));

vi.mock('$lib/exodusBrowser', () => ({
  canUseNativeWebview: () => true,
  tabWebviewLabel: (id: string) => `exodus-tab-${id}`,
}));

import { useExtensions } from './useExtensions';
import { syncExtensionTabs } from '$lib/extensions/syncTabs';

describe('useExtensions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invokeMock.mockResolvedValue(undefined);
  });

  it('syncRegistry calls syncExtensionTabs with active tab id', async () => {
    const tabs = [{ id: 'a', title: 'A', url: 'https://a.test' }];
    const contentHost = ref<HTMLElement | undefined>(document.createElement('div'));
    const onTabOps = vi.fn();

    const ext = useExtensions({
      getTabs: () => tabs,
      getActiveTabId: () => 'a',
      contentHost,
      onTabOps,
      onTabCreates: async () => [],
    });

    ext.syncRegistry();
    expect(syncExtensionTabs).toHaveBeenCalledWith(tabs, 'a');
  });

  it('setup registers extension listeners and starts pump timer', async () => {
    vi.useFakeTimers();
    const onTabOps = vi.fn();
    const contentHost = ref<HTMLElement | undefined>(document.createElement('div'));

    const ext = useExtensions({
      getTabs: () => [{ id: 'a', title: 'A', url: 'https://a.test' }],
      getActiveTabId: () => 'a',
      contentHost,
      onTabOps,
      onTabCreates: async () => [],
    });

    await ext.setup();
    expect(onTabOps).toHaveBeenCalled();
    expect(syncExtensionTabs).toHaveBeenCalled();

    ext.teardown();
    vi.useRealTimers();
  });

  it('dismissPermPrompt clears active request', async () => {
    const contentHost = ref<HTMLElement | undefined>(undefined);
    const ext = useExtensions({
      getTabs: () => [],
      getActiveTabId: () => null,
      contentHost,
      onTabOps: vi.fn(),
      onTabCreates: async () => [],
    });

    ext.permRequest.value = {
      extensionId: 'x',
      extensionName: 'X',
      requestId: '1',
      permissions: [],
    };
    ext.dismissPermPrompt();
    expect(ext.permRequest.value).toBeNull();
  });
});
