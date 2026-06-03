import { describe, it, expect, beforeEach } from 'vitest';
import { useBookmarks } from './useBookmarks';

describe('useBookmarks', () => {
  beforeEach(() => {
    localStorage.clear();
    useBookmarks().clearBookmarks();
  });

  describe('Input validation and error handling', () => {
    it('handles null/undefined URL gracefully', () => {
      const { addBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Test', null as any);
      addBookmark('Test', undefined as any);
      addBookmark('Test', '' as any);
      
      expect(bookmarks.value.length).toBe(0);
    });

    it('handles non-string URL gracefully', () => {
      const { addBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Test', 123 as any);
      addBookmark('Test', {} as any);
      addBookmark('Test', [] as any);
      
      expect(bookmarks.value.length).toBe(0);
    });

    it('rejects dangerous URL schemes', () => {
      const { addBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Data URL', 'data:text/plain,hello');
      addBookmark('JS URL', 'javascript:alert(1)');
      addBookmark('VBS URL', 'vbscript:msgbox(1)');
      
      expect(bookmarks.value.length).toBe(0);
    });

    it('handles invalid URL gracefully', () => {
      const { addBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Invalid', 'not-a-valid-url');
      addBookmark('Invalid', 'htp://invalid');
      
      // These URLs get https:// prefix added and become valid
      expect(bookmarks.value.length).toBe(2);
    });

    it('handles null/undefined title gracefully', () => {
      const { addBookmark, bookmarks } = useBookmarks();
      
      addBookmark(null as any, 'https://example.com');
      addBookmark(undefined as any, 'https://example2.com');
      addBookmark('' as any, 'https://example3.com');
      
      // Empty string is treated as valid (trimmed to empty, becomes 'Untitled')
      expect(bookmarks.value.length).toBe(3);
      expect(bookmarks.value.every(b => b.title === 'Untitled')).toBe(true);
    });

    it('handles null/undefined folder gracefully', () => {
      const { addBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Test', 'https://example.com', null as any);
      addBookmark('Test2', 'https://example2.com', undefined as any);
      
      expect(bookmarks.value.length).toBe(2);
      expect(bookmarks.value.every(b => !b.folder)).toBe(true);
    });

    it('handles removeBookmark with invalid id', () => {
      const { addBookmark, removeBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Test', 'https://example.com');
      const initialCount = bookmarks.value.length;
      
      removeBookmark(null as any);
      removeBookmark(undefined as any);
      removeBookmark('' as any);
      removeBookmark(123 as any);
      
      expect(bookmarks.value.length).toBe(initialCount);
    });

    it('handles updateBookmark with invalid parameters', () => {
      const { addBookmark, updateBookmark, bookmarks } = useBookmarks();
      
      addBookmark('Test', 'https://example.com');
      const bookmark = bookmarks.value[0];
      
      updateBookmark(null as any, { title: 'Updated' });
      updateBookmark(undefined as any, { title: 'Updated' });
      updateBookmark(bookmark.id, null as any);
      updateBookmark(bookmark.id, undefined as any);
      
      expect(bookmarks.value[0].title).toBe('Test');
    });

    it('handles searchBookmarks with invalid query', () => {
      const { addBookmark, searchBookmarks } = useBookmarks();
      
      addBookmark('Google', 'https://google.com');
      addBookmark('GitHub', 'https://github.com');
      
      const results1 = searchBookmarks(null as any);
      const results2 = searchBookmarks(undefined as any);
      const results3 = searchBookmarks('' as any);
      const results4 = searchBookmarks(123 as any);
      
      expect(results1.length).toBeGreaterThan(0);
      expect(results2.length).toBeGreaterThan(0);
      expect(results3.length).toBeGreaterThan(0);
      expect(results4.length).toBeGreaterThan(0);
    });

    it('handles addFolder with invalid name', () => {
      const { addFolder, folders } = useBookmarks();
      
      addFolder(null as any);
      addFolder(undefined as any);
      addFolder('' as any);
      addFolder(123 as any);
      addFolder('   ');
      
      expect(folders.value.length).toBe(3); // default folders
    });

    it('handles removeFolder with invalid name', () => {
      const { addFolder, removeFolder, folders } = useBookmarks();
      
      addFolder('Test');
      const initialCount = folders.value.length;
      
      removeFolder(null as any);
      removeFolder(undefined as any);
      removeFolder('' as any);
      removeFolder(123 as any);
      
      expect(folders.value.length).toBe(initialCount);
    });

    it('handles importBookmarks with invalid data', () => {
      const { importBookmarks, bookmarks } = useBookmarks();
      
      const initialCount = bookmarks.value.length;
      
      importBookmarks(null as any);
      importBookmarks(undefined as any);
      importBookmarks([] as any);
      importBookmarks(123 as any);
      importBookmarks('string' as any);
      importBookmarks([null, undefined, {}, { id: 123, url: 'https://test.com' }] as any);
      
      expect(bookmarks.value.length).toBe(initialCount);
    });

    it('handles corrupted localStorage data', () => {
      const { loadBookmarks, bookmarks } = useBookmarks();
      
      localStorage.setItem('browser-bookmarks', 'invalid json');
      localStorage.setItem('bookmark-folders', 'invalid json');
      
      loadBookmarks();
      
      expect(bookmarks.value.length).toBe(0);
    });

    it('handles localStorage with invalid bookmark objects', () => {
      const { loadBookmarks, bookmarks } = useBookmarks();
      
      localStorage.setItem('browser-bookmarks', JSON.stringify([
        null,
        undefined,
        {},
        { id: 123, url: 'https://test.com' },
        { id: 'valid', url: 123 },
        { id: 'valid2', url: 'https://test2.com' }
      ]));
      
      loadBookmarks();
      
      expect(bookmarks.value.length).toBe(1);
      expect(bookmarks.value[0].id).toBe('valid2');
    });
  });

  it('adds bookmark', () => {
    const { addBookmark, bookmarks } = useBookmarks();
    
    addBookmark('Example Page', 'https://example.com');
    
    expect(bookmarks.value.length).toBe(1);
    expect(bookmarks.value[0].title).toBe('Example Page');
    expect(bookmarks.value[0].url).toBe('https://example.com');
  });

  it('does not add data URLs as bookmarks', () => {
    const { addBookmark, bookmarks } = useBookmarks();
    
    addBookmark('Data URL', 'data:text/plain,hello');
    
    expect(bookmarks.value.length).toBe(0);
  });

  it('updates existing bookmark instead of duplicating', () => {
    const { addBookmark, bookmarks } = useBookmarks();
    
    addBookmark('Example Page', 'https://example.com');
    addBookmark('Updated Page', 'https://example.com');
    
    expect(bookmarks.value.length).toBe(1);
    expect(bookmarks.value[0].title).toBe('Updated Page');
  });

  it('removes bookmark', () => {
    const { addBookmark, removeBookmark, bookmarks } = useBookmarks();
    
    addBookmark('Example Page', 'https://example.com');
    const bookmark = bookmarks.value[0];
    removeBookmark(bookmark.id);
    
    expect(bookmarks.value.length).toBe(0);
  });

  it('checks if URL is bookmarked', () => {
    const { addBookmark, isBookmarked } = useBookmarks();
    
    addBookmark('Example Page', 'https://example.com');
    
    expect(isBookmarked('https://example.com')).toBe(true);
    expect(isBookmarked('https://other.com')).toBe(false);
  });

  it('searches bookmarks by title and URL', () => {
    const { addBookmark, searchBookmarks } = useBookmarks();
    
    addBookmark('Google', 'https://google.com');
    addBookmark('GitHub', 'https://github.com');
    addBookmark('Example', 'https://example.com');
    
    const results = searchBookmarks('git');
    
    expect(results.length).toBe(1);
    expect(results[0].title).toBe('GitHub');
  });

  it('adds folder', () => {
    const { addFolder, folders } = useBookmarks();
    
    addFolder('Test Folder');
    
    expect(folders.value).toContain('Test Folder');
  });

  it('does not add duplicate folder', () => {
    const { addFolder, folders } = useBookmarks();
    
    addFolder('Test Folder');
    addFolder('Test Folder');
    
    expect(folders.value.filter(f => f === 'Test Folder').length).toBe(1);
  });

  it('removes folder and moves bookmarks to root', () => {
    const { addBookmark, addFolder, removeFolder, getBookmarks } = useBookmarks();
    
    addFolder('Test Folder');
    addBookmark('Example', 'https://example.com', 'Test Folder');
    removeFolder('Test Folder');
    
    const bookmarks = getBookmarks();
    expect(bookmarks.length).toBe(1);
    expect(bookmarks[0].folder).toBeUndefined();
  });

  it('clears all bookmarks', () => {
    const { addBookmark, clearBookmarks, bookmarks } = useBookmarks();
    
    addBookmark('Example', 'https://example.com');
    clearBookmarks();
    
    expect(bookmarks.value.length).toBe(0);
  });

  it('exports bookmarks as JSON', () => {
    const { addBookmark, exportBookmarks } = useBookmarks();
    
    addBookmark('Example', 'https://example.com');
    const exported = exportBookmarks();
    
    const parsed = JSON.parse(exported);
    expect(parsed).toHaveLength(1);
    expect(parsed[0].title).toBe('Example');
  });

  it('limits bookmarks to MAX_BOOKMARKS', () => {
    const { addBookmark, bookmarks } = useBookmarks();
    
    for (let i = 0; i < 5100; i++) {
      addBookmark(`Bookmark ${i}`, `https://example${i}.com`);
    }
    
    expect(bookmarks.value.length).toBe(5000);
  });

  it('handles localStorage quota exceeded', () => {
    const { addBookmark, bookmarks } = useBookmarks();
    
    // Mock localStorage to throw QuotaExceededError
    const originalSetItem = localStorage.setItem;
    let callCount = 0;
    localStorage.setItem = function(key: string, value: string) {
      callCount++;
      if (callCount === 1) {
        throw new DOMException('Quota exceeded', 'QuotaExceededError');
      }
      return originalSetItem.call(this, key, value);
    };
    
    addBookmark('Test', 'https://example.com');
    
    // Should not crash
    expect(bookmarks.value.length).toBe(1);
    
    localStorage.setItem = originalSetItem;
  });
});
