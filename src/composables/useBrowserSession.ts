/**
 * Exodus Browser — save/restore tab session via Tauri `save_session` / `load_session`.
 */
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Ref } from 'vue';
import { shouldPersistSession } from '@/lib/privacySettings';
import { isNewTabUrl } from '@/lib/newTabPage';

export type SessionTabLike = {
  id: string;
  url: string;
  title: string;
  pinned?: boolean;
};

export type SessionRestorePayload = {
  restored: SessionTabLike[];
  targetId: string;
};

export type UseBrowserSessionOptions = {
  getTabs: () => SessionTabLike[];
  getActiveTabId: () => string | null;
  getSortedTabs: () => SessionTabLike[];
  sessionRestore: Ref<boolean>;
  privateMode: Ref<boolean>;
  createTabId: () => string;
  activateTab: (id: string) => Promise<void>;
  onStatus: (message: string) => void;
  newTabPageUrl: string;
};

/**
 * Persist and restore browser tabs when session restore is enabled.
 */
export function useBrowserSession(options: UseBrowserSessionOptions) {
  async function saveSession(): Promise<void> {
    if (!isTauri()) return;
    if (!shouldPersistSession(options.sessionRestore.value, options.privateMode.value)) {
      return;
    }
    try {
      const currentTabs = options.getSortedTabs().map((t) => ({
        id: t.id,
        url: t.url,
        title: t.title,
        active: t.id === options.getActiveTabId(),
      }));
      await invoke('save_session', {
        tabs: currentTabs,
        activeTabId: options.getActiveTabId(),
      });
    } catch (error) {
      console.error('Failed to save session:', error);
    }
  }

  /**
   * Restore last session when only the default new-tab is open.
   */
  async function loadSession(): Promise<SessionRestorePayload | undefined> {
    if (!isTauri()) return undefined;
    if (!shouldPersistSession(options.sessionRestore.value, options.privateMode.value)) {
      return undefined;
    }
    try {
      const session = await invoke<{
        tabs: Array<{ id: string; url: string; title: string; active?: boolean }>;
        activeTabId?: string;
      } | null>('load_session');
      if (!session?.tabs?.length) return undefined;

      const tabs = options.getTabs();
      if (tabs.length !== 1 || !isNewTabUrl(tabs[0].url)) return undefined;

      const restored = session.tabs.map((tab) => ({
        id: tab.id || options.createTabId(),
        title: tab.title || tab.url || 'New Tab',
        url: tab.url || options.newTabPageUrl,
        pinned: false,
      }));

      const targetId =
        session.activeTabId && restored.some((t) => t.id === session.activeTabId)
          ? session.activeTabId
          : restored[0].id;

      return { restored, targetId };
    } catch (error) {
      console.error('Failed to load session:', error);
      return undefined;
    }
  }

  return {
    saveSession,
    loadSession,
  };
}
