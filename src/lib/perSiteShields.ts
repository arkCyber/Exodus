/**
 * Per-Site Shield Controls API for Exodus Browser
 * Provides granular privacy settings per website
 */

import { invoke } from '@tauri-apps/api/core';

export type ShieldType =
  | 'ad_blocking'
  | 'tracker_blocking'
  | 'cookie_blocking'
  | 'fingerprinting_protection'
  | 'https_only'
  | 'script_blocking'
  | 'canvas_fingerprinting'
  | 'webgl_fingerprinting'
  | 'audio_fingerprinting'
  | 'font_fingerprinting';

export interface SiteShieldSettings {
  origin: string;
  shields: Record<ShieldType, boolean>;
  custom_rules: string[];
}

/**
 * Get shield settings for a site
 */
export async function getSiteShieldSettings(origin: string): Promise<SiteShieldSettings> {
  return invoke('get_site_shield_settings', { origin });
}

/**
 * Set shield settings for a site
 */
export async function setSiteShieldSettings(
  origin: string,
  settings: SiteShieldSettings
): Promise<void> {
  return invoke('set_site_shield_settings', { origin, settings });
}

/**
 * Enable a shield for a site
 */
export async function enableSiteShield(origin: string, shieldType: ShieldType): Promise<void> {
  return invoke('enable_site_shield', { origin, shieldType });
}

/**
 * Disable a shield for a site
 */
export async function disableSiteShield(origin: string, shieldType: ShieldType): Promise<void> {
  return invoke('disable_site_shield', { origin, shieldType });
}

/**
 * Check if a shield is enabled for a site
 */
export async function isSiteShieldEnabled(
  origin: string,
  shieldType: ShieldType
): Promise<boolean> {
  return invoke('is_site_shield_enabled', { origin, shieldType });
}

/**
 * Set default shield settings
 */
export async function setDefaultShieldSettings(settings: SiteShieldSettings): Promise<void> {
  return invoke('set_default_shield_settings', { settings });
}

/**
 * Get default shield settings
 */
export async function getDefaultShieldSettings(): Promise<SiteShieldSettings> {
  return invoke('get_default_shield_settings');
}

/**
 * Add custom rule for a site
 */
export async function addSiteCustomRule(origin: string, rule: string): Promise<void> {
  return invoke('add_site_custom_rule', { origin, rule });
}

/**
 * Remove custom rule from a site
 */
export async function removeSiteCustomRule(origin: string, rule: string): Promise<void> {
  return invoke('remove_site_custom_rule', { origin, rule });
}

/**
 * Get all sites with custom settings
 */
export async function getAllShieldSites(): Promise<string[]> {
  return invoke('get_all_shield_sites');
}

/**
 * Reset site to default settings
 */
export async function resetSiteShields(origin: string): Promise<void> {
  return invoke('reset_site_shields', { origin });
}

/**
 * Clear all custom settings
 */
export async function clearAllSiteShields(): Promise<void> {
  return invoke('clear_all_site_shields');
}
