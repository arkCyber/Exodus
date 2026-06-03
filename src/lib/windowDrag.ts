/**
 * Exodus Browser — Tauri window drag from chrome blank areas (toolbar / tab strip).
 * Requires `core:window:allow-start-dragging` in Tauri capabilities.
 */

import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

/** Selectors for elements that must not initiate window drag. */
const DRAG_BLOCK_SELECTOR =
  'button,a,input,textarea,select,option,label,form,[data-no-drag],[contenteditable="true"],.no-window-drag,.tab-item,.tab-new,.tab-close';

/**
 * Whether the event target is an interactive control (clicks should not move the window).
 */
export function isWindowDragBlockedTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return true;
  return !!target.closest(DRAG_BLOCK_SELECTOR);
}

/**
 * Start native window drag on primary-button mousedown in title bar blank space.
 * Must run during mousedown (Tauri macOS requirement); do not await before returning.
 */
export function startWindowDragFromMouseDown(e: MouseEvent): void {
  if (e.button !== 0) return;
  if (e.defaultPrevented) return;
  if (isWindowDragBlockedTarget(e.target)) return;
  if (!isTauri()) return;
  void getCurrentWindow()
    .startDragging()
    .catch((error) => {
      console.error('[windowDrag] startDragging failed — add core:window:allow-start-dragging:', error);
    });
}

/** @deprecated use startWindowDragFromMouseDown (sync) */
export async function startWindowDragFromMouseDownAsync(e: MouseEvent): Promise<void> {
  startWindowDragFromMouseDown(e);
}
