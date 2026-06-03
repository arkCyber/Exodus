/**
 * Enhanced Mobile Sync API for Exodus Browser
 * Provides enhanced synchronization features for mobile devices
 */

import { invoke } from '@tauri-apps/api/core';

export interface MobileSyncSettings {
  enabled: boolean;
  auto_sync: boolean;
  sync_interval_minutes: number;
  sync_over_wifi_only: boolean;
  sync_bookmarks: boolean;
  sync_history: boolean;
  sync_passwords: boolean;
  sync_reading_list: boolean;
}

/**
 * Enable mobile sync
 */
export async function enableMobileSync(): Promise<void> {
  return invoke('enable_mobile_sync');
}

/**
 * Disable mobile sync
 */
export async function disableMobileSync(): Promise<void> {
  return invoke('disable_mobile_sync');
}

/**
 * Check if mobile sync is enabled
 */
export async function isMobileSyncEnabled(): Promise<boolean> {
  return invoke('is_mobile_sync_enabled');
}

/**
 * Set mobile sync auto
 */
export async function setMobileSyncAuto(auto: boolean): Promise<void> {
  return invoke('set_mobile_sync_auto', { auto });
}

/**
 * Get mobile sync auto
 */
export async function getMobileSyncAuto(): Promise<boolean> {
  return invoke('get_mobile_sync_auto');
}

/**
 * Set mobile sync interval
 */
export async function setMobileSyncInterval(minutes: number): Promise<void> {
  return invoke('set_mobile_sync_interval', { minutes });
}

/**
 * Get mobile sync interval
 */
export async function getMobileSyncInterval(): Promise<number> {
  return invoke('get_mobile_sync_interval');
}

/**
 * Set mobile sync wifi only
 */
export async function setMobileSyncWifiOnly(wifiOnly: boolean): Promise<void> {
  return invoke('set_mobile_sync_wifi_only', { wifiOnly });
}

/**
 * Get mobile sync wifi only
 */
export async function getMobileSyncWifiOnly(): Promise<boolean> {
  return invoke('get_mobile_sync_wifi_only');
}

/**
 * Set mobile sync bookmarks
 */
export async function setMobileSyncBookmarks(sync: boolean): Promise<void> {
  return invoke('set_mobile_sync_bookmarks', { sync });
}

/**
 * Get mobile sync bookmarks
 */
export async function getMobileSyncBookmarks(): Promise<boolean> {
  return invoke('get_mobile_sync_bookmarks');
}

/**
 * Set mobile sync history
 */
export async function setMobileSyncHistory(sync: boolean): Promise<void> {
  return invoke('set_mobile_sync_history', { sync });
}

/**
 * Get mobile sync history
 */
export async function getMobileSyncHistory(): Promise<boolean> {
  return invoke('get_mobile_sync_history');
}

/**
 * Set mobile sync passwords
 */
export async function setMobileSyncPasswords(sync: boolean): Promise<void> {
  return invoke('set_mobile_sync_passwords', { sync });
}

/**
 * Get mobile sync passwords
 */
export async function getMobileSyncPasswords(): Promise<boolean> {
  return invoke('get_mobile_sync_passwords');
}

/**
 * Set mobile sync reading list
 */
export async function setMobileSyncReadingList(sync: boolean): Promise<void> {
  return invoke('set_mobile_sync_reading_list', { sync });
}

/**
 * Get mobile sync reading list
 */
export async function getMobileSyncReadingList(): Promise<boolean> {
  return invoke('get_mobile_sync_reading_list');
}

/**
 * Trigger manual mobile sync
 */
export async function triggerMobileSync(): Promise<void> {
  return invoke('trigger_mobile_sync');
}

/**
 * Get last mobile sync time
 */
export async function getMobileSyncLastSync(): Promise<string | null> {
  return invoke('get_mobile_sync_last_sync');
}

/**
 * Get mobile sync settings
 */
export async function getMobileSyncSettings(): Promise<MobileSyncSettings> {
  return invoke('get_mobile_sync_settings');
}
