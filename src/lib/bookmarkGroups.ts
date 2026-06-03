/**
 * Exodus Browser — bookmark bar groups (Chrome tab-group style folders).
 * Persistence, validation, and display helpers with self-healing storage.
 */

import { logStartup } from '@/lib/startupLog';
import type { BookmarkItem } from './browserTypes';
import { bookmarksInFolder, bookmarkFolderNames } from './bookmarks';
import { tabGroupColorCss, TAB_GROUP_COLORS } from './tabGroups';

/** Reserved folder names for Chrome right-pinned “All bookmarks” (not user folders). */
export const RESERVED_BOOKMARK_BAR_FOLDERS = new Set<string>([
  'All bookmarks',
  '所有书签',
]);

logStartup('bookmarkGroups module loaded');

/** Maximum characters for a bookmark group / folder name. */
export const MAX_BOOKMARK_GROUP_NAME_LENGTH = 64;

/** Persisted bookmark bar group (folder may be empty). */
export type SavedBookmarkBarGroup = {
  name: string;
  color: string;
};

/** Group row for the bookmark bar groups menu. */
export type BookmarkBarGroupEntry = {
  name: string;
  color: string;
  count: number;
};

/** Validation failure codes for group names. */
export type BookmarkGroupValidationError =
  | 'empty'
  | 'reserved'
  | 'exists'
  | 'too_long'
  | 'invalid_chars';

const GROUPS_STORAGE_KEY = 'exodus-bookmark-bar-groups';
const COLORS_STORAGE_KEY = 'exodus-bookmark-folder-colors';

const VALID_COLOR_IDS = new Set(TAB_GROUP_COLORS.map((c) => c.toLowerCase()));

/** Trim and collapse internal whitespace. */
export function normalizeBookmarkGroupName(name: string): string {
  if (typeof name !== 'string') return '';
  return name.trim().replace(/\s+/g, ' ');
}

/** True when folder name is reserved for Chrome “All bookmarks” (not a user group). */
export function isReservedBookmarkGroupName(name: string): boolean {
  const normalized = normalizeBookmarkGroupName(name);
  if (!normalized) return true;
  const lower = normalized.toLowerCase();
  for (const reserved of RESERVED_BOOKMARK_BAR_FOLDERS) {
    if (reserved.toLowerCase() === lower) return true;
  }
  return false;
}

/** Case-insensitive duplicate check against existing folder / group names. */
export function isBookmarkGroupNameTaken(name: string, existingNames: readonly string[]): boolean {
  const target = normalizeBookmarkGroupName(name).toLowerCase();
  if (!target) return false;
  return existingNames.some((n) => normalizeBookmarkGroupName(n).toLowerCase() === target);
}

/**
 * Validate a proposed bookmark group name.
 * Returns null when valid, otherwise a machine-readable error code.
 */
export function validateBookmarkGroupName(
  name: string,
  existingNames: readonly string[],
): BookmarkGroupValidationError | null {
  const normalized = normalizeBookmarkGroupName(name);
  if (!normalized) return 'empty';
  if (normalized.length > MAX_BOOKMARK_GROUP_NAME_LENGTH) return 'too_long';
  if (/[\x00-\x1f<>]/.test(normalized)) return 'invalid_chars';
  if (isReservedBookmarkGroupName(normalized)) return 'reserved';
  if (isBookmarkGroupNameTaken(normalized, existingNames)) return 'exists';
  return null;
}

/** Default color when none is stored for a folder. */
export function defaultBookmarkGroupColor(name: string): string {
  if (!name) return 'grey';
  let hash = 0;
  for (let i = 0; i < name.length; i += 1) {
    hash = (hash * 31 + name.charCodeAt(i)) >>> 0;
  }
  const palette = TAB_GROUP_COLORS.filter((c) => c !== 'grey');
  return palette[hash % palette.length] ?? 'blue';
}

/** Normalize color id to a known tab-group palette entry. */
export function normalizeBookmarkGroupColor(color: string, fallbackName = ''): string {
  const lower = (color || '').trim().toLowerCase();
  if (VALID_COLOR_IDS.has(lower)) return lower;
  return defaultBookmarkGroupColor(fallbackName);
}

/** CSS color for a bookmark group dot / stripe. */
export function bookmarkGroupColorCss(color: string): string {
  return tabGroupColorCss(color);
}

