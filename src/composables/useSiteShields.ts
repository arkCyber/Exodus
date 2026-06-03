/**
 * Exodus Browser — per-site tracker shields (Brave-style) + global tracking protection.
 */
import { ref } from 'vue';
import { loadTrackingProtectionSettings } from '@/lib/browserIntegrations';
import {
  getSiteShieldAllowTrackers,
  hostFromPageUrl,
  setSiteShieldAllowTrackers,
} from '@/lib/siteShields';
import { isNewTabUrl } from '@/lib/newTabPage';

export type UseSiteShieldsOptions = {
  onStatus: (message: string) => void;
  reloadActiveTab?: () => Promise<void>;
};

/**
 * Reactive shields state for the address bar and privacy settings.
 */
export function useSiteShields(options: UseSiteShieldsOptions) {
  const trackingProtectionEnabled = ref(true);
  const siteAllowTrackers = ref(false);

  const shieldsEnabled = () => trackingProtectionEnabled.value && !siteAllowTrackers.value;

  async function loadTrackingProtection(): Promise<void> {
    try {
      const tp = await loadTrackingProtectionSettings();
      trackingProtectionEnabled.value = tp?.enabled ?? true;
    } catch (error) {
      console.error('loadTrackingProtectionSettings failed:', error);
      trackingProtectionEnabled.value = true;
    }
  }

  /** Refresh per-site override for the current page URL. */
  async function refreshSiteShieldForUrl(url: string): Promise<void> {
    const host = hostFromPageUrl(url);
    if (!host || isNewTabUrl(url)) {
      siteAllowTrackers.value = false;
      return;
    }
    siteAllowTrackers.value = await getSiteShieldAllowTrackers(host);
  }

  /** Toggle allow/block trackers for the current site hostname. */
  async function toggleSiteShieldAllowTrackers(url: string): Promise<void> {
    const host = hostFromPageUrl(url);
    if (!host) return;
    const next = !siteAllowTrackers.value;
    try {
      await setSiteShieldAllowTrackers(host, next);
      siteAllowTrackers.value = next;
      options.onStatus(next ? `Shields down for ${host}` : `Shields up for ${host}`);
      await options.reloadActiveTab?.();
    } catch (error) {
      console.error('set_site_shield_override failed:', error);
      options.onStatus('Failed to update site shields');
    }
  }

  return {
    trackingProtectionEnabled,
    siteAllowTrackers,
    shieldsEnabled,
    loadTrackingProtection,
    refreshSiteShieldForUrl,
    toggleSiteShieldAllowTrackers,
  };
}
