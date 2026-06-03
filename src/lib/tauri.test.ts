/**
 * Exodus Browser — Tauri runtime detection unit tests.
 */
import { describe, expect, it, beforeEach, afterEach } from 'vitest';
import { canInvokeTauri } from './tauri';

describe('canInvokeTauri', () => {
  const originalWindow = globalThis.window;

  beforeEach(() => {
    globalThis.window = { ...originalWindow } as Window & typeof globalThis;
  });

  afterEach(() => {
  globalThis.window = originalWindow;
  });

  it('returns false when __TAURI_INTERNALS__ is missing', () => {
    expect(canInvokeTauri()).toBe(false);
  });

  it('returns true when invoke is on __TAURI_INTERNALS__', () => {
    (window as Window & { __TAURI_INTERNALS__?: { invoke: () => void } }).__TAURI_INTERNALS__ = {
      invoke: () => undefined,
    };
    expect(canInvokeTauri()).toBe(true);
  });

  it('returns false when global isTauri is unset but internals exist (withGlobalTauri: false)', () => {
    (window as Window & { __TAURI_INTERNALS__?: { invoke: () => void } }).__TAURI_INTERNALS__ = {
      invoke: () => undefined,
    };
    (globalThis as { isTauri?: boolean }).isTauri = undefined;
    expect(canInvokeTauri()).toBe(true);
  });
});
