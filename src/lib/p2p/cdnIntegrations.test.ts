/**
 * Exodus Browser — P2P CDN integration helper tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  CDN_AUTO_ANNOUNCE_MIN_BYTES,
  announcePageUrlToCdn,
  cdnKindFromName,
  extractAnnounceableUrls,
  isLikelyLargeAssetUrl,
  maybeAnnounceIndexedPage,
  suggestUrlsForCdnAnnounce,
} from './cdnIntegrations';

describe('cdnIntegrations', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('isLikelyLargeAssetUrl detects model files', () => {
    expect(isLikelyLargeAssetUrl('https://x.com/m.gguf')).toBe(true);
    expect(isLikelyLargeAssetUrl('https://x.com/page.html')).toBe(false);
  });

  it('suggestUrlsForCdnAnnounce filters large assets', () => {
    const urls = suggestUrlsForCdnAnnounce('a https://x.com/m.gguf b https://x.com/p.html');
    expect(urls).toEqual(['https://x.com/m.gguf']);
  });

  it('extractAnnounceableUrls finds links', () => {
    const urls = extractAnnounceableUrls('See https://example.com/a and http://x.org/b');
    expect(urls).toHaveLength(2);
  });

  it('announcePageUrlToCdn invokes announce_url_hot', async () => {
    await announcePageUrlToCdn('https://example.com/m', 'Model', 'lobby');
    expect(invokeMock).toHaveBeenCalledWith(
      'p2p_cdn_announce_url_hot',
      expect.objectContaining({ url: 'https://example.com/m', roomId: 'lobby' }),
    );
  });

  it('cdnKindFromName detects ai models', () => {
    expect(cdnKindFromName('model.gguf')).toBe('ai_model');
    expect(cdnKindFromName('clip.mp4')).toBe('video_model');
  });

  it('maybeAnnounceIndexedPage skips small pages', async () => {
    const ok = await maybeAnnounceIndexedPage('https://a.com', 'T', 1000);
    expect(ok).toBe(false);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('maybeAnnounceIndexedPage announces large pages', async () => {
    const ok = await maybeAnnounceIndexedPage(
      'https://a.com/big',
      'Big',
      CDN_AUTO_ANNOUNCE_MIN_BYTES + 1,
    );
    expect(ok).toBe(true);
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_join_room', { roomId: 'lobby' });
    expect(invokeMock).toHaveBeenCalledWith(
      'p2p_cdn_announce_url_hot',
      expect.objectContaining({ url: 'https://a.com/big' }),
    );
  });
});
