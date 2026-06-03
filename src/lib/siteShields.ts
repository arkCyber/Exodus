/**
 * Exodus Browser — per-site tracker shield overrides (Brave-style).
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Read whether trackers are allowed on `host` (normalized server-side).
 */
export async function getSiteShieldAllowTrackers(host: string): Promise<boolean> {
  try {
    return await invoke<boolean>('get_site_shield_override', { host });
  } catch (error) {
    console.error('get_site_shield_override failed:', error);
    return false;
  }
}

/**
 * Allow or block trackers for a site hostname.
 */
export async function setSiteShieldAllowTrackers(
  host: string,
  allowTrackers: boolean,
): Promise<void> {
  try {
    await invoke('set_site_shield_override', { host, allowTrackers });
  } catch (error) {
    console.error('set_site_shield_override failed:', error);
    throw error;
  }
}

/**
 * Refresh tracker blocklist subscription (embedded fallback when URL unavailable).
 */
export async function refreshTrackerBlocklist(url?: string): Promise<number> {
  try {
    return await invoke<number>('refresh_tracker_blocklist', { url: url ?? null });
  } catch (error) {
    console.error('refresh_tracker_blocklist failed:', error);
    throw error;
  }
}

/** Extract hostname from a page URL for shield APIs. */
export function hostFromPageUrl(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return '';
  }
}
