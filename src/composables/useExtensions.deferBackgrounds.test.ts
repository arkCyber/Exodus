/**
 * Extension backgrounds are lazy — not created on setup or idle callback.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('$lib/perfLog', () => ({
  logPerf: vi.fn(),
  perfAsync: <T>(_: string, fn: () => Promise<T>) => fn(),
  perfStart: vi.fn(),
  perfEnd: vi.fn(),
}));

vi.mock('$lib/extensions/syncTabs', () => ({
  syncExtensionTabs: vi.fn(),
}));

const ensureExtensionBackgrounds = vi.fn(async () => {});

vi.mock('$lib/extensions/backgroundHosts', () => ({
  ensureExtensionBackgrounds,
}));

vi.mock('$lib/extensions/tabOps', () => ({
  listenExtensionTabOps: vi.fn(async () => () => {}),
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

vi.mock('$lib/extensions/api', () => ({
  listExtensions: vi.fn(async () => []),
}));

vi.mock('$lib/exodusBrowser', () => ({
  canUseNativeWebview: () => true,
  tabWebviewLabel: (id: string) => `exodus-tab-${id}`,
}));

describe('useExtensions.setup', () => {
  beforeEach(() => {
    ensureExtensionBackgrounds.mockClear();
  });

  it('does not create background webviews on setup or idle timer', async () => {
    vi.useFakeTimers();
    const idleSpy = vi.fn((cb: IdleRequestCallback) => {
      window.setTimeout(() => cb({ didTimeout: false, timeRemaining: () => 0 } as IdleDeadline), 100);
    });
    vi.stubGlobal('requestIdleCallback', idleSpy);

    const { useExtensions } = await import('./useExtensions');
    const contentHost = ref(document.createElement('div'));
    document.body.appendChild(contentHost.value);

    const ext = useExtensions({
      getTabs: () => [],
      getActiveTabId: () => null,
      contentHost,
      onTabOps: async () => {},
      onTabCreates: async () => [],
    });

    await ext.setup();
    expect(ensureExtensionBackgrounds).not.toHaveBeenCalled();
    expect(idleSpy).not.toHaveBeenCalled();

    vi.advanceTimersByTime(60_000);
    await Promise.resolve();
    expect(ensureExtensionBackgrounds).not.toHaveBeenCalled();

    await ext.pump();
    expect(ensureExtensionBackgrounds).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
    ext.teardown();
    contentHost.value.remove();
  });
});
