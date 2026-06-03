/**
 * Exodus Browser — CDN page status label tests.
 */
import { describe, expect, it } from 'vitest';
import { cdnUrlStatusLabel } from './cdnPageStatus';

describe('cdnPageStatus', () => {
  it('cdnUrlStatusLabel returns null when no status', () => {
    expect(cdnUrlStatusLabel(null)).toBeNull();
  });

  it('cdnUrlStatusLabel formats peer count', () => {
    expect(
      cdnUrlStatusLabel({
        url: 'u',
        discoveryHash: 'h',
        announced: true,
        peerCount: 2,
        localComplete: false,
      }),
    ).toBe('P2P · 2');
  });

  it('cdnUrlStatusLabel prefers cached', () => {
    expect(
      cdnUrlStatusLabel({
        url: 'u',
        discoveryHash: 'h',
        announced: false,
        peerCount: 0,
        localComplete: true,
      }),
    ).toBe('P2P · cached');
  });
});
