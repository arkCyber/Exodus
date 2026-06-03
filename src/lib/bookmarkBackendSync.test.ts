/**
 * bookmarkBackendSync — Tauri bookmark store mirror helpers.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  mirrorBookmarksToLocalStorage,
  fetchBookmarksFromBackend,
  syncBookmarksFromBackendIfTauri,
} from './bookmarkBackendSync';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe('bookmarkBackendSync', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('mirrors bookmarks into localStorage', () => {
    mirrorBookmarksToLocalStorage([
      { id: '1', title: 'Example', url: 'https://example.com', created_at: '' },
    ]);
    expect(JSON.parse(localStorage.getItem('browser-bookmarks') || '[]')).toHaveLength(1);
  });

  it('fetches bookmarks from backend invoke', async () => {
    invokeMock.mockResolvedValue([{ id: '1', title: 'A', url: 'https://a.com', created_at: '' }]);
    const list = await fetchBookmarksFromBackend();
    expect(invokeMock).toHaveBeenCalledWith('list_bookmarks');
    expect(list).toHaveLength(1);
  });

  it('syncs backend bookmarks into localStorage', async () => {
    invokeMock.mockResolvedValue([{ id: '1', title: 'A', url: 'https://a.com', created_at: '' }]);
    const ok = await syncBookmarksFromBackendIfTauri();
    expect(ok).toBe(true);
    expect(JSON.parse(localStorage.getItem('browser-bookmarks') || '[]')).toHaveLength(1);
  });
});
