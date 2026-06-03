/**
 * Exodus Browser — wire Rust browser services (suggestions, privacy, translate, reading mode).
 */

import { invoke } from '@tauri-apps/api/core';
import { evalTabReturning } from '$lib/exodusBrowser';

/** Omnibox row from `get_suggestions`. */
export type OmniboxSuggestion = {
  id: string;
  text: string;
  url: string;
  suggestion_type: string;
  relevance: number;
  visit_count: number;
  last_visited: number;
  favicon_url?: string | null;
};

/** Result from `translate_text`. */
export type TranslationResult = {
  original_text: string;
  translated_text: string;
  source_language: { code: string; name: string };
  target_language: { code: string; name: string };
  confidence: number;
};

/** Options for `clear_browsing_data`. */
export type ClearBrowsingDataOptions = {
  clearCache?: boolean;
  clearCookies?: boolean;
  clearLocalStorage?: boolean;
  clearHistory?: boolean;
};

/** Fetch Chrome-style omnibox suggestions (history, bookmarks, popular). */
export async function fetchOmniboxSuggestions(
  query: string,
  limit = 8,
): Promise<OmniboxSuggestion[]> {
  const q = query.trim();
  if (q.length < 1) return [];
  try {
    return await invoke<OmniboxSuggestion[]>('get_suggestions', { query: q, limit });
  } catch (error) {
    console.error('get_suggestions failed:', error);
    return [];
  }
}

/** Feed visit into the suggestions index. */
export async function syncSuggestionHistory(url: string, title: string): Promise<void> {
  try {
    await invoke('add_suggestion_history_entry', { url, title });
  } catch (error) {
    console.error('add_suggestion_history_entry failed:', error);
  }
}

/** Feed bookmark into the suggestions index. */
export async function syncSuggestionBookmark(url: string, title: string): Promise<void> {
  try {
    await invoke('add_suggestion_bookmark', { url, title });
  } catch (error) {
    console.error('add_suggestion_bookmark failed:', error);
  }
}

/** Clear selected browsing data categories. */
export async function clearBrowsingData(options: ClearBrowsingDataOptions): Promise<string> {
  return invoke<string>('clear_browsing_data', {
    clearCache: options.clearCache ?? false,
    clearCookies: options.clearCookies ?? false,
    clearLocalStorage: options.clearLocalStorage ?? false,
    clearHistory: options.clearHistory ?? false,
  });
}

/** Translate a text snippet (page excerpt or selection). */
export async function translateText(
  text: string,
  targetLang: string,
): Promise<TranslationResult> {
  return invoke<TranslationResult>('translate_text', { text, targetLang });
}

/** Reading-mode CSS from backend presets. */
export async function fetchReadingModeCss(): Promise<string> {
  return invoke<string>('generate_reading_mode_css');
}

/** Mark URL as reading-mode enabled in backend store. */
export async function enableReadingModeForUrl(url: string): Promise<void> {
  await invoke('enable_reading_mode', { url });
}

/** Mark URL as reading-mode disabled in backend store. */
export async function disableReadingModeForUrl(url: string): Promise<void> {
  await invoke('disable_reading_mode', { url });
}

/** Safe Browsing threat row from `check_url_safe`. */
export type SafeBrowsingThreat = {
  id: string;
  url_pattern: string;
  threat_type: string;
  severity: number;
  added_at: number;
  block_count: number;
};

/** Safe Browsing settings from backend. */
export type SafeBrowsingSettings = {
  enabled: boolean;
  block_malware: boolean;
  block_phishing: boolean;
  block_unwanted_software: boolean;
  show_warnings: boolean;
  allow_proceed: boolean;
  list_url?: string | null;
  last_list_refresh?: number;
};

/** Tracking protection settings from backend. */
export type TrackingProtectionSettings = {
  enabled: boolean;
  block_advertising: boolean;
  block_analytics: boolean;
  block_fingerprinting: boolean;
  block_cryptomining: boolean;
  block_tracking: boolean;
  block_social: boolean;
  subscription_url?: string | null;
  subscription_refresh_hours?: number;
  last_subscription_refresh?: number;
};

/** Encrypted sync settings from backend. */
export type EncryptedSyncSettings = {
  enabled: boolean;
  has_passphrase: boolean;
  last_sync_at: number;
  sync_server_url?: string | null;
  sync_token?: string | null;
  device_id?: string | null;
};

/** Result of a navigation guard check. */
export type NavigationGuardResult = {
  allowed: boolean;
  reason: string;
  canProceed: boolean;
  threatType?: string;
};

/** Check URL against Safe Browsing before navigation. */
export async function checkNavigationGuard(url: string): Promise<NavigationGuardResult> {
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    return { allowed: true, reason: '', canProceed: false };
  }
  try {
    const settings = await invoke<SafeBrowsingSettings>('get_safe_browsing_settings');
    if (!settings.enabled) {
      return { allowed: true, reason: '', canProceed: false };
    }
    const threat = await invoke<SafeBrowsingThreat | null>('check_url_safe', { url });
    if (!threat) {
      return { allowed: true, reason: '', canProceed: false };
    }
    const label = threat.threat_type.replace(/_/g, ' ');
    const reason = `Safe Browsing blocked this page (${label}: ${threat.url_pattern})`;
    const canProceed = settings.show_warnings && settings.allow_proceed;
    return {
      allowed: false,
      reason,
      canProceed,
      threatType: threat.threat_type,
    };
  } catch (error) {
    console.error('checkNavigationGuard failed:', error);
    return { allowed: true, reason: '', canProceed: false };
  }
}

