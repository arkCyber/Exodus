/**
 * Exodus Browser — useCdnPageStatus unit tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { useCdnPageStatus } from './useCdnPageStatus';

vi.mock('$lib/p2p/cdnPageStatus', () => ({
  fetchCdnPageStatus: vi.fn(async () => ({
    announced: true,
    peerCount: 2,
    localComplete: false,
  })),
  cdnUrlStatusLabel: (s: { peerCount: number } | null) => (s ? `P2P · ${s.peerCount}` : null),
}));

describe('useCdnPageStatus', () => {
  it('refreshCdnPageStatus sets label', async () => {
    const cdn = useCdnPageStatus({
      getPageUrl: () => 'https://example.com',
      getRoomId: () => 'lobby',
    });
    await cdn.refreshCdnPageStatus();
    expect(cdn.cdnStatusLabel.value).toBe('P2P · 2');
  });
});
