/**
 * Exodus Browser — useBrowserSidebar composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ref } from 'vue';
import { useBrowserSidebar } from './useBrowserSidebar';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

vi.mock('@/lib/sidebarAiChat', () => ({
  checkSidebarAiOnline: vi.fn(),
  streamSidebarChat: vi.fn(),
}));

vi.mock('@/lib/hermesClient', () => ({
  hermesAnalyzePage: vi.fn(),
  hermesPlanAgentAction: vi.fn(),
  hermesPlanAutomationSteps: vi.fn(),
  hermesRunStrategySteps: vi.fn(),
  hermesSyncAgentContext: vi.fn(),
}));

vi.mock('@/lib/hermesStrategies', () => ({
  listHermesStrategyTemplates: vi.fn(() => []),
  actionJsonFromStepResult: vi.fn(),
}));

vi.mock('@/lib/exodusBrowser', () => ({
  evalInTab: vi.fn(),
  evalTabReturning: vi.fn(),
  getTabHtml: vi.fn(),
}));

vi.mock('@/lib/historyManager', () => ({
  loadMergedBrowsingHistory: vi.fn(),
}));

describe('useBrowserSidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    const getCurrentUrl = () => 'https://example.com';
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const getContentDocument = () => null;
    const navigate = vi.fn();
    const onStatus = vi.fn();
    const aiPort = ref(11435);
    const aiModel = ref('llama3');
    const loadBookmarks = vi.fn();
    const getBookmarks = () => [];
    const removeBookmark = vi.fn();
    const updateBookmarkFolder = vi.fn();

    const sidebar = useBrowserSidebar({
      getCurrentUrl,
      getActiveTabLabel,
      useNativeWebview,
      getContentDocument,
      navigate,
      onStatus,
      aiPort,
      aiModel,
      loadBookmarks,
      getBookmarks,
      removeBookmark,
      updateBookmarkFolder,
    });

    expect(sidebar.sidebarOpen.value).toBe(false);
    expect(sidebar.sidebarPanel.value).toBe('ai');
    expect(sidebar.agentPanelOpen.value).toBe(false);
    expect(sidebar.aiChatHistory.value).toEqual([]);
  });

  it('opens a panel', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.openPanel('bookmarks');

    expect(sidebar.sidebarPanel.value).toBe('bookmarks');
    expect(sidebar.sidebarOpen.value).toBe(true);
    expect(sidebar.agentPanelOpen.value).toBe(false);
  });

  it('toggles sidebar', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.toggleSidebar();

    expect(sidebar.sidebarOpen.value).toBe(true);
  });

  it('closes sidebar', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.closeSidebar();

    expect(sidebar.sidebarOpen.value).toBe(false);
  });

  it('toggles agent panel', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.toggleAgentPanel();

    expect(sidebar.agentPanelOpen.value).toBe(true);
    expect(sidebar.sidebarPanel.value).toBe('ai');
  });

  it('switches back to AI panel from agent', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.agentPanelOpen.value = true;
    sidebar.agentBackToAi();

    expect(sidebar.agentPanelOpen.value).toBe(false);
    expect(sidebar.sidebarPanel.value).toBe('ai');
  });

  it('cancels chat', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.isLoading.value = true;
    sidebar.chatStreamBuffer.value = 'test';
    sidebar.cancelChat();

    expect(sidebar.chatStreamBuffer.value).toBe('');
    expect(sidebar.isLoading.value).toBe(false);
    expect(sidebar.aiStreamMode.value).toBe('none');
  });

  it('adds agent log', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.agentLog.value = ['Previous log'];
    sidebar.agentLog.value = [...sidebar.agentLog.value, 'Test log'];

    expect(sidebar.agentLog.value).toContain('Test log');
  });

  it('filters bookmarks by search query', () => {
    const mockBookmarks = [
      { id: '1', title: 'Work', url: 'https://work.com', folder: 'Work', created_at: Date.now().toString() },
      { id: '2', title: 'Personal', url: 'https://personal.com', folder: 'Personal', created_at: Date.now().toString() },
    ];

    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => mockBookmarks,
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.bookmarkSearchQuery.value = 'work';

    expect(sidebar.filteredBookmarks.value.length).toBe(1);
    expect(sidebar.filteredBookmarks.value[0].title).toBe('Work');
  });

  it('filters memory and history groups by search query', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const { loadMergedBrowsingHistory } = await import('@/lib/historyManager');
    vi.mocked(invoke).mockResolvedValue([
      { id: '1', title: 'Docs', url: 'https://docs.example.com', timestamp: new Date().toISOString() },
    ]);
    vi.mocked(loadMergedBrowsingHistory).mockResolvedValue([
      { id: '2', title: 'News', url: 'https://news.example.com', timestamp: new Date().toISOString() },
    ]);

    const sidebar = useBrowserSidebar({
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    });

    await sidebar.loadIndexedMemory();
    await sidebar.refreshHistory();
    sidebar.memorySearchQuery.value = 'docs';

    const indexedUrls = sidebar.filteredIndexedMemoryGroups.value.flatMap((g) => g.pages.map((p) => p.url));
    expect(indexedUrls.some((u) => u.includes('docs'))).toBe(true);
    expect(sidebar.filteredHistoryGroups.value.flatMap((g) => g.pages).length).toBe(0);
  });

  it('checks if page can be announced', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    expect(sidebar.canAnnouncePage.value).toBe(true);
  });

  it('returns false for canAnnouncePage on new tab', () => {
    const options = {
      getCurrentUrl: () => 'new-tab://',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    expect(sidebar.canAnnouncePage.value).toBe(false);
  });

  it('computes indexed count', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.indexedPages.value = [
      { id: '1', url: 'https://example.com', title: 'Test', timestamp: Date.now().toString() },
    ];

    expect(sidebar.indexedCount.value).toBe(1);
  });

  it('computes history count', () => {
    const options = {
      getCurrentUrl: () => 'https://example.com',
      getActiveTabLabel: () => 'tab-1',
      useNativeWebview: ref(true),
      getContentDocument: () => null,
      navigate: vi.fn(),
      onStatus: vi.fn(),
      aiPort: ref(11435),
      aiModel: ref('llama3'),
      loadBookmarks: vi.fn(),
      getBookmarks: () => [],
      removeBookmark: vi.fn(),
      updateBookmarkFolder: vi.fn(),
    };

    const sidebar = useBrowserSidebar(options);
    sidebar.historyPages.value = [
      { id: '1', url: 'https://example.com', title: 'Test', timestamp: Date.now().toString() },
    ];

    expect(sidebar.historyCount.value).toBe(1);
  });
});
