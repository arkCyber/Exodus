/**
 * BrowserPage must not hit Vue's recursive update guard on mount.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import BrowserPage from './BrowserPage.vue';

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
    homepageUrl: { value: 'https://duckduckgo.com' },
    aiPort: { value: 11435 },
    aiModel: { value: 'exodus-default' },
    sessionRestore: { value: false },
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
vi.mock('@/composables/useBrowserSidebar', () => ({
  useBrowserSidebar: () => ({
    sidebarOpen: { value: true },
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
    indexedCount: { value: 0 },
    historyCount: { value: 0 },
    filteredBookmarks: { value: [] },
    p2pRoomId: { value: 'lobby' },
    canAnnouncePage: { value: false },
    toggleSidebar: vi.fn(),
    closeSidebar: vi.fn(),
    openPanel: vi.fn(),
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
    runAgentHermesStrategy: vi.fn(),
    initSidebar: vi.fn(),
    probeAiOnline: vi.fn(),
    bookmarkSearchQuery: { value: '' },
  }),
}));
vi.mock('@/components/BrowserSidebar.vue', () => ({
  default: { name: 'BrowserSidebar', template: '<aside />' },
}));
vi.mock('@/components/DownloadPanel.vue', () => ({
  default: { name: 'DownloadPanel', template: '<div />' },
}));
vi.mock('@/composables/useBrowserTabGroups', () => ({
  useBrowserTabGroups: () => ({
    tabGroups: { value: [] },
    sortedTabs: { value: [] },
    loadTabGroups: vi.fn(),
  }),
}));
vi.mock('$lib/appLifecycle', () => ({ bindLifecycleRecovery: vi.fn(async () => () => {}) }));
vi.mock('@/components/SettingsModal.vue', () => ({ default: { name: 'SettingsModal', template: '<div />' } }));

describe('BrowserPage render stability', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.spyOn(console, 'warn').mockImplementation((msg: unknown) => {
      const text = String(msg);
      if (text.includes('Maximum recursive updates')) {
        throw new Error(text);
      }
    });
  });

  it('mounts without recursive update errors', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    await flushPromises();
    await flushPromises();
    expect(wrapper.find('.browser-page').exists()).toBe(true);
  });
});
