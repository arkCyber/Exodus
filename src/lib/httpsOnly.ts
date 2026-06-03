/**
 * HTTPS-Only Mode API for Exodus Browser
 * Automatically upgrades HTTP connections to HTTPS
 */

import { invoke } from '@tauri-apps/api/core';

export interface HttpsOnlySettings {
  enabled: boolean;
  exceptions: string[];
}

/**
 * Enable HTTPS-Only mode
 */
export async function enableHttpsOnly(): Promise<void> {
  return invoke('enable_https_only');
}

/**
 * Disable HTTPS-Only mode
 */
export async function disableHttpsOnly(): Promise<void> {
  return invoke('disable_https_only');
}

/**
 * Check if HTTPS-Only mode is enabled
 */
export async function isHttpsOnlyEnabled(): Promise<boolean> {
  return invoke('is_https_only_enabled');
}

/**
 * Add an exception (domain that can use HTTP)
 */
export async function addHttpsOnlyException(domain: string): Promise<void> {
  return invoke('add_https_only_exception', { domain });
}

/**
 * Remove an exception
 */
export async function removeHttpsOnlyException(domain: string): Promise<void> {
  return invoke('remove_https_only_exception', { domain });
}

/**
 * Get all exceptions
 */
export async function getHttpsOnlyExceptions(): Promise<string[]> {
  return invoke('get_https_only_exceptions');
}

/**
 * Check if a URL should be upgraded
 */
export async function shouldUpgradeUrl(url: string): Promise<boolean> {
  return invoke('should_upgrade_url', { url });
}

/**
 * Upgrade a URL to HTTPS
 */
export async function upgradeToHttps(url: string): Promise<string | null> {
  return invoke('upgrade_to_https', { url });
}

/**
 * Get HTTPS-Only settings
 */
export async function getHttpsOnlySettings(): Promise<HttpsOnlySettings> {
  return invoke('get_https_only_settings');
}
