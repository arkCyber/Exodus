import { shallowRef } from 'vue';

import type { BookmarkItem as SharedBookmarkItem } from '../lib/browserTypes';

// Aerospace-level security validation patterns
const VALID_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_URL_PATTERN = /^https?:\/\/.+/;
const VALID_TITLE_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const VALID_FOLDER_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const MAX_URL_LENGTH = 2000;
const MAX_TITLE_LENGTH = 500;
const MAX_FOLDER_LENGTH = 100;
const MAX_BOOKMARKS = 5000;

/**
 * Aerospace-level validation for bookmark ID format.
 */
function validateBookmarkId(id: string): boolean {
  if (!id || typeof id !== 'string') {
    console.error('[UseBookmarks] Invalid bookmark ID');
    return false;
  }
  return VALID_ID_PATTERN.test(id);
}

/**
 * Aerospace-level validation for URL format.
 */
function validateUrl(url: string): boolean {
  if (!url || typeof url !== 'string') {
    console.error('[UseBookmarks] Invalid URL');
    return false;
  }
  if (url.length > MAX_URL_LENGTH) {
    console.error('[UseBookmarks] URL too long');
    return false;
  }
  return VALID_URL_PATTERN.test(url);
}

/**
 * Aerospace-level validation for title format.
 */
function validateTitle(title: string): boolean {
  if (!title || typeof title !== 'string') {
    console.error('[UseBookmarks] Invalid title');
    return false;
  }
  if (title.length > MAX_TITLE_LENGTH) {
    console.error('[UseBookmarks] Title too long');
    return false;
  }
  return VALID_TITLE_PATTERN.test(title);
}

/**
 * Aerospace-level validation for folder name format.
 */
function validateFolder(folder: string): boolean {
  if (!folder || typeof folder !== 'string') {
    console.error('[UseBookmarks] Invalid folder name');
    return false;
  }
  if (folder.length > MAX_FOLDER_LENGTH) {
    console.error('[UseBookmarks] Folder name too long');
    return false;
  }
  return VALID_FOLDER_PATTERN.test(folder);
}

interface BookmarkItem extends Omit<SharedBookmarkItem, 'created_at'> {
  created_at: string;
  createdAt: number;
}

const bookmarks = shallowRef<BookmarkItem[]>([]);
const folders = shallowRef<string[]>(['Favorites', 'Work', 'News']);

const DEFAULT_BOOKMARKS = [
  { title: 'DuckDuckGo', url: 'https://duckduckgo.com' },
  { title: 'Wikipedia', url: 'https://en.wikipedia.org' },
  { title: 'GitHub', url: 'https://github.com' },
  { title: 'Hacker News', url: 'https://news.ycombinator.com' },
  { title: 'Stack Overflow', url: 'https://stackoverflow.com' },
  { title: 'Reddit', url: 'https://www.reddit.com' },
];

