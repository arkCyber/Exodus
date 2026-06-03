/**
 * Exodus Browser — vertical tab layout settings API.
 */

import { invoke } from '@tauri-apps/api/core';

/** Vertical tab layout settings from backend. */
export type VerticalTabSettings = {
  enabled: boolean;
  position: 'Left' | 'Right' | string;
  width_mode: 'Fixed' | 'Auto' | 'Compact' | string;
  fixed_width: number;
  show_icons: boolean;
  show_titles: boolean;
  show_close_buttons: boolean;
  collapse_inactive: boolean;
  tab_spacing: number;
};

const STORAGE_KEY = 'exodus-vertical-tabs-enabled';

/** Load vertical tab settings from Rust store. */
export async function loadVerticalTabSettings(): Promise<VerticalTabSettings> {
  return invoke<VerticalTabSettings>('get_vertical_tab_settings');
}

/** Persist vertical tab settings. */
export async function saveVerticalTabSettings(settings: VerticalTabSettings): Promise<void> {
  await invoke('update_vertical_tab_settings', { settings });
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY, settings.enabled ? '1' : '0');
    } catch (error) {
      console.error('vertical tabs localStorage failed:', error);
    }
  }
}

/** Cached enable flag for instant layout before async settings load. */
export function readVerticalTabsCached(): boolean | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === '1') return true;
    if (v === '0') return false;
  } catch {
    /* ignore */
  }
  return null;
}

/** CSS width for the vertical tab strip. */
export function verticalTabStripWidth(settings: VerticalTabSettings): number {
  if (settings.width_mode === 'Compact' || settings.width_mode === 'compact') return 140;
  if (settings.width_mode === 'Fixed' || settings.width_mode === 'fixed') {
    return Math.min(400, Math.max(160, settings.fixed_width || 250));
  }
  return 220;
}

/** Whether tabs sit on the right edge. */
export function isVerticalTabsRight(settings: VerticalTabSettings): boolean {
  const p = String(settings.position).toLowerCase();
  return p === 'right';
}
