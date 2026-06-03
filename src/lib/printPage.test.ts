/**
 * Unit tests — printActivePage (iframe + chrome internal guards).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { printActivePage } from './printPage';
import { NEWTAB_INTERNAL_URL } from './newTabPage';

vi.mock('./tauri', () => ({
  canInvokeTauri: () => false,
}));

vi.mock('./exodusBrowser', () => ({
  evalInTab: vi.fn(),
  tabWebviewLabel: (id: string) => `tab-${id}`,
}));

describe('printActivePage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns false for new-tab and chrome internal URLs', async () => {
    expect(
      await printActivePage({
        useNativeWebview: false,
        activeTabId: 't1',
        tabUrl: NEWTAB_INTERNAL_URL,
        iframe: null,
      }),
    ).toBe(false);
    expect(
      await printActivePage({
        useNativeWebview: false,
        activeTabId: 't1',
        tabUrl: 'chrome://settings',
        iframe: null,
      }),
    ).toBe(false);
  });

  it('calls iframe contentWindow.print when available', async () => {
    const printFn = vi.fn();
    const iframe = {
      contentWindow: { print: printFn },
    } as unknown as HTMLIFrameElement;

    const ok = await printActivePage({
      useNativeWebview: false,
      activeTabId: 't1',
      tabUrl: 'https://example.com',
      iframe,
    });

    expect(ok).toBe(true);
    expect(printFn).toHaveBeenCalledOnce();
  });
});
