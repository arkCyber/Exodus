/**
 * Exodus Browser — Firefox-style sidebar preferences (tools, position, vertical tabs).
 */

import type { SidebarPanel } from '$lib/browserTypes';

/** Panels that can be toggled in Customize sidebar (gear). */
export type SidebarToolId = Exclude<SidebarPanel, 'customize'>;

export type SidebarPosition = 'left' | 'right';

export interface SidebarPreferences {
  position: SidebarPosition;
  /** Show open tabs in sidebar instead of top tab strip (Firefox 136). */
  verticalTabsInSidebar: boolean;
  enabledTools: SidebarToolId[];
}

const STORAGE_KEY = 'exodus.sidebar.preferences';

/** Tool metadata for icon rail and customize checkboxes. */
export const SIDEBAR_TOOL_CATALOG: {
  id: SidebarToolId;
  title: string;
  icon: string;
  firefoxLabel: string;
}[] = [
  { id: 'tabs', title: 'Tabs', icon: 'tabs', firefoxLabel: 'Vertical tabs' },
  { id: 'ai', title: 'AI Chat', icon: 'ai', firefoxLabel: 'AI assistant' },
  { id: 'memory', title: 'History', icon: 'history', firefoxLabel: 'History' },
  { id: 'bookmarks', title: 'Bookmarks', icon: 'bookmarks', firefoxLabel: 'Bookmarks' },
  { id: 'synced', title: 'Synced tabs', icon: 'synced', firefoxLabel: 'Tabs from other devices' },
  { id: 'reading', title: 'Reading list', icon: 'reading', firefoxLabel: 'Reading list' },
  { id: 'pocket', title: 'Pocket', icon: 'pocket', firefoxLabel: 'Pocket saves' },
  { id: 'p2p', title: 'P2P', icon: 'p2p', firefoxLabel: 'P2P & chat' },
];

const DEFAULT_ENABLED: SidebarToolId[] = SIDEBAR_TOOL_CATALOG.map((t) => t.id);

const DEFAULT_PREFS: SidebarPreferences = {
  position: 'right',
  verticalTabsInSidebar: false,
  enabledTools: [...DEFAULT_ENABLED],
};

/**
 * Load sidebar preferences from localStorage (Firefox Customize sidebar persistence).
 */
export function loadSidebarPreferences(): SidebarPreferences {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_PREFS, enabledTools: [...DEFAULT_ENABLED] };
    const parsed = JSON.parse(raw) as Partial<SidebarPreferences>;
    const enabled = Array.isArray(parsed.enabledTools)
      ? parsed.enabledTools.filter((id): id is SidebarToolId =>
          SIDEBAR_TOOL_CATALOG.some((t) => t.id === id),
        )
      : [...DEFAULT_ENABLED];
    return {
      position: parsed.position === 'left' ? 'left' : 'right',
      verticalTabsInSidebar: Boolean(parsed.verticalTabsInSidebar),
      enabledTools: enabled.length > 0 ? enabled : [...DEFAULT_ENABLED],
    };
  } catch (error) {
    console.error('loadSidebarPreferences failed:', error);
    return { ...DEFAULT_PREFS, enabledTools: [...DEFAULT_ENABLED] };
  }
}

/** Persist preferences. */
export function saveSidebarPreferences(prefs: SidebarPreferences): void {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        ...prefs,
        enabledTools: [...prefs.enabledTools],
      }),
    );
  } catch (error) {
    console.error('saveSidebarPreferences failed:', error);
  }
}

/** Whether a tool appears in the icon rail. */
export function isToolEnabled(prefs: SidebarPreferences, tool: SidebarToolId): boolean {
  return prefs.enabledTools.includes(tool);
}

/** First enabled panel for default selection. */
export function defaultSidebarPanel(prefs: SidebarPreferences): SidebarPanel {
  if (prefs.verticalTabsInSidebar && isToolEnabled(prefs, 'tabs')) return 'tabs';
  const first = prefs.enabledTools[0];
  return first ?? 'ai';
}

/** Merge partial updates into prefs and save. */
export function applySidebarPreferencesPatch(
  current: SidebarPreferences,
  patch: Partial<SidebarPreferences>,
): SidebarPreferences {
  const next: SidebarPreferences = {
    position: patch.position ?? current.position,
    verticalTabsInSidebar: patch.verticalTabsInSidebar ?? current.verticalTabsInSidebar,
    enabledTools: patch.enabledTools
      ? patch.enabledTools.filter((id) => SIDEBAR_TOOL_CATALOG.some((t) => t.id === id))
      : [...current.enabledTools],
  };
  if (next.enabledTools.length === 0) {
    next.enabledTools = ['ai'];
  }
  saveSidebarPreferences(next);
  return next;
}
