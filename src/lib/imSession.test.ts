/**
 * Exodus Browser — IM session helpers (unit tests).
 */
import { describe, expect, it } from 'vitest';
import { isLikelyPeerNodeId } from './imSession';

describe('isLikelyPeerNodeId', () => {
  it('rejects legacy local user id', () => {
    expect(isLikelyPeerNodeId('exodus-local-user')).toBe(false);
  });

  it('accepts long node-like ids', () => {
    expect(isLikelyPeerNodeId('abc123def456ghi789')).toBe(true);
  });
});
