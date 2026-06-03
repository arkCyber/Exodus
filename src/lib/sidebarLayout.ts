/**
 * Exodus Browser — Firefox-style sidebar layout (48px rail + resizable content).
 */

import { CHROME_LAYOUT } from '$lib/chromeLayout';

const STORAGE_WIDTH = 'exodus.sidebar.contentWidth';
const STORAGE_COLLAPSED = 'exodus.sidebar.contentCollapsed';

/** Default content panel width (Firefox ~320px). */
export const SIDEBAR_CONTENT_DEFAULT_PX = 320;

/** Minimum / maximum content width when dragging the resize handle. */
export const SIDEBAR_CONTENT_MIN_PX = 260;
/** Max content width (~75% viewport, Firefox sidebar limit). */
export function sidebarContentMaxPx(): number {
  if (typeof window !== 'undefined' && window.innerWidth > 0) {
    return Math.round(window.innerWidth * 0.75);
  }
  return 900;
}

/** Total sidebar width = icon rail + content (when expanded). */
export function sidebarTotalWidthPx(contentWidthPx: number, contentCollapsed: boolean): number {
  if (contentCollapsed) return CHROME_LAYOUT.sidebarIconRail;
  return CHROME_LAYOUT.sidebarIconRail + contentWidthPx;
}

/** Read persisted content width from localStorage. */
export function loadSidebarContentWidth(): number {
  try {
    const raw = localStorage.getItem(STORAGE_WIDTH);
    if (!raw) return SIDEBAR_CONTENT_DEFAULT_PX;
    const n = Number.parseInt(raw, 10);
    if (!Number.isFinite(n)) return SIDEBAR_CONTENT_DEFAULT_PX;
    return Math.min(sidebarContentMaxPx(), Math.max(SIDEBAR_CONTENT_MIN_PX, n));
  } catch {
    return SIDEBAR_CONTENT_DEFAULT_PX;
  }
}

/** Persist content width. */
export function saveSidebarContentWidth(px: number): void {
  try {
    localStorage.setItem(STORAGE_WIDTH, String(px));
  } catch {
    /* ignore quota */
  }
}

/** Read whether the content panel is collapsed (icon rail only). */
export function loadSidebarContentCollapsed(): boolean {
  try {
    return localStorage.getItem(STORAGE_COLLAPSED) === '1';
  } catch {
    return false;
  }
}

/** Persist collapsed state. */
export function saveSidebarContentCollapsed(collapsed: boolean): void {
  try {
    localStorage.setItem(STORAGE_COLLAPSED, collapsed ? '1' : '0');
  } catch {
    /* ignore */
  }
}
