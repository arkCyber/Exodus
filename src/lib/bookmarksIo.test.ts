/**
 * Exodus Browser — bookmark import/export E2E-style tests (Vitest, no Tauri).
 */

import { describe, expect, it } from 'vitest';
import {
  BookmarksIoError,
  bookmarksExportFilename,
  bookmarksExportImportRoundTrip,
  parseBookmarksExportJson,
  serializeBookmarksForImport,
} from '$lib/bookmarksIo';

const sampleExport = JSON.stringify([
  {
    id: 'b1',
    url: 'https://example.com',
    title: 'Example',
    created_at: '2026-05-18T12:00:00Z',
    folder: '',
  },
  {
    id: 'b2',
    url: 'https://work.example/doc',
    title: 'Work doc',
    created_at: '2026-05-18T12:01:00Z',
    folder: 'Work',
  },
]);

describe('bookmarksExportFilename', () => {
  it('uses ISO date in filename', () => {
    expect(bookmarksExportFilename(new Date('2026-05-18T12:00:00Z'))).toBe(
      'exodus-bookmarks-2026-05-18.json',
    );
  });

  it('uses current date when no date provided', () => {
    const filename = bookmarksExportFilename();
    expect(filename).toMatch(/^exodus-bookmarks-\d{4}-\d{2}-\d{2}\.json$/);
  });
});

describe('bookmarksIo E2E flow', () => {
  it('parses export JSON from backend shape', () => {
    const items = parseBookmarksExportJson(sampleExport);
    expect(items).toHaveLength(2);
    expect(items[0].url).toBe('https://example.com');
    expect(items[1].folder).toBe('Work');
  });

  it('round-trips export → import payload without losing URLs', () => {
    const { items, importJson } = bookmarksExportImportRoundTrip(sampleExport);
    expect(items).toHaveLength(2);
    const reparsed = parseBookmarksExportJson(importJson);
    expect(reparsed.map((b) => b.url)).toEqual(items.map((b) => b.url));
    expect(reparsed.map((b) => b.folder)).toEqual(items.map((b) => b.folder));
  });

  it('serializeBookmarksForImport produces valid import JSON', () => {
    const json = serializeBookmarksForImport([
      { id: 'x', url: 'https://a.com', title: 'A', created_at: '', folder: '' },
    ]);
    const parsed = JSON.parse(json) as { url: string }[];
    expect(parsed[0].url).toBe('https://a.com');
  });

  it('rejects invalid JSON', () => {
    expect(() => parseBookmarksExportJson('not json')).toThrow(BookmarksIoError);
  });

  it('rejects non-array export', () => {
    expect(() => parseBookmarksExportJson('{"url":"x"}')).toThrow(BookmarksIoError);
  });

  it('rejects bookmark without url', () => {
    expect(() => parseBookmarksExportJson('[{"title":"No URL"}]')).toThrow(BookmarksIoError);
  });
});
