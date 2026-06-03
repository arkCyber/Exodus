/**
 * Exodus Browser — sync bookmark bar state with Tauri RagDatabase backend.
 * Aerospace-level error handling, security validation, and input validation.
 */

import { shellLog } from '@/lib/diagnosticLog';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { BookmarkItem } from '@/lib/browserTypes';

shellLog.info('bookmarkBackendSync module loaded');

// Aerospace-level security validation patterns
const VALID_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_URL_PATTERN = /^https?:\/\/.+/;
const VALID_TITLE_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const VALID_FOLDER_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const MAX_URL_LENGTH = 2000;
const MAX_TITLE_LENGTH = 500;
const MAX_FOLDER_LENGTH = 100;

/**
 * Aerospace-level validation for bookmark ID format.
 */
function validateBookmarkId(id: string): boolean {
  if (!id || typeof id !== 'string') {
    shellLog.error('Invalid bookmark ID');
    return false;
  }
  return VALID_ID_PATTERN.test(id);
}

/**
 * Aerospace-level validation for URL format.
 */
function validateUrl(url: string): boolean {
  if (!url || typeof url !== 'string') {
    shellLog.error('Invalid URL');
    return false;
  }
  if (url.length > MAX_URL_LENGTH) {
    shellLog.error('URL too long');
    return false;
  }
  return VALID_URL_PATTERN.test(url);
}

/**
 * Aerospace-level validation for title format.
 */
function validateTitle(title: string): boolean {
  if (!title || typeof title !== 'string') {
    shellLog.error('Invalid title');
    return false;
  }
  if (title.length > MAX_TITLE_LENGTH) {
    shellLog.error('Title too long');
    return false;
  }
  return VALID_TITLE_PATTERN.test(title);
}

/**
 * Aerospace-level validation for folder name format.
 */
function validateFolder(folder: string): boolean {
  if (!folder || typeof folder !== 'string') {
    shellLog.error('Invalid folder name');
    return false;
  }
  if (folder.length > MAX_FOLDER_LENGTH) {
    shellLog.error('Folder name too long');
    return false;
  }
  return VALID_FOLDER_PATTERN.test(folder);
}

/** Mirror backend bookmark rows into localStorage for `useBookmarks`. */
export function mirrorBookmarksToLocalStorage(bookmarks: BookmarkItem[]): void {
  // Aerospace-level input validation for bookmarks array
  if (!Array.isArray(bookmarks)) {
    shellLog.error('Invalid bookmarks array');
    return;
  }

  try {
    localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks));
  } catch (error) {
    shellLog.error('mirrorBookmarksToLocalStorage failed', error);
  }
}

/** Fetch bookmarks from Rust store when running in Tauri. */
export async function fetchBookmarksFromBackend(): Promise<BookmarkItem[] | null> {
  if (!isTauri()) return null;
  try {
    return await invoke<BookmarkItem[]>('list_bookmarks');
  } catch (error) {
    shellLog.error('list_bookmarks failed', error);
    return null;
  }
}

/** Pull backend bookmarks into localStorage (no-op in Vite dev). */
export async function syncBookmarksFromBackendIfTauri(): Promise<boolean> {
  const remote = await fetchBookmarksFromBackend();
  if (!remote) return false;
  mirrorBookmarksToLocalStorage(remote);
  return true;
}

/** Persist add/upsert to backend when in Tauri. */
export async function persistBookmarkAddToBackend(
  url: string,
  title: string,
  folder?: string,
): Promise<BookmarkItem | null> {
  if (!isTauri()) return null;
  
  // Aerospace-level input validation
  if (!validateUrl(url)) {
    shellLog.error('Invalid URL in persistBookmarkAddToBackend', url);
    return null;
  }
  if (!validateTitle(title)) {
    shellLog.error('Invalid title in persistBookmarkAddToBackend', title);
    return null;
  }
  if (folder && !validateFolder(folder)) {
    shellLog.error('Invalid folder in persistBookmarkAddToBackend', folder);
    return null;
  }
  
  try {
    return await invoke<BookmarkItem>('add_bookmark', {
      url,
      title,
      folder: folder?.trim() || '',
    });
  } catch (error) {
    shellLog.error('add_bookmark failed', error);
    return null;
  }
}

/** Remove bookmark in backend when in Tauri. */
export async function persistBookmarkRemoveFromBackend(id: string): Promise<boolean> {
  if (!isTauri()) return false;
  
  // Aerospace-level input validation
  if (!validateBookmarkId(id)) {
    shellLog.error('Invalid bookmark ID in persistBookmarkRemoveFromBackend', id);
    return false;
  }
  
  try {
    await invoke('remove_bookmark', { id });
    return true;
  } catch (error) {
    shellLog.error('remove_bookmark failed', error);
    return false;
  }
}
