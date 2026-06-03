/**
 * Exodus Browser — extension tab sync tests.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('$lib/exodusBrowser', () => ({
  tabWebviewLabel: (id: string) => `exodus-tab-${id}`,
}));

import { syncExtensionTabs } from '$lib/extensions/syncTabs';

describe('syncExtensionTabs', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('calls extension_sync_tabs with webview labels', async () => {
    await syncExtensionTabs(
      [
        { id: 'a', title: 'A', url: 'https://a.com', webview: null },
        { id: 'b', title: 'B', url: 'https://b.com', webview: null },
      ],
      'b',
    );
    expect(invokeMock).toHaveBeenCalledWith('extension_sync_tabs', {
      tabs: [
        {
          id: 'a',
          chromeTabId: 1,
          index: 0,
          webviewLabel: 'exodus-tab-a',
          url: 'https://a.com',
          title: 'A',
          active: false,
        },
        {
          id: 'b',
          chromeTabId: 2,
          index: 1,
          webviewLabel: 'exodus-tab-b',
          url: 'https://b.com',
          title: 'B',
          active: true,
        },
      ],
    });
  });
});
