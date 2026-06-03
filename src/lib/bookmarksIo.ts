/**
 * Exodus Browser — bookmark import/export JSON helpers.
 * Used by Settings UI and Vitest E2E-style round-trip tests.
 */

import type { BookmarkItem } from '$lib/browserTypes';

/** Raw bookmark shape from `export_bookmarks` (Rust serde). */
export type ExportedBookmark = {
  id: string;
  url: string;
  title: string;
  created_at: string;
  folder?: string;
};

/** Error thrown when bookmark JSON is invalid or empty. */
export class BookmarksIoError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'BookmarksIoError';
  }
}

/**
 * Parse exported bookmark JSON into UI bookmark items.
 * Accepts the array returned by `export_bookmarks`.
 */
export function parseBookmarksExportJson(json: string): BookmarkItem[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch {
    throw new BookmarksIoError('Invalid JSON');
  }
  if (!Array.isArray(parsed)) {
    throw new BookmarksIoError('Expected a JSON array of bookmarks');
  }
  const items: BookmarkItem[] = [];
  for (let i = 0; i < parsed.length; i++) {
    const row = parsed[i];
    if (!row || typeof row !== 'object') {
      throw new BookmarksIoError(`Invalid bookmark at index ${i}`);
    }
    const rec = row as Record<string, unknown>;
    const url = typeof rec.url === 'string' ? rec.url.trim() : '';
    const title = typeof rec.title === 'string' ? rec.title.trim() : '';
    if (!url) {
      throw new BookmarksIoError(`Bookmark at index ${i} is missing url`);
    }
    items.push({
      id: typeof rec.id === 'string' ? rec.id : `import-${i}`,
      url,
      title: title || url,
      created_at: typeof rec.created_at === 'string' ? rec.created_at : '',
      folder: typeof rec.folder === 'string' ? rec.folder : '',
    });
  }
  return items;
}

/**
 * Serialize bookmarks for `import_bookmarks` (preserves export fields when present).
 */
export function serializeBookmarksForImport(items: BookmarkItem[]): string {
  const payload: ExportedBookmark[] = items.map((b) => ({
    id: b.id,
    url: b.url,
    title: b.title,
    created_at: b.created_at || new Date().toISOString(),
    folder: b.folder ?? '',
  }));
  return JSON.stringify(payload);
}

/** Default filename for bookmark export downloads. */
export function bookmarksExportFilename(date = new Date()): string {
  const day = date.toISOString().split('T')[0];
  return `exodus-bookmarks-${day}.json`;
}

/**
 * Full client-side round-trip: export JSON → parse → re-serialize for import.
 */
export function bookmarksExportImportRoundTrip(exportJson: string): {
  items: BookmarkItem[];
  importJson: string;
} {
  const items = parseBookmarksExportJson(exportJson);
  return { items, importJson: serializeBookmarksForImport(items) };
}
