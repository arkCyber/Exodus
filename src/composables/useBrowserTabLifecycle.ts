/**
 * Exodus Browser — register tabs with Rust tab freezer / sleep managers.
 */
import type { Ref } from 'vue';
import {
  markTabActiveLifecycle,
  registerTabLifecycle,
  syncAllTabsLifecycle,
  unregisterTabLifecycle,
  type TabLifecycleRow,
} from '@/lib/tabLifecycle';

export type UseBrowserTabLifecycleOptions = {
  getTabs: () => TabLifecycleRow[];
  getActiveTabId: () => string | null;
  useNativeWebview: Ref<boolean>;
};

/**
 * Wire tab create/close/activate to lifecycle backends (native webview only).
 */
export function useBrowserTabLifecycle(options: UseBrowserTabLifecycleOptions) {
  function shouldTrack(): boolean {
    return options.useNativeWebview.value;
  }

  async function registerTab(tab: TabLifecycleRow): Promise<void> {
    if (!shouldTrack()) return;
    await registerTabLifecycle(tab);
  }

  async function unregisterTab(tabId: string): Promise<void> {
    if (!shouldTrack()) return;
    await unregisterTabLifecycle(tabId);
  }

  async function markTabActive(tabId: string): Promise<void> {
    if (!shouldTrack()) return;
    await markTabActiveLifecycle(tabId);
  }

  /** After session restore or startup, sync all open tabs. */
  async function syncAllTabs(): Promise<void> {
    if (!shouldTrack()) return;
    const activeId = options.getActiveTabId();
    if (!activeId) return;
    await syncAllTabsLifecycle(options.getTabs(), activeId);
  }

  return {
    registerTab,
    unregisterTab,
    markTabActive,
    syncAllTabs,
  };
}
