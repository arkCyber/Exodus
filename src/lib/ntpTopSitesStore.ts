/**
 * Exodus Browser — persisted new-tab top-site grid (user-driven add / remove).
 * First launch shows bundled defaults; after any edit, only user choices remain (no auto-refill).
 */

import type { QuickLink } from '@/lib/browserTypes';
import { DEFAULT_NTP_TOP_SITES } from '@/lib/presetBookmarks';
import { isValidNtpSiteUrl, ntpHostLabel } from '@/lib/newTabPage';
import {
  isNtpLayoutCustomized,
  markNtpLayoutCustomized,
  NTP_PINNED_SITES_STORAGE_KEY,
  NTP_REMOVED_SITES_STORAGE_KEY,
  NTP_TOP_SITES_STORAGE_KEY,
} from '@/lib/ntpLayoutStore';

export {
  NTP_PINNED_SITES_STORAGE_KEY,
  NTP_REMOVED_SITES_STORAGE_KEY,
  NTP_TOP_SITES_STORAGE_KEY,
} from '@/lib/ntpLayoutStore';

/** Maximum tiles shown on the 2×4 grid. */
export const NTP_TOP_SITES_MAX = 8;

/** Stored site row for the NTP grid. */
export type NtpStoredSite = {
  url: string;
  title: string;
  favicon?: string;
};

/**
 * Normalize a site URL for dedupe and persistence (origin + path without hash).
 */
export function normalizeNtpSiteUrl(url: string): string {
  try {
    const href = url.trim().startsWith('http') ? url.trim() : `https://${url.trim()}`;
    const parsed = new URL(href);
    parsed.hash = '';
    return parsed.href;
  } catch {
    return url.trim();
  }
}

/** Read JSON array from localStorage with validation and error handling. */
function readSiteList(key: string): NtpStoredSite[] {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((row) => row && typeof row === 'object' && typeof row.url === 'string')
      .map((row) => ({
        url: normalizeNtpSiteUrl(row.url),
        title: typeof row.title === 'string' ? row.title.trim() : '',
        favicon: typeof row.favicon === 'string' ? row.favicon : undefined,
      }))
      .filter((row) => isValidNtpSiteUrl(row.url));
  } catch (error) {
    console.error(`[ntpTopSitesStore] read ${key} failed:`, error);
    return [];
  }
}

/** Persist a site list to localStorage. */
function writeSiteList(key: string, sites: NtpStoredSite[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(sites));
  } catch (error) {
    console.error(`[ntpTopSitesStore] write ${key} failed:`, error);
  }
}