/** Sanitize and dedupe loaded group rows (case-insensitive by name). */
export function sanitizeSavedBookmarkBarGroups(
  rows: SavedBookmarkBarGroup[],
): SavedBookmarkBarGroup[] {
  const seen = new Set<string>();
  const out: SavedBookmarkBarGroup[] = [];
  for (const row of rows) {
    const name = normalizeBookmarkGroupName(row?.name ?? '');
    if (!name || isReservedBookmarkGroupName(name)) continue;
    const key = name.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    out.push({
      name,
      color: normalizeBookmarkGroupColor(row?.color ?? '', name),
    });
  }
  return out.sort((a, b) => a.name.localeCompare(b.name));
}

function persistSavedGroups(groups: SavedBookmarkBarGroup[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    const sanitized = sanitizeSavedBookmarkBarGroups(groups);
    localStorage.setItem(GROUPS_STORAGE_KEY, JSON.stringify(sanitized));
  } catch (error) {
    console.error('persistSavedGroups failed:', error);
  }
}

/** Load saved empty / named groups from localStorage (self-healing). */
export function loadSavedBookmarkBarGroups(): SavedBookmarkBarGroup[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(GROUPS_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      persistSavedGroups([]);
      return [];
    }
    const rows: SavedBookmarkBarGroup[] = [];
    for (const item of parsed) {
      if (!item || typeof item !== 'object') continue;
      const g = item as SavedBookmarkBarGroup;
      if (typeof g.name !== 'string') continue;
      rows.push({
        name: g.name,
        color: typeof g.color === 'string' ? g.color : '',
      });
    }
    const sanitized = sanitizeSavedBookmarkBarGroups(rows);
    try {
      const current = localStorage.getItem(GROUPS_STORAGE_KEY);
      const next = JSON.stringify(sanitized);
      if (current !== next) {
        localStorage.setItem(GROUPS_STORAGE_KEY, next);
      }
    } catch (error) {
      console.error('loadSavedBookmarkBarGroups persist failed:', error);
    }
    return sanitized;
  } catch (error) {
    console.error('loadSavedBookmarkBarGroups failed:', error);
    return [];
  }
}

/** Re-read storage and rewrite sanitized groups + color map (call after bookmark sync). */
export function reconcileBookmarkBarGroupsStorage(bookmarks: BookmarkItem[]): void {
  const groups = loadSavedBookmarkBarGroups();
  persistSavedGroups(groups);
  const folderNames = bookmarkFolderNames(bookmarks);
  const colors = loadBookmarkFolderColors();
  for (const folder of folderNames) {
    if (!colors[folder]) {
      setBookmarkFolderColor(folder, defaultBookmarkGroupColor(folder));
    }
  }
  for (const key of Object.keys(colors)) {
    if (isReservedBookmarkGroupName(key)) {
      delete colors[key];
    }
  }
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(COLORS_STORAGE_KEY, JSON.stringify(colors));
    }
  } catch (error) {
    console.error('reconcileBookmarkBarGroupsStorage colors failed:', error);
  }
}

/**
 * Persist a new or updated bookmark bar group.
 * Returns false when validation fails or storage is unavailable.
 */
export function saveBookmarkBarGroup(
  name: string,
  color: string,
  existingNames?: readonly string[],
): boolean {
  if (typeof localStorage === 'undefined') return false;
  const normalized = normalizeBookmarkGroupName(name);
  const existing =
    existingNames ??
    mergeBookmarkFolderNames([], loadSavedBookmarkBarGroups());
  const err = validateBookmarkGroupName(normalized, existing);
  if (err) return false;
  const normalizedColor = normalizeBookmarkGroupColor(color, normalized);
  try {
    const groups = loadSavedBookmarkBarGroups().filter(
      (g) => g.name.toLowerCase() !== normalized.toLowerCase(),
    );
    groups.push({ name: normalized, color: normalizedColor });
    persistSavedGroups(groups);
    setBookmarkFolderColor(normalized, normalizedColor);
    return true;
  } catch (error) {
    console.error('saveBookmarkBarGroup failed:', error);
    return false;
  }
}

/** Remove a saved empty group and its color entry. */
export function removeBookmarkBarGroup(name: string): void {
  if (typeof localStorage === 'undefined') return;
  const normalized = normalizeBookmarkGroupName(name);
  if (!normalized) return;
  try {
    const groups = loadSavedBookmarkBarGroups().filter(
      (g) => g.name.toLowerCase() !== normalized.toLowerCase(),
    );
    persistSavedGroups(groups);
    const colors = loadBookmarkFolderColors();
    for (const key of Object.keys(colors)) {
      if (key.toLowerCase() === normalized.toLowerCase()) delete colors[key];
    }
    localStorage.setItem(COLORS_STORAGE_KEY, JSON.stringify(colors));
  } catch (error) {
    console.error('removeBookmarkBarGroup failed:', error);
  }
}

