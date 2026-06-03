/**
 * Tab Search API for Exodus Browser
 * Allows searching through open tabs by title, URL, or content
 */

import { invoke } from '@tauri-apps/api/core';

export interface TabInfo {
  label: string;
  title: string;
  url: string;
  favicon?: string;
  is_active: boolean;
  is_pinned: boolean;
  is_muted: boolean;
}

export interface TabSearchRequest {
  query: string;
  limit?: number;
}

export interface TabSearchResult {
  tabs: TabInfo[];
  total_count: number;
}

/**
 * Search tabs by query
 */
export async function searchTabs(request: TabSearchRequest): Promise<TabSearchResult> {
  return invoke('search_tabs', { request });
}

/**
 * Get all registered tabs
 */
export async function getAllTabs(): Promise<TabInfo[]> {
  return invoke('get_all_search_tabs');
}

/**
 * Register a tab for search
 */
export async function registerTab(tab: TabInfo): Promise<void> {
  return invoke('register_search_tab', { tab });
}

/**
 * Unregister a tab
 */
export async function unregisterTab(label: string): Promise<void> {
  return invoke('unregister_search_tab', { label });
}

/**
 * Update tab information
 */
export async function updateTab(params: {
  label: string;
  title?: string;
  url?: string;
  favicon?: string;
  is_active?: boolean;
  is_pinned?: boolean;
  is_muted?: boolean;
}): Promise<void> {
  return invoke('update_search_tab', params );
}
