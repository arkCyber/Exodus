/**
 * Exodus Browser — sync open tabs to the Web Extension tab registry (chrome.tabs).
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import type { BrowserTab } from '$lib/browserTypes';
import { tabWebviewLabel } from '$lib/exodusBrowser';

/** Tab payload for `extension_sync_tabs`. */
export type ExtensionTabSync = {
  id: string;
  chromeTabId: number;
  index: number;
  webviewLabel: string;
  url: string;
  title: string;
  active: boolean;
};

/**
 * Push current UI tabs to the Rust tab registry for extension APIs.
 */
export async function syncExtensionTabs(
  tabs: BrowserTab[],
  activeTabId: string,
): Promise<void> {
  if (!isTauri()) return;
  const payload: ExtensionTabSync[] = tabs.map((t, index) => ({
    id: t.id,
    chromeTabId: index + 1,
    index,
    webviewLabel: tabWebviewLabel(t.id),
    url: t.url,
    title: t.title,
    active: t.id === activeTabId,
  }));
  try {
    await invoke('extension_sync_tabs', { tabs: payload });
  } catch (error) {
    console.error('extension_sync_tabs failed:', error);
  }
}
