/**
 * BrowserPage — Firefox sidebar wiring (reading list save, customize).
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import BrowserPage from './BrowserPage.vue';

const openPanel = vi.fn();
const pocketSaveArticle = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => false,
  invoke: vi.fn(),
}));

vi.mock('@/lib/perfLog', () => ({
  logPerf: vi.fn(),
  perfAsync: <T>(_: string, fn: () => Promise<T>) => fn(),
  perfStart: vi.fn(),
  perfEnd: vi.fn(),
  startFrameGapMonitor: () => () => {},
}));

vi.mock('$lib/exodusBrowser', () => ({
  canUseNativeWebview: vi.fn(() => false),
  tabWebviewLabel: (id: string) => `exodus-tab-${id}`,
  createTabWebview: vi.fn(),
  showTabWebview: vi.fn(),
  hideTabWebview: vi.fn(),
  closeTabWebview: vi.fn(),
  navigateTab: vi.fn(),
  goBackTab: vi.fn(),
  goForwardTab: vi.fn(),
  reloadTab: vi.fn(),
  getTabNavState: vi.fn(),
  setTabPopupBlocking: vi.fn(),
  toggleTabDevTools: vi.fn(),
  watchWebviewLayout: () => () => {},
}));

vi.mock('@/lib/localPocket', () => ({
  pocketSaveArticle: (...args: unknown[]) => pocketSaveArticle(...args),
}));

vi.mock('$lib/extensions/syncTabs', () => ({ syncExtensionTabs: vi.fn() }));
vi.mock('$lib/extensions/backgroundHosts', () => ({ ensureExtensionBackgrounds: vi.fn() }));
vi.mock('$lib/extensions/tabOps', () => ({ listenExtensionTabOps: vi.fn(async () => () => {}) }));
vi.mock('$lib/extensions/extensionEvents', () => ({
  flushExtensionTab: vi.fn(),
  pumpExtensionRuntime: vi.fn(),
  listenExtensionTabCreates: vi.fn(async () => () => {}),
  listenExtensionPermissionRequests: vi.fn(async () => () => {}),
  listenExtensionHostInstallRequests: vi.fn(async () => () => {}),
  listenExtensionNotifications: vi.fn(async () => () => {}),
  listenExtensionHostDenied: vi.fn(async () => () => {}),
}));
vi.mock('$lib/extensions/api', () => ({
  listExtensions: vi.fn(async () => []),
  validateExtensionHostAccess: vi.fn(async () => true),
}));

vi.mock('@/composables/useBrowserConfig', () => ({
  useBrowserConfig: () => ({
    showBookmarkBar: { value: true },
    httpsOnly: { value: false },
    privateMode: { value: false },
    blockPopups: { value: true },
    searchEngineUrl: { value: 'https://duckduckgo.com/?q={query}' },
    aiPort: { value: 11435 },
    aiModel: { value: 'exodus-default' },
    sessionRestore: { value: true },
    load: vi.fn(),
  }),
}));

vi.mock('@/composables/useBrowserDownloads', () => ({
  useBrowserDownloads: () => ({
    downloads: { value: [] },
    showDownloadsPanel: { value: false },
    activeDownloadsCount: { value: 0 },
    openDownloadsPanel: vi.fn(),
    closeDownloadsPanel: vi.fn(),
    clearDownloads: vi.fn(),
    openDownloadsDir: vi.fn(),
    openDownloadFile: vi.fn(),
    revealDownloadFile: vi.fn(),
  }),
}));

vi.mock('@/composables/useSidebarPreferences', () => ({
  useSidebarPreferences: () => ({
    prefs: {
      value: {
        position: 'right',
        verticalTabsInSidebar: false,
        enabledTools: ['ai', 'memory', 'bookmarks', 'tabs', 'synced', 'reading', 'pocket', 'p2p'],
      },
    },
    iconItems: { value: [] },
    sidebarOnLeft: { value: false },
    verticalTabsInSidebar: { value: false },
    hideHorizontalTabBar: { value: false },
    updatePrefs: vi.fn(),
    toggleTool: vi.fn(),
    setPosition: vi.fn(),
    setVerticalTabsInSidebar: vi.fn(async () => undefined),
    loadPrefs: vi.fn(async () => undefined),
    resolvePanel: (p: string) => p,
    defaultPanel: () => 'ai',
  }),
}));

vi.mock('@/composables/useBrowserSidebar', () => ({
  useBrowserSidebar: () => ({
    sidebarOpen: { value: false },
    sidebarPanel: { value: 'ai' },
    agentPanelOpen: { value: false },
    aiChatHistory: { value: [] },
    chatStreamBuffer: { value: '' },
    aiStreamMode: { value: 'none' },
    isLoading: { value: false },
    aiOnline: { value: false },
    aiChatInput: { value: '' },
    agentCommand: { value: '' },
    agentLog: { value: [] },
    agentDomSummary: { value: '' },
    isAgentExecuting: { value: false },
    indexedMemoryGroups: { value: [] },
    historyGroups: { value: [] },
    filteredIndexedMemoryGroups: { value: [] },
    filteredHistoryGroups: { value: [] },
    memorySearchQuery: { value: '' },
    indexedCount: { value: 0 },
    historyCount: { value: 0 },
    filteredBookmarks: { value: [] },
    p2pRoomId: { value: 'lobby' },
    canAnnouncePage: { value: false },
    toggleSidebar: vi.fn(),
    closeSidebar: vi.fn(),
    openPanel,
    sendAiChat: vi.fn(),
    cancelChat: vi.fn(),
    toggleAgentPanel: vi.fn(),
    loadIndexedMemory: vi.fn(),
    removeIndexedPage: vi.fn(),
    clearIndexedMemory: vi.fn(),
    clearBrowsingHistory: vi.fn(),
    executeAgentCommand: vi.fn(),
    compressCurrentDom: vi.fn(),
    agentBackToAi: vi.fn(),
    onAgentPreset: vi.fn(),
    askAgentWithAllama: vi.fn(),
    runAgentHermesStrategy: vi.fn(),
    initSidebar: vi.fn(),
    probeAiOnline: vi.fn(),
    bookmarkSearchQuery: { value: '' },
  }),
}));

vi.mock('@/components/BrowserSidebar.vue', () => ({
  default: { name: 'BrowserSidebar', template: '<aside class="browser-sidebar-stub" />' },
}));
vi.mock('@/components/DownloadPanel.vue', () => ({
  default: { name: 'DownloadPanel', template: '<div />' },
}));
vi.mock('@/composables/useBrowserTabGroups', () => ({
  useBrowserTabGroups: () => ({
    tabGroups: { value: [] },
    tabContextMenu: { value: null },
    tabGroupEditOffer: { value: null },
    tabGroupEditBusy: { value: false },
    tabGroupDeleteTitle: { value: null },
    tabGroupDeleteBusy: { value: false },
    sortedTabs: { value: [] },
    loadTabGroups: vi.fn(),
    openTabContextMenu: vi.fn(),
    closeTabContextMenu: vi.fn(),
    newTabGroupFromTab: vi.fn(),
    addTabToExistingGroup: vi.fn(),
    removeTabGroupMembership: vi.fn(),
    renameTabGroupPrompt: vi.fn(),
    saveTabGroupEdit: vi.fn(),
    cancelTabGroupEdit: vi.fn(),
    cycleTabGroupColor: vi.fn(),
    deleteTabGroupById: vi.fn(),
    confirmTabGroupDelete: vi.fn(),
    cancelTabGroupDelete: vi.fn(),
    toggleTabGroupCollapse: vi.fn(),
  }),
}));
vi.mock('@/composables/usePasswordSaveOffer', () => ({
  usePasswordSaveOffer: () => ({
    passwordSaveOffer: { value: null },
    passwordSaveBusy: { value: false },
    runPasswordAutofillHooks: vi.fn(),
    confirmPasswordSave: vi.fn(),
    dismissPasswordSave: vi.fn(),
    neverSavePasswordForSite: vi.fn(),
  }),
}));
vi.mock('@/components/PasswordSavePrompt.vue', () => ({ default: { template: '<div />' } }));
vi.mock('@/components/TabGroupEditPrompt.vue', () => ({ default: { template: '<div />' } }));
vi.mock('@/components/TabGroupDeletePrompt.vue', () => ({ default: { template: '<div />' } }));
vi.mock('$lib/extensions/contextMenus', () => ({
  listExtensionContextMenus: vi.fn(async () => []),
  fireExtensionContextMenuClick: vi.fn(),
}));
vi.mock('@/components/SettingsModal.vue', () => ({
  default: { name: 'SettingsModal', template: '<div />' },
}));
vi.mock('$lib/appLifecycle', () => ({ bindLifecycleRecovery: vi.fn(async () => () => {}) }));
vi.mock('$lib/browserIntegrations', () => ({
  loadTrackingProtectionSettings: vi.fn(async () => ({ enabled: true })),
  translateText: vi.fn(async () => ({ translated_text: 'ok' })),
  fetchReadingModeCss: vi.fn(async () => 'body {}'),
  checkNavigationGuard: vi.fn(async () => ({ allowed: true, reason: '', canProceed: false })),
  recordMaliciousSiteBlocked: vi.fn(),
}));
vi.mock('@/composables/useBrowserSitePermissions', () => ({
  useBrowserSitePermissions: () => ({
    sitePermRequest: { value: null },
    setupSitePermissionListener: vi.fn(),
    advanceSitePermQueue: vi.fn(),
    teardownSitePermissionListener: vi.fn(),
  }),
}));

describe('BrowserPage sidebar wiring', () => {
  beforeEach(() => {
    localStorage.clear();
    openPanel.mockClear();
    pocketSaveArticle.mockReset();
    pocketSaveArticle.mockResolvedValue({
      id: 'art-1',
      url: 'https://example.com',
      title: 'Example',
      content: '',
      excerpt: '',
      author: null,
      tags: ['reading-list'],
      saved_at: new Date().toISOString(),
      read_at: null,
      is_favorite: false,
      is_archived: false,
      reading_time_minutes: 1,
      word_count: 100,
    });
  });

  it('saves page to reading list and opens reading panel', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    const addressBar = wrapper.findComponent({ name: 'AddressBar' });
    await addressBar.vm.$emit('navigate', 'https://example.com');
    await flushPromises();
    await addressBar.vm.$emit('saveToReadingList');
    await flushPromises();

    expect(pocketSaveArticle).toHaveBeenCalledWith(
      expect.objectContaining({
        url: 'https://example.com',
        tags: ['reading-list'],
      }),
    );
    expect(openPanel).toHaveBeenCalledWith('reading');
  });

  it('opens customize panel from address bar', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    const addressBar = wrapper.findComponent({ name: 'AddressBar' });
    await addressBar.vm.$emit('openSidebarCustomize');
    await flushPromises();
    expect(openPanel).toHaveBeenCalledWith('customize');
  });
});
