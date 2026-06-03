/**
 * Exodus Browser — status notifier unit tests.
 */

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import {
  STATUS_CLEAR_MS,
  createStatusNotifier,
  formatStatusError,
} from './statusMessage';

describe('createStatusNotifier', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('sets message and clears after default delay', () => {
    let current = '';
    const { show, dispose } = createStatusNotifier((m) => (current = m));
    show('Saved');
    expect(current).toBe('Saved');
    vi.advanceTimersByTime(STATUS_CLEAR_MS);
    expect(current).toBe('');
    dispose();
  });

  it('persist skips auto-clear', () => {
    let current = '';
    const { show, dispose } = createStatusNotifier((m) => (current = m));
    show('Working…', { persist: true });
    vi.advanceTimersByTime(STATUS_CLEAR_MS * 2);
    expect(current).toBe('Working…');
    dispose();
  });

  it('replaces pending clear when showing new message', () => {
    let current = '';
    const { show, dispose } = createStatusNotifier((m) => (current = m));
    show('First');
    vi.advanceTimersByTime(1000);
    show('Second');
    vi.advanceTimersByTime(STATUS_CLEAR_MS - 1000);
    expect(current).toBe('Second');
    dispose();
  });
});

describe('formatStatusError', () => {
  it('includes Error message', () => {
    expect(formatStatusError(new Error('timeout'), 'Download failed')).toBe(
      'Download failed: timeout',
    );
  });
});
