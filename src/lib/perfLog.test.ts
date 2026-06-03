/**
 * Unit tests for perfLog helpers.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { logPerf, perfStart, perfEnd, perfSync } from './perfLog';

describe('perfLog', () => {
  beforeEach(() => {
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
  });

  it('logPerf writes to console', () => {
    logPerf('test_step', { ok: true });
    expect(console.log).toHaveBeenCalled();
  });

  it('perfEnd logs duration for started span', () => {
    perfStart('unit_span');
    perfEnd('unit_span');
    expect(console.log).toHaveBeenCalled();
  });

  it('perfSync returns fn result', () => {
    const n = perfSync('sync_span', () => 42);
    expect(n).toBe(42);
  });
});
