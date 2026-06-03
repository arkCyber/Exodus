/**
 * Exodus Browser — Firefox-style preset bookmark bar and new-tab quick links.
 */

import { invoke } from '@tauri-apps/api/core';
import type { BookmarkItem, QuickLink } from '$lib/browserTypes';

/** localStorage flag: preset bar was seeded once (do not re-seed after user clears all). */
export const PRESET_BOOKMARK_STORAGE_KEY = 'exodus-preset-bookmarks-seeded-v1';

/**
 * Default sites for the bookmark bar (Firefox toolbar style) and new-tab quick links.
 * Order is left-to-right on the bar.
 */
export const PRESET_BOOKMARK_SITES: QuickLink[] = [
  { title: 'DuckDuckGo', url: 'https://duckduckgo.com' },
  { title: 'Wikipedia', url: 'https://en.wikipedia.org' },
  { title: 'GitHub', url: 'https://github.com' },
  { title: 'Hacker News', url: 'https://news.ycombinator.com' },
  { title: 'Stack Overflow', url: 'https://stackoverflow.com' },
  { title: 'Reddit', url: 'https://www.reddit.com' },
];

/** First four presets shown as new-tab quick-link chips. */
export const DEFAULT_QUICK_LINKS: QuickLink[] = PRESET_BOOKMARK_SITES.slice(0, 4);

function markPresetBookmarksSeeded(): void {
  try {
    localStorage.setItem(PRESET_BOOKMARK_STORAGE_KEY, '1');
  } catch (error) {
    console.error('markPresetBookmarksSeeded failed:', error);
  }
}

function isPresetBookmarksSeeded(): boolean {
  try {
    return localStorage.getItem(PRESET_BOOKMARK_STORAGE_KEY) === '1';
  } catch {
    return false;
  }
}

/**
 * Seed the bookmark bar on first run when the store is empty (Firefox-style defaults).
 * Always reads from the backend store (never trust in-memory Svelte state).
 * Returns true if any bookmarks were added.
 */
export async function seedPresetBookmarksIfEmpty(): Promise<boolean> {
  if (isPresetBookmarksSeeded()) return false;

  let bookmarks: BookmarkItem[];
  try {
    bookmarks = (await invoke('list_bookmarks')) as BookmarkItem[];
  } catch (error) {
    console.error('seedPresetBookmarksIfEmpty list_bookmarks failed:', error);
    return false;
  }

  if (bookmarks.length > 0) {
    markPresetBookmarksSeeded();
    return false;
  }

  try {
    for (const site of PRESET_BOOKMARK_SITES) {
      await invoke('add_bookmark', {
        url: site.url,
        title: site.title,
        folder: '',
      });
    }
    markPresetBookmarksSeeded();
    return true;
  } catch (error) {
    console.error('seedPresetBookmarksIfEmpty add_bookmark failed:', error);
    return false;
  }
}
