/**
 * Exodus Browser — extension context menu host API tests.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { listExtensionContextMenus, fireExtensionContextMenuClick } from './contextMenus';

describe('contextMenus host API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue([]);
  });

  it('listExtensionContextMenus invokes list_host command', async () => {
    await listExtensionContextMenus('https://example.com', 'page');
    expect(invokeMock).toHaveBeenCalledWith('extension_context_menus_list_host', {
      pageUrl: 'https://example.com',
      hostContext: 'page',
    });
  });

  it('fireExtensionContextMenuClick invokes click command', async () => {
    await fireExtensionContextMenuClick('ext-1', 'ext-1:item', 'https://a.test');
    expect(invokeMock).toHaveBeenCalledWith('extension_context_menu_clicked', {
      extensionId: 'ext-1',
      menuItemId: 'ext-1:item',
      pageUrl: 'https://a.test',
    });
  });
});