/** Record a blocked malicious site on the privacy dashboard. */
export async function recordMaliciousSiteBlocked(url: string): Promise<void> {
  try {
    const host = new URL(url).hostname;
    await invoke('record_malicious_site_blocked', { domain: host });
  } catch (error) {
    console.error('record_malicious_site_blocked failed:', error);
  }
}

/** Load Safe Browsing settings. */
export async function loadSafeBrowsingSettings(): Promise<SafeBrowsingSettings> {
  return invoke<SafeBrowsingSettings>('get_safe_browsing_settings');
}

/** Persist Safe Browsing settings. */
export async function saveSafeBrowsingSettings(settings: SafeBrowsingSettings): Promise<void> {
  await invoke('update_safe_browsing_settings', { settings });
}

/** Load tracking protection settings. */
export async function loadTrackingProtectionSettings(): Promise<TrackingProtectionSettings> {
  return invoke<TrackingProtectionSettings>('get_tracking_settings');
}

/** Persist tracking protection settings. */
/** Refresh Safe Browsing threat list from configured `list_url`. */
export async function refreshSafeBrowsingList(url?: string): Promise<number> {
  return invoke<number>('refresh_safe_browsing_list', { url: url ?? null });
}

/** Configure tracker blocklist subscription URL and refresh interval (hours). */
export async function setTrackingSubscription(
  subscriptionUrl: string | null,
  refreshHours?: number,
): Promise<void> {
  await invoke('set_tracking_subscription', {
    subscription_url: subscriptionUrl,
    refresh_hours: refreshHours ?? null,
  });
}

/** Load encrypted sync settings. */
export async function loadEncryptedSyncSettings(): Promise<EncryptedSyncSettings> {
  return invoke<EncryptedSyncSettings>('encrypted_sync_get_settings');
}

/** Set sync passphrase (min 8 chars) and enable encrypted vault. */
export async function setEncryptedSyncPassphrase(passphrase: string): Promise<void> {
  await invoke('encrypted_sync_set_passphrase', { passphrase });
}

/** Encrypt bookmark JSON into local vault. */
export async function storeEncryptedBookmarkVault(bookmarksJson: string): Promise<string> {
  return invoke<string>('encrypted_sync_store_bookmarks', { bookmarksJson });
}

export async function setEncryptedSyncServer(
  syncServerUrl: string | null,
  syncToken: string | null,
): Promise<void> {
  await invoke('encrypted_sync_set_server', {
    sync_server_url: syncServerUrl,
    sync_token: syncToken,
  });
}

export async function uploadEncryptedVault(): Promise<string> {
  return invoke<string>('encrypted_sync_upload_vault');
}

export async function downloadEncryptedVault(): Promise<number> {
  return invoke<number>('encrypted_sync_download_vault');
}

export async function saveTrackingProtectionSettings(
  settings: TrackingProtectionSettings,
): Promise<void> {
  await invoke('update_tracking_settings', { settings });
}

/** Report in-page tracker blocks to the privacy dashboard. */
export async function flushTrackerBlockReports(tabLabel: string): Promise<number> {
  if (!tabLabel) return 0;
  try {
    const raw = await evalTabReturning(
      tabLabel,
      `(function(){
        var a = window.__exodusTrackerBlocked || [];
        window.__exodusTrackerBlocked = [];
        return JSON.stringify(a);
      })()`,
    );
    if (!raw) return 0;
    const hosts = JSON.parse(raw) as string[];
    if (!Array.isArray(hosts)) return 0;
    const unique = [...new Set(hosts.filter((h) => typeof h === 'string' && h.length > 0))];
    for (const domain of unique) {
      await invoke('record_tracker_blocked', { domain });
    }
    return unique.length;
  } catch (error) {
    console.error('flushTrackerBlockReports failed:', error);
    return 0;
  }
}

/** Import visit history into the omnibox suggestion index (startup sync). */
export async function seedOmniboxFromVisits(
  visits: Array<{ url: string; title: string }>,
  max = 150,
): Promise<void> {
  for (const visit of visits.slice(0, max)) {
    if (!visit.url.startsWith('http://') && !visit.url.startsWith('https://')) continue;
    await syncSuggestionHistory(visit.url, visit.title || visit.url);
  }
}

/** Label for omnibox suggestion type chip. */
export function omniboxSuggestionTypeLabel(type: string): string {
  switch (type) {
    case 'History':
      return 'History';
    case 'Bookmark':
      return 'Bookmark';
    case 'Search':
      return 'Search';
    case 'Popular':
      return 'Popular';
    default:
      return type;
  }
}
