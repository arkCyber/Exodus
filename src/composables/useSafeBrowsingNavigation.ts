/**
 * Exodus Browser — Safe Browsing gate before loading http(s) URLs.
 */
import { ref } from 'vue';
import {
  checkNavigationGuard,
  recordMaliciousSiteBlocked,
} from '@/lib/browserIntegrations';
import { isNewTabUrl } from '@/lib/newTabPage';

export type SafeBrowsingOffer = {
  url: string;
  reason: string;
};

export type UseSafeBrowsingNavigationOptions = {
  onStatus: (message: string) => void;
};

/**
 * Pending navigation offer when Safe Browsing allows proceed-after-warning.
 */
export function useSafeBrowsingNavigation(options: UseSafeBrowsingNavigationOptions) {
  const safeBrowsingOffer = ref<SafeBrowsingOffer | null>(null);

  /**
   * Returns true when navigation may continue immediately; false when blocked or awaiting user choice.
   */
  async function ensureNavigationAllowed(url: string): Promise<boolean> {
    if (isNewTabUrl(url) || url.startsWith('extension://') || url.startsWith('data:')) {
      return true;
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      return true;
    }
    const guard = await checkNavigationGuard(url);
    if (guard.allowed) {
      return true;
    }
    void recordMaliciousSiteBlocked(url);
    if (guard.canProceed) {
      safeBrowsingOffer.value = { url, reason: guard.reason };
      return false;
    }
    options.onStatus(guard.reason);
    return false;
  }

  function cancelSafeBrowsing(): void {
    safeBrowsingOffer.value = null;
  }

  /** URL from the pending offer, if any (caller should clear offer after use). */
  function takePendingUrl(): string | null {
    const pending = safeBrowsingOffer.value?.url ?? null;
    safeBrowsingOffer.value = null;
    return pending;
  }

  return {
    safeBrowsingOffer,
    ensureNavigationAllowed,
    cancelSafeBrowsing,
    takePendingUrl,
  };
}
