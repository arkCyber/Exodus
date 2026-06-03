/**
 * Unit tests for Firefox-style preset bookmark seeding.
 */

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import {
  DEFAULT_QUICK_LINKS,
  PRESET_BOOKMARK_SITES,
  PRESET_BOOKMARK_STORAGE_KEY,
  seedPresetBookmarksIfEmpty,
} from './presetBookmarks';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('PRESET_BOOKMARK_SITES', () => {
  it('defines six toolbar-style defaults', () => {
    expect(PRESET_BOOKMARK_SITES).toHaveLength(6);
    expect(PRESET_BOOKMARK_SITES[0].url).toContain('duckduckgo');
  });

  it('exposes four sites as new-tab quick links', () => {
    expect(DEFAULT_QUICK_LINKS).toHaveLength(4);
    expect(DEFAULT_QUICK_LINKS.map((l) => l.url)).toEqual(
      PRESET_BOOKMARK_SITES.slice(0, 4).map((l) => l.url),
    );
  });
});

function mockLocalStorage(): Storage {
  const store = new Map<string, string>();
  return {
    get length() {
      return store.size;
    },
    clear: () => store.clear(),
    getItem: (key: string) => store.get(key) ?? null,
    key: (index: number) => [...store.keys()][index] ?? null,
    removeItem: (key: string) => {
      store.delete(key);
    },
    setItem: (key: string, value: string) => {
      store.set(key, value);
    },
  };
}

describe('seedPresetBookmarksIfEmpty', () => {
  beforeEach(() => {
    vi.stubGlobal('localStorage', mockLocalStorage());
    vi.mocked(invoke).mockReset();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('skips when already seeded flag is set', async () => {
    localStorage.setItem(PRESET_BOOKMARK_STORAGE_KEY, '1');
    const result = await seedPresetBookmarksIfEmpty();
    expect(result).toBe(false);
    expect(invoke).not.toHaveBeenCalled();
  });

  it('skips when bookmarks already exist', async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      { id: '1', url: 'https://example.com', title: 'Ex', created_at: '' },
    ]);
    const result = await seedPresetBookmarksIfEmpty();
    expect(result).toBe(false);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem(PRESET_BOOKMARK_STORAGE_KEY)).toBe('1');
  });

  it('adds preset bookmarks when store is empty', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([])
      .mockResolvedValue({});
    const result = await seedPresetBookmarksIfEmpty();
    expect(result).toBe(true);
    expect(invoke).toHaveBeenCalledTimes(1 + PRESET_BOOKMARK_SITES.length);
    expect(localStorage.getItem(PRESET_BOOKMARK_STORAGE_KEY)).toBe('1');
  });
});
