/**
 * Exodus Browser — Bookmark Shortcuts Composable
 * 
 * This composable provides keyboard shortcut functionality for bookmark operations.
 */

import { onMounted, onUnmounted } from 'vue';

export interface BookmarkShortcutConfig {
  enabled: boolean;
  shortcuts: {
    addBookmark: string;
    openBookmarkManager: string;
    toggleBookmarkBar: string;
  };
}

export function useBookmarkShortcuts() {
  const defaultConfig: BookmarkShortcutConfig = {
    enabled: true,
    shortcuts: {
      addBookmark: 'CmdOrCtrl+D',
      openBookmarkManager: 'CmdOrCtrl+Shift+O',
      toggleBookmarkBar: 'CmdOrCtrl+Shift+B',
    },
  };

  let shortcutHandlers: Array<{ key: string; handler: (e: KeyboardEvent) => void }> = [];

  function parseShortcut(shortcut: string): { key: string; modifiers: string[] } {
    const parts = shortcut.split('+');
    const key = parts.pop()?.toLowerCase() || '';
    const modifiers = parts.map(p => p.toLowerCase());
    return { key, modifiers };
  }

  function matchesShortcut(event: KeyboardEvent, shortcut: string): boolean {
    const { key, modifiers } = parseShortcut(shortcut);
    
    const eventKey = event.key.toLowerCase();
    const eventModifiers: string[] = [];
    
    if (event.metaKey) eventModifiers.push('cmdorctrl');
    if (event.ctrlKey) eventModifiers.push('cmdorctrl');
    if (event.altKey) eventModifiers.push('alt');
    if (event.shiftKey) eventModifiers.push('shift');

    // Check if all required modifiers are present
    const hasAllModifiers = modifiers.every(m => eventModifiers.includes(m));
    // Check if no extra modifiers are present
    const hasOnlyRequiredModifiers = eventModifiers.every(m => modifiers.includes(m));

    return hasAllModifiers && hasOnlyRequiredModifiers && eventKey === key;
  }

  function registerShortcut(
    shortcut: string,
    handler: (e: KeyboardEvent) => void
  ): void {
    shortcutHandlers.push({ key: shortcut, handler });
  }

  function unregisterShortcut(shortcut: string): void {
    shortcutHandlers = shortcutHandlers.filter(h => h.key !== shortcut);
  }

  function handleKeyDown(event: KeyboardEvent): void {
    // Don't trigger shortcuts when typing in input fields
    const target = event.target as HTMLElement;
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable
    ) {
      return;
    }

    for (const { key, handler } of shortcutHandlers) {
      if (matchesShortcut(event, key)) {
        event.preventDefault();
        handler(event);
        break;
      }
    }
  }

  function setupDefaultShortcuts(
    callbacks: {
      onAddBookmark?: (e: KeyboardEvent) => void;
      onOpenBookmarkManager?: (e: KeyboardEvent) => void;
      onToggleBookmarkBar?: (e: KeyboardEvent) => void;
    }
  ): void {
    if (callbacks.onAddBookmark) {
      registerShortcut(defaultConfig.shortcuts.addBookmark, callbacks.onAddBookmark);
    }
    if (callbacks.onOpenBookmarkManager) {
      registerShortcut(defaultConfig.shortcuts.openBookmarkManager, callbacks.onOpenBookmarkManager);
    }
    if (callbacks.onToggleBookmarkBar) {
      registerShortcut(defaultConfig.shortcuts.toggleBookmarkBar, callbacks.onToggleBookmarkBar);
    }
  }

  function clearShortcuts(): void {
    shortcutHandlers = [];
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
    clearShortcuts();
  });

  return {
    registerShortcut,
    unregisterShortcut,
    setupDefaultShortcuts,
    clearShortcuts,
    defaultConfig,
  };
}
