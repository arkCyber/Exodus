/**
 * Exodus Browser — address-bar P2P CDN status for the active page URL.
 */

import { p2pCdnUrlStatus, type CdnUrlStatus } from './cdn';

export type { CdnUrlStatus };

/** Human label for the omnibox CDN badge. */
export function cdnUrlStatusLabel(status: CdnUrlStatus | null): string | null {
  if (!status) return null;
  if (status.localComplete) return 'P2P · cached';
  if (status.peerCount > 0) return `P2P · ${status.peerCount}`;
  if (status.announced) return 'P2P · listed';
  return null;
}

/** Fetch CDN status for a page URL (debounced by caller). */
export async function fetchCdnPageStatus(
  url: string,
  roomId: string,
): Promise<CdnUrlStatus | null> {
  if (!url.startsWith('http://') && !url.startsWith('https://')) return null;
  try {
    return await p2pCdnUrlStatus(roomId, url);
  } catch (error) {
    console.error('p2p_cdn_url_status failed:', error);
    return null;
  }
}
