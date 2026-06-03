/**
 * Exodus Browser — built-in new tab page (data URL underlay + Svelte overlay UI).
 */

import type { QuickLink } from '$lib/browserTypes';
import { DEFAULT_QUICK_LINKS } from '$lib/presetBookmarks';
import {
  loadWallpaperId,
  resolveWallpaperBackgroundUrl,
  resolveWallpaperDataUrl,
} from '$lib/newTabWallpaper';

export { DEFAULT_QUICK_LINKS };

/** Marker inside HTML so `isNewTabUrl` recognizes regenerated pages. */
export const NEWTAB_PAGE_MARKER = 'Exodus-New-Tab-Page';

/** @deprecated Use wallpaper library via `resolveWallpaperDataUrl`. */
export const NEWTAB_WALLPAPER_DATA_URL = resolveWallpaperBackgroundUrl();

export type NewTabPageOptions = {
  wallpaperId?: string;
  wallpaperDataUrl?: string;
  aiOnline?: boolean;
  aiModel?: string;
};

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/**
 * Minimal HTML underlay (wallpaper only). Main UI is the Svelte `NewTabPage` overlay.
 */
export function buildNewTabHtml(options: NewTabPageOptions = {}): string {
  const wallpaperId = options.wallpaperId ?? loadWallpaperId();
  const wallpaper =
    options.wallpaperDataUrl ??
    resolveWallpaperDataUrl(wallpaperId) ??
    resolveWallpaperBackgroundUrl(wallpaperId);

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <meta name="description" content="${NEWTAB_PAGE_MARKER}" />
  <title>Exodus — New Tab</title>
  <style>
    html, body { margin: 0; height: 100%; background: #0a0a0f; }
    body {
      background-image:
        linear-gradient(180deg, rgba(6,8,14,0.2) 0%, rgba(6,6,10,0.45) 100%),
        url('${wallpaper}');
      background-size: cover;
      background-position: center;
      background-attachment: fixed;
    }
  </style>
</head>
<body></body>
</html>`;
}

/** Build data URL for WebView navigation. */
export function buildNewTabDataUrl(options: NewTabPageOptions = {}): string {
  return 'data:text/html;charset=utf-8,' + encodeURIComponent(buildNewTabHtml(options));
}

/** Legacy constant (updated on `initNewTabPage`). */
export let NEWTAB_PAGE_URL = buildNewTabDataUrl();

let cachedUrl = NEWTAB_PAGE_URL;

/** Current new-tab URL (may include live AI status). */
export function getNewTabPageUrl(): string {
  return cachedUrl;
}

/**
 * Refresh new-tab HTML (wallpaper + options). Returns the data URL.
 */
export function initNewTabPage(options: NewTabPageOptions = {}): string {
  const wallpaperId = options.wallpaperId ?? loadWallpaperId();
  cachedUrl = buildNewTabDataUrl({ ...options, wallpaperId });
  NEWTAB_PAGE_URL = cachedUrl;
  return cachedUrl;
}

/** Whether a URL represents the internal new-tab page. */
export function isNewTabUrl(url: string): boolean {
  if (!url) return false;
  if (url.startsWith('data:text/html') && url.includes(NEWTAB_PAGE_MARKER)) return true;
  if (url === cachedUrl || url === NEWTAB_PAGE_URL) return true;
  return url.startsWith('data:text/html') && url.includes('Exodus');
}
