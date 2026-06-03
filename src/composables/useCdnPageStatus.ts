/**
 * Exodus Browser — P2P CDN status badge for the active page URL in the omnibox.
 */
import { ref, computed } from 'vue';
import { cdnUrlStatusLabel, fetchCdnPageStatus, type CdnUrlStatus } from '@/lib/p2p/cdnPageStatus';

export type UseCdnPageStatusOptions = {
  getPageUrl: () => string;
  getRoomId: () => string;
};

/**
 * Reactive CDN page status for the address bar badge.
 */
export function useCdnPageStatus(options: UseCdnPageStatusOptions) {
  const cdnPageStatus = ref<CdnUrlStatus | null>(null);

  const cdnStatusLabel = computed(() => cdnUrlStatusLabel(cdnPageStatus.value));

  /** Refresh CDN status for the current page URL. */
  async function refreshCdnPageStatus(): Promise<void> {
    const url = options.getPageUrl();
    const roomId = options.getRoomId();
    cdnPageStatus.value = await fetchCdnPageStatus(url, roomId);
  }

  return {
    cdnPageStatus,
    cdnStatusLabel,
    refreshCdnPageStatus,
  };
}
