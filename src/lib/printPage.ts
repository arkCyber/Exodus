/**
 * Exodus Browser — print active page (native WebView + iframe fallback).
 */

import { canInvokeTauri } from '@/lib/tauri';
import { evalInTab, tabWebviewLabel } from '@/lib/exodusBrowser';
import { isChromeInternalUrl } from '@/lib/chromeInternal';
import { isNewTabUrl } from '@/lib/newTabPage';

export type PrintActivePageOptions = {
  useNativeWebview: boolean;
  activeTabId: string | null;
  tabUrl: string;
  iframe?: HTMLIFrameElement | null;
};

/**
 * Open the system print dialog for the active tab (Chrome ⌘P behavior).
 */
export async function printActivePage(options: PrintActivePageOptions): Promise<boolean> {
  const { useNativeWebview, activeTabId, tabUrl, iframe } = options;
  if (!activeTabId) return false;
  if (isNewTabUrl(tabUrl) || isChromeInternalUrl(tabUrl)) {
    return false;
  }

  try {
    if (useNativeWebview && canInvokeTauri()) {
      await evalInTab(tabWebviewLabel(activeTabId), 'window.print()');
      return true;
    }
    iframe?.contentWindow?.print();
    return true;
  } catch (error) {
    console.error('printActivePage failed:', error);
    return false;
  }
}