/** Read removed bundled URLs from localStorage. */
function readRemovedUrls(): Set<string> {
  try {
    const raw = localStorage.getItem(NTP_REMOVED_SITES_STORAGE_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return new Set();
    return new Set(
      parsed
        .filter((url): url is string => typeof url === 'string')
        .map((url) => normalizeNtpSiteUrl(url)),
    );
  } catch (error) {
    console.error('[ntpTopSitesStore] read removed sites failed:', error);
    return new Set();
  }
}

/** Persist removed URLs. */
function writeRemovedUrls(urls: Set<string>): void {
  try {
    localStorage.setItem(NTP_REMOVED_SITES_STORAGE_KEY, JSON.stringify([...urls]));
  } catch (error) {
    console.error('[ntpTopSitesStore] write removed sites failed:', error);
  }
}

/** Convert a quick link into a stored site row. */
export function toStoredSite(site: QuickLink): NtpStoredSite {
  const url = normalizeNtpSiteUrl(site.url);
  const title = site.title?.trim() || ntpHostLabel({ title: '', url });
  return { url, title };
}

/** Load user custom top sites. */
export function loadCustomTopSites(): NtpStoredSite[] {
  return readSiteList(NTP_TOP_SITES_STORAGE_KEY);
}

/** Load pinned top sites. */
export function loadPinnedTopSites(): NtpStoredSite[] {
  return readSiteList(NTP_PINNED_SITES_STORAGE_KEY);
}

/** Whether a URL is currently pinned on the NTP grid. */
export function isNtpTopSitePinned(url: string): boolean {
  const normalized = normalizeNtpSiteUrl(url);
  return loadPinnedTopSites().some((site) => site.url === normalized);
}

/** URLs currently pinned (for UI state). */
export function listPinnedNtpSiteUrls(): string[] {
  return loadPinnedTopSites().map((site) => site.url);
}

/**
 * Build the visible top-site grid.
 * First run: eight bundled defaults. After customization: user list only (no fallback refill).
 */
export function buildNtpTopSitesGrid(): QuickLink[] {
  const customized = isNtpLayoutCustomized();
  const pinned = loadPinnedTopSites();
  const custom = loadCustomTopSites();
  const removed = readRemovedUrls();
  const seen = new Set<string>();
  const grid: QuickLink[] = [];

  const pushSite = (site: NtpStoredSite | QuickLink): void => {
    if (grid.length >= NTP_TOP_SITES_MAX) return;
    const stored = toStoredSite(site);
    if (removed.has(stored.url) || seen.has(stored.url)) return;
    seen.add(stored.url);
    grid.push({ title: stored.title, url: stored.url });
  };

  if (!customized) {
    for (const site of DEFAULT_NTP_TOP_SITES) {
      pushSite(site);
    }
    return grid;
  }

  for (const site of pinned) pushSite(site);
  for (const site of custom) pushSite(site);
  for (const site of DEFAULT_NTP_TOP_SITES) pushSite(site);

  return grid;
}

/** Whether the visible top-site grid has reached its slot limit. */
export function isNtpTopSitesGridFull(): boolean {
  return buildNtpTopSitesGrid().length >= NTP_TOP_SITES_MAX;
}

/** Whether a URL can be added to the visible top-site grid. */
export function canAddNtpTopSite(url: string): boolean {
  if (!isValidNtpSiteUrl(url)) return false;
  const normalized = normalizeNtpSiteUrl(url);
  const grid = buildNtpTopSitesGrid();
  if (grid.length < NTP_TOP_SITES_MAX) return true;
  return grid.some((site) => normalizeNtpSiteUrl(site.url) === normalized);
}

/** Add a site to the custom top-site list (also clears removed flag). */
export function addNtpTopSite(site: QuickLink): boolean {
  if (!canAddNtpTopSite(site.url)) return false;
  markNtpLayoutCustomized();
  const stored = toStoredSite(site);
  const removed = readRemovedUrls();
  removed.delete(stored.url);
  writeRemovedUrls(removed);

  const custom = loadCustomTopSites().filter((row) => row.url !== stored.url);
  custom.unshift(stored);
  writeSiteList(NTP_TOP_SITES_STORAGE_KEY, custom.slice(0, NTP_TOP_SITES_MAX));
  return true;
}

/** Pin a site (add + move to pinned list). */
export function pinNtpTopSite(site: QuickLink): boolean {
  if (!addNtpTopSite(site)) return false;
  const stored = toStoredSite(site);
  const pinned = loadPinnedTopSites().filter((row) => row.url !== stored.url);
  pinned.unshift(stored);
  writeSiteList(NTP_PINNED_SITES_STORAGE_KEY, pinned.slice(0, NTP_TOP_SITES_MAX));
  return true;
}

/** Unpin a site (keeps it on the grid if still custom/default). */
export function unpinNtpTopSite(site: QuickLink): boolean {
  markNtpLayoutCustomized();
  const normalized = normalizeNtpSiteUrl(site.url);
  const pinned = loadPinnedTopSites().filter((row) => row.url !== normalized);
  writeSiteList(NTP_PINNED_SITES_STORAGE_KEY, pinned);
  return true;
}

/** Remove a site from the grid (does not auto-refill). */
export function removeNtpTopSite(site: QuickLink): boolean {
  markNtpLayoutCustomized();
  const normalized = normalizeNtpSiteUrl(site.url);
  const removed = readRemovedUrls();
  removed.add(normalized);
  writeRemovedUrls(removed);

  writeSiteList(
    NTP_TOP_SITES_STORAGE_KEY,
    loadCustomTopSites().filter((row) => row.url !== normalized),
  );
  writeSiteList(
    NTP_PINNED_SITES_STORAGE_KEY,
    loadPinnedTopSites().filter((row) => row.url !== normalized),
  );
  return true;
}

/** Reset top sites to bundled factory defaults. */
export function resetNtpTopSites(): void {
  try {
    localStorage.removeItem(NTP_TOP_SITES_STORAGE_KEY);
    localStorage.removeItem(NTP_PINNED_SITES_STORAGE_KEY);
    localStorage.removeItem(NTP_REMOVED_SITES_STORAGE_KEY);
  } catch (error) {
    console.error('[ntpTopSitesStore] reset failed:', error);
  }
}
