/**
 * Exodus Browser — bookmark bar / folder helpers.
 */

import type { BookmarkItem } from '$lib/browserTypes';

/** Max chips on the bookmark bar before overflow menu. */
export const BOOKMARK_BAR_MAX = 10;

/** Bookmarks shown on the bookmark bar (empty folder), sorted by `bar_order`. */
export function bookmarksOnBar(bookmarks: BookmarkItem[]): BookmarkItem[] {
  return bookmarks
    .filter((b) => !b.folder)
    .sort((a, b) => (a.bar_order ?? 0) - (b.bar_order ?? 0) || a.title.localeCompare(b.title));
}

/** Distinct bookmark folder names (panel-only). */
export function bookmarkFolderNames(bookmarks: BookmarkItem[]): string[] {
  const names = new Set<string>();
  for (const b of bookmarks) {
    if (b.folder) names.add(b.folder);
  }
  return [...names].sort();
}

/** Bookmarks inside a named folder. */
export function bookmarksInFolder(bookmarks: BookmarkItem[], name: string): BookmarkItem[] {
  return bookmarks.filter((b) => b.folder === name);
}
