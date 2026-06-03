/**
 * Exodus Browser — Brave-style new tab wallpaper library (static + app_data custom).
 */

import { invoke } from '@tauri-apps/api/core';
import manifest from '../../static/newtab/wallpapers/manifest.json';

/** Wallpaper catalog entry from manifest.json. */
export type WallpaperEntry = {
  id: string;
  name: string;
  file: string;
  accent: string;
  description?: string;
  custom?: boolean;
};

export type WallpaperManifest = {
  version: number;
  defaultId: string;
  wallpapers: WallpaperEntry[];
};

export const WALLPAPER_STORAGE_KEY = 'exodus-ntp-wallpaper-id';

const bundledCatalog = manifest as WallpaperManifest;

/** In-memory catalog (bundled + optional custom from backend). */
let runtimeCatalog: WallpaperEntry[] = [...bundledCatalog.wallpapers];

/** data: URLs for WebView underlay (data: HTML cannot load relative /http assets). */
const wallpaperDataUrlCache = new Map<string, string>();

/** Bundled wallpaper library directory (Vite static). */
export const WALLPAPER_LIBRARY_PATH = '/newtab/wallpapers';

/** Resolve fetch base for wallpaper SVG assets. */
export function wallpaperFetchBase(): string {
  if (typeof window !== 'undefined' && window.location?.origin) {
    return window.location.origin.replace(/\/?$/, '');
  }
  return '';
}

/** All wallpapers (refresh with `loadWallpaperCatalog` for custom entries). */
export function listWallpapers(): WallpaperEntry[] {
  return [...runtimeCatalog];
}

/** Default wallpaper id. */
export function defaultWallpaperId(): string {
  return bundledCatalog.defaultId;
}

/** Lookup wallpaper by id. */
export function getWallpaperById(id: string | null | undefined): WallpaperEntry {
  const found = runtimeCatalog.find((w) => w.id === id);
  if (found) return found;
  return (
    runtimeCatalog.find((w) => w.id === bundledCatalog.defaultId) ?? runtimeCatalog[0]
  );
}

/**
 * Public URL for overlay CSS (Svelte UI — same origin as the app).
 */
export function wallpaperAssetUrl(file: string): string {
  const base = (import.meta.env.BASE_URL ?? '/').replace(/\/?$/, '/');
  return `${base}newtab/wallpapers/${file}`;
}

/**
 * Absolute asset URL for fetch preload (dev + production).
 */
export function wallpaperAbsoluteAssetUrl(file: string): string {
  const origin = wallpaperFetchBase();
  const path = wallpaperAssetUrl(file);
  if (origin) return `${origin}${path.startsWith('/') ? path : `/${path}`}`;
  return path;
}

/**
 * CSS background URL for overlay (custom wallpapers may use file:// via backend path).
 */
export function resolveWallpaperBackgroundUrl(id?: string | null): string {
  const entry = getWallpaperById(id);
  const cached = wallpaperDataUrlCache.get(entry.id);
  if (cached) return cached;
  if (entry.custom) {
    return customWallpaperFileUrl(entry.file);
  }
  return wallpaperAssetUrl(entry.file);
}

/** Cached data URL for native WebView data: HTML underlay. */
export function resolveWallpaperDataUrl(id?: string | null): string {
  const entry = getWallpaperById(id);
  return wallpaperDataUrlCache.get(entry.id) ?? resolveWallpaperBackgroundUrl(id);
}

let customLibraryBase: string | null = null;

function customWallpaperFileUrl(file: string): string {
  if (!customLibraryBase) return wallpaperAssetUrl(file);
  const base = customLibraryBase.replace(/\/$/, '');
  return `file://${base}/${file}`;
}

/**
 * Merge Rust-backed custom wallpapers from app_data library folder.
 */
export async function loadWallpaperCatalog(): Promise<WallpaperEntry[]> {
  try {
    const merged = await invoke<WallpaperEntry[]>('ntp_list_wallpaper_catalog');
    if (merged.length > 0) runtimeCatalog = merged;
    customLibraryBase = await invoke<string>('ntp_get_wallpaper_library_path');
  } catch (error) {
    console.error('loadWallpaperCatalog failed:', error);
    runtimeCatalog = [...bundledCatalog.wallpapers];
  }
  return listWallpapers();
}

