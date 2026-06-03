/**
 * Exodus Browser — useBrowserSession unit tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useBrowserSession } from './useBrowserSession';
import { NEWTAB_PAGE_URL } from '$lib/newTabPage';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

describe('useBrowserSession', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('saveSession invokes save_session when restore enabled', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const sessionRestore = ref(true);
    const privateMode = ref(false);
    const session = useBrowserSession({
      getTabs: () => [{ id: 't1', url: NEWTAB_PAGE_URL, title: 'New Tab' }],
      getActiveTabId: () => 't1',
      getSortedTabs: () => [{ id: 't1', url: NEWTAB_PAGE_URL, title: 'New Tab' }],
      sessionRestore,
      privateMode,
      createTabId: () => 't2',
      activateTab: vi.fn(),
      onStatus: vi.fn(),
      newTabPageUrl: NEWTAB_PAGE_URL,
    });

    await session.saveSession();
    expect(invoke).toHaveBeenCalledWith('save_session', {
      tabs: [{ id: 't1', url: NEWTAB_PAGE_URL, title: 'New Tab', active: true }],
      activeTabId: 't1',
    });
  });

  it('skips save when private mode', async () => {
    const session = useBrowserSession({
      getTabs: () => [],
      getActiveTabId: () => null,
      getSortedTabs: () => [],
      sessionRestore: ref(true),
      privateMode: ref(true),
      createTabId: () => 't',
      activateTab: vi.fn(),
      onStatus: vi.fn(),
      newTabPageUrl: NEWTAB_PAGE_URL,
    });
    await session.saveSession();
    expect(invoke).not.toHaveBeenCalled();
  });

  it('loadSession returns payload for single new-tab', async () => {
    vi.mocked(invoke).mockResolvedValue({
      tabs: [
        { id: 'a', url: 'https://example.com', title: 'Ex' },
        { id: 'b', url: 'https://b.com', title: 'B' },
      ],
      activeTabId: 'b',
    });

    const session = useBrowserSession({
      getTabs: () => [{ id: 't1', url: NEWTAB_PAGE_URL, title: 'New Tab' }],
      getActiveTabId: () => 't1',
      getSortedTabs: () => [{ id: 't1', url: NEWTAB_PAGE_URL, title: 'New Tab' }],
      sessionRestore: ref(true),
      privateMode: ref(false),
      createTabId: () => 'new-id',
      activateTab: vi.fn(),
      onStatus: vi.fn(),
      newTabPageUrl: NEWTAB_PAGE_URL,
    });

    const payload = await session.loadSession();
    expect(payload?.targetId).toBe('b');
    expect(payload?.restored).toHaveLength(2);
  });
});
