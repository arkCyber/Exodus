/**
 * Private Search Tab API for Exodus Browser
 * Provides dedicated private search tab functionality
 */

import { invoke } from '@tauri-apps/api/core';

export interface PrivateSearchSettings {
  enabled: boolean;
  default_search_engine: string;
  block_trackers: boolean;
  clear_on_close: boolean;
  separate_history: boolean;
}

/**
 * Enable private search tab
 */
export async function enablePrivateSearchTab(): Promise<void> {
  return invoke('enable_private_search_tab');
}

/**
 * Disable private search tab
 */
export async function disablePrivateSearchTab(): Promise<void> {
  return invoke('disable_private_search_tab');
}

/**
 * Check if private search tab is enabled
 */
export async function isPrivateSearchTabEnabled(): Promise<boolean> {
  return invoke('is_private_search_tab_enabled');
}

/**
 * Set private search engine
 */
export async function setPrivateSearchEngine(engine: string): Promise<void> {
  return invoke('set_private_search_engine', { engine });
}

/**
 * Get private search engine
 */
export async function getPrivateSearchEngine(): Promise<string> {
  return invoke('get_private_search_engine');
}

/**
 * Set private search block trackers
 */
export async function setPrivateSearchBlockTrackers(block: boolean): Promise<void> {
  return invoke('set_private_search_block_trackers', { block });
}

/**
 * Get private search block trackers
 */
export async function getPrivateSearchBlockTrackers(): Promise<boolean> {
  return invoke('get_private_search_block_trackers');
}

/**
 * Set private search clear on close
 */
export async function setPrivateSearchClearOnClose(clear: boolean): Promise<void> {
  return invoke('set_private_search_clear_on_close', { clear });
}

/**
 * Get private search clear on close
 */
export async function getPrivateSearchClearOnClose(): Promise<boolean> {
  return invoke('get_private_search_clear_on_close');
}

/**
 * Set private search separate history
 */
export async function setPrivateSearchSeparateHistory(separate: boolean): Promise<void> {
  return invoke('set_private_search_separate_history', { separate });
}

/**
 * Get private search separate history
 */
export async function getPrivateSearchSeparateHistory(): Promise<boolean> {
  return invoke('get_private_search_separate_history');
}

/**
 * Get private search settings
 */
export async function getPrivateSearchSettings(): Promise<PrivateSearchSettings> {
  return invoke('get_private_search_settings');
}
