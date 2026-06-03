/**
 * Unit tests for new-tab wallpaper library.
 */

import { describe, expect, it } from 'vitest';
import {
  defaultWallpaperId,
  getWallpaperById,
  listWallpapers,
  resolveWallpaperBackgroundUrl,
  WALLPAPER_LIBRARY_PATH,
} from './newTabWallpaper';

describe('newTabWallpaper', () => {
  it('exposes wallpaper library path', () => {
    expect(WALLPAPER_LIBRARY_PATH).toBe('/newtab/wallpapers');
  });

  it('lists bundled wallpapers from manifest', () => {
    const list = listWallpapers();
    expect(list.length).toBeGreaterThanOrEqual(6);
    expect(list.some((w) => w.id === 'aurora')).toBe(true);
  });

  it('resolves asset URL for wallpaper file', () => {
    const url = resolveWallpaperBackgroundUrl('ocean');
    expect(url).toContain('newtab/wallpapers/ocean.svg');
  });

  it('falls back to default for unknown id', () => {
    const wp = getWallpaperById('does-not-exist');
    expect(wp.id).toBe(defaultWallpaperId());
  });

  it('uses nebula as bundled default wallpaper', () => {
    expect(defaultWallpaperId()).toBe('nebula');
  });
});
