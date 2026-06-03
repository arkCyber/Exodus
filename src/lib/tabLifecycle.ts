/**
 * Exodus Browser — wire tab freezer + tab sleep managers (Chrome-style background tab discipline).
 */

import { invoke } from '@tauri-apps/api/core';
import { tabWebviewLabel } from '$lib/exodusBrowser';

export type TabLifecycleRow = {
  id: string;
  url: string;
  title: string;
  pinned?: boolean;
};

/** Optional discard hook (destroy WebView for sleeping tabs). */
export type TabLifecycleDiscardHook = (
  tabId: string,
  url: string,
) => Promise<void>;

/**
 * Register a tab with memory-management backends.
 */
export async function registerTabLifecycle(tab: TabLifecycleRow): Promise<void> {
  const label = tabWebviewLabel(tab.id);
  try {
    await invoke('register_tab', { label, url: tab.url, title: tab.title });
    await invoke('tab_sleep_register', {
      tabId: tab.id,
      label,
      url: tab.url,
      title: tab.title,
      isPinned: tab.pinned ?? false,
    });
  } catch (error) {
    console.error('registerTabLifecycle failed:', error);
  }
}

/**
 * Mark tab active (unfreeze + update last-active timestamps).
 */
export async function markTabActiveLifecycle(tabId: string): Promise<void> {
  const label = tabWebviewLabel(tabId);
  try {
    await invoke('update_tab_activity', { label });
    await invoke('tab_sleep_mark_active', { tabId });
    await invoke('tab_sleep_wake', { tabId });
  } catch (error) {
    console.error('markTabActiveLifecycle failed:', error);
  }
}

/**
 * Remove tab from lifecycle managers.
 */
export async function unregisterTabLifecycle(tabId: string): Promise<void> {
  const label = tabWebviewLabel(tabId);
  try {
    await invoke('unregister_tab', { label });
    await invoke('tab_sleep_unregister', { tabId });
  } catch (error) {
    console.error('unregisterTabLifecycle failed:', error);
  }
}

export type TabLifecycleMaintenanceResult = {
  newlyFrozen: number;
  totalFrozen: number;
  sleepCandidates: number;
};

/**
 * Periodic maintenance: freeze inactive tabs and mark sleep candidates.
 */
export async function runTabLifecycleMaintenance(
  activeTabId: string,
  tabs: TabLifecycleRow[] = [],
  onDiscard?: TabLifecycleDiscardHook,
): Promise<TabLifecycleMaintenanceResult> {
  const empty = { newlyFrozen: 0, totalFrozen: 0, sleepCandidates: 0 };
  try {
    const frozen = await invoke<string[]>('auto_freeze_inactive_tabs');
    const candidates = await invoke<string[]>('tab_sleep_get_candidates');
    for (const tabId of candidates) {
      if (tabId === activeTabId) continue;
      await invoke('tab_sleep_mark_sleeping', { tabId });
      if (onDiscard) {
        const row = tabs.find((t) => t.id === tabId);
        if (row && !row.pinned && row.url.startsWith('http')) {
          try {
            await onDiscard(tabId, row.url);
          } catch (error) {
            console.error('tab discard failed for', tabId, error);
          }
        }
      }
    }
    let totalFrozen = 0;
    try {
      const allFrozen = await invoke<unknown[]>('get_frozen_tabs');
      totalFrozen = allFrozen.length;
    } catch (error) {
      console.error('get_frozen_tabs failed:', error);
    }
    return {
      newlyFrozen: frozen.length,
      totalFrozen,
      sleepCandidates: candidates.length,
    };
  } catch (error) {
    console.error('runTabLifecycleMaintenance failed:', error);
    return empty;
  }
}

/**
 * Sync all open tabs with backends (startup / session restore).
 */
export async function syncAllTabsLifecycle(
  tabs: TabLifecycleRow[],
  activeTabId: string,
): Promise<void> {
  for (const tab of tabs) {
    await registerTabLifecycle(tab);
  }
  if (activeTabId) {
    await markTabActiveLifecycle(activeTabId);
  }
}
