/**
 * Exodus Browser — useFindInPage composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ref } from 'vue';
import { useFindInPage } from './useFindInPage';

vi.mock('@/lib/exodusBrowser', () => ({
  evalTabReturning: vi.fn(),
  findInTab: vi.fn(),
  tabWebviewLabel: (id: string) => `tab-${id}`,
}));

describe('useFindInPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('counts find matches in native webview', async () => {
    const { evalTabReturning } = await import('@/lib/exodusBrowser');
    vi.mocked(evalTabReturning).mockResolvedValue('5');

    const useNativeWebview = ref(true);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => null;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { countFindMatches } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    const count = await countFindMatches();
    expect(count).toBe(5);
    expect(evalTabReturning).toHaveBeenCalledWith('tab-tab-1', expect.any(String));
  });

  it('counts find matches in iframe fallback', async () => {
    const mockDoc = {
      body: { innerText: 'test test test' },
    } as unknown as Document;

    const useNativeWebview = ref(false);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => mockDoc;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { countFindMatches } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    const count = await countFindMatches();
    expect(count).toBe(3);
  });

  it('returns 0 for empty query', async () => {
    const useNativeWebview = ref(true);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => null;
    const findQuery = ref('');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { countFindMatches } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    const count = await countFindMatches();
    expect(count).toBe(0);
  });

  it('handles regex special characters in query', async () => {
    const mockDoc = {
      body: { innerText: 'test.test test*test' },
    } as unknown as Document;

    const useNativeWebview = ref(false);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => mockDoc;
    const findQuery = ref('test.test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { countFindMatches } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    const count = await countFindMatches();
    expect(count).toBe(1);
  });

  it('finds next match in native webview', async () => {
    const { evalTabReturning, findInTab } = await import('@/lib/exodusBrowser');
    vi.mocked(evalTabReturning).mockResolvedValue('3');
    vi.mocked(findInTab).mockResolvedValue(true);

    const useNativeWebview = ref(true);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => null;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { findInPage } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    await findInPage('next');
    expect(findResults.value).toBe(3);
    expect(currentFindIndex.value).toBe(1);
    expect(findInTab).toHaveBeenCalledWith('tab-tab-1', 'test', true);
  });

  it('finds previous match in native webview', async () => {
    const { evalTabReturning, findInTab } = await import('@/lib/exodusBrowser');
    vi.mocked(evalTabReturning).mockResolvedValue('3');
    vi.mocked(findInTab).mockResolvedValue(true);

    const useNativeWebview = ref(true);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => null;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(2);

    const { findInPage } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    await findInPage('prev');
    expect(currentFindIndex.value).toBe(1);
    expect(findInTab).toHaveBeenCalledWith('tab-tab-1', 'test', false);
  });

  it('finds next match in iframe fallback', async () => {
    const mockWin = {
      find: vi.fn().mockReturnValue(true),
    } as unknown as Window & { find: (s: string, a: boolean, b: boolean, c: boolean) => boolean };

    const mockDoc = {
      body: { innerText: 'test test test' },
      defaultView: mockWin,
    } as unknown as Document;

    const useNativeWebview = ref(false);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => mockDoc;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { findInPage } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    await findInPage('next');
    expect(findResults.value).toBe(3);
    expect(currentFindIndex.value).toBe(1);
    expect(mockWin.find).toHaveBeenCalledWith('test', false, false, true);
  });

  it('resets index when no matches found', async () => {
    const { evalTabReturning, findInTab } = await import('@/lib/exodusBrowser');
    vi.mocked(evalTabReturning).mockResolvedValue('0');
    vi.mocked(findInTab).mockResolvedValue(false);

    const useNativeWebview = ref(true);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => null;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(5);

    const { findInPage } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    await findInPage('next');
    expect(currentFindIndex.value).toBe(0);
  });

  it('recalculates matches on query input', async () => {
    const mockDoc = {
      body: { innerText: 'test test' },
    } as unknown as Document;

    const useNativeWebview = ref(false);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => mockDoc;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(5);

    const { onFindQueryInput } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    await onFindQueryInput();
    expect(currentFindIndex.value).toBe(0);
    expect(findResults.value).toBe(2);
  });

  it('handles errors gracefully', async () => {
    const { evalTabReturning } = await import('@/lib/exodusBrowser');
    vi.mocked(evalTabReturning).mockRejectedValue(new Error('Test error'));

    const useNativeWebview = ref(true);
    const getActiveTabId = () => 'tab-1';
    const getContentDocument = () => null;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { countFindMatches } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    const count = await countFindMatches();
    expect(count).toBe(0);
  });

  it('handles null active tab id', async () => {
    const useNativeWebview = ref(true);
    const getActiveTabId = () => null;
    const getContentDocument = () => null;
    const findQuery = ref('test');
    const findResults = ref(0);
    const currentFindIndex = ref(0);

    const { countFindMatches } = useFindInPage({
      useNativeWebview,
      getActiveTabId,
      getContentDocument,
      findQuery,
      findResults,
      currentFindIndex,
    });

    const count = await countFindMatches();
    expect(count).toBe(0);
  });
});
