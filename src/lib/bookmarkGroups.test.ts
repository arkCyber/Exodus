/**
 * Exodus Browser — bookmark bar group helper tests (validation & storage).
 */

import { describe, expect, it, beforeEach } from 'vitest';
import type { BookmarkItem } from './browserTypes';
import {
  MAX_BOOKMARK_GROUP_NAME_LENGTH,
  allKnownBookmarkGroupNames,
  bookmarkFolderColor,
  buildBookmarkBarGroupEntries,
  defaultBookmarkGroupColor,
  isBookmarkGroupNameTaken,
  isReservedBookmarkGroupName,
  loadSavedBookmarkBarGroups,
  mergeBookmarkFolderNames,
  normalizeBookmarkGroupColor,
  normalizeBookmarkGroupName,
  reconcileBookmarkBarGroupsStorage,
  sanitizeSavedBookmarkBarGroups,
  saveBookmarkBarGroup,
  validateBookmarkGroupName,
} from './bookmarkGroups';

describe('bookmarkGroups', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('normalizeBookmarkGroupName', () => {
    it('trims and collapses whitespace', () => {
      expect(normalizeBookmarkGroupName('  Work   Projects  ')).toBe('Work Projects');
    });

    it('returns empty for non-string', () => {
      expect(normalizeBookmarkGroupName(null as unknown as string)).toBe('');
    });
  });

  describe('isReservedBookmarkGroupName', () => {
    it('blocks reserved labels case-insensitively', () => {
      expect(isReservedBookmarkGroupName('All bookmarks')).toBe(true);
      expect(isReservedBookmarkGroupName('all bookmarks')).toBe(true);
      expect(isReservedBookmarkGroupName('所有书签')).toBe(true);
      expect(isReservedBookmarkGroupName('Work')).toBe(false);
    });

    it('treats empty as reserved', () => {
      expect(isReservedBookmarkGroupName('   ')).toBe(true);
    });
  });

  describe('validateBookmarkGroupName', () => {
    it('rejects empty, reserved, duplicate, long, and invalid chars', () => {
      expect(validateBookmarkGroupName('', [])).toBe('empty');
      expect(validateBookmarkGroupName('All bookmarks', [])).toBe('reserved');
      expect(validateBookmarkGroupName('Work', ['Work'])).toBe('exists');
      expect(validateBookmarkGroupName('work', ['Work'])).toBe('exists');
      expect(
        validateBookmarkGroupName('x'.repeat(MAX_BOOKMARK_GROUP_NAME_LENGTH + 1), []),
      ).toBe('too_long');
      expect(validateBookmarkGroupName('bad<>name', [])).toBe('invalid_chars');
      expect(validateBookmarkGroupName('Valid Group', [])).toBeNull();
    });
  });

  describe('isBookmarkGroupNameTaken', () => {
    it('is case-insensitive', () => {
      expect(isBookmarkGroupNameTaken('WORK', ['work'])).toBe(true);
    });
  });

  describe('sanitizeSavedBookmarkBarGroups', () => {
    it('removes reserved, dedupes case-insensitive, normalizes colors', () => {
      const out = sanitizeSavedBookmarkBarGroups([
        { name: '  Work ', color: 'blue' },
        { name: 'work', color: 'red' },
        { name: 'All bookmarks', color: 'green' },
        { name: '所有书签', color: 'yellow' },
        { name: 'News', color: 'not-a-color' },
      ]);
      expect(out).toHaveLength(2);
      expect(out.map((g) => g.name)).toEqual(['News', 'Work']);
      expect(out.find((g) => g.name === 'Work')?.color).toBe('blue');
      expect(normalizeBookmarkGroupColor('not-a-color', 'News')).toBeTruthy();
    });
  });

  describe('persistence', () => {
    it('saveBookmarkBarGroup rejects names that exist as bookmark folders', () => {
      const bookmarks: BookmarkItem[] = [
        { id: '1', url: 'https://a.com', title: 'A', folder: 'Work' },
      ];
      const existing = mergeBookmarkFolderNames(bookmarks);
      expect(saveBookmarkBarGroup('Work', 'blue', existing)).toBe(false);
      expect(saveBookmarkBarGroup('work', 'red', existing)).toBe(false);
    });

    it('saveBookmarkBarGroup returns false for invalid names', () => {
      expect(saveBookmarkBarGroup('', 'blue')).toBe(false);
      expect(saveBookmarkBarGroup('All bookmarks', 'blue')).toBe(false);
      expect(loadSavedBookmarkBarGroups()).toEqual([]);
    });

    it('saveBookmarkBarGroup persists and updates color on rename-case conflict', () => {
      saveBookmarkBarGroup('Projects', 'green');
      expect(saveBookmarkBarGroup('projects', 'red')).toBe(false);
      expect(loadSavedBookmarkBarGroups()).toEqual([{ name: 'Projects', color: 'green' }]);
    });

    it('loadSavedBookmarkBarGroups self-heals corrupt storage', () => {
      localStorage.setItem(
        'exodus-bookmark-bar-groups',
        JSON.stringify([
          { name: '  Alpha ', color: 'blue' },
          { name: 'alpha', color: 'red' },
          { name: 'All bookmarks', color: 'green' },
        ]),
      );
      const loaded = loadSavedBookmarkBarGroups();
      expect(loaded).toEqual([{ name: 'Alpha', color: 'blue' }]);
      expect(JSON.parse(localStorage.getItem('exodus-bookmark-bar-groups')!)).toEqual(loaded);
    });

    it('mergeBookmarkFolderNames includes saved empty groups', () => {
      saveBookmarkBarGroup('Empty Group', 'blue');
      const bookmarks: BookmarkItem[] = [
        { id: '1', url: 'https://a.com', title: 'A', folder: 'Work' },
      ];
      expect(mergeBookmarkFolderNames(bookmarks)).toEqual(['Empty Group', 'Work']);
    });
  });

  describe('buildBookmarkBarGroupEntries', () => {
    it('reports bookmark counts per folder', () => {
      const bookmarks: BookmarkItem[] = [
        { id: '1', url: 'https://a.com', title: 'A', folder: 'Work' },
        { id: '2', url: 'https://b.com', title: 'B', folder: 'Work' },
      ];
      const entries = buildBookmarkBarGroupEntries(bookmarks, ['Work']);
      expect(entries).toHaveLength(1);
      expect(entries[0]).toEqual({ name: 'Work', color: bookmarkFolderColor('Work'), count: 2 });
    });
  });

  describe('allKnownBookmarkGroupNames', () => {
    it('merges bookmarks, saved groups, and prop folder names', () => {
      saveBookmarkBarGroup('Saved Only', 'purple');
      const bookmarks: BookmarkItem[] = [{ id: '1', url: 'https://a.com', title: 'A', folder: 'Work' }];
      const names = allKnownBookmarkGroupNames(bookmarks, ['Extra'], loadSavedBookmarkBarGroups());
      expect(names).toContain('Work');
      expect(names).toContain('Saved Only');
      expect(names).toContain('Extra');
    });
  });

  describe('reconcileBookmarkBarGroupsStorage', () => {
    it('strips reserved colors from color map', () => {
      localStorage.setItem(
        'exodus-bookmark-folder-colors',
        JSON.stringify({ Work: 'blue', 'All bookmarks': 'red' }),
      );
      reconcileBookmarkBarGroupsStorage([]);
      const colors = JSON.parse(localStorage.getItem('exodus-bookmark-folder-colors')!);
      expect(colors['All bookmarks']).toBeUndefined();
      expect(colors.Work).toBe('blue');
    });
  });

  describe('defaultBookmarkGroupColor', () => {
    it('is stable for a name', () => {
      expect(defaultBookmarkGroupColor('Work')).toBe(defaultBookmarkGroupColor('Work'));
      expect(defaultBookmarkGroupColor('Work')).not.toBe(defaultBookmarkGroupColor('Personal'));
    });
  });
});
