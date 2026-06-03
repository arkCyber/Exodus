/**
 * Tab bar handler builder tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { buildTabBarHandlers } from './useBrowserTabBarHandlers';

describe('buildTabBarHandlers', () => {
  it('wires middle-click to close tab', () => {
    const closeTab = vi.fn();
    const handlers = buildTabBarHandlers({
      activateTab: vi.fn(),
      createNewTab: vi.fn(),
      closeTab,
      toggleTabPin: vi.fn(),
      duplicateTab: vi.fn(),
      reorderTabs: vi.fn(),
      tabGroups: {
        openTabContextMenu: vi.fn(),
        closeTabContextMenu: vi.fn(),
      } as never,
    });
    const ev = { button: 1, preventDefault: vi.fn() } as unknown as MouseEvent;
    handlers.tabMouseDown(ev, 'tab-1');
    expect(ev.preventDefault).toHaveBeenCalled();
    expect(closeTab).toHaveBeenCalledWith('tab-1', true);
  });
});
