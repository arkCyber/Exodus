/**
 * Exodus Browser — useOmnibox unit tests (/ask memory search).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useOmnibox } from './useOmnibox';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

vi.mock('$lib/browserIntegrations', () => ({
  fetchOmniboxSuggestions: vi.fn(async () => []),
}));

vi.mock('$lib/extensions/omniboxHost', () => ({
  listExtensionOmniboxKeywords: vi.fn(async () => []),
  matchExtensionOmniboxKeyword: vi.fn(() => null),
  dispatchExtensionOmniboxEvent: vi.fn(),
}));

describe('useOmnibox', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('performSemanticSearch invokes semantic_search', async () => {
    vi.mocked(invoke).mockResolvedValue([
      { page: { url: 'https://a.test', title: 'A', timestamp: '1' }, score: 0.9 },
    ]);
    const omnibox = useOmnibox();
    await omnibox.performSemanticSearch('hello');
    expect(invoke).toHaveBeenCalledWith('semantic_search', { query: 'hello' });
    expect(omnibox.showSearchResults.value).toBe(true);
    expect(omnibox.searchResults.value).toHaveLength(1);
  });

  it('handleOmniboxSubmit runs ask search for /ask prefix', async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const omnibox = useOmnibox();
    const result = await omnibox.handleOmniboxSubmit('/ask rust browser');
    expect(result).toBe('ask');
    expect(invoke).toHaveBeenCalledWith('semantic_search', { query: 'rust browser' });
  });

  it('skips suggestions when input is /ask', async () => {
    const { fetchOmniboxSuggestions } = await import('$lib/browserIntegrations');
    const omnibox = useOmnibox();
    omnibox.scheduleSuggestions('/ask foo');
    await new Promise((r) => setTimeout(r, 150));
    expect(fetchOmniboxSuggestions).not.toHaveBeenCalled();
    expect(omnibox.showSuggestions.value).toBe(false);
  });
});
