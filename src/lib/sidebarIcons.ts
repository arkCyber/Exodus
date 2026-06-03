/**
 * Exodus Browser — Firefox-style sidebar icon SVG paths (24×24 viewBox).
 */

import type { SidebarPanel } from '$lib/browserTypes';
import {
  SIDEBAR_TOOL_CATALOG,
  type SidebarPreferences,
  type SidebarToolId,
  isToolEnabled,
} from '$lib/sidebarPreferences';

export type SidebarIconId =
  | 'tabs'
  | 'ai'
  | 'history'
  | 'bookmarks'
  | 'synced'
  | 'reading'
  | 'p2p'
  | 'pocket'
  | 'customize'
  | 'collapse'
  | 'expand'
  | 'close';

/** Icon rail entry derived from preferences. */
export type SidebarIconItem = {
  panel: SidebarPanel;
  icon: SidebarIconId;
  title: string;
};

/**
 * Build visible icon rail items from Firefox-style preferences.
 */
export function sidebarIconItemsFromPrefs(prefs: SidebarPreferences): SidebarIconItem[] {
  return SIDEBAR_TOOL_CATALOG.filter((t) => isToolEnabled(prefs, t.id)).map((t) => ({
    panel: t.id,
    icon: t.icon as SidebarIconId,
    title: t.title,
  }));
}

/**
 * Returns SVG path `d` for a sidebar icon (stroke icons, no fill).
 */
export function sidebarIconPath(icon: SidebarIconId): string {
  switch (icon) {
    case 'tabs':
      return 'M4 6h16M4 12h16M4 18h16';
    case 'ai':
      return 'M12 3a7 7 0 0 0-4 12.7V21l4-2 4 2v-5.3A7 7 0 0 0 12 3z';
    case 'history':
      return 'M12 7v5l3 2 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z';
    case 'bookmarks':
      return 'M4 19.5A2.5 2.5 0 0 1 6.5 17H20 M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15z';
    case 'synced':
      return 'M9 17H7A5 5 0 0 1 7 7h2M15 7h2a5 5 0 1 1 0 10h-2M8 12h8';
    case 'reading':
      return 'M4 19.5A2.5 2.5 0 0 1 6.5 17H20 M6 2v15.5M12 2v20M18 2v15.5';
    case 'p2p':
      return 'M6 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm12-5a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm0 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm-6 5a2 2 0 1 0 0-4 2 2 0 0 0 0 4z M8 12h5 M13 10l3-3 M13 14l3 3';
    case 'pocket':
      return 'M4 19.5A2.5 2.5 0 0 1 6.5 17H20 M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15z M12 7v10 M9 10l3-3 3 3';
    case 'customize':
      return 'M12 15.5A3.5 3.5 0 1 0 12 8.5a3.5 3.5 0 0 0 0 7z M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z';
    case 'collapse':
      return 'M11 19l-7-7 7-7M19 19l-7-7 7-7';
    case 'expand':
      return 'M13 5l7 7-7 7M5 5l7 7-7 7';
    case 'close':
      return 'M18 6L6 18M6 6l12 12';
    default:
      return '';
  }
}

/** @deprecated Use sidebarIconItemsFromPrefs */
export const SIDEBAR_ICON_PANELS: { panel: SidebarToolId; icon: SidebarIconId; title: string }[] =
  SIDEBAR_TOOL_CATALOG.map((t) => ({
    panel: t.id,
    icon: t.icon as SidebarIconId,
    title: t.title,
  }));