/** Per-folder color overrides (folder name → color id). */
export function loadBookmarkFolderColors(): Record<string, string> {
  if (typeof localStorage === 'undefined') return {};
  try {
    const raw = localStorage.getItem(COLORS_STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object') return {};
    const out: Record<string, string> = {};
    for (const [key, value] of Object.entries(parsed)) {
      if (typeof key !== 'string' || typeof value !== 'string') continue;
      const trimmed = normalizeBookmarkGroupName(key);
      if (!trimmed || isReservedBookmarkGroupName(trimmed)) continue;
      out[trimmed] = normalizeBookmarkGroupColor(value, trimmed);
    }
    return out;
  } catch (error) {
    console.error('loadBookmarkFolderColors failed:', error);
    return {};
  }
}

/** Store color for a bookmark folder / group. */
export function setBookmarkFolderColor(folderName: string, color: string): void {
  if (typeof localStorage === 'undefined') return;
  const trimmed = normalizeBookmarkGroupName(folderName);
  if (!trimmed || isReservedBookmarkGroupName(trimmed)) return;
  try {
    const colors = loadBookmarkFolderColors();
    colors[trimmed] = normalizeBookmarkGroupColor(color, trimmed);
    localStorage.setItem(COLORS_STORAGE_KEY, JSON.stringify(colors));
  } catch (error) {
    console.error('setBookmarkFolderColor failed:', error);
  }
}

/** Resolve display color for a folder. */
export function bookmarkFolderColor(
  folderName: string,
  savedGroups?: SavedBookmarkBarGroup[],
): string {
  const trimmed = normalizeBookmarkGroupName(folderName);
  if (!trimmed) return 'grey';
  const groups = savedGroups ?? loadSavedBookmarkBarGroups();
  const fromSaved = groups.find((g) => g.name.toLowerCase() === trimmed.toLowerCase());
  if (fromSaved?.color) return fromSaved.color;
  const fromStore = loadBookmarkFolderColors()[trimmed];
  if (fromStore) return fromStore;
  return defaultBookmarkGroupColor(trimmed);
}

/**
 * Merge folder names from bookmarks with saved empty groups (stable sort).
 */
export function mergeBookmarkFolderNames(
  bookmarks: BookmarkItem[],
  savedGroups?: SavedBookmarkBarGroup[],
): string[] {
  const names = new Set<string>(bookmarkFolderNames(bookmarks));
  for (const group of savedGroups ?? loadSavedBookmarkBarGroups()) {
    if (group.name && !isReservedBookmarkGroupName(group.name)) names.add(group.name);
  }
  return [...names].sort((a, b) => a.localeCompare(b));
}

/**
 * Build group menu rows: name, color, bookmark count (Chrome “N tabs” parity).
 */
export function buildBookmarkBarGroupEntries(
  bookmarks: BookmarkItem[],
  folderNames: string[],
  savedGroups?: SavedBookmarkBarGroup[],
): BookmarkBarGroupEntry[] {
  const groups = savedGroups ?? loadSavedBookmarkBarGroups();
  const names = new Set<string>();
  for (const folder of folderNames) {
    if (folder && !isReservedBookmarkGroupName(folder)) names.add(normalizeBookmarkGroupName(folder));
  }
  for (const g of groups) {
    if (g.name && !isReservedBookmarkGroupName(g.name)) names.add(g.name);
  }
  return [...names]
    .sort((a, b) => a.localeCompare(b))
    .map((name) => ({
      name,
      color: bookmarkFolderColor(name, groups),
      count: bookmarksInFolder(bookmarks, name).length,
    }));
}

/** Collect all known group names for validation (bookmarks + saved + bar props). */
export function allKnownBookmarkGroupNames(
  bookmarks: BookmarkItem[],
  folderNames: readonly string[],
  savedGroups?: SavedBookmarkBarGroup[],
): string[] {
  const names = new Set<string>(mergeBookmarkFolderNames(bookmarks, savedGroups));
  for (const folder of folderNames) {
    const n = normalizeBookmarkGroupName(folder);
    if (n && !isReservedBookmarkGroupName(n)) names.add(n);
  }
  return [...names].sort((a, b) => a.localeCompare(b));
}
