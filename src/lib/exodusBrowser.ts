/**
 * Exodus Browser — native Tauri content WebView helpers.
 * Supports per-tab labels; falls back when not running inside Tauri.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import { Webview } from '@tauri-apps/api/webview';

export type PageCapture = {
  url: string;
  title: string;
  content: string;
};

export type SelectionCapture = {
  text: string;
};

export type NavState = {
  url: string;
  can_go_back: boolean;
  can_go_forward: boolean;
};

/** Build a stable webview label for a tab id. */
export function tabWebviewLabel(tabId: string): string {
  const safe = tabId.replace(/[^a-zA-Z0-9_-]/g, '');
  return `exodus-tab-${safe}`;
}

/** Whether the app can use native child WebViews (Tauri desktop only). */
export function canUseNativeWebview(): boolean {
  return isTauri();
}

/** Layout a webview over a DOM container. */
export async function layoutWebview(
  webview: Webview,
  container: HTMLElement,
): Promise<void> {
  const rect = container.getBoundingClientRect();
  await webview.setPosition(new LogicalPosition(rect.left, rect.top));
  await webview.setSize(
    new LogicalSize(Math.max(rect.width, 100), Math.max(rect.height, 100)),
  );
}

/** Create a child WebView for a tab. */
export async function createTabWebview(
  container: HTMLElement,
  label: string,
  url: string,
): Promise<Webview> {
  const rect = container.getBoundingClientRect();

  await invoke('browser_create_tab', {
    label,
    url,
    x: rect.left,
    y: rect.top,
    width: Math.max(rect.width, 100),
    height: Math.max(rect.height, 100),
  });

  const webview = await Webview.getByLabel(label);
  if (!webview) {
    throw new Error(`Tab webview not found after create: ${label}`);
  }

  await layoutWebview(webview, container);
  return webview;
}

/** Get an existing tab webview or null. */
export async function getTabWebview(label: string): Promise<Webview | null> {
  return Webview.getByLabel(label);
}

export async function showTabWebview(webview: Webview, container: HTMLElement): Promise<void> {
  await layoutWebview(webview, container);
  await webview.show();
  await webview.setFocus();
}

export async function hideTabWebview(webview: Webview): Promise<void> {
  await webview.hide();
}

/** Toggle DevTools for a tab webview (debug builds). */
export async function toggleTabDevTools(label: string): Promise<boolean> {
  return invoke<boolean>('browser_toggle_devtools', { label });
}

/** Whether tab webview was discarded (destroyed, snapshot kept). */
export async function isTabDiscarded(label: string): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    return await invoke<boolean>('browser_is_tab_discarded', { label });
  } catch (error) {
    console.error('browser_is_tab_discarded failed:', error);
    return false;
  }
}

/**
 * Discard tab: destroy WebView and keep URL/layout for restore (true tab discard).
 */
export async function discardTabWebview(
  label: string,
  url: string,
  container: HTMLElement,
): Promise<void> {
  const rect = container.getBoundingClientRect();
  const wv = await Webview.getByLabel(label);
  if (wv) {
    await wv.close();
  }
  await invoke('browser_discard_tab', {
    label,
    url,
    x: rect.left,
    y: rect.top,
    width: Math.max(rect.width, 100),
    height: Math.max(rect.height, 100),
  });
  try {
    await invoke('browser_clear_tab_nav', { label });
  } catch (error) {
    console.error('browser_clear_tab_nav after discard failed:', error);
  }
}

/** Restore a discarded tab webview from snapshot. */
export async function restoreDiscardedTab(
  container: HTMLElement,
  label: string,
): Promise<Webview | null> {
  const restored = await invoke<boolean>('browser_restore_discarded_tab', { label });
  if (!restored) return null;
  const wv = await Webview.getByLabel(label);
  if (wv) {
    await layoutWebview(wv, container);
    await wv.show();
  }
  return wv;
}

export async function closeTabWebview(label: string): Promise<void> {
  const wv = await Webview.getByLabel(label);
  if (wv) {
    await wv.close();
  }
  try {
    await invoke('browser_clear_tab_nav', { label });
  } catch (error) {
    console.error('browser_clear_tab_nav failed:', error);
  }
}

export async function navigateTab(label: string, url: string): Promise<void> {
  await invoke('browser_navigate', { label, url });
}

/** Sync popup-block flag with privacy settings on an existing tab webview. */
export async function setTabPopupBlocking(label: string, block: boolean): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke('browser_set_popup_blocking', { label, block });
  } catch (error) {
    console.error('browser_set_popup_blocking failed:', error);
  }
}

export async function goBackTab(label: string): Promise<void> {
  await invoke('browser_go_back', { label });
}

export async function goForwardTab(label: string): Promise<void> {
  await invoke('browser_go_forward', { label });
}

export async function reloadTab(label: string): Promise<void> {
  await invoke('browser_reload', { label });
}

export async function captureTabPage(label: string): Promise<PageCapture> {
  return invoke<PageCapture>('browser_capture_content', { label });
}

export async function getTabSelection(label: string): Promise<SelectionCapture> {
  return invoke<SelectionCapture>('browser_get_selection', { label });
}

export async function getTabHtml(label: string): Promise<string> {
  return invoke<string>('browser_get_html', { label });
}

export async function evalInTab(label: string, script: string): Promise<void> {
  await invoke('browser_eval', { label, script });
}

/** Evaluate JS in a tab and return the JSON-serialized result. */
export async function evalTabReturning(label: string, script: string): Promise<string> {
  return invoke<string>('browser_eval_return', { label, script });
}

/** Set zoom on a tab webview (scale 0.5–3.0, 1.0 = 100%). */
export async function setTabZoom(label: string, scale: number): Promise<void> {
  await invoke('browser_set_zoom', { label, scale });
}

/** Read document.title from a tab webview. */
export async function getTabTitle(label: string): Promise<string> {
  return invoke<string>('browser_get_title', { label });
}

/** Read back/forward availability and current URL from a tab webview. */
export async function getTabNavState(label: string): Promise<NavState> {
  return invoke<NavState>('browser_get_nav_state', { label });
}

/** Find text in page; returns whether a match was found. */
export async function findInTab(
  label: string,
  query: string,
  forward = true,
): Promise<boolean> {
  return invoke<boolean>('browser_find_in_page', { label, query, forward });
}

/** Keep webview aligned when the container or window resizes. */
export function watchWebviewLayout(
  container: HTMLElement,
  getWebview: () => Webview | null | undefined,
): () => void {
  const relayout = () => {
    const wv = getWebview();
    if (wv) {
      layoutWebview(wv, container).catch(console.error);
    }
  };

  const observer = new ResizeObserver(relayout);
  observer.observe(container);
  window.addEventListener('resize', relayout);

  return () => {
    observer.disconnect();
    window.removeEventListener('resize', relayout);
  };
}