export function useBookmarks() {
  function addBookmark(title: string, url: string, folder?: string, favicon?: string, description?: string): SharedBookmarkItem | undefined {
    // Aerospace-level input validation for URL
    if (!url || typeof url !== 'string') return;
    const trimmedUrl = url.trim();
    if (!trimmedUrl || trimmedUrl.startsWith('data:') || trimmedUrl.startsWith('javascript:') || trimmedUrl.startsWith('vbscript:')) return undefined;
    
    // Normalize URL first
    let normalizedUrl = trimmedUrl;
    if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
      normalizedUrl = `https://${normalizedUrl}`;
    }
    
    // Aerospace-level validation for normalized URL
    if (!validateUrl(normalizedUrl)) {
      console.error('[UseBookmarks] Invalid URL in addBookmark:', normalizedUrl);
      return undefined;
    }
    
    // Aerospace-level input validation for title
    const safeTitle = (title && typeof title === 'string') ? title.trim() : 'Untitled';
    if (!validateTitle(safeTitle)) {
      console.error('[UseBookmarks] Invalid title in addBookmark:', safeTitle);
      return undefined;
    }
    
    // Aerospace-level input validation for folder
    const safeFolder = (folder && typeof folder === 'string') ? folder.trim() : undefined;
    if (safeFolder && !validateFolder(safeFolder)) {
      console.error('[UseBookmarks] Invalid folder in addBookmark:', safeFolder);
      return undefined;
    }
    
    // Validate favicon
    const safeFavicon = (favicon && typeof favicon === 'string') ? favicon.trim() : undefined;
    
    // Validate description
    const safeDescription = (description && typeof description === 'string') ? description.trim() : undefined;
    
    // Check for duplicates
    const existingIndex = bookmarks.value.findIndex(b => b && b.url === normalizedUrl);
    if (existingIndex > -1) {
      // Update existing bookmark
      bookmarks.value[existingIndex] = {
        ...bookmarks.value[existingIndex],
        title: safeTitle,
        folder: safeFolder,
        favicon: safeFavicon,
        description: safeDescription,
      };
    } else {
      // Add new bookmark
      const now = Date.now();
      const bookmark: BookmarkItem = {
        id: `bookmark-${now}-${Math.random().toString(36).substring(2, 11)}`,
        title: safeTitle,
        url: normalizedUrl,
        folder: safeFolder,
        favicon: safeFavicon,
        description: safeDescription,
        created_at: new Date(now).toISOString(),
        createdAt: now,
      };
      bookmarks.value.push(bookmark);
    }
    
    // Limit bookmark size
    if (bookmarks.value.length > MAX_BOOKMARKS) {
      bookmarks.value = bookmarks.value.slice(0, MAX_BOOKMARKS);
    }
    
    // Persist to localStorage with quota handling
    try {
      localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks.value));
    } catch (e) {
      if (e instanceof DOMException && e.name === 'QuotaExceededError') {
        console.warn('LocalStorage quota exceeded, clearing old bookmarks');
        // Remove oldest bookmarks to free space
        bookmarks.value = bookmarks.value.slice(-MAX_BOOKMARKS / 2);
        try {
          localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks.value));
        } catch (retryError) {
          console.error('Failed to save bookmarks after cleanup:', retryError);
        }
      } else {
        console.error('Failed to save bookmarks:', e);
      }
    }

    const saved = bookmarks.value.find((b) => b && b.url === normalizedUrl);
    if (!saved) return undefined;
    return {
      id: saved.id,
      url: saved.url,
      title: saved.title || saved.url,
      created_at: saved.created_at || '',
      folder: saved.folder,
      favicon: saved.favicon,
      bar_order: saved.bar_order,
      description: saved.description,
    };
  }
  
  function removeBookmark(id: string) {
    // Aerospace-level input validation for ID
    if (!id || typeof id !== 'string') return;
    if (!validateBookmarkId(id)) {
      console.error('[UseBookmarks] Invalid bookmark ID in removeBookmark:', id);
      return;
    }
    const index = bookmarks.value.findIndex(b => b && b.id === id);
    if (index > -1) {
      bookmarks.value.splice(index, 1);
      try {
        localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks.value));
      } catch (e) {
        console.error('Failed to save bookmarks:', e);
      }
    }
  }
  
  function updateBookmark(id: string, updates: Partial<BookmarkItem>) {
    // Aerospace-level input validation for ID
    if (!id || typeof id !== 'string') return;
    if (!validateBookmarkId(id)) {
      console.error('[UseBookmarks] Invalid bookmark ID in updateBookmark:', id);
      return;
    }
    if (!updates) return;
    
    // Aerospace-level input validation for updates
    if (updates.url && !validateUrl(updates.url)) {
      console.error('[UseBookmarks] Invalid URL in updateBookmark:', updates.url);
      return;
    }
    if (updates.title && !validateTitle(updates.title)) {
      console.error('[UseBookmarks] Invalid title in updateBookmark:', updates.title);
      return;
    }
    if (updates.folder && !validateFolder(updates.folder)) {
      console.error('[UseBookmarks] Invalid folder in updateBookmark:', updates.folder);
      return;
    }
    
    const index = bookmarks.value.findIndex(b => b && b.id === id);
    if (index > -1) {
      bookmarks.value[index] = { ...bookmarks.value[index], ...updates };
      try {
        localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks.value));
      } catch (e) {
        console.error('Failed to save bookmarks:', e);
      }
    }
  }
  
  function getBookmarks(folder?: string): SharedBookmarkItem[] {
    const validBookmarks = bookmarks.value.filter(b => b && b.id && b.url);
    if (folder) {
      return validBookmarks.filter(b => b.folder === folder).map(b => ({
        id: b.id,
        url: b.url,
        title: b.title || b.url,
        created_at: b.created_at || '',
        folder: b.folder,
        favicon: b.favicon,
        bar_order: b.bar_order,
        description: b.description,
      }));
    }
    return validBookmarks.filter(b => !b.folder).map(b => ({
      id: b.id,
      url: b.url,
      title: b.title || b.url,
      created_at: b.created_at || '',
      folder: b.folder,
      favicon: b.favicon,
      bar_order: b.bar_order,
      description: b.description,
    }));
  }
  
  function getBookmarksInFolder(folder: string): BookmarkItem[] {
    if (!folder || typeof folder !== 'string') return [];
    return bookmarks.value.filter(b => b && b.folder === folder);
  }
  
  function isBookmarked(url: string): boolean {
    if (!url || typeof url !== 'string') return false;
    return bookmarks.value.some(b => b && b.url === url);
  }
  
  function getBookmarkByUrl(url: string): BookmarkItem | undefined {
    if (!url || typeof url !== 'string') return undefined;
    return bookmarks.value.find(b => b && b.url === url);
  }
  
  function searchBookmarks(query: string): BookmarkItem[] {
    if (!query || typeof query !== 'string') return bookmarks.value.filter(b => b && b.id && b.url);
    const lowerQuery = query.toLowerCase();
    return bookmarks.value.filter(b => 
      b && 
      (b.title?.toLowerCase().includes(lowerQuery) || 
       b.url?.toLowerCase().includes(lowerQuery))
    );
  }
  
  function addFolder(name: string) {
    if (!name || typeof name !== 'string') return;
    const trimmedName = name.trim();
    if (!trimmedName) return;
    if (!folders.value.includes(trimmedName)) {
      folders.value.push(trimmedName);
      try {
        localStorage.setItem('bookmark-folders', JSON.stringify(folders.value));
      } catch (e) {
        console.error('Failed to save folders:', e);
      }
    }
  }
  
  function removeFolder(name: string) {
    if (!name || typeof name !== 'string') return;
    const index = folders.value.indexOf(name);
    if (index > -1) {
      folders.value.splice(index, 1);
      // Move bookmarks from this folder to root
      bookmarks.value.forEach(b => {
        if (b && b.folder === name) {
          b.folder = undefined;
        }
      });
      try {
        localStorage.setItem('bookmark-folders', JSON.stringify(folders.value));
        localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks.value));
      } catch (e) {
        console.error('Failed to save:', e);
      }
    }
  }
  
  function loadBookmarks() {
    try {
      const savedBookmarks = localStorage.getItem('browser-bookmarks');
      if (savedBookmarks) {
        const parsed = JSON.parse(savedBookmarks);
        // Validate parsed data is an array
        if (Array.isArray(parsed)) {
          bookmarks.value = parsed.filter((b: any) =>
            b && typeof b === 'object' && b.id && typeof b.id === 'string' && b.url && typeof b.url === 'string'
          );
        }
      } else {
        // Add default bookmarks if none exist
        for (const site of DEFAULT_BOOKMARKS) {
          addBookmark(site.title, site.url);
        }
      }

      const savedFolders = localStorage.getItem('bookmark-folders');
      if (savedFolders) {
        const parsed = JSON.parse(savedFolders);
        // Validate parsed data is an array
        if (Array.isArray(parsed)) {
          folders.value = parsed.filter((f: any) => typeof f === 'string' && f.trim()).map((f: string) => f.trim());
        }
      }
    } catch (e) {
      console.error('Failed to load bookmarks:', e);
      // Reset to defaults on error
      bookmarks.value = [];
      folders.value = ['Favorites', 'Work', 'News'];
    }
  }
  
  function clearBookmarks() {
    bookmarks.value = [];
    try {
      localStorage.removeItem('browser-bookmarks');
    } catch (e) {
      console.error('Failed to clear bookmarks:', e);
    }
  }
  
  function importBookmarks(importedBookmarks: BookmarkItem[]) {
    if (!Array.isArray(importedBookmarks)) return;
    const validBookmarks = importedBookmarks.filter(b => 
      b && b.id && typeof b.id === 'string' && b.url && typeof b.url === 'string'
    );
    bookmarks.value = [...bookmarks.value, ...validBookmarks];
    // Limit after import
    if (bookmarks.value.length > MAX_BOOKMARKS) {
      bookmarks.value = bookmarks.value.slice(0, MAX_BOOKMARKS);
    }
    try {
      localStorage.setItem('browser-bookmarks', JSON.stringify(bookmarks.value));
    } catch (e) {
      console.error('Failed to import bookmarks:', e);
    }
  }
  
  function exportBookmarks(): string {
    return JSON.stringify(bookmarks.value, null, 2);
  }
  
  return {
    bookmarks,
    folders,
    addBookmark,
    removeBookmark,
    updateBookmark,
    getBookmarks,
    getBookmarksInFolder,
    isBookmarked,
    getBookmarkByUrl,
    searchBookmarks,
    addFolder,
    removeFolder,
    loadBookmarks,
    clearBookmarks,
    importBookmarks,
    exportBookmarks,
  };
}
