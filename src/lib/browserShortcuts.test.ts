/**
 * Exodus Browser — keyboard shortcut unit tests.
 */

import { describe, expect, it, vi } from 'vitest';
import { handleBrowserShortcut, type BrowserShortcutActions } from './browserShortcuts';

/** Minimal KeyboardEvent stub for Vitest (no DOM). */
function key(
  keyName: string,
  opts: { meta?: boolean; ctrl?: boolean; shift?: boolean } = {},
): KeyboardEvent {
  return {
    key: keyName,
    metaKey: opts.meta ?? false,
    ctrlKey: opts.ctrl ?? false,
    shiftKey: opts.shift ?? false,
    preventDefault: () => {},
  } as KeyboardEvent;
}

function actions(overrides: Partial<BrowserShortcutActions> = {}): BrowserShortcutActions {
  return {
    focusOmnibox: vi.fn(),
    reload: vi.fn(),
    newTab: vi.fn(),
    restoreClosedTab: vi.fn(),
    closeActiveTab: vi.fn(),
    zoomIn: vi.fn(),
    zoomOut: vi.fn(),
    resetZoom: vi.fn(),
    toggleBookmark: vi.fn(),
    toggleBookmarkBar: vi.fn(),
    openHistory: vi.fn(),
    toggleSidebar: vi.fn(),
    toggleFindBar: vi.fn(),
    print: vi.fn(),
    goBack: vi.fn(),
    goForward: vi.fn(),
    switchToTabIndex: vi.fn(),
    onEscape: vi.fn(),
    tabIdsInOrder: () => ['a', 'b', 'c'],
    ...overrides,
  };
}

describe('handleBrowserShortcut', () => {
  it('focuses omnibox on mod+l', () => {
    const a = actions();
    const prevent = vi.fn();
    const e = { ...key('l', { meta: true }), preventDefault: prevent };
    expect(handleBrowserShortcut(e as KeyboardEvent, a)).toBe(true);
    expect(a.focusOmnibox).toHaveBeenCalled();
    expect(prevent).toHaveBeenCalled();
  });

  it('opens new tab on mod+t', () => {
    const a = actions();
    handleBrowserShortcut(key('t', { meta: true }), a);
    expect(a.newTab).toHaveBeenCalled();
    expect(a.restoreClosedTab).not.toHaveBeenCalled();
  });

  it('restores closed tab on mod+shift+t', () => {
    const a = actions();
    handleBrowserShortcut(key('t', { meta: true, shift: true }), a);
    expect(a.restoreClosedTab).toHaveBeenCalled();
    expect(a.newTab).not.toHaveBeenCalled();
  });

  it('toggles private mode on mod+shift+n when handler provided', () => {
    const togglePrivateMode = vi.fn();
    const a = actions({ togglePrivateMode });
    handleBrowserShortcut(key('n', { meta: true, shift: true }), a);
    expect(togglePrivateMode).toHaveBeenCalled();
    expect(a.newTab).not.toHaveBeenCalled();
  });

  it('switches to tab by mod+1', () => {
    const a = actions();
    handleBrowserShortcut(key('1', { meta: true }), a);
    expect(a.switchToTabIndex).toHaveBeenCalledWith(0);
  });

  it('ignores mod+9 when tab missing', () => {
    const a = actions({ tabIdsInOrder: () => ['only'] });
    const e = key('9', { meta: true });
    expect(handleBrowserShortcut(e, a)).toBe(false);
    expect(a.switchToTabIndex).not.toHaveBeenCalled();
  });

  it('calls onEscape without modifier', () => {
    const a = actions();
    handleBrowserShortcut(key('Escape'), a);
    expect(a.onEscape).toHaveBeenCalled();
    expect(a.newTab).not.toHaveBeenCalled();
  });

  it('navigates back on mod+[', () => {
    const a = actions();
    handleBrowserShortcut(key('[', { meta: true }), a);
    expect(a.goBack).toHaveBeenCalled();
  });

  it('ignores unmodified letter keys', () => {
    const a = actions();
    expect(handleBrowserShortcut(key('t'), a)).toBe(false);
    expect(a.newTab).not.toHaveBeenCalled();
  });
});
