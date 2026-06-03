import { describe, it, expect, beforeEach, vi } from 'vitest';
import { computed } from 'vue';
import { mount, flushPromises } from '@vue/test-utils';
import BrowserPage from './BrowserPage.vue';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => false,
  invoke: vi.fn(),
}));

vi.mock('vue-router', () => ({
  useRoute: () => ({ path: '/', params: {}, query: {} }),
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
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

vi.mock('$lib/extensions/syncTabs', () => ({
  syncExtensionTabs: vi.fn(),
}));

vi.mock('$lib/extensions/backgroundHosts', () => ({
  ensureExtensionBackgrounds: vi.fn(),
}));

vi.mock('$lib/extensions/tabOps', () => ({
  listenExtensionTabOps: vi.fn(async () => () => {}),
}));

vi.mock('@/composables/useBrowserDownloads', () => ({
  useBrowserDownloads: vi.fn(() => ({
    startDownload: vi.fn(),
    openDownloadsPanel: vi.fn(),
  })),
}));

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
  default: { name: 'DownloadPanel', template: '<div class="download-panel-stub" />' },
}));

vi.mock('@/composables/useBrowserTabGroups', () => ({
  useBrowserTabGroups: () => ({
    tabGroups: { value: [] },
    tabContextMenu: { value: null },
    tabGroupEditOffer: { value: null },
    tabGroupEditBusy: { value: false },
    tabGroupDeleteTitle: { value: null },
    tabGroupDeleteBusy: { value: false },
    sortedTabs: computed(() => []),
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

vi.mock('@/components/PasswordSavePrompt.vue', () => ({
  default: { name: 'PasswordSavePrompt', template: '<div />' },
}));
vi.mock('@/components/TabGroupEditPrompt.vue', () => ({
  default: { name: 'TabGroupEditPrompt', template: '<div />' },
}));
vi.mock('@/components/TabGroupDeletePrompt.vue', () => ({
  default: { name: 'TabGroupDeletePrompt', template: '<div />' },
}));

vi.mock('$lib/extensions/contextMenus', () => ({
  listExtensionContextMenus: vi.fn(async () => []),
  fireExtensionContextMenuClick: vi.fn(),
}));

vi.mock('@/components/SettingsModal.vue', () => ({
  default: { name: 'SettingsModal', template: '<div />' },
}));

vi.mock('@/components/ImMessenger.vue', () => ({
  default: {
    name: 'ImMessenger',
    props: ['fullWidth'],
    template: '<div class="im-messenger webchat-main-view" :class="{ \'full-width\': fullWidth }" />',
  },
}));

vi.mock('$lib/appLifecycle', () => ({
  bindLifecycleRecovery: vi.fn(async () => () => {}),
}));

vi.mock('@/lib/newTabWallpaper', () => ({
  WALLPAPER_FEATURE_ENABLED: true,
  peekCachedWallpaperDisplayUrl: vi.fn(() => ''),
  ensureWallpaperDataUrl: vi.fn(async () => 'blob:mock-wallpaper'),
  saveWallpaperIdAndSync: vi.fn(async () => undefined),
  invalidateWallpaperCache: vi.fn(),
  getWallpaperById: vi.fn((id: string) => ({
    id,
    name: 'Test',
    accent: '#6366f1',
    file: 'ishaan-sen-OQRkj2erTPI-unsplash.jpg',
  })),
  wallpaperAssetUrl: vi.fn(() => '/newtab/wallpapers/ishaan-sen-OQRkj2erTPI-unsplash.jpg'),
  wallpaperAbsoluteAssetUrl: vi.fn(() => 'http://localhost/newtab/wallpapers/ishaan-sen-OQRkj2erTPI-unsplash.jpg'),
  peekWallpaperForWindowSession: vi.fn(() => 'ishaan-sen'),
  pickRandomWallpaperForNewTabAsync: vi.fn(async () => 'aivars-vilks'),
  pickWallpaperForWindowSession: vi.fn(async () => 'ishaan-sen'),
  readLaunchWallpaperId: vi.fn(() => null),
  resolveWallpaperDisplayUrl: vi.fn(async () => 'blob:mock-wallpaper'),
}));

vi.mock('$lib/browserIntegrations', () => ({
  loadTrackingProtectionSettings: vi.fn(async () => ({ enabled: true })),
  translateText: vi.fn(async () => ({ translated_text: 'ok' })),
  fetchReadingModeCss: vi.fn(async () => 'body { max-width: 40rem; }'),
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

describe('BrowserPage', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
  });

  it('renders browser page with all components', () => {
    const wrapper = mount(BrowserPage);
    
    expect(wrapper.find('.browser-page').exists()).toBe(true);
    expect(wrapper.findComponent({ name: 'AddressBar' }).exists()).toBe(true);
    expect(wrapper.findComponent({ name: 'BrowserTabBar' }).exists()).toBe(true);
    expect(wrapper.findComponent({ name: 'BookmarkBar' }).exists()).toBe(true);
  });

  it('creates initial tab on mount', async () => {
    const wrapper = mount(BrowserPage);
    
    // Wait for onMounted to complete
    await wrapper.vm.$nextTick();
    expect(wrapper.vm.tabs.length).toBeGreaterThanOrEqual(1);
  });

  it('uses lightweight new-tab URL (no data URL)', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    expect(wrapper.vm.activeTab?.url).toMatch(/^about:blank#exodus-new-tab/);
    expect(wrapper.vm.activeTab?.url.startsWith('data:')).toBe(false);
  });

  it('does not create webview for new-tab URL when native webviews enabled', async () => {
    const exodus = await import('$lib/exodusBrowser');
    vi.mocked(exodus.canUseNativeWebview).mockReturnValue(true);
    vi.mocked(exodus.createTabWebview).mockClear();

    const wrapper = mount(BrowserPage);
    await flushPromises();

    await wrapper.vm.createNewTab();
    await flushPromises();

    expect(exodus.createTabWebview).not.toHaveBeenCalled();
    vi.mocked(exodus.canUseNativeWebview).mockReturnValue(false);
  });

  it('emits navigate event when address bar navigates', async () => {
    const wrapper = mount(BrowserPage);
    await wrapper.vm.$nextTick();

    const addressBar = wrapper.findComponent({ name: 'AddressBar' });
    await addressBar.vm.$emit('navigate', 'https://example.com');
    await wrapper.vm.$nextTick();

    expect(wrapper.vm.activeTab?.url).toBe('https://example.com');
  });

  it('handles keyboard shortcuts', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    await wrapper.findComponent({ name: 'AddressBar' }).vm.$emit('navigate', 'https://example.com');
    await flushPromises();
    const before = wrapper.vm.tabs.length;
    window.dispatchEvent(
      new KeyboardEvent('keydown', { key: 't', metaKey: true, bubbles: true }),
    );
    await flushPromises();
    expect(wrapper.vm.tabs.length).toBeGreaterThan(before);
  });

  it('toggles find bar on Cmd+F', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    await flushPromises();
    window.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'f', metaKey: true, bubbles: true }),
    );
    await flushPromises();
    expect(wrapper.vm.showFindBar).toBe(true);
  });

  it('closes find bar on Escape', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    await flushPromises();
    wrapper.vm.showFindBar = true;
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await flushPromises();
    expect(wrapper.vm.showFindBar).toBe(false);
  });

  it('shows context menu with Chrome-like items', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    
    // Set up active tab to enable navigation
    wrapper.vm.activeTab = { id: 'test', url: 'https://example.com', title: 'Example' };
    wrapper.vm.canGoBack = true;
    wrapper.vm.canGoForward = true;
    
    // Simulate right-click event
    const event = new MouseEvent('contextmenu', {
      clientX: 100,
      clientY: 200,
      bubbles: true,
      cancelable: true,
    });
    window.dispatchEvent(event);
    await flushPromises();
    
    expect(wrapper.vm.showContextMenu).toBe(true);
    expect(wrapper.vm.contextMenuItems.length).toBeGreaterThan(0);
    
    // Check for key menu items
    const menuIds = wrapper.vm.contextMenuItems.map((item: any) => item.id);
    expect(menuIds).toContain('back');
    expect(menuIds).toContain('forward');
    expect(menuIds).toContain('reload');
    expect(menuIds).toContain('save');
    expect(menuIds).toContain('print');
    expect(menuIds).toContain('bookmark');
    expect(menuIds).toContain('zoom-in');
    expect(menuIds).toContain('zoom-out');
    expect(menuIds).toContain('zoom-reset');
  });

  it('disables save and print for new tab pages', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();
    
    // Set active tab to new tab URL
    wrapper.vm.activeTab = { id: 'test', url: 'exodus://newtab', title: 'New Tab' };
    
    const event = new MouseEvent('contextmenu', {
      clientX: 100,
      clientY: 200,
      bubbles: true,
      cancelable: true,
    });
    window.dispatchEvent(event);
    await flushPromises();
    
    const saveItem = wrapper.vm.contextMenuItems.find((item: any) => item.id === 'save');
    const printItem = wrapper.vm.contextMenuItems.find((item: any) => item.id === 'print');
    
    expect(saveItem?.disabled).toBe(true);
    expect(printItem?.disabled).toBe(true);
  });

  it('toggles WebChat full view in main content area', async () => {
    const wrapper = mount(BrowserPage);
    await flushPromises();

    expect(wrapper.find('.webchat-main-view').exists()).toBe(false);

    const addressBar = wrapper.findComponent({ name: 'AddressBar' });
    await addressBar.vm.$emit('toggleWebChat');
    await flushPromises();

    expect(wrapper.vm.showWebChatView).toBe(true);
    expect(wrapper.find('.browser-content .webchat-main-view.im-messenger').exists()).toBe(true);
    expect(wrapper.find('.exodus-new-tab').exists()).toBe(false);

    await addressBar.vm.$emit('toggleWebChat');
    await flushPromises();

    expect(wrapper.vm.showWebChatView).toBe(false);
    expect(wrapper.find('.webchat-main-view').exists()).toBe(false);
  });
});
