/**
 * Exodus Browser — privacy dashboard stats helpers.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

/** Subset of privacy stats shown in chrome UI. */
export type PrivacyStatsSummary = {
  trackers_blocked: number;
  malicious_sites_blocked: number;
  cookies_blocked: number;
  fingerprinting_blocked: number;
};

/** Load privacy stats from backend. */
export async function fetchPrivacyStats(): Promise<PrivacyStatsSummary | null> {
  if (!isTauri()) {
    return null;
  }
  try {
    return await invoke<PrivacyStatsSummary>('get_privacy_stats');
  } catch (error) {
    console.error('get_privacy_stats failed:', error);
    return null;
  }
}
