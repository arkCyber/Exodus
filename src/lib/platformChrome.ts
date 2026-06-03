/**
 * Exodus Browser — platform-specific chrome layout (macOS overlay title bar inset).
 */

import { isTauri } from '@tauri-apps/api/core';

/** Left inset so tabs sit to the right of macOS traffic-light buttons (px). */
export const MAC_TITLEBAR_INSET_PX = 80;

/**
 * Whether the shell runs in Tauri on macOS with overlay title bar (traffic lights).
 */
export function isMacTauriOverlayTitlebar(): boolean {
  if (typeof navigator === 'undefined') return false;
  try {
    return isTauri() && /Mac|Macintosh|macOS/i.test(navigator.userAgent);
  } catch {
    return false;
  }
}

/**
 * Apply document classes for platform chrome (call once at app bootstrap).
 */
export function applyPlatformChromeClasses(): void {
  if (typeof document === 'undefined') return;
  if (isMacTauriOverlayTitlebar()) {
    document.documentElement.classList.add('exodus-macos-overlay-titlebar');
  }
}
