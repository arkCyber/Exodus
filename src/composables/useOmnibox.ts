/**
 * Exodus Browser — address bar omnibox (history, /ask memory search, extension keywords).
 */

import { ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { fetchOmniboxSuggestions, type OmniboxSuggestion } from '@/lib/browserIntegrations';
import type { SearchHit } from '@/lib/browserTypes';
import {
  dispatchExtensionOmniboxEvent,
  listExtensionOmniboxKeywords,
  matchExtensionOmniboxKeyword,
  type ExtensionOmniboxKeyword,
} from '@/lib/extensions/omniboxHost';

/**
 * Omnibox state for AddressBar: history/bookmark suggestions and extension keyword mode.
 */
export function useOmnibox() {
  const suggestions = ref<OmniboxSuggestion[]>([]);
  const showSuggestions = ref(false);
  const extensionKeywords = ref<ExtensionOmniboxKeyword[]>([]);
  const activeExtensionKeyword = ref<ExtensionOmniboxKeyword | null>(null);
  const searchResults = ref<SearchHit[]>([]);
  const isSearching = ref(false);
  const showSearchResults = ref(false);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  function isAskInput(input: string): boolean {
    return input.trim().startsWith('/ask ');
  }

  async function loadExtensionKeywords(): Promise<void> {
    extensionKeywords.value = await listExtensionOmniboxKeywords();
  }

  /** Refresh suggestion list for current input. */
  async function refreshSuggestions(input: string): Promise<void> {
    const trimmed = input.trim();
    const extMatch = matchExtensionOmniboxKeyword(trimmed, extensionKeywords.value);
    activeExtensionKeyword.value = extMatch?.entry ?? null;

    if (extMatch) {
      await dispatchExtensionOmniboxEvent(extMatch.entry.extensionId, 'onInputChanged', extMatch.query);
      suggestions.value = [];
      showSuggestions.value = false;
      return;
    }

    if (trimmed.length < 1 || isAskInput(trimmed)) {
      suggestions.value = [];
      showSuggestions.value = false;
      return;
    }

    const rows = await fetchOmniboxSuggestions(trimmed, 8);
    suggestions.value = rows;
    showSuggestions.value = rows.length > 0 && !showSearchResults.value;
  }

  /** Run RAG semantic search for `/ask <query>` omnibox input. */
  async function performSemanticSearch(query: string): Promise<void> {
    const q = query.trim();
    if (!q) {
      searchResults.value = [];
      showSearchResults.value = false;
      return;
    }
    if (!isTauri()) {
      searchResults.value = [];
      showSearchResults.value = true;
      return;
    }
    isSearching.value = true;
    showSearchResults.value = true;
    showSuggestions.value = false;
    searchResults.value = [];
    try {
      searchResults.value = await invoke<SearchHit[]>('semantic_search', { query: q });
    } catch (error) {
      console.error('semantic_search failed:', error);
      searchResults.value = [];
    } finally {
      isSearching.value = false;
    }
  }

  function clearSearchResults(): void {
    showSearchResults.value = false;
    searchResults.value = [];
    isSearching.value = false;
  }

  function scheduleSuggestions(input: string): void {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      void refreshSuggestions(input);
    }, 120);
    void refreshSuggestions(input);
  }

  function hideSuggestions(): void {
    showSuggestions.value = false;
    suggestions.value = [];
  }

  /**
   * Handle omnibox submit: extension keyword, `/ask` search, or normal navigation payload.
   * @returns `'ask'` when memory search ran; `'extension'` when extension consumed input; `null` otherwise.
   */
  async function handleOmniboxSubmit(
    input: string,
  ): Promise<'ask' | 'extension' | null> {
    const trimmed = input.trim();
    if (!trimmed) return null;
    if (trimmed.startsWith('/ask ')) {
      await performSemanticSearch(trimmed.slice(5));
      return 'ask';
    }
    if (await submitExtensionKeyword(trimmed)) {
      return 'extension';
    }
    hideSuggestions();
    clearSearchResults();
    return null;
  }

  /** Notify extension onInputEntered when user submits in keyword mode. */
  async function submitExtensionKeyword(input: string): Promise<boolean> {
    const match = matchExtensionOmniboxKeyword(input, extensionKeywords.value);
    if (!match) return false;
    await dispatchExtensionOmniboxEvent(match.entry.extensionId, 'onInputEntered', match.query);
    hideSuggestions();
    return true;
  }

  void loadExtensionKeywords();

  return {
    suggestions,
    showSuggestions,
    extensionKeywords,
    activeExtensionKeyword,
    searchResults,
    isSearching,
    showSearchResults,
    scheduleSuggestions,
    hideSuggestions,
    clearSearchResults,
    performSemanticSearch,
    handleOmniboxSubmit,
    submitExtensionKeyword,
    loadExtensionKeywords,
    isAskInput,
  };
}
