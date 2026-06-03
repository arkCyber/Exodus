/**
 * Omnibox Image Search API for Exodus Browser
 * Provides image search functionality from the omnibox
 */

import { invoke } from '@tauri-apps/api/core';

export interface OmniboxImageSearchSettings {
  enabled: boolean;
  default_engine: string;
  show_preview: boolean;
  safe_search: boolean;
}

/**
 * Enable omnibox image search
 */
export async function enableOmniboxImageSearch(): Promise<void> {
  return invoke('enable_omnibox_image_search');
}

/**
 * Disable omnibox image search
 */
export async function disableOmniboxImageSearch(): Promise<void> {
  return invoke('disable_omnibox_image_search');
}

/**
 * Check if omnibox image search is enabled
 */
export async function isOmniboxImageSearchEnabled(): Promise<boolean> {
  return invoke('is_omnibox_image_search_enabled');
}

/**
 * Set omnibox image search engine
 */
export async function setOmniboxImageSearchEngine(engine: string): Promise<void> {
  return invoke('set_omnibox_image_search_engine', { engine });
}

/**
 * Get omnibox image search engine
 */
export async function getOmniboxImageSearchEngine(): Promise<string> {
  return invoke('get_omnibox_image_search_engine');
}

/**
 * Set omnibox image search show preview
 */
export async function setOmniboxImageSearchShowPreview(show: boolean): Promise<void> {
  return invoke('set_omnibox_image_search_show_preview', { show });
}

/**
 * Get omnibox image search show preview
 */
export async function getOmniboxImageSearchShowPreview(): Promise<boolean> {
  return invoke('get_omnibox_image_search_show_preview');
}

/**
 * Set omnibox image search safe search
 */
export async function setOmniboxImageSearchSafeSearch(safe: boolean): Promise<void> {
  return invoke('set_omnibox_image_search_safe_search', { safe });
}

/**
 * Get omnibox image search safe search
 */
export async function getOmniboxImageSearchSafeSearch(): Promise<boolean> {
  return invoke('get_omnibox_image_search_safe_search');
}

/**
 * Get omnibox image search settings
 */
export async function getOmniboxImageSearchSettings(): Promise<OmniboxImageSearchSettings> {
  return invoke('get_omnibox_image_search_settings');
}
