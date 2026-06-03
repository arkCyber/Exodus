/**
 * Unit tests for frontend startup logging helpers.
 */

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { logStartup, logStartupError } from './startupLog';

describe('startupLog', () => {
  beforeEach(() => {
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('logStartup writes prefixed message', () => {
    logStartup('test step');
    expect(console.log).toHaveBeenCalled();
    const arg = String(vi.mocked(console.log).mock.calls[0][0]);
    expect(arg).toContain('[Exodus]');
    expect(arg).toContain('test step');
  });

  it('logStartupError writes to console.error', () => {
    logStartupError('failed', new Error('x'));
    expect(console.error).toHaveBeenCalled();
  });
});
