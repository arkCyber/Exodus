/**
 * Exodus Browser — Sidebar Search functionality
 * Aerospace-level error handling, security validation, and input validation.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

// Aerospace-level security validation patterns
const VALID_QUERY_PATTERN = /^[a-zA-Z0-9_\-\s\u4e00-\u9fa5]+$/;
const VALID_ENGINE_PATTERN = /^[a-zA-Z0-9_-]+$/;

/**
 * Validate search query for security.
 * Aerospace-level validation to prevent injection attacks.
 */
function validateQuery(query: string): boolean {
  if (!query || typeof query !== 'string') {
    console.error('[SidebarSearch] Invalid query');
    return false;
  }
  if (query.length > 500) {
    console.error('[SidebarSearch] Query too long');
    return false;
  }
  return VALID_QUERY_PATTERN.test(query);
}

/**
 * Validate search engine ID for security.
 */
function validateEngine(engine: string): boolean {
  if (!engine || typeof engine !== 'string') {
    console.error('[SidebarSearch] Invalid engine');
    return false;
  }
  return VALID_ENGINE_PATTERN.test(engine);
}

/** Search engine configuration */
export type SearchEngine = {
  id: string;
  name: string;
  url: string;
  icon?: string;
};

/** Search result */
export type SearchResult = {
  title: string;
  url: string;
  snippet?: string;
  engine: string;
};

/** Search history entry */
export type SearchHistoryEntry = {
  query: string;
  timestamp: number;
  engine: string;
};

/** Default search engines */
export const DEFAULT_SEARCH_ENGINES: SearchEngine[] = [
  {
    id: 'duckduckgo',
    name: 'DuckDuckGo',
    url: 'https://duckduckgo.com/?q={query}',
  },
  {
    id: 'google',
    name: 'Google',
    url: 'https://www.google.com/search?q={query}',
  },
  {
    id: 'bing',
    name: 'Bing',
    url: 'https://www.bing.com/search?q={query}',
  },
  {
    id: 'brave',
    name: 'Brave',
    url: 'https://search.brave.com/search?q={query}',
  },
];

/** Get all configured search engines */
export async function getSearchEngines(): Promise<SearchEngine[]> {
  if (!isTauri()) return DEFAULT_SEARCH_ENGINES;
  
  try {
    const engines = await invoke<SearchEngine[]>('get_search_engines');
    return engines.length > 0 ? engines : DEFAULT_SEARCH_ENGINES;
  } catch (error) {
    console.error('[SidebarSearch] Failed to get search engines:', error);
    return DEFAULT_SEARCH_ENGINES;
  }
}

/** Set default search engine */
export async function setDefaultSearchEngine(engineId: string): Promise<void> {
  if (!isTauri()) return;
  
  if (!validateEngine(engineId)) {
    console.error('[SidebarSearch] Invalid engine ID for setDefaultSearchEngine');
    return;
  }
  
  try {
    await invoke('set_default_search_engine', { engineId });
  } catch (error) {
    console.error('[SidebarSearch] Failed to set default search engine:', error);
  }
}

/** Get default search engine */
export async function getDefaultSearchEngine(): Promise<string> {
  if (!isTauri()) return 'duckduckgo';
  
  try {
    return await invoke<string>('get_default_search_engine');
  } catch (error) {
    console.error('[SidebarSearch] Failed to get default search engine:', error);
    return 'duckduckgo';
  }
}

/** Add custom search engine */
export async function addSearchEngine(engine: SearchEngine): Promise<void> {
  if (!isTauri()) return;
  
  if (!validateEngine(engine.id)) {
    console.error('[SidebarSearch] Invalid engine ID for addSearchEngine');
    return;
  }
  
  try {
    await invoke('add_search_engine', { engine });
  } catch (error) {
    console.error('[SidebarSearch] Failed to add search engine:', error);
  }
}

/** Remove custom search engine */
export async function removeSearchEngine(engineId: string): Promise<void> {
  if (!isTauri()) return;
  
  if (!validateEngine(engineId)) {
    console.error('[SidebarSearch] Invalid engine ID for removeSearchEngine');
    return;
  }
  
  try {
    await invoke('remove_search_engine', { engineId });
  } catch (error) {
    console.error('[SidebarSearch] Failed to remove search engine:', error);
  }
}

/** Perform search with specified engine */
export function performSearch(query: string, engine: SearchEngine): string {
  if (!validateQuery(query)) {
    console.error('[SidebarSearch] Invalid query for performSearch');
    return '';
  }
  
  return engine.url.replace('{query}', encodeURIComponent(query));
}

/** Get search history */
export async function getSearchHistory(limit: number = 10): Promise<SearchHistoryEntry[]> {
  if (!isTauri()) return [];
  
  try {
    return await invoke<SearchHistoryEntry[]>('get_search_history', { limit });
  } catch (error) {
    console.error('[SidebarSearch] Failed to get search history:', error);
    return [];
  }
}

/** Add query to search history */
export async function addToSearchHistory(query: string, engine: string): Promise<void> {
  if (!isTauri()) return;
  
  if (!validateQuery(query)) {
    console.error('[SidebarSearch] Invalid query for addToSearchHistory');
    return;
  }
  
  if (!validateEngine(engine)) {
    console.error('[SidebarSearch] Invalid engine for addToSearchHistory');
    return;
  }
  
  try {
    await invoke('add_to_search_history', { query, engine });
  } catch (error) {
    console.error('[SidebarSearch] Failed to add to search history:', error);
  }
}

/** Clear search history */
export async function clearSearchHistory(): Promise<void> {
  if (!isTauri()) return;
  
  try {
    await invoke('clear_search_history');
  } catch (error) {
    console.error('[SidebarSearch] Failed to clear search history:', error);
  }
}

/** Remove specific search history entry */
export async function removeFromSearchHistory(query: string): Promise<void> {
  if (!isTauri()) return;
  
  if (!validateQuery(query)) {
    console.error('[SidebarSearch] Invalid query for removeFromSearchHistory');
    return;
  }
  
  try {
    await invoke('remove_from_search_history', { query });
  } catch (error) {
    console.error('[SidebarSearch] Failed to remove from search history:', error);
  }
}

/** Get search suggestions from local history */
export async function getSearchSuggestions(query: string, limit: number = 5): Promise<string[]> {
  if (!isTauri()) return [];
  
  if (!validateQuery(query)) {
    console.error('[SidebarSearch] Invalid query for getSearchSuggestions');
    return [];
  }
  
  try {
    return await invoke<string[]>('get_search_suggestions', { query, limit });
  } catch (error) {
    console.error('[SidebarSearch] Failed to get search suggestions:', error);
    return [];
  }
}