/**
 * Preload SVG wallpapers as data: URLs for WebView new-tab underlay.
 */
export async function preloadWallpaperDataUrls(): Promise<void> {
  await loadWallpaperCatalog();
  for (const wp of runtimeCatalog) {
    if (wallpaperDataUrlCache.has(wp.id)) continue;
    try {
      if (wp.custom) {
        const dataUrl = await invoke<string>('ntp_wallpaper_file_data_url', { file: wp.file });
        wallpaperDataUrlCache.set(wp.id, dataUrl);
        continue;
      }
      const url = wallpaperAbsoluteAssetUrl(wp.file);
      const res = await fetch(url);
      if (!res.ok) continue;
      const body = await res.text();
      const dataUrl = wp.file.endsWith('.svg')
        ? `data:image/svg+xml,${encodeURIComponent(body)}`
        : URL.createObjectURL(await res.blob());
      wallpaperDataUrlCache.set(wp.id, dataUrl);
    } catch (error) {
      console.error('preloadWallpaperDataUrls failed for', wp.id, error);
    }
  }
}

/** Load persisted wallpaper id. */
export function loadWallpaperId(): string {
  try {
    const saved = localStorage.getItem(WALLPAPER_STORAGE_KEY);
    if (saved && runtimeCatalog.some((w) => w.id === saved)) return saved;
  } catch {
    /* ignore */
  }
  return defaultWallpaperId();
}

/** Persist wallpaper selection to localStorage. */
export function saveWallpaperId(id: string): void {
  try {
    localStorage.setItem(WALLPAPER_STORAGE_KEY, id);
  } catch (error) {
    console.error('saveWallpaperId failed:', error);
  }
}

/** Persist wallpaper id to Rust `new_tab_settings.json` (survives localStorage clears). */
export async function persistWallpaperIdToBackend(id: string): Promise<void> {
  try {
    await invoke('set_new_tab_wallpaper_id', { id });
  } catch (error) {
    console.error('persistWallpaperIdToBackend failed:', error);
  }
}

/** Save wallpaper locally and on disk via Tauri. */
export async function saveWallpaperIdAndSync(id: string): Promise<void> {
  saveWallpaperId(id);
  await persistWallpaperIdToBackend(id);
}

/**
 * Resolve startup wallpaper: localStorage → Rust settings → manifest default.
 * Call after `loadWallpaperCatalog()` / `preloadWallpaperDataUrls()`.
 */
export async function syncNtpWallpaperOnStartup(): Promise<string> {
  await loadWallpaperCatalog();

  let id: string | null = null;
  try {
    const saved = localStorage.getItem(WALLPAPER_STORAGE_KEY);
    if (saved && runtimeCatalog.some((w) => w.id === saved)) id = saved;
  } catch {
    /* ignore */
  }

  if (!id) {
    try {
      const settings = await invoke<{ wallpaper_id?: string }>('get_new_tab_settings');
      const backendId = settings?.wallpaper_id;
      if (backendId && runtimeCatalog.some((w) => w.id === backendId)) id = backendId;
    } catch (error) {
      console.error('syncNtpWallpaperOnStartup get_new_tab_settings failed:', error);
    }
  }

  if (!id) id = defaultWallpaperId();
  saveWallpaperId(id);
  void persistWallpaperIdToBackend(id);
  return id;
}

/** Reset wallpaper to the bundled default (`manifest.json` → `defaultId`). */
export async function resetWallpaperToDefault(): Promise<string> {
  const id = defaultWallpaperId();
  await saveWallpaperIdAndSync(id);
  return id;
}

/** Random wallpaper id. */
export function randomWallpaperId(): string {
  const list = listWallpapers();
  if (!list.length) return defaultWallpaperId();
  return list[Math.floor(Math.random() * list.length)].id;
}

/** User wallpaper library path on disk (app_data). */
export async function getWallpaperLibraryPath(): Promise<string> {
  try {
    return await invoke<string>('ntp_get_wallpaper_library_path');
  } catch (error) {
    console.error('getWallpaperLibraryPath failed:', error);
    return '';
  }
}
