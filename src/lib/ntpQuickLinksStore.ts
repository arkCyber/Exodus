/**
 * Exodus Browser — persisted new-tab quick-link chips (user-driven add / remove).
 * First launch shows four bundled defaults; after any edit, user list only (no auto-refill).
 */

import type { QuickLink } from '@/lib/browserTypes';
import { DEFAULT_QUICK_LINKS } from '@/lib/presetBookmarks';
import { isValidNtpSiteUrl, ntpHostLabel } from '@/lib/newTabPage';
import {
  isNtpLayoutCustomized,
  markNtpLayoutCustomized,
  NTP_QUICK_LINKS_STORAGE_KEY,
  NTP_REMOVED_QUICK_LINKS_STORAGE_KEY,
} from '@/lib/ntpLayoutStore';
import { normalizeNtpSiteUrl } from '@/lib/ntpTopSitesStore';

export {
  NTP_QUICK_LINKS_STORAGE_KEY,
  NTP_REMOVED_QUICK_LINKS_STORAGE_KEY,
} from '@/lib/ntpLayoutStore';

/** Soft cap for quick-link chips (row can wrap). */
export const NTP_QUICK_LINKS_MAX = 16;

/** Stored quick-link row. */
export type NtpStoredQuickLink = {
  url: string;
  title: string;
};

/** Read JSON quick-link list from localStorage. */
function readQuickLinkList(key: string): NtpStoredQuickLink[] {
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
      }))
      .filter((row) => isValidNtpSiteUrl(row.url));
  } catch (error) {
    console.error(`[ntpQuickLinksStore] read ${key} failed:`, error);
    return [];
  }
}

/** Persist quick-link list to localStorage. */
function writeQuickLinkList(key: string, links: NtpStoredQuickLink[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(links));
  } catch (error) {
    console.error(`[ntpQuickLinksStore] write ${key} failed:`, error);
  }
}

/** Read removed default chip URLs. */
function readRemovedQuickLinkUrls(): Set<string> {
  try {
    const raw = localStorage.getItem(NTP_REMOVED_QUICK_LINKS_STORAGE_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return new Set();
    return new Set(
      parsed
        .filter((url): url is string => typeof url === 'string')
        .map((url) => normalizeNtpSiteUrl(url)),
    );
  } catch (error) {
    console.error('[ntpQuickLinksStore] read removed quick links failed:', error);
    return new Set();
  }
}

/** Persist removed chip URLs. */
function writeRemovedQuickLinkUrls(urls: Set<string>): void {
  try {
    localStorage.setItem(NTP_REMOVED_QUICK_LINKS_STORAGE_KEY, JSON.stringify([...urls]));
  } catch (error) {
    console.error('[ntpQuickLinksStore] write removed quick links failed:', error);
  }
}

/** Convert a quick link into a stored chip row. */
export function toStoredQuickLink(link: QuickLink): NtpStoredQuickLink {
  const url = normalizeNtpSiteUrl(link.url);
  const title = link.title?.trim() || ntpHostLabel({ title: '', url });
  return { url, title };
}

/** Load user custom quick links. */
export function loadCustomQuickLinks(): NtpStoredQuickLink[] {
  return readQuickLinkList(NTP_QUICK_LINKS_STORAGE_KEY);
}

/** Whether a URL is already shown as a quick-link chip. */
export function isNtpQuickLink(url: string): boolean {
  const normalized = normalizeNtpSiteUrl(url);
  return buildNtpQuickLinks().some((link) => normalizeNtpSiteUrl(link.url) === normalized);
}

/**
 * Build the visible quick-link chip row.
 * First run: four bundled defaults. After customization: user list only (no fallback refill).
 */
export function buildNtpQuickLinks(): QuickLink[] {
  const customized = isNtpLayoutCustomized();
  const custom = loadCustomQuickLinks();
  const removed = readRemovedQuickLinkUrls();
  const seen = new Set<string>();
  const chips: QuickLink[] = [];

  const pushLink = (link: NtpStoredQuickLink | QuickLink): void => {
    if (chips.length >= NTP_QUICK_LINKS_MAX) return;
    const stored = toStoredQuickLink(link);
    if (removed.has(stored.url) || seen.has(stored.url)) return;
    seen.add(stored.url);
    chips.push({ title: stored.title, url: stored.url });
  };

  if (!customized) {
    for (const link of DEFAULT_QUICK_LINKS) {
      pushLink(link);
    }
    return chips;
  }

  for (const link of custom) pushLink(link);
  for (const link of DEFAULT_QUICK_LINKS) pushLink(link);

  return chips;
}

/** Whether the visible quick-link row has reached its slot limit. */
export function isNtpQuickLinksFull(): boolean {
  return buildNtpQuickLinks().length >= NTP_QUICK_LINKS_MAX;
}

/** Whether a URL can be added to the visible quick-link row. */
export function canAddNtpQuickLink(url: string): boolean {
  if (!isValidNtpSiteUrl(url)) return false;
  const normalized = normalizeNtpSiteUrl(url);
  const chips = buildNtpQuickLinks();
  if (chips.length < NTP_QUICK_LINKS_MAX) return true;
  return chips.some((link) => normalizeNtpSiteUrl(link.url) === normalized);
}

/** Add a quick-link chip (also clears removed flag). */
export function addNtpQuickLink(link: QuickLink): boolean {
  if (!canAddNtpQuickLink(link.url)) return false;
  markNtpLayoutCustomized();
  const stored = toStoredQuickLink(link);
  const removed = readRemovedQuickLinkUrls();
  removed.delete(stored.url);
  writeRemovedQuickLinkUrls(removed);

  const custom = loadCustomQuickLinks().filter((row) => row.url !== stored.url);
  custom.unshift(stored);
  writeQuickLinkList(NTP_QUICK_LINKS_STORAGE_KEY, custom.slice(0, NTP_QUICK_LINKS_MAX));
  return true;
}

/** Remove a quick-link chip (does not auto-refill). */
export function removeNtpQuickLink(link: QuickLink): boolean {
  markNtpLayoutCustomized();
  const normalized = normalizeNtpSiteUrl(link.url);
  const removed = readRemovedQuickLinkUrls();
  removed.add(normalized);
  writeRemovedQuickLinkUrls(removed);

  writeQuickLinkList(
    NTP_QUICK_LINKS_STORAGE_KEY,
    loadCustomQuickLinks().filter((row) => row.url !== normalized),
  );
  return true;
}

/** Reset quick links to bundled factory defaults. */
export function resetNtpQuickLinks(): void {
  try {
    localStorage.removeItem(NTP_QUICK_LINKS_STORAGE_KEY);
    localStorage.removeItem(NTP_REMOVED_QUICK_LINKS_STORAGE_KEY);
  } catch (error) {
    console.error('[ntpQuickLinksStore] reset failed:', error);
  }
}
