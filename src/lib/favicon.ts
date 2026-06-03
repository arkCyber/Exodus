/**
 * Exodus Browser — favicon URL helpers for tabs and bookmark bar.
 */

import { isNewTabUrl } from '$lib/newTabPage';

/**
 * Build a favicon URL for a page (Google s2 service by domain).
 * Returns null for new-tab data URLs and invalid inputs.
 */
export function faviconUrlFor(pageUrl: string): string | null {
  if (!pageUrl || isNewTabUrl(pageUrl) || pageUrl.startsWith('data:')) {
    return null;
  }
  try {
    const href = pageUrl.startsWith('http://') || pageUrl.startsWith('https://')
      ? pageUrl
      : `https://${pageUrl}`;
    const host = new URL(href).hostname;
    if (!host) return null;
    return `https://www.google.com/s2/favicons?domain=${encodeURIComponent(host)}&sz=32`;
  } catch {
    return null;
  }
}

/**
 * Whether the address bar should show a secure (HTTPS) indicator.
 */
export function isSecureUrl(pageUrl: string): boolean {
  try {
    const href =
      pageUrl.startsWith('http://') || pageUrl.startsWith('https://')
        ? pageUrl
        : `https://${pageUrl}`;
    return new URL(href).protocol === 'https:';
  } catch {
    return false;
  }
}
