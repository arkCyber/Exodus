/**
 * Exodus Browser — Tauri runtime detection helpers.
 *
 * With `withGlobalTauri: false`, `globalThis.isTauri` may be unset even when
 * `window.__TAURI_INTERNALS__.invoke` is available — detect IPC via internals.
 */

export { isTauri } from '@tauri-apps/api/core';

/**
 * True when the Tauri IPC bridge is available (safe to call `invoke`).
 */
export function canInvokeTauri(): boolean {
  if (typeof window === 'undefined') return false;
  const w = window as Window & {
    __TAURI_INTERNALS__?: { invoke?: (...args: unknown[]) => unknown };
  };
  return typeof w.__TAURI_INTERNALS__?.invoke === 'function';
}
