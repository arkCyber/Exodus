/**
 * Tests for browser settings helpers.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  readShowBookmarkBar,
  SHOW_BOOKMARK_BAR_KEY,
  sidecarStateLabel,
  writeShowBookmarkBar,
} from './browserSettings';

describe('sidecarStateLabel', () => {
  it('maps known sidecar states', () => {
    expect(sidecarStateLabel('disabled')).toBe('Disabled');
    expect(sidecarStateLabel('not_found')).toBe('Binary not found');
    expect(sidecarStateLabel('spawn_failed')).toBe('Failed to start');
    expect(sidecarStateLabel('running')).toBe('Running');
    expect(sidecarStateLabel('exited')).toBe('Exited');
  });

  it('returns Unknown for unrecognized states', () => {
    expect(sidecarStateLabel('')).toBe('Unknown');
    expect(sidecarStateLabel('custom')).toBe('Unknown');
  });
});

describe('bookmark bar persistence', () => {
  const store = new Map<string, string>();

  beforeEach(() => {
    store.clear();
    vi.stubGlobal('localStorage', {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => {
        store.set(k, v);
      },
      removeItem: (k: string) => {
        store.delete(k);
      },
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('defaults to visible', () => {
    expect(readShowBookmarkBar()).toBe(true);
  });

  it('persists hidden state', () => {
    writeShowBookmarkBar(false);
    expect(readShowBookmarkBar()).toBe(false);
    writeShowBookmarkBar(true);
  });
});
