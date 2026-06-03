/**
 * Exodus Browser — global keyboard shortcuts (Chrome-style).
 */

/** Handlers invoked when a shortcut matches. */
export type BrowserShortcutActions = {
  focusOmnibox: () => void;
  reload: () => void;
  newTab: () => void;
  restoreClosedTab: () => void;
  closeActiveTab: () => void;
  zoomIn: () => void;
  zoomOut: () => void;
  resetZoom: () => void;
  toggleBookmark: () => void;
  toggleBookmarkBar: () => void;
  openHistory: () => void;
  toggleSidebar: () => void;
  toggleFindBar: () => void;
  print: () => void;
  goBack: () => void;
  goForward: () => void;
  switchToTabIndex: (index: number) => void;
  /** Close modals, menus, find bar, etc. */
  onEscape: () => void;
  /** Ordered tab ids for ⌘1–⌘9 (pinned first). */
  tabIdsInOrder: () => string[];
  /** Toggle DevTools on the active tab (optional). */
  toggleDevTools?: () => void;
  /** Toggle private / incognito mode (optional). */
  togglePrivateMode?: () => void;
};

const OMNIBOX_INPUT_ID = 'exodus-omnibox-input';

/** Focus the address bar omnibox input. */
export function focusOmniboxInput(): void {
  const el = document.getElementById(OMNIBOX_INPUT_ID);
  if (el instanceof HTMLInputElement) {
    el.focus();
    el.select();
  }
}

/** Omnibox input element id for use in AddressBar. */
export { OMNIBOX_INPUT_ID };

/**
 * Handle a keydown event; calls matching actions and prevents default when consumed.
 * @returns true if the event was handled
 */
export function handleBrowserShortcut(e: KeyboardEvent, actions: BrowserShortcutActions): boolean {
  if (e.key === 'Escape') {
    actions.onEscape();
    return true;
  }

  if (e.key === 'F12' && actions.toggleDevTools) {
    e.preventDefault();
    actions.toggleDevTools();
    return true;
  }

  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return false;

  const key = e.key;

  if (key === 'l') {
    e.preventDefault();
    actions.focusOmnibox();
    return true;
  }
  if (key === 'r') {
    e.preventDefault();
    actions.reload();
    return true;
  }
  if (key === '[') {
    e.preventDefault();
    actions.goBack();
    return true;
  }
  if (key === ']') {
    e.preventDefault();
    actions.goForward();
    return true;
  }
  if (key === 't' && e.shiftKey) {
    e.preventDefault();
    actions.restoreClosedTab();
    return true;
  }
  if (key === 'n' && e.shiftKey && actions.togglePrivateMode) {
    e.preventDefault();
    actions.togglePrivateMode();
    return true;
  }
  if (key === 't') {
    e.preventDefault();
    actions.newTab();
    return true;
  }
  if (key === 'w') {
    e.preventDefault();
    actions.closeActiveTab();
    return true;
  }
  if (key === '=' || key === '+') {
    e.preventDefault();
    actions.zoomIn();
    return true;
  }
  if (key === '-') {
    e.preventDefault();
    actions.zoomOut();
    return true;
  }
  if (key === '0') {
    e.preventDefault();
    actions.resetZoom();
    return true;
  }
  if (key === 'd') {
    e.preventDefault();
    actions.toggleBookmark();
    return true;
  }
  if (key === 'b' && !e.shiftKey) {
    e.preventDefault();
    actions.toggleBookmarkBar();
    return true;
  }
  if (key === 'h') {
    e.preventDefault();
    actions.openHistory();
    return true;
  }
  if (key === 'B' && e.shiftKey) {
    e.preventDefault();
    actions.toggleSidebar();
    return true;
  }
  if (key === 'f' && !e.shiftKey) {
    e.preventDefault();
    actions.toggleFindBar();
    return true;
  }
  if (key === 'p' && !e.shiftKey) {
    e.preventDefault();
    actions.print();
    return true;
  }
  if (key >= '1' && key <= '9') {
    const idx = Number(key) - 1;
    const ordered = actions.tabIdsInOrder();
    if (ordered[idx]) {
      e.preventDefault();
      actions.switchToTabIndex(idx);
      return true;
    }
  }

  return false;
}

/**
 * Register window keydown listener for browser shortcuts.
 * @returns cleanup function
 */
export function mountBrowserShortcuts(actions: BrowserShortcutActions): () => void {
  const handler = (e: KeyboardEvent) => {
    handleBrowserShortcut(e, actions);
  };
  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler);
}
