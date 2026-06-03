/**
 * Exodus Browser — useBrowserTabLifecycle unit tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useBrowserTabLifecycle } from './useBrowserTabLifecycle';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => undefined),
  isTauri: () => true,
}));

describe('useBrowserTabLifecycle', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockClear();
  });

  it('registers tab when native webview enabled', async () => {
    const useNative = ref(true);
    const lifecycle = useBrowserTabLifecycle({
      getTabs: () => [{ id: 't1', url: 'https://ex.test', title: 'Ex' }],
      getActiveTabId: () => 't1',
      useNativeWebview: useNative,
    });
    await lifecycle.registerTab({ id: 't1', url: 'https://ex.test', title: 'Ex' });
    expect(invoke).toHaveBeenCalledWith('register_tab', expect.any(Object));
    expect(invoke).toHaveBeenCalledWith('tab_sleep_register', expect.any(Object));
  });

  it('skips register when not native webview', async () => {
    const useNative = ref(false);
    const lifecycle = useBrowserTabLifecycle({
      getTabs: () => [],
      getActiveTabId: () => null,
      useNativeWebview: useNative,
    });
    await lifecycle.registerTab({ id: 't1', url: 'https://ex.test', title: 'Ex' });
    expect(invoke).not.toHaveBeenCalled();
  });
});
