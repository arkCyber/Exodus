<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { Webview } from '@tauri-apps/api/webview';
  import type {
    AiChatMessage,
    BookmarkItem,
    BrowserTab,
    HistoryPage,
    ClosedTabSnapshot,
    DownloadRecord,
    SearchHit,
    IndexedPage,
    SidebarPanel,
    QuickLink,
  } from '$lib/browserTypes';
  import {
    canUseNativeWebview,
    captureTabPage,
    closeTabWebview,
    discardTabWebview,
    isTabDiscarded,
    restoreDiscardedTab,
    createTabWebview,
    evalInTab,
    evalTabReturning,
    findInTab,
    getTabHtml,
    getTabNavState,
    getTabSelection,
    getTabTitle,
    goBackTab,
    goForwardTab,
    hideTabWebview,
    navigateTab,
    reloadTab,
    toggleTabDevTools,
    showTabWebview,
    tabWebviewLabel,
    watchWebviewLayout,
    setTabZoom,
    setTabPopupBlocking,
  } from '$lib/exodusBrowser';
  import { faviconUrlFor, isSecureUrl } from '$lib/favicon';
  import { resolveOmniboxInput } from '$lib/omnibox';
  import { groupHistoryByDate } from '$lib/historyGroups';
  import { announcePageUrlToCdn, maybeAnnounceIndexedPage, suggestUrlsForCdnAnnounce } from '$lib/p2p/cdnIntegrations';
  import { cdnUrlStatusLabel, fetchCdnPageStatus, type CdnUrlStatus } from '$lib/p2p/cdnPageStatus';
  import {
    disableReadingModeForUrl,
    enableReadingModeForUrl,
    fetchOmniboxSuggestions,
    fetchReadingModeCss,
    syncSuggestionBookmark,
    syncSuggestionHistory,
    translateText,
    checkNavigationGuard,
    recordMaliciousSiteBlocked,
    flushTrackerBlockReports,
    seedOmniboxFromVisits,
    loadTrackingProtectionSettings,
    type OmniboxSuggestion,
  } from '$lib/browserIntegrations';
  import {
    addTabToGroup,
    collapseTabGroup,
    createTabGroup,
    deleteTabGroup,
    expandTabGroup,
    listTabGroups,
    removeTabFromGroup,
    sortTabsWithGroups,
    TAB_GROUP_COLORS,
    updateTabGroup,
    type TabGroup,
  } from '$lib/tabGroups';
  import {
    applyPasswordAutofill,
    getPasswordForPage,
    loadPasswordManagerSettings,
    pullPasswordCapture,
    savePasswordCapture,
    type PasswordCapturePayload,
  } from '$lib/passwordAutofill';
  import { groupIndexedByDate } from '$lib/indexedMemory';
  import { applyHttpsOnly, parsePrivacyTuple, shouldPersistSession } from '$lib/privacySettings';
  import { ensureExtensionBackgrounds } from '$lib/extensions/backgroundHosts';
  import {
    flushExtensionTab,
    listenExtensionTabCreates,
    listenExtensionNotifications,
    listenExtensionHostDenied,
    listenExtensionPermissionRequests,
    listenExtensionHostInstallRequests,
    listenBrowserSitePermissionRequests,
    pumpExtensionRuntime,
    type ExtensionPermissionRequestEvent,
    type ExtensionHostInstallRequestEvent,
    type BrowserSitePermissionRequestEvent,
  } from '$lib/extensions/extensionEvents';
  import { listenExtensionTabOps, type ExtensionTabOp } from '$lib/extensions/tabOps';
  import ExtensionPermissionPrompt from '$lib/components/ExtensionPermissionPrompt.svelte';
  import PasswordSavePrompt from '$lib/components/PasswordSavePrompt.svelte';
  import SafeBrowsingPrompt from '$lib/components/SafeBrowsingPrompt.svelte';
  import TabGroupEditPrompt, {
    type TabGroupEditOffer,
  } from '$lib/components/TabGroupEditPrompt.svelte';
  import TabGroupDeletePrompt from '$lib/components/TabGroupDeletePrompt.svelte';
  import { addNeverSavePasswordHost, isNeverSavePasswordUrl } from '$lib/passwordNeverSave';
  import { fetchPrivacyStats, type PrivacyStatsSummary } from '$lib/privacyStats';
  import ExtensionHostInstallPrompt from '$lib/components/ExtensionHostInstallPrompt.svelte';
  import BrowserSitePermissionPrompt from '$lib/components/BrowserSitePermissionPrompt.svelte';
  import type { ExtensionTabCreateRequest } from '$lib/extensions/extensionEvents';
  import type { TabCreateAck } from '$lib/extensions/types';
  import { syncExtensionTabs } from '$lib/extensions/syncTabs';
  import ExtensionActionBar from '$lib/components/ExtensionActionBar.svelte';
  import { initNewTabPage, isNewTabUrl, NEWTAB_PAGE_URL } from '$lib/newTabPage';
  import {
    preloadWallpaperDataUrls,
    saveWallpaperIdAndSync,
    syncNtpWallpaperOnStartup,
  } from '$lib/newTabWallpaper';
  import { buildTopSitesFromHistory } from '$lib/newTabTopSites';
  import {
    markTabActiveLifecycle,
    registerTabLifecycle,
    runTabLifecycleMaintenance,
    syncAllTabsLifecycle,
    unregisterTabLifecycle,
  } from '$lib/tabLifecycle';
  import { getRecentManagedHistory } from '$lib/historyManager';
  import { loadPersistedDownloads } from '$lib/downloadsPersist';
  import {
    getSiteShieldAllowTrackers,
    hostFromPageUrl,
    setSiteShieldAllowTrackers,
  } from '$lib/siteShields';
  import { navFlagsFromTrack, recordTabNavigation } from '$lib/tabNavStack';
  import {
    parseAgentCommandInput,
    agentActionReturnsValue,
    summarizeCompressedDom,
  } from '$lib/agentActions';
  import { checkEmbeddingsOnline } from '$lib/allamaClient';
  import { DEFAULT_CHAT_MODEL, loadAiConfig, normalizeChatModelFromAllama } from '$lib/aiConfig';
  import {
    checkSidebarAiOnline,
    streamSidebarChat,
    streamSidebarSummarize,
  } from '$lib/sidebarAiChat';
  import BrowserTabBar from '$lib/components/BrowserTabBar.svelte';
  import SettingsModal, {
    type SettingsScrollSection,
  } from '$lib/components/SettingsModal.svelte';
  import {
    addManagedHistoryEntry,
    clearAllManagedHistory,
    loadMergedBrowsingHistory,
    searchMergedBrowsingHistory,
  } from '$lib/historyManager';
  import ConfirmPrompt, { type ConfirmOffer } from '$lib/components/ConfirmPrompt.svelte';
  import {
    isVerticalTabsRight,
    loadVerticalTabSettings,
    readVerticalTabsCached,
    verticalTabStripWidth,
    type VerticalTabSettings,
  } from '$lib/verticalTabs';
  import {
    applyFormAutofillOnLoad,
    flushFormCaptures,
  } from '$lib/formAutofill';
  import AddressBar from '$lib/components/AddressBar.svelte';
  import BrowserSidebar from '$lib/components/BrowserSidebar.svelte';
  import FindBar from '$lib/components/FindBar.svelte';
  import BookmarkBar from '$lib/components/BookmarkBar.svelte';
  import BrowserPanels from '$lib/components/BrowserPanels.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import SelectionPopup from '$lib/components/SelectionPopup.svelte';
  import BrowserContent from '$lib/components/BrowserContent.svelte';
  import { bookmarksOnBar, bookmarkFolderNames } from '$lib/bookmarks';
  import { seedPresetBookmarksIfEmpty } from '$lib/presetBookmarks';
  import { logStartup, logStartupError } from '$lib/startupLog';
  import { bindLifecycleRecovery } from '$lib/appLifecycle';
  import { focusOmniboxInput, mountBrowserShortcuts } from '$lib/browserShortcuts';
  import { enqueuePrompt } from '$lib/promptQueue';
  import { createStatusNotifier, formatStatusError, STATUS_CLEAR_MS } from '$lib/statusMessage';
  import {
    readShowBookmarkBar,
    writeShowBookmarkBar,
    type ExodusConfigDto,
    type SidecarStatusDto,
  } from '$lib/browserSettings';
  import '$lib/styles/browser-chrome.css';
  import '$lib/styles/browser-theme.css';

  const THEME_STORAGE_KEY = 'exodus-theme';
  const AUTO_INDEX_KEY = 'exodus-auto-index';
  const WELCOME_AI_KEY = 'exodus-welcome-ai-v1';
  const MAX_CLOSED_TABS = 25;

  type AiStreamMode = 'none' | 'summary' | 'chat';

  function createTabId(): string {
    return crypto.randomUUID().slice(0, 8);
  }

  const firstTabId = createTabId();
  let newTabPageUrl = $state(NEWTAB_PAGE_URL);
  let tabs = $state<BrowserTab[]>([
    { id: firstTabId, title: 'New Tab', url: NEWTAB_PAGE_URL, webview: null },
  ]);
  let activeTabId = $state(firstTabId);
  let currentUrl = $state(NEWTAB_PAGE_URL);
  let urlInput = $state(NEWTAB_PAGE_URL);
  let webviewFrame: HTMLIFrameElement | undefined = $state();
  let contentHost: HTMLDivElement | undefined = $state();
  let useNativeWebview = $state(canUseNativeWebview());
  let aiOnline = $state(false);
  let aiPort = $state(11435);
  let aiModel = $state(DEFAULT_CHAT_MODEL);
  let embeddingModel = $state('nomic-embed-text');
  let httpsOnly = $state(false);
  let privateMode = $state(false);
  let blockPopups = $state(false);
  let sessionRestore = $state(true);
  let embeddingsOnline = $state(false);
  let homepageUrl = $state('https://duckduckgo.com');
  let searchEngineUrl = $state('https://duckduckgo.com/?q={query}');
  let showSettings = $state(false);
  let settingsScrollSection = $state<SettingsScrollSection | null>(null);
  let confirmOffer = $state<ConfirmOffer | null>(null);
  let confirmBusy = $state(false);
  let pendingConfirmAction = $state<(() => Promise<void>) | null>(null);
  let verticalTabSettings = $state<VerticalTabSettings | null>(
    readVerticalTabsCached() !== null
      ? {
          enabled: readVerticalTabsCached()!,
          position: 'Left',
          width_mode: 'Auto',
          fixed_width: 250,
          show_icons: true,
          show_titles: true,
          show_close_buttons: true,
          collapse_inactive: false,
          tab_spacing: 4,
        }
      : null,
  );
  let indexedPageCount = $state<number | null>(null);
  let indexedPagesLoading = $state(false);
  let aiSidebarOpen = $state(true);
  let showBookmarkBar = $state(readShowBookmarkBar());
  let showMenu = $state(false);
  let zoomLevel = $state(100);
  let isDarkTheme = $state(true);
  let downloads = $state<DownloadRecord[]>([]);
  let extensionPermRequest = $state<ExtensionPermissionRequestEvent | null>(null);
  let extensionPermQueue: ExtensionPermissionRequestEvent[] = [];
  let extensionHostInstallRequest = $state<ExtensionHostInstallRequestEvent | null>(null);
  let extensionHostInstallQueue: ExtensionHostInstallRequestEvent[] = [];
  let browserSitePermRequest = $state<BrowserSitePermissionRequestEvent | null>(null);
  let browserSitePermQueue: BrowserSitePermissionRequestEvent[] = [];
  let showDownloadsPanel = $state(false);
  let tabBarEl: HTMLDivElement | undefined = $state();
  let tabContextMenu = $state<{ tabId: string; x: number; y: number } | null>(null);
  let canGoBack = $state(false);
  let canGoForward = $state(false);
  let closedTabs = $state<ClosedTabSnapshot[]>([]);
  let findQuery = $state('');
  let showFindBar = $state(false);
  let findResults = $state(0);
  let currentFindIndex = $state(0);
  let sidebarPanel = $state<SidebarPanel>('ai');
  let p2pRoomId = $state('lobby');
  let cdnPageStatus = $state<CdnUrlStatus | null>(null);
  let aiCdnSuggestUrls = $state<string[]>([]);
  let cdnStatusTimer: ReturnType<typeof setTimeout> | undefined;
  let selectedText = $state('');
  let aiResponse = $state('');
  let aiStreamMode = $state<AiStreamMode>('none');
  let aiChatHistory = $state<AiChatMessage[]>([]);
  let aiChatInput = $state('');
  let chatStreamBuffer = $state('');
  /** Aborts in-flight sidebar Allama HTTP streams. */
  let chatAbortController: AbortController | null = null;
  let isLoading = $state(false);
  let showSelectionPopup = $state(false);
  let popupPosition = $state({ x: 0, y: 0 });
  let showSearchResults = $state(false);
  let searchResults = $state<SearchHit[]>([]);
  let isSearching = $state(false);
  let omniboxSuggestions = $state<OmniboxSuggestion[]>([]);
  let showOmniboxSuggestions = $state(false);
  let omniboxSuggestTimer: ReturnType<typeof setTimeout> | undefined;
  let readingModeOn = $state(false);
  let tabGroups = $state<TabGroup[]>([]);
  let passwordSaveOffer = $state<PasswordCapturePayload | null>(null);
  let passwordSaveBusy = $state(false);
  let tabGroupEditOffer = $state<TabGroupEditOffer | null>(null);
  let tabGroupEditBusy = $state(false);
  let tabGroupDeleteOffer = $state<{ groupId: string; title: string } | null>(null);
  let tabGroupDeleteBusy = $state(false);
  let privacyStats = $state<PrivacyStatsSummary | null>(null);
  let newTabTopSites = $state<QuickLink[]>([]);
  let newTabWallpaperId = $state('nebula');
  let trackingProtectionEnabled = $state(true);
  let siteAllowTrackers = $state(false);
  let safeBrowsingOffer = $state<{ url: string; reason: string } | null>(null);
  let lastCapturedUrl = $state('');
  let autoIndexPages = $state(true);
  let autoIndexTimer: ReturnType<typeof setTimeout> | undefined;
  const iframeNavStacks = new Map<string, { stack: string[]; index: number }>();
  let agentPanelOpen = $state(false);
  let agentLog = $state<string[]>([]);
  let isAgentExecuting = $state(false);
  let agentCommand = $state('');
  let agentDomSummary = $state('');
  let historyPages = $state<HistoryPage[]>([]);
  let indexedPages = $state<IndexedPage[]>([]);
  let bookmarks = $state<BookmarkItem[]>([]);
  let isBookmarked = $state(false);
  let statusMessage = $state('');
  let statusClearMs = $state(STATUS_CLEAR_MS);
  let spawnSidecar = $state(false);
  let spawnAllama = $state(true);
  let sidecarStatus = $state<SidecarStatusDto | null>(null);
  const statusNotifier = createStatusNotifier(
    (m) => (statusMessage = m),
    () => statusClearMs,
  );
  const showStatus = statusNotifier.show;

  function activeLabel(): string {
    return tabWebviewLabel(activeTabId);
  }

  function activeTab(): BrowserTab | undefined {
    return tabs.find((t) => t.id === activeTabId);
  }

  /** Sync open tabs to the Web Extension registry (chrome.tabs). */
  function pushExtensionTabRegistry() {
    void syncExtensionTabs(tabs, activeTabId);
  }

  const barBookmarks = $derived(bookmarksOnBar(bookmarks));
  const bookmarkFolders = $derived(bookmarkFolderNames(bookmarks));

  /** Count of in-progress downloads for toolbar badge. */
  function activeDownloadsCount(): number {
    return downloads.filter((d) => d.status === 'downloading' || d.status === 'pending').length;
  }

  /** Tabs with pinned first, then tab groups (Chrome-style). */
  function sortedTabs(): BrowserTab[] {
    const order = sortTabsWithGroups(tabs, tabGroups, activeTabId);
    const byId = new Map(tabs.map((t) => [t.id, t]));
    return order.map((id) => byId.get(id)).filter((t): t is BrowserTab => !!t);
  }

  async function loadTabGroups() {
    tabGroups = await listTabGroups();
  }

  async function newTabGroupFromTab(tabId: string) {
    const tab = tabs.find((t) => t.id === tabId);
    const title = (tab?.title || 'Group').slice(0, 28);
    try {
      const groupId = await createTabGroup(title, 'blue');
      await addTabToGroup(groupId, tabId);
      await loadTabGroups();
      showStatus(`Tab group "${title}" created`);
    } catch (error) {
      console.error('newTabGroupFromTab failed:', error);
      showStatus(formatStatusError(error, 'Failed to create tab group'));
    }
    closeTabContextMenu();
  }

  async function addTabToExistingGroup(tabId: string, groupId: string) {
    try {
      await addTabToGroup(groupId, tabId);
      await loadTabGroups();
      showStatus('Tab added to group');
    } catch (error) {
      console.error('addTabToExistingGroup failed:', error);
      showStatus(formatStatusError(error, 'Failed to add tab to group'));
    }
    closeTabContextMenu();
  }

  async function removeTabGroupMembership(tabId: string) {
    try {
      await removeTabFromGroup(tabId);
      await loadTabGroups();
    } catch (error) {
      console.error('removeTabFromGroup failed:', error);
    }
    closeTabContextMenu();
  }

  function renameTabGroupPrompt(groupId: string) {
    const group = tabGroups.find((g) => g.id === groupId);
    if (!group) return;
    tabGroupEditOffer = {
      groupId,
      title: group.title,
      color: group.color,
    };
    closeTabContextMenu();
  }

  async function saveTabGroupEdit(title: string, color: string) {
    if (!tabGroupEditOffer) return;
    tabGroupEditBusy = true;
    const { groupId } = tabGroupEditOffer;
    try {
      await updateTabGroup(groupId, title, color);
      await loadTabGroups();
      showStatus('Tab group updated');
      tabGroupEditOffer = null;
    } catch (error) {
      console.error('saveTabGroupEdit failed:', error);
      showStatus(formatStatusError(error, 'Failed to update group'));
    } finally {
      tabGroupEditBusy = false;
    }
  }

  function cancelTabGroupEdit() {
    tabGroupEditOffer = null;
  }

  async function cycleTabGroupColor(groupId: string) {
    const group = tabGroups.find((g) => g.id === groupId);
    if (!group) return;
    const colors = TAB_GROUP_COLORS;
    const idx = colors.findIndex((c) => c === group.color.toLowerCase());
    const next = colors[(idx + 1) % colors.length];
    try {
      await updateTabGroup(groupId, undefined, next);
      await loadTabGroups();
    } catch (error) {
      console.error('cycleTabGroupColor failed:', error);
    }
    closeTabContextMenu();
  }

  function deleteTabGroupById(groupId: string) {
    const group = tabGroups.find((g) => g.id === groupId);
    if (!group) return;
    tabGroupDeleteOffer = { groupId, title: group.title };
    closeTabContextMenu();
  }

  async function confirmTabGroupDelete() {
    if (!tabGroupDeleteOffer) return;
    tabGroupDeleteBusy = true;
    const { groupId } = tabGroupDeleteOffer;
    try {
      await deleteTabGroup(groupId);
      await loadTabGroups();
      showStatus('Tab group deleted');
      tabGroupDeleteOffer = null;
    } catch (error) {
      console.error('confirmTabGroupDelete failed:', error);
      showStatus(formatStatusError(error, 'Failed to delete group'));
    } finally {
      tabGroupDeleteBusy = false;
    }
  }

  function cancelTabGroupDelete() {
    tabGroupDeleteOffer = null;
  }

  async function refreshPrivacyStats() {
    if (privateMode) {
      privacyStats = null;
      return;
    }
    privacyStats = await fetchPrivacyStats();
  }

  async function runPasswordAutofillHooks(url: string) {
    if (privateMode || !useNativeWebview || isNewTabUrl(url)) return;
    if (!url.startsWith('http://') && !url.startsWith('https://')) return;
    try {
      const settings = await loadPasswordManagerSettings();
      if (settings.auto_fill) {
        const entry = await getPasswordForPage(url);
        if (entry) {
          await applyPasswordAutofill(activeLabel(), entry);
        }
      }
      setTimeout(() => {
        void offerPasswordSave();
      }, 2500);
    } catch (error) {
      console.error('runPasswordAutofillHooks failed:', error);
    }
  }

  async function offerPasswordSave() {
    if (privateMode || !useNativeWebview) return;
    try {
      const settings = await loadPasswordManagerSettings();
      if (!settings.auto_save) return;
      const capture = await pullPasswordCapture(activeLabel());
      if (!capture) return;
      if (isNeverSavePasswordUrl(capture.url)) return;
      const existing = await getPasswordForPage(capture.url);
      if (
        existing &&
        existing.username === capture.username &&
        existing.password === capture.password
      ) {
        return;
      }
      passwordSaveOffer = capture;
    } catch (error) {
      console.error('offerPasswordSave failed:', error);
    }
  }

  async function confirmPasswordSave() {
    if (!passwordSaveOffer) return;
    passwordSaveBusy = true;
    const capture = passwordSaveOffer;
    try {
      await savePasswordCapture(capture.url, capture.username, capture.password);
      showStatus('Password saved');
      passwordSaveOffer = null;
    } catch (error) {
      console.error('confirmPasswordSave failed:', error);
      showStatus(formatStatusError(error, 'Failed to save password'));
    } finally {
      passwordSaveBusy = false;
    }
  }

  function dismissPasswordSave() {
    passwordSaveOffer = null;
  }

  function neverSavePasswordForSite() {
    if (passwordSaveOffer) {
      addNeverSavePasswordHost(passwordSaveOffer.url);
      showStatus('Passwords will not be saved for this site');
    }
    passwordSaveOffer = null;
  }

  async function flushPrivacyReports() {
    if (!useNativeWebview || privateMode) return;
    const n = await flushTrackerBlockReports(activeLabel());
    if (n > 0) {
      showStatus(`Blocked ${n} tracker request(s)`);
    }
    await refreshPrivacyStats();
  }

  async function toggleTabGroupCollapse(groupId: string, collapsed: boolean) {
    try {
      if (collapsed) {
        await collapseTabGroup(groupId);
      } else {
        await expandTabGroup(groupId);
      }
      await loadTabGroups();
    } catch (error) {
      console.error('toggleTabGroupCollapse failed:', error);
    }
    closeTabContextMenu();
  }

  /** Visit history grouped by Today / Yesterday / date. */
  function historyGroups() {
    return groupHistoryByDate(historyPages);
  }

  /** RAG indexed pages grouped by date. */
  function indexedMemoryGroups() {
    return groupIndexedByDate(indexedPages);
  }

  function openTabContextMenu(e: MouseEvent, tabId: string) {
    e.preventDefault();
    tabContextMenu = { tabId, x: e.clientX, y: e.clientY };
  }

  function closeTabContextMenu() {
    tabContextMenu = null;
  }

  function togglePinTab(tabId: string) {
    tabs = tabs.map((t) => (t.id === tabId ? { ...t, pinned: !t.pinned } : t));
    closeTabContextMenu();
    scrollActiveTabIntoView();
  }

  /** Scroll the active tab chip into view in the tab bar. */
  function scrollActiveTabIntoView() {
    requestAnimationFrame(() => {
      tabBarEl?.querySelector('.tab-item.active')?.scrollIntoView({
        inline: 'nearest',
        block: 'nearest',
        behavior: 'smooth',
      });
    });
  }

  function onTabMouseDown(e: MouseEvent, tabId: string) {
    if (e.button === 1) {
      e.preventDefault();
      const tab = tabs.find((t) => t.id === tabId);
      if (!tab?.pinned) void closeTab(tabId);
    }
  }

  async function duplicateTab(id: string) {
    const tab = tabs.find((t) => t.id === id);
    if (!tab) return;
    const newId = createTabId();
    tabs = [
      ...tabs,
      {
        id: newId,
        title: tab.title,
        url: tab.url,
        webview: null,
        favicon: faviconUrlFor(tab.url),
      },
    ];
    await switchTab(newId);
    if (useNativeWebview && contentHost) {
      const wv = await createTabWebview(contentHost, tabWebviewLabel(newId), tab.url);
      tabs = tabs.map((t) => (t.id === newId ? { ...t, webview: wv } : t));
    }
    scrollActiveTabIntoView();
  }

  function persistActiveTabUrl() {
    tabs = tabs.map((t) =>
      t.id === activeTabId ? { ...t, url: currentUrl, title: t.title === 'New Tab' ? currentUrl : t.title } : t,
    );
  }

  async function ensureActiveWebview(): Promise<Webview | null> {
    if (!useNativeWebview || !contentHost) return null;
    const tab = activeTab();
    if (!tab) return null;

    const label = activeLabel();
    if (await isTabDiscarded(label)) {
      const restored = await restoreDiscardedTab(contentHost, label);
      if (restored) {
        await setTabPopupBlocking(label, blockPopups);
        tabs = tabs.map((t) => (t.id === activeTabId ? { ...t, webview: restored } : t));
        return restored;
      }
    }

    if (tab.webview) {
      await showTabWebview(tab.webview, contentHost);
      return tab.webview;
    }

    const wv = await createTabWebview(contentHost, label, tab.url);
    await setTabPopupBlocking(activeLabel(), blockPopups);
    tabs = tabs.map((t) => (t.id === activeTabId ? { ...t, webview: wv } : t));
    return wv;
  }

  /** Open a URL in a new tab (used for window.open / target=_blank routing). */
  async function openUrlInNewTab(url: string) {
    const targetUrl = applyHttpsOnly(url, httpsOnly);
    const id = createTabId();
    tabs = [
      ...tabs,
      {
        id,
        title: 'New Tab',
        url: targetUrl,
        webview: null,
        favicon: faviconUrlFor(targetUrl),
      },
    ];
    await switchTab(id);
    if (useNativeWebview && contentHost) {
      const label = tabWebviewLabel(id);
      const wv = await createTabWebview(contentHost, label, targetUrl);
      await setTabPopupBlocking(label, blockPopups);
      tabs = tabs.map((t) => (t.id === id ? { ...t, webview: wv } : t));
      for (const t of tabs) {
        if (t.id !== id && t.webview) await hideTabWebview(t.webview);
      }
    } else if (webviewFrame) {
      webviewFrame.src = targetUrl;
    }
    currentUrl = targetUrl;
    urlInput = targetUrl;
    void recordVisit(targetUrl);
    void saveSession();
    scrollActiveTabIntoView();
    void registerTabLifecycle({ id, url: targetUrl, title: 'New Tab' });
  }

  async function switchTab(id: string) {
    if (id === activeTabId) return;

    persistActiveTabUrl();

    const prev = tabs.find((t) => t.id === activeTabId);
    if (prev?.webview && useNativeWebview) {
      await flushExtensionTab(tabWebviewLabel(prev.id));
      await hideTabWebview(prev.webview);
    }

    activeTabId = id;
    const tab = tabs.find((t) => t.id === id);
    if (!tab) return;

    void saveSession();

    currentUrl = tab.url;
    urlInput = tab.url;

    if (useNativeWebview && contentHost) {
      await ensureActiveWebview();
    } else if (webviewFrame) {
      webviewFrame.src = applyHttpsOnly(tab.url, httpsOnly);
    }
    scrollActiveTabIntoView();
    pushExtensionTabRegistry();
    void markTabActiveLifecycle(id);
  }

  async function newTab() {
    const id = createTabId();
    tabs = [...tabs, { id, title: 'New Tab', url: newTabPageUrl, webview: null }];
    await switchTab(id);
    if (useNativeWebview && contentHost) {
      const label = tabWebviewLabel(id);
      const wv = await createTabWebview(contentHost, label, newTabPageUrl);
      await setTabPopupBlocking(label, blockPopups);
      tabs = tabs.map((t) => (t.id === id ? { ...t, webview: wv } : t));
      for (const t of tabs) {
        if (t.id !== id && t.webview) await hideTabWebview(t.webview);
      }
    }
    currentUrl = newTabPageUrl;
    urlInput = newTabPageUrl;
    scrollActiveTabIntoView();
    pushExtensionTabRegistry();
    void registerTabLifecycle({ id, url: newTabPageUrl, title: 'New Tab' });
  }

  /** Map chrome tab id (1-based index) to internal tab id. */
  function tabIdFromChromeId(chromeTabId: number): string | undefined {
    const idx = chromeTabId - 1;
    return tabs[idx]?.id;
  }

  /** Handle extension tab ops (update / remove / reload). */
  async function handleExtensionTabOps(ops: ExtensionTabOp[]) {
    for (const op of ops) {
      if (op.op === 'remove') {
        for (const chromeId of op.tabIds ?? []) {
          const id = tabIdFromChromeId(chromeId);
          if (id) await closeTab(id, true);
        }
      } else if (op.op === 'reload') {
        const id = tabIdFromChromeId(op.chromeTabId ?? 0);
        if (id && useNativeWebview) {
          await reloadTab(tabWebviewLabel(id));
        }
      } else if (op.op === 'update') {
        const id = tabIdFromChromeId(op.chromeTabId ?? 0);
        if (!id) continue;
        if (op.updateProperties?.active) await switchTab(id);
        if (op.updateProperties?.url && useNativeWebview) {
          const url = op.updateProperties.url;
          if (op.extensionId) {
            const allowed = await invoke<boolean>('extension_validate_host_access', {
              extensionId: op.extensionId,
              url,
            });
            if (!allowed) {
              showStatus(`Extension blocked navigation to ${url}`);
              continue;
            }
          }
          await navigateTab(tabWebviewLabel(id), url);
        }
      }
    }
    pushExtensionTabRegistry();
  }

  async function closeTab(id: string, force = false) {
    if (tabs.length <= 1) return;
    const tab = tabs.find((t) => t.id === id);
    if (tab?.pinned && !force) return;
    if (tab) {
      closedTabs = [
        { title: tab.title, url: tab.url, pinned: tab.pinned },
        ...closedTabs,
      ].slice(0, MAX_CLOSED_TABS);
    }
    if (tab?.webview && useNativeWebview) {
      await flushExtensionTab(tabWebviewLabel(id));
      await closeTabWebview(tabWebviewLabel(id));
    }
    void unregisterTabLifecycle(id);
    iframeNavStacks.delete(id);
    void removeTabFromGroup(id);
    const wasActive = activeTabId === id;
    tabs = tabs.filter((t) => t.id !== id);
    if (wasActive) {
      await switchTab(tabs[tabs.length - 1].id);
    }
    void saveSession();
    pushExtensionTabRegistry();
  }

  /** Restore the most recently closed tab (⌘⇧T). */
  async function restoreClosedTab() {
    const snap = closedTabs[0];
    if (!snap) {
      showStatus('No recently closed tabs');
      return;
    }
    closedTabs = closedTabs.slice(1);
    const id = createTabId();
    tabs = [
      ...tabs,
      {
        id,
        title: snap.title,
        url: snap.url,
        webview: null,
        pinned: snap.pinned,
        favicon: faviconUrlFor(snap.url),
      },
    ];
    await switchTab(id);
    checkIfBookmarked();
    showStatus('Restored closed tab');
    void registerTabLifecycle({ id, url: snap.url, title: snap.title, pinned: snap.pinned });
  }

  function openShieldsSettings() {
    openSettingsSection('privacy');
  }

  async function refreshSiteShieldForUrl(url: string) {
    const host = hostFromPageUrl(url);
    if (!host || isNewTabUrl(url)) {
      siteAllowTrackers = false;
      return;
    }
    siteAllowTrackers = await getSiteShieldAllowTrackers(host);
  }

  async function toggleSiteShieldAllowTrackers() {
    const host = hostFromPageUrl(currentUrl);
    if (!host) return;
    const next = !siteAllowTrackers;
    try {
      await setSiteShieldAllowTrackers(host, next);
      siteAllowTrackers = next;
      showStatus(next ? `Shields down for ${host}` : `Shields up for ${host}`);
      if (useNativeWebview && activeTab()?.webview) {
        await reloadTab(activeLabel());
      }
    } catch (error) {
      showStatus(formatStatusError(error, 'Failed to update site shields'));
    }
  }

  /** Recreate all tab webviews after private-mode toggle (incognito flag). */
  async function recreateWebviewsForPrivateMode() {
    if (!useNativeWebview || !contentHost) return;
    const snapshot = tabs.map((t) => ({ id: t.id, url: t.url, title: t.title }));
    for (const tab of tabs) {
      if (tab.webview) {
        await closeTabWebview(tabWebviewLabel(tab.id));
      }
    }
    tabs = snapshot.map((t) => ({ ...t, webview: null }));
    await switchTab(activeTabId);
    for (const t of tabs) {
      if (t.id === activeTabId) continue;
      if (isNewTabUrl(t.url)) continue;
      const label = tabWebviewLabel(t.id);
      const wv = await createTabWebview(contentHost, label, t.url);
      tabs = tabs.map((row) => (row.id === t.id ? { ...row, webview: wv } : row));
      await hideTabWebview(wv);
    }
  }

  async function refreshNewTabTopSites() {
    try {
      const recent = await getRecentManagedHistory(80);
      newTabTopSites = buildTopSitesFromHistory(recent, 8);
    } catch (error) {
      console.error('refreshNewTabTopSites failed:', error);
      newTabTopSites = [];
    }
  }

  function openSettingsSection(section: SettingsScrollSection) {
    settingsScrollSection = section;
    showSettings = true;
  }

  function openConfirmDialog(offer: ConfirmOffer, action: () => Promise<void>) {
    confirmOffer = offer;
    pendingConfirmAction = action;
  }

  function cancelConfirmDialog() {
    confirmOffer = null;
    pendingConfirmAction = null;
    confirmBusy = false;
  }

  async function runConfirmDialog() {
    const action = pendingConfirmAction;
    if (!action) return;
    confirmBusy = true;
    try {
      await action();
    } finally {
      cancelConfirmDialog();
    }
  }

  function requestClearVisitHistory() {
    openConfirmDialog(
      {
        title: 'Clear browsing history?',
        message:
          'Removes sidebar visit history and the full history store. This cannot be undone.',
        confirmLabel: 'Clear history',
        danger: true,
      },
      async () => {
        try {
          await invoke('clear_visit_history');
          await clearAllManagedHistory();
          historyPages = [];
          showStatus('Browsing history cleared');
        } catch (error) {
          console.error('Clear history failed:', error);
          showStatus('Failed to clear browsing history');
        }
      },
    );
  }

  async function clearVisitHistory() {
    requestClearVisitHistory();
  }

  function applyVerticalTabLayout(settings: VerticalTabSettings) {
    verticalTabSettings = settings;
  }

  const verticalTabsOn = $derived(verticalTabSettings?.enabled ?? false);
  const verticalTabWidth = $derived(
    verticalTabSettings ? verticalTabStripWidth(verticalTabSettings) : 220,
  );
  const verticalTabsRight = $derived(
    verticalTabSettings ? isVerticalTabsRight(verticalTabSettings) : false,
  );

  function applyIframeNavFlags(tabId: string) {
    const flags = navFlagsFromTrack(iframeNavStacks.get(tabId));
    canGoBack = flags.canGoBack;
    canGoForward = flags.canGoForward;
  }

  function trackIframeNavigation(tabId: string, url: string) {
    if (isNewTabUrl(url)) return;
    recordTabNavigation(iframeNavStacks, tabId, url);
    if (tabId === activeTabId) {
      applyIframeNavFlags(tabId);
    }
  }

  async function refreshNavState() {
    if (!useNativeWebview) {
      applyIframeNavFlags(activeTabId);
      try {
        const href = webviewFrame?.contentWindow?.location.href;
        if (href && !isNewTabUrl(href)) {
          trackIframeNavigation(activeTabId, href);
        }
      } catch {
        /* cross-origin */
      }
      return;
    }
    try {
      const state = await getTabNavState(activeLabel());
      canGoBack = state.can_go_back;
      canGoForward = state.can_go_forward;
      if (state.url && state.url !== currentUrl && !isNewTabUrl(state.url)) {
        currentUrl = state.url;
        urlInput = state.url;
      }
    } catch {
      /* webview not ready */
    }
  }

  /** Persist a browsing-history visit (local DB). */
  async function recordVisit(url = currentUrl, title?: string) {
    if (privateMode) return;
    if (isNewTabUrl(url) || (!url.startsWith('http://') && !url.startsWith('https://'))) return;
    const pageTitle = title || activeTab()?.title || url;
    try {
      await invoke('record_visit', { url, title: pageTitle });
      void addManagedHistoryEntry(url, pageTitle);
      void syncSuggestionHistory(url, pageTitle);
    } catch (error) {
      console.error('record_visit failed:', error);
    }
  }

  async function runFormAutofillHooks(url: string) {
    if (privateMode || !useNativeWebview || isNewTabUrl(url)) return;
    if (!url.startsWith('http://') && !url.startsWith('https://')) return;
    try {
      await applyFormAutofillOnLoad(activeLabel(), url);
      setTimeout(() => {
        void flushFormCaptures(activeLabel()).then((n) => {
          if (n > 0) showStatus(`Saved ${n} form field(s)`);
        });
      }, 3500);
    } catch (error) {
      console.error('runFormAutofillHooks failed:', error);
    }
  }

  function scheduleOmniboxSuggestions() {
    if (omniboxSuggestTimer) clearTimeout(omniboxSuggestTimer);
    omniboxSuggestTimer = setTimeout(() => {
      void refreshOmniboxSuggestions();
    }, 200);
  }

  async function refreshOmniboxSuggestions() {
    const input = urlInput.trim();
    if (!input || input.startsWith('/ask ')) {
      omniboxSuggestions = [];
      showOmniboxSuggestions = false;
      return;
    }
    const rows = await fetchOmniboxSuggestions(input, 8);
    omniboxSuggestions = rows;
    showOmniboxSuggestions = rows.length > 0 && !showSearchResults;
  }

  async function selectOmniboxSuggestion(url: string) {
    urlInput = url;
    showOmniboxSuggestions = false;
    omniboxSuggestions = [];
    await navigate();
  }

  async function translateCurrentPage() {
    showMenu = false;
    try {
      let snippet = '';
      if (useNativeWebview) {
        snippet = await evalTabReturning(
          activeLabel(),
          `(document.body?.innerText || '').slice(0, 2500)`,
        );
      } else if (webviewFrame?.contentDocument?.body) {
        snippet = (webviewFrame.contentDocument.body.innerText || '').slice(0, 2500);
      }
      if (!snippet.trim()) {
        showStatus('Nothing to translate on this page');
        return;
      }
      const result = await translateText(snippet, 'en');
      aiChatHistory = [
        ...aiChatHistory,
        { role: 'assistant', content: result.translated_text },
      ];
      sidebarPanel = 'ai';
      aiSidebarOpen = true;
      showStatus('Page translated — see AI sidebar');
    } catch (error) {
      console.error('translateCurrentPage failed:', error);
      showStatus(formatStatusError(error, 'Translation failed'));
    }
  }

  async function toggleReadingMode() {
    showMenu = false;
    try {
      const css = await fetchReadingModeCss();
      const script = `(function(){
        const id='exodus-reading-mode-style';
        const existing=document.getElementById(id);
        if(existing){ existing.remove(); return 'off'; }
        const s=document.createElement('style');
        s.id=id;
        s.textContent=${JSON.stringify(css)};
        document.head.appendChild(s);
        return 'on';
      })()`;
      let nextOn = !readingModeOn;
      if (useNativeWebview) {
        const result = await evalTabReturning(activeLabel(), script);
        nextOn = result === 'on';
      } else if (webviewFrame?.contentDocument) {
        const doc = webviewFrame.contentDocument;
        const existing = doc.getElementById('exodus-reading-mode-style');
        if (existing) {
          existing.remove();
          nextOn = false;
        } else {
          const style = doc.createElement('style');
          style.id = 'exodus-reading-mode-style';
          style.textContent = css;
          doc.head.appendChild(style);
          nextOn = true;
        }
      }
      readingModeOn = nextOn;
      if (nextOn) {
        await enableReadingModeForUrl(currentUrl);
        showStatus('Reading mode on');
      } else {
        await disableReadingModeForUrl(currentUrl);
        showStatus('Reading mode off');
      }
    } catch (error) {
      console.error('toggleReadingMode failed:', error);
      showStatus(formatStatusError(error, 'Reading mode failed'));
    }
  }

  async function goBack() {
    if (useNativeWebview) {
      await goBackTab(activeLabel());
      setTimeout(() => {
        void refreshNavState();
        void refreshActiveTabTitle();
        void recordVisit();
        void scheduleAutoIndex();
      }, 400);
      return;
    }
    const track = iframeNavStacks.get(activeTabId);
    if (track && track.index > 0) {
      track.index -= 1;
      applyIframeNavFlags(activeTabId);
    }
    try {
      webviewFrame?.contentWindow?.history.back();
      setTimeout(() => {
        void refreshNavState();
        void scheduleAutoIndex();
      }, 300);
    } catch {
      /* cross-origin */
    }
  }

  async function goForward() {
    if (useNativeWebview) {
      await goForwardTab(activeLabel());
      setTimeout(() => {
        void refreshNavState();
        void refreshActiveTabTitle();
        void recordVisit();
        void scheduleAutoIndex();
      }, 400);
      return;
    }
    const track = iframeNavStacks.get(activeTabId);
    if (track && track.index + 1 < track.stack.length) {
      track.index += 1;
      applyIframeNavFlags(activeTabId);
    }
    try {
      webviewFrame?.contentWindow?.history.forward();
      setTimeout(() => {
        void refreshNavState();
        void scheduleAutoIndex();
      }, 300);
    } catch {
      /* cross-origin */
    }
  }

  async function reloadPage() {
    if (useNativeWebview) {
      await reloadTab(activeLabel());
      return;
    }
    if (webviewFrame) webviewFrame.src = currentUrl;
  }

  function onFrameLoad() {
    urlInput = currentUrl;
    trackIframeNavigation(activeTabId, currentUrl);
    void refreshActiveTabTitle();
    void recordVisit();
    void refreshNavState();
    void scheduleAutoIndex();
  }

  /** Update the active tab label from document.title. */
  async function refreshActiveTabTitle() {
    if (isNewTabUrl(currentUrl)) {
      document.title = 'New Tab - Exodus Browser';
      return;
    }
    try {
      let title = '';
      if (useNativeWebview) {
        title = await getTabTitle(activeTabId);
      } else if (webviewFrame?.contentDocument) {
        title = webviewFrame.contentDocument.title.trim();
      }
      if (!title) {
        document.title = 'Exodus Browser';
        return;
      }
      const short = title.length > 48 ? `${title.slice(0, 45)}…` : title;
      tabs = tabs.map((t) =>
        t.id === activeTabId ? { ...t, title: short, favicon: faviconUrlFor(currentUrl) } : t,
      );
      document.title = `${title} - Exodus Browser`;
    } catch {
      // Ignore errors
    }
  }

  function zoomIn() {
    if (zoomLevel < 200) {
      zoomLevel += 10;
      applyZoom();
    }
  }

  function zoomOut() {
    if (zoomLevel > 50) {
      zoomLevel -= 10;
      applyZoom();
    }
  }

  function resetZoom() {
    zoomLevel = 100;
    applyZoom();
  }

  async function applyZoom() {
    const scale = zoomLevel / 100;
    if (useNativeWebview) {
      try {
        await setTabZoom(activeLabel(), scale);
      } catch (e) {
        console.error('Native zoom failed:', e);
      }
      return;
    }
    if (webviewFrame) {
      webviewFrame.style.transform = `scale(${scale})`;
      webviewFrame.style.transformOrigin = 'top left';
      webviewFrame.style.width = scale !== 1 ? `${100 / scale}%` : '100%';
      webviewFrame.style.height = scale !== 1 ? `${100 / scale}%` : '100%';
    }
  }

  /** Safe Browsing gate before loading a URL. */
  async function ensureNavigationAllowed(url: string): Promise<boolean> {
    if (isNewTabUrl(url) || url.startsWith('extension://') || url.startsWith('data:')) {
      return true;
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      return true;
    }
    const guard = await checkNavigationGuard(url);
    if (guard.allowed) return true;
    void recordMaliciousSiteBlocked(url);
    if (guard.canProceed) {
      safeBrowsingOffer = { url, reason: guard.reason };
      return false;
    }
    showStatus(guard.reason);
    return false;
  }

  function cancelSafeBrowsing() {
    safeBrowsingOffer = null;
  }

  async function proceedSafeBrowsing() {
    const offer = safeBrowsingOffer;
    safeBrowsingOffer = null;
    if (!offer) return;
    await commitNavigation(offer.url);
  }

  async function commitNavigation(targetUrl: string) {
    currentUrl = targetUrl;
    urlInput = targetUrl;
    showSearchResults = false;
    showOmniboxSuggestions = false;
    omniboxSuggestions = [];
    tabs = tabs.map((t) =>
      t.id === activeTabId
        ? {
            ...t,
            url: currentUrl,
            title: t.title === 'New Tab' ? currentUrl : t.title,
            favicon: faviconUrlFor(currentUrl),
          }
        : t,
    );
    checkIfBookmarked();

    trackIframeNavigation(activeTabId, currentUrl);

    if (useNativeWebview) {
      await navigateTab(activeLabel(), currentUrl);
      setTimeout(() => {
        void flushExtensionTab(activeLabel());
        void refreshActiveTabTitle();
        void refreshNavState();
        void scheduleAutoIndex();
      }, 1500);
    } else if (webviewFrame) {
      webviewFrame.src = currentUrl;
    }
    void recordVisit(currentUrl);
    void saveSession();
    pushExtensionTabRegistry();
    void runPasswordAutofillHooks(targetUrl);
    void runFormAutofillHooks(targetUrl);
    void refreshSiteShieldForUrl(targetUrl);
    setTimeout(() => {
      void flushPrivacyReports();
    }, 3000);
  }

  async function navigate() {
    const resolved = resolveOmniboxInput(urlInput, searchEngineUrl);
    if (!resolved) return;

    if (resolved.kind === 'ask') {
      await performSemanticSearch(resolved.query);
      return;
    }

    const targetUrl = applyHttpsOnly(resolved.url, httpsOnly);
    if (!(await ensureNavigationAllowed(targetUrl))) return;
    await commitNavigation(targetUrl);
  }

  async function navigateToResult(url: string) {
    urlInput = url;
    showSearchResults = false;
    if (!(await ensureNavigationAllowed(url))) return;
    await commitNavigation(url);
  }

  async function navigateToBookmark(url: string) {
    urlInput = url;
    if (!(await ensureNavigationAllowed(url))) return;
    await commitNavigation(url);
  }

  async function removeBookmark(id: string) {
    try {
      await invoke('remove_bookmark', { id });
      await loadBookmarks();
      showStatus('Bookmark removed successfully');
    } catch (error) {
      console.error('Remove bookmark failed:', error);
      showStatus(formatStatusError(error, 'Failed to remove bookmark'));
    }
  }

  /** Load RAG indexed pages (`get_history`) for sidebar and settings count. */
  async function loadIndexedMemory() {
    try {
      const pages = (await invoke('get_history')) as IndexedPage[];
      indexedPages = pages;
      indexedPageCount = pages.length;
    } catch (error) {
      console.error('Failed to load indexed memory:', error);
      indexedPages = [];
      indexedPageCount = null;
    }
  }

  /** Refresh indexed count for settings (shows loading state). */
  async function refreshIndexedMemoryCount() {
    indexedPagesLoading = true;
    try {
      await loadIndexedMemory();
    } finally {
      indexedPagesLoading = false;
    }
  }

  function requestClearIndexedMemory() {
    openConfirmDialog(
      {
        title: 'Clear local memory?',
        message: 'Removes all RAG-indexed pages used for /ask search. This cannot be undone.',
        confirmLabel: 'Clear memory',
        danger: true,
      },
      async () => {
        try {
          await invoke('clear_rag_data');
          indexedPages = [];
          indexedPageCount = 0;
          showStatus('Local memory cleared successfully');
        } catch (error) {
          console.error('Failed to clear local memory:', error);
          showStatus(formatStatusError(error, 'Failed to clear local memory'));
        }
      },
    );
  }

  async function clearIndexedMemory() {
    requestClearIndexedMemory();
  }

  async function removeIndexedPage(id: string) {
    try {
      await invoke('delete_indexed_page', { id });
      indexedPages = indexedPages.filter((p) => p.id !== id);
      indexedPageCount = indexedPages.length;
      showStatus('Removed from indexed memory');
    } catch (error) {
      console.error('Failed to remove indexed page:', error);
      showStatus(formatStatusError(error, 'Failed to remove page'));
    }
  }

  async function clearLocalMemory() {
    await clearIndexedMemory();
    showSettings = false;
  }

  /** Debounced background RAG index after navigation (when enabled in settings). */
  function scheduleAutoIndex() {
    if (privateMode || !autoIndexPages || isNewTabUrl(currentUrl)) return;
    if (autoIndexTimer) clearTimeout(autoIndexTimer);
    autoIndexTimer = setTimeout(() => {
      void maybeAutoIndexPage();
    }, 2000);
  }

  async function maybeAutoIndexPage() {
    if (!autoIndexPages || isNewTabUrl(currentUrl)) return;
    const target =
      currentUrl.startsWith('http://') || currentUrl.startsWith('https://') ? currentUrl : '';
    if (!target || target === lastCapturedUrl) return;
    await capturePageContent(true);
  }

  async function capturePageContent(silent = false) {
    if (isNewTabUrl(currentUrl)) return;
    try {
      if (useNativeWebview) {
        const captured = await captureTabPage(activeLabel());
        await invoke('capture_page', {
          url: captured.url || currentUrl,
          title: captured.title || currentUrl,
          content: captured.content,
        });
        const pageUrl = captured.url || currentUrl;
        const byteLen = new TextEncoder().encode(captured.content).length;
        void maybeAnnounceIndexedPage(
          pageUrl,
          captured.title || currentUrl,
          byteLen,
        ).then((announced) => {
          if (announced) showStatus('Large page announced to P2P CDN lobby');
        });
        lastCapturedUrl = pageUrl;
        if (captured.url && captured.url !== currentUrl) {
          currentUrl = captured.url;
          urlInput = captured.url;
        }
        if (captured.title) {
          tabs = tabs.map((t) =>
            t.id === activeTabId ? { ...t, title: captured.title, url: captured.url || t.url } : t,
          );
        }
        if (!silent) showStatus('Page indexed to local memory');
        void loadIndexedMemory();
        return;
      }

      const iframe = webviewFrame;
      if (!iframe?.contentDocument) return;

      const content = (iframe.contentDocument.body?.innerText || '').substring(0, 10000);
      await invoke('capture_page', {
        url: currentUrl,
        title: iframe.contentDocument.title || currentUrl,
        content,
      });
      void maybeAnnounceIndexedPage(
        currentUrl,
        iframe.contentDocument.title || currentUrl,
        new TextEncoder().encode(content).length,
      ).then((announced) => {
        if (announced) showStatus('Large page announced to P2P CDN lobby');
      });
      lastCapturedUrl = currentUrl;
      if (!silent) showStatus('Page indexed (iframe)');
      void loadIndexedMemory();
    } catch (error) {
      console.error('Failed to capture page:', error);
      if (!silent) showStatus('Index failed — open page in Tauri native view');
    }
  }

  async function performSemanticSearch(query: string) {
    isSearching = true;
    showSearchResults = true;
    searchResults = [];
    try {
      searchResults = (await invoke('semantic_search', { query })) as SearchHit[];
    } catch (error) {
      console.error('Search failed:', error);
      showStatus(formatStatusError(error, 'Memory search failed'));
    } finally {
      isSearching = false;
    }
  }

  async function loadHistory(): Promise<HistoryPage[]> {
    try {
      historyPages = await loadMergedBrowsingHistory();
      void refreshNewTabTopSites();
      return historyPages;
    } catch (error) {
      console.error('Failed to load history:', error);
      historyPages = [];
      return [];
    }
  }

  async function updateBookmarkFolder(id: string, folder: string) {
    try {
      await invoke('update_bookmark_folder', { id, folder: folder.trim() });
      await loadBookmarks();
      showStatus(folder.trim() ? `Moved to folder “${folder.trim()}”` : 'Moved to bookmark bar');
    } catch (error) {
      console.error('Update bookmark folder failed:', error);
      showStatus('Failed to update bookmark folder');
    }
  }

  async function loadBookmarks() {
    try {
      bookmarks = (await invoke('list_bookmarks')) as BookmarkItem[];
      checkIfBookmarked();
      for (const b of bookmarks) {
        void syncSuggestionBookmark(b.url, b.title);
      }
    } catch (error) {
      console.error('Failed to load bookmarks:', error);
      bookmarks = [];
    }
  }

  async function reorderBookmarkBar(orderedIds: string[]) {
    try {
      await invoke('reorder_bookmarks_bar', { orderedIds });
      await loadBookmarks();
    } catch (error) {
      console.error('reorder_bookmarks_bar failed:', error);
      showStatus('Failed to reorder bookmarks');
    }
  }

  function checkIfBookmarked() {
    isBookmarked = bookmarks.some((b) => b.url === currentUrl);
  }

  async function bookmarkCurrentPage() {
    const title = activeTab()?.title || currentUrl;
    try {
      await invoke('add_bookmark', { url: currentUrl, title, folder: '' });
      void syncSuggestionBookmark(currentUrl, title);
      await loadBookmarks();
      showStatus('Bookmark saved');
    } catch (error) {
      console.error('Add bookmark failed:', error);
      showStatus(formatStatusError(error, 'Failed to save bookmark'));
    }
  }

  async function toggleBookmark() {
    showMenu = false;
    const existing = bookmarks.find((b) => b.url === currentUrl);
    if (existing) {
      await removeBookmark(existing.id);
      checkIfBookmarked();
    } else {
      await bookmarkCurrentPage();
    }
  }

  function openBookmarksPanel() {
    showMenu = false;
    openPanel('bookmarks');
  }

  function openHistoryPanel() {
    showMenu = false;
    openPanel('memory');
  }

  function toggleSidebar() {
    aiSidebarOpen = !aiSidebarOpen;
    setTimeout(() => {
      const tab = activeTab();
      if (tab?.webview && contentHost) {
        showTabWebview(tab.webview, contentHost).catch(console.error);
      }
    }, 350);
  }

  function openPanel(panel: SidebarPanel) {
    sidebarPanel = panel;
    aiSidebarOpen = true;
    agentPanelOpen = panel === 'ai' ? agentPanelOpen : false;
    if (panel === 'memory') {
      void loadIndexedMemory();
      void loadHistory();
    }
    if (panel === 'bookmarks') loadBookmarks();
  }

  /** Announce a URL to the shared P2P CDN room (lobby or group id). */
  async function announceUrlToCdn(url: string, title: string) {
    try {
      await announcePageUrlToCdn(url, title, p2pRoomId);
      showStatus(`Announced to P2P CDN · ${p2pRoomId}`);
      void refreshCdnPageStatus();
    } catch (error) {
      console.error('announcePageUrlToCdn failed:', error);
      showStatus(formatStatusError(error, 'P2P CDN announce failed'));
    }
  }

  async function announceCurrentPageToCdn() {
    const title = activeTab()?.title || currentUrl;
    await announceUrlToCdn(currentUrl, title);
    void refreshCdnPageStatus();
  }

  async function refreshCdnPageStatus() {
    cdnPageStatus = await fetchCdnPageStatus(currentUrl, p2pRoomId);
  }

  function scheduleCdnPageStatusRefresh() {
    if (cdnStatusTimer) clearTimeout(cdnStatusTimer);
    cdnStatusTimer = setTimeout(() => {
      void refreshCdnPageStatus();
    }, 600);
  }

  async function announceAllAiCdnUrls() {
    const urls = [...aiCdnSuggestUrls];
    for (const url of urls) {
      await announceUrlToCdn(url, url);
    }
    aiCdnSuggestUrls = [];
    showStatus(`Announced ${urls.length} link(s) to P2P CDN`);
    void refreshCdnPageStatus();
  }

  function handleAiReplyForCdn(content: string) {
    const urls = suggestUrlsForCdnAnnounce(content);
    if (urls.length > 0) {
      aiCdnSuggestUrls = urls;
      showStatus(`AI shared ${urls.length} large-file link(s) — use “Announce all” in sidebar`);
    }
  }

  /** Refresh Allama / embeddings status when settings open. */
  $effect(() => {
    if (showSettings) {
      void refreshAiOnline();
    }
  });

  $effect(() => {
    if (typeof window === 'undefined') return;
    void currentUrl;
    void p2pRoomId;
    scheduleCdnPageStatusRefresh();
  });

  function toggleAgentPanel() {
    agentPanelOpen = !agentPanelOpen;
    sidebarPanel = 'ai';
  }

  async function compressCurrentDom() {
    try {
      const html = useNativeWebview
        ? await getTabHtml(activeLabel())
        : webviewFrame?.contentDocument?.documentElement.outerHTML || '';

      if (!html) {
        addAgentLog('Cannot read DOM');
        return;
      }

      const compressed = await invoke<string>('compress_dom', { html, url: currentUrl });
      agentDomSummary = summarizeCompressedDom(compressed);
      addAgentLog(agentDomSummary);
    } catch (error) {
      addAgentLog(`DOM compression failed: ${error}`);
    }
  }

  async function runAgentAction(actionJson: string) {
    isAgentExecuting = true;
    addAgentLog(`Executing: ${actionJson.substring(0, 120)}`);

    try {
      const jsCode = await invoke<string>('execute_agent_action_with_context', {
        actionJson,
        currentUrl,
      });

      const returnsValue = agentActionReturnsValue(actionJson);

      if (useNativeWebview) {
        if (returnsValue) {
          const result = await evalTabReturning(activeLabel(), jsCode);
          const preview = result.length > 1200 ? `${result.slice(0, 1200)}…` : result;
          addAgentLog(preview || '(empty)');
        } else {
          await evalInTab(activeLabel(), jsCode);
          addAgentLog(`Ran: ${jsCode.substring(0, 80)}`);
        }
      } else {
        executeInIframe(jsCode);
        addAgentLog(returnsValue ? 'Ran (iframe — result may be cross-origin)' : `Ran: ${jsCode.substring(0, 80)}`);
      }
    } catch (error) {
      addAgentLog(`Execution failed: ${error}`);
    } finally {
      isAgentExecuting = false;
    }
  }

  async function executeAgentCommand() {
    const cmd = agentCommand.trim();
    if (!cmd) return;
    const askMatch = /^ask[:\s]+(.+)$/i.exec(cmd);
    if (askMatch?.[1]?.trim()) {
      agentCommand = '';
      await askAgentWithAllama(askMatch[1].trim());
      return;
    }
    const actionJson =
      parseAgentCommandInput(cmd) ?? (cmd.startsWith('{') ? cmd : null);
    if (!actionJson) {
      addAgentLog('Use JSON or: scroll down / scroll up / get text / links / ask: question');
      return;
    }
    agentCommand = '';
    await runAgentAction(actionJson);
  }

  /** Open AI chat with agent DOM context (or command text as the question). */
  async function askAgentWithAllama(question: string) {
    agentPanelOpen = false;
    sidebarPanel = 'ai';
    aiSidebarOpen = true;
    const prompt = agentDomSummary
      ? `Page context:\n${agentDomSummary}\n\nUser question: ${question}`
      : question;
    aiChatInput = '';
    const historyWithUser: AiChatMessage[] = [
      ...aiChatHistory,
      { role: 'user', content: prompt },
    ];
    aiChatHistory = historyWithUser;
    chatStreamBuffer = '';
    aiStreamMode = 'chat';
    isLoading = true;
    chatAbortController?.abort();
    chatAbortController = new AbortController();
    const signal = chatAbortController.signal;
    if (!aiOnline) {
      aiChatHistory = [
        ...aiChatHistory,
        {
          role: 'assistant',
          content: 'Error: Allama is offline — open Settings and start Allama (port 11435)',
        },
      ];
      isLoading = false;
      aiStreamMode = 'none';
      chatAbortController = null;
      return;
    }
    await streamSidebarChat(
      historyWithUser,
      { port: aiPort, model: aiModel, signal },
      {
        onChunk: (content) => {
          chatStreamBuffer += content;
        },
        onDone: () => {
          if (chatStreamBuffer.trim()) {
            const reply = chatStreamBuffer;
            aiChatHistory = [...aiChatHistory, { role: 'assistant', content: reply }];
            handleAiReplyForCdn(reply);
          }
          chatStreamBuffer = '';
          isLoading = false;
          aiStreamMode = 'none';
          chatAbortController = null;
        },
        onError: (message) => {
          if (signal.aborted) {
            chatStreamBuffer = '';
            isLoading = false;
            aiStreamMode = 'none';
            chatAbortController = null;
            return;
          }
          aiChatHistory = [
            ...aiChatHistory,
            { role: 'assistant', content: `Error: ${message}` },
          ];
          chatStreamBuffer = '';
          isLoading = false;
          aiStreamMode = 'none';
          chatAbortController = null;
        },
      },
    );
  }

  function onAskAgentAi() {
    const q = agentCommand.trim() || 'What is this page about? Summarize the main points.';
    void askAgentWithAllama(q);
  }

  function cancelAiChat() {
    chatAbortController?.abort();
    chatAbortController = null;
    if (chatStreamBuffer.trim()) {
      aiChatHistory = [
        ...aiChatHistory,
        { role: 'assistant', content: chatStreamBuffer },
      ];
    }
    chatStreamBuffer = '';
    isLoading = false;
    aiStreamMode = 'none';
  }

  function onAgentPreset(actionJson: string) {
    void runAgentAction(actionJson);
  }

  function executeInIframe(jsCode: string) {
    const iframe = webviewFrame;
    if (!iframe?.contentWindow) return;
    const fn = new Function(jsCode);
    fn.call(iframe.contentWindow);
  }

  function addAgentLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    agentLog = [...agentLog, `[${timestamp}] ${message}`];
  }

  function handleChromeSelection() {
    if (useNativeWebview) return;
    const selection = window.getSelection();
    const text = selection?.toString().trim();
    if (text && selection) {
      selectedText = text;
      const range = selection.getRangeAt(0);
      const rect = range.getBoundingClientRect();
      popupPosition = { x: rect.left + rect.width / 2, y: rect.top - 50 };
      showSelectionPopup = true;
    } else {
      showSelectionPopup = false;
    }
  }

  async function handleContentMouseUp() {
    if (!useNativeWebview) return;
    try {
      const sel = await getTabSelection(activeLabel());
      if (sel.text.length > 2) {
        selectedText = sel.text;
        popupPosition = { x: window.innerWidth / 2, y: 100 };
        showSelectionPopup = true;
      }
    } catch {
      /* ignore */
    }
  }

  async function refreshEmbeddingsOnline() {
    try {
      embeddingsOnline = await checkEmbeddingsOnline(aiPort, embeddingModel);
    } catch {
      embeddingsOnline = false;
    }
  }

  async function refreshAiOnline() {
    try {
      aiOnline = await checkSidebarAiOnline(aiPort);
    } catch {
      aiOnline = false;
    }
    await refreshEmbeddingsOnline();
    await refreshNewTabPage();
  }

  /** Regenerate built-in new-tab HTML (wallpaper + AI status) and update open new tabs. */
  async function applyNewTabWallpaper(id: string) {
    newTabWallpaperId = id;
    await saveWallpaperIdAndSync(id);
    await refreshNewTabPage();
  }

  async function refreshNewTabPage() {
    await preloadWallpaperDataUrls();
    const url = initNewTabPage({ aiOnline, aiModel, wallpaperId: newTabWallpaperId });
    newTabPageUrl = url;
    tabs = tabs.map((t) => (isNewTabUrl(t.url) ? { ...t, url } : t));
    if (isNewTabUrl(currentUrl)) {
      currentUrl = url;
      urlInput = url;
    }
    if (!useNativeWebview) return;
    for (const t of tabs) {
      if (!isNewTabUrl(t.url) || !t.webview) continue;
      try {
        await navigateTab(tabWebviewLabel(t.id), url);
      } catch (error) {
        console.error('refreshNewTabPage navigate failed:', error);
      }
    }
  }

  /** One-time welcome message in the AI sidebar after first launch. */
  function seedWelcomeAiChat() {
    try {
      if (localStorage.getItem(WELCOME_AI_KEY)) return;
      localStorage.setItem(WELCOME_AI_KEY, '1');
    } catch {
      return;
    }
    aiChatHistory = [
      {
        role: 'assistant',
        content: aiOnline
          ? `Hello! Local AI is ready (model: ${aiModel}). Ask anything here, or type /ask in the address bar.`
          : 'Welcome to Exodus AI. Run `sh scripts/start-exodus-ai.sh` to start Allama, then send a message here.',
      },
    ];
    aiSidebarOpen = true;
    sidebarPanel = 'ai';
  }

  async function summarizeText() {
    showSelectionPopup = false;
    isLoading = true;
    aiResponse = '';
    aiStreamMode = 'summary';
    sidebarPanel = 'ai';
    aiSidebarOpen = true;
    if (!aiOnline) {
      aiResponse = 'Error: Allama is offline — check Settings → Allama (port 11435)';
      isLoading = false;
      aiStreamMode = 'none';
      return;
    }
    chatAbortController?.abort();
    chatAbortController = new AbortController();
    const signal = chatAbortController.signal;
    await streamSidebarSummarize(
      selectedText,
      { port: aiPort, model: aiModel, signal },
      {
        onChunk: (content) => {
          aiResponse += content;
        },
        onDone: () => {
          isLoading = false;
          aiStreamMode = 'none';
          chatAbortController = null;
        },
        onError: (message) => {
          if (signal.aborted) {
            isLoading = false;
            aiStreamMode = 'none';
            chatAbortController = null;
            return;
          }
          aiResponse = `Error: ${message}`;
          isLoading = false;
          aiStreamMode = 'none';
          chatAbortController = null;
        },
      },
    );
  }

  /** Send a message to the sidebar AI chat (direct Allama HTTP on port 11435). */
  async function sendAiChat() {
    const prompt = aiChatInput.trim();
    if (!prompt || isLoading) return;
    aiChatInput = '';
    const historyWithUser: AiChatMessage[] = [
      ...aiChatHistory,
      { role: 'user', content: prompt },
    ];
    aiChatHistory = historyWithUser;
    chatStreamBuffer = '';
    aiStreamMode = 'chat';
    isLoading = true;
    sidebarPanel = 'ai';
    aiSidebarOpen = true;
    if (!aiOnline) {
      aiChatHistory = [
        ...aiChatHistory,
        {
          role: 'assistant',
          content: 'Error: Allama is offline — open Settings and start Allama (port 11435)',
        },
      ];
      isLoading = false;
      aiStreamMode = 'none';
      return;
    }
    chatAbortController?.abort();
    chatAbortController = new AbortController();
    const signal = chatAbortController.signal;
    await streamSidebarChat(
      historyWithUser,
      { port: aiPort, model: aiModel, signal },
      {
        onChunk: (content) => {
          chatStreamBuffer += content;
        },
        onDone: () => {
          if (chatStreamBuffer.trim()) {
            const reply = chatStreamBuffer;
            aiChatHistory = [...aiChatHistory, { role: 'assistant', content: reply }];
            handleAiReplyForCdn(reply);
          }
          chatStreamBuffer = '';
          isLoading = false;
          aiStreamMode = 'none';
          chatAbortController = null;
        },
        onError: (message) => {
          if (signal.aborted) {
            chatStreamBuffer = '';
            isLoading = false;
            aiStreamMode = 'none';
            chatAbortController = null;
            return;
          }
          aiChatHistory = [
            ...aiChatHistory,
            { role: 'assistant', content: `Error: ${message}` },
          ];
          chatStreamBuffer = '';
          isLoading = false;
          aiStreamMode = 'none';
          chatAbortController = null;
        },
      },
    );
  }

  async function goHome() {
    urlInput = homepageUrl;
    await navigate();
  }

  async function refreshSidecarStatus() {
    try {
      sidecarStatus = await invoke<SidecarStatusDto>('get_sidecar_status');
    } catch (error) {
      console.error('Sidecar status failed:', error);
    }
  }

  async function restartSidecar() {
    try {
      sidecarStatus = await invoke<SidecarStatusDto>('restart_sidecar');
      await refreshAiOnline();
      showStatus('Inference engine restarted');
    } catch (error) {
      showStatus(formatStatusError(error, 'Sidecar restart failed'));
    }
  }

  async function saveAiSettings() {
    try {
      await invoke('set_ai_config', {
        aiPort,
        aiModel,
        embeddingModel,
        homepageUrl,
        searchEngineUrl,
        statusClearMs,
        spawnSidecar,
        spawnAllama,
      });
      await refreshAiOnline();
      showStatus('Settings saved');
    } catch (error) {
      console.error('Failed to save AI settings:', error);
      showStatus(formatStatusError(error, 'Failed to save settings'));
    }
  }

  async function savePrivacySettings() {
    try {
      await invoke('set_privacy_settings', {
        httpsOnly,
        privateMode,
        blockPopups,
        sessionRestore,
      });
      if (useNativeWebview) {
        for (const tab of tabs) {
          if (tab.webview) {
            await setTabPopupBlocking(tabWebviewLabel(tab.id), blockPopups);
          }
        }
      }
      if (!sessionRestore || privateMode) {
        await invoke('clear_session');
      }
      await recreateWebviewsForPrivateMode();
      void refreshSiteShieldForUrl(currentUrl);
      showStatus('Privacy settings saved');
    } catch (error) {
      console.error('Failed to save privacy settings:', error);
      showStatus(formatStatusError(error, 'Failed to save privacy settings'));
    }
  }

  async function exportBookmarks() {
    try {
      const bookmarksJson = await invoke<string>('export_bookmarks');
      const { parseBookmarksExportJson, bookmarksExportFilename } = await import('$lib/bookmarksIo');
      parseBookmarksExportJson(bookmarksJson);

      const blob = new Blob([bookmarksJson], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = bookmarksExportFilename();
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      
      showStatus('Bookmarks exported successfully');
    } catch (error) {
      console.error('Failed to export bookmarks:', error);
      showStatus(formatStatusError(error, 'Failed to export bookmarks'));
    }
  }

  async function importBookmarks() {
    try {
      // Create a file input element
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = 'application/json,.json';
      
      input.onchange = async (e) => {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (!file) return;
        
        const text = await file.text();
        const { parseBookmarksExportJson, serializeBookmarksForImport } = await import(
          '$lib/bookmarksIo'
        );
        const items = parseBookmarksExportJson(text);
        const importedCount = await invoke<number>('import_bookmarks', {
          bookmarksJson: serializeBookmarksForImport(items),
        });
        
        showStatus(`Imported ${importedCount} bookmarks`);
        await loadBookmarks();
      };
      
      input.click();
    } catch (error) {
      console.error('Failed to import bookmarks:', error);
      showStatus(formatStatusError(error, 'Failed to import bookmarks'));
    }
  }

  async function searchBookmarks(query: string) {
    if (!query.trim()) {
      await loadBookmarks();
      return;
    }
    try {
      const results = await invoke<BookmarkItem[]>('search_bookmarks', { query });
      bookmarks = results;
    } catch (error) {
      console.error('Failed to search bookmarks:', error);
      showStatus(formatStatusError(error, 'Search failed'));
    }
  }

  async function saveSession() {
    if (!isTauri()) return;
    if (!shouldPersistSession(sessionRestore, privateMode)) return;
    try {
      const currentTabs = sortedTabs().map(t => ({
        id: t.id,
        url: t.url,
        title: t.title,
        active: t.id === activeTabId,
      }));
      const currentActiveTabId = activeTabId || null;
      await invoke('save_session', { tabs: currentTabs, activeTabId: currentActiveTabId });
    } catch (error) {
      console.error('Failed to save session:', error);
    }
  }

  /** Restore last session when only the default new-tab is open and session restore is enabled. */
  async function loadSession() {
    if (!isTauri()) return;
    if (!shouldPersistSession(sessionRestore, privateMode)) return;
    try {
      const session = await invoke<{
        tabs: Array<{ id: string; url: string; title: string; active?: boolean }>;
        activeTabId?: string;
      } | null>('load_session');
      if (!session?.tabs?.length) return;
      if (tabs.length !== 1 || !isNewTabUrl(tabs[0].url)) return;

      const restored: BrowserTab[] = session.tabs.map((tab) => ({
        id: tab.id || createTabId(),
        title: tab.title || tab.url || 'New Tab',
        url: tab.url || newTabPageUrl,
        webview: null,
      }));
      tabs = restored;
      const targetId =
        session.activeTabId && restored.some((t) => t.id === session.activeTabId)
          ? session.activeTabId
          : restored[0].id;
      await switchTab(targetId);
      showStatus('Session restored');
    } catch (error) {
      console.error('Failed to load session:', error);
    }
  }

  /** Filter indexed pages and visit history in the memory sidebar. */
  async function searchMemoryPanel(query: string) {
    if (!query.trim()) {
      await loadIndexedMemory();
      await loadHistory();
      return;
    }
    try {
      const [pages, mergedHistory] = await Promise.all([
        invoke<IndexedPage[]>('search_indexed_pages', { query }),
        searchMergedBrowsingHistory(query),
      ]);
      indexedPages = pages;
      historyPages = mergedHistory;
    } catch (error) {
      console.error('Failed to search memory panel:', error);
      showStatus(formatStatusError(error, 'Search failed'));
    }
  }

  function closeMenu() {
    showMenu = false;
  }

  function applyTheme() {
    document.documentElement.classList.toggle('light-theme', !isDarkTheme);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, isDarkTheme ? 'dark' : 'light');
    } catch {
      /* private mode */
    }
  }

  function toggleTheme() {
    isDarkTheme = !isDarkTheme;
    applyTheme();
  }

  function openDownloadsPanel() {
    showDownloadsPanel = true;
  }

  function closeDownloadsPanel() {
    showDownloadsPanel = false;
  }

  function clearDownloads() {
    downloads = [];
  }

  /** Start downloading a URL via the Rust download manager. */
  async function startDownload(url: string, filename?: string) {
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      showStatus('Cannot download this URL');
      return;
    }
    const id = crypto.randomUUID().slice(0, 8);
    const record: DownloadRecord = {
      id,
      url,
      filename: filename || url.split('/').pop()?.split('?')[0] || 'download',
      status: 'pending',
      progress: 0,
      received: 0,
      total: 0,
    };
    downloads = [record, ...downloads];
    showDownloadsPanel = true;
    try {
      downloads = downloads.map((d) =>
        d.id === id ? { ...d, status: 'downloading' as const } : d,
      );
      await invoke('download_url', { id, url, filename: filename ?? null });
    } catch (error) {
      downloads = downloads.map((d) =>
        d.id === id ? { ...d, status: 'failed' as const } : d,
      );
      showStatus(formatStatusError(error, 'Download failed'));
    }
  }

  async function downloadCurrentPage() {
    showMenu = false;
    await startDownload(currentUrl);
  }

  async function openDownloadsDir() {
    try {
      await invoke('open_downloads_folder');
    } catch (error) {
      console.error('Open downloads folder failed:', error);
      showStatus('Could not open downloads folder');
    }
  }

  function toggleFindBar() {
    showFindBar = !showFindBar;
    if (showFindBar) {
      setTimeout(() => {
        const findInput = document.querySelector('.find-input') as HTMLInputElement;
        if (findInput) findInput.focus();
      }, 100);
    }
  }

  function closeFindBar() {
    showFindBar = false;
    findQuery = '';
    findResults = 0;
    currentFindIndex = 0;
  }

  async function countFindMatches(): Promise<number> {
    if (!findQuery.trim()) return 0;
    const escaped = findQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    try {
      if (useNativeWebview) {
        const script = `(function() {
          const content = document.body.innerText || '';
          const m = content.match(new RegExp(${JSON.stringify(escaped)}, 'gi'));
          return m ? m.length : 0;
        })()`;
        const raw = await evalTabReturning(activeLabel(), script);
        const count = typeof raw === 'string' ? JSON.parse(raw) : raw;
        return Number(count) || 0;
      }
      const content = webviewFrame?.contentDocument?.body?.innerText || '';
      const matches = content.match(new RegExp(escaped, 'gi'));
      return matches?.length || 0;
    } catch {
      return 0;
    }
  }

  async function findInPage(direction: 'next' | 'prev' = 'next') {
    if (!findQuery.trim()) return;
    const forward = direction === 'next';
    try {
      findResults = await countFindMatches();
      let found = false;
      if (useNativeWebview) {
        found = await findInTab(activeLabel(), findQuery, forward);
      } else if (webviewFrame?.contentWindow) {
        const win = webviewFrame.contentWindow as Window & {
          find: (s: string, a: boolean, b: boolean, c: boolean) => boolean;
        };
        found = win.find(findQuery, false, !forward, true);
      }
      if (found) {
        if (forward) {
          currentFindIndex =
            findResults > 0 ? (currentFindIndex % findResults) + 1 : currentFindIndex + 1;
        } else {
          currentFindIndex =
            findResults > 0
              ? currentFindIndex <= 1
                ? findResults
                : currentFindIndex - 1
              : Math.max(1, currentFindIndex - 1);
        }
      } else if (findResults === 0) {
        currentFindIndex = 0;
      }
    } catch (error) {
      console.error('Find failed:', error);
    }
  }

  async function printPage() {
    try {
      if (useNativeWebview) {
        await evalInTab(activeLabel(), 'window.print()');
      } else if (webviewFrame?.contentWindow) {
        webviewFrame.contentWindow.print();
      }
    } catch (error) {
      console.error('Print failed:', error);
      showStatus('Print failed');
    }
  }

  $effect(() => {
    writeShowBookmarkBar(showBookmarkBar);
  });

  onMount(() => {
    logStartup('+page onMount begin', {
      href: typeof window !== 'undefined' ? window.location.href : '',
      nativeWebview: canUseNativeWebview(),
    });
    try {
      showBookmarkBar = readShowBookmarkBar();
      const saved = localStorage.getItem(THEME_STORAGE_KEY);
      if (saved === 'light') isDarkTheme = false;
      else if (saved === 'dark') isDarkTheme = true;
      const autoIdx = localStorage.getItem(AUTO_INDEX_KEY);
      if (autoIdx === '0') autoIndexPages = false;
    } catch {
      /* ignore */
    }
    applyTheme();

    document.addEventListener('selectionchange', handleChromeSelection);
    const unbindShortcuts = mountBrowserShortcuts({
      focusOmnibox: focusOmniboxInput,
      reload: reloadPage,
      goBack,
      goForward,
      newTab,
      restoreClosedTab: () => void restoreClosedTab(),
      closeActiveTab: () => closeTab(activeTabId),
      zoomIn,
      zoomOut,
      resetZoom,
      toggleBookmark,
      toggleBookmarkBar: () => (showBookmarkBar = !showBookmarkBar),
      openHistory: openHistoryPanel,
      toggleSidebar,
      toggleFindBar,
      print: printPage,
      switchToTabIndex: (idx) => {
        const ordered = sortedTabs();
        if (ordered[idx]) void switchTab(ordered[idx].id);
      },
      tabIdsInOrder: () => sortedTabs().map((t) => t.id),
      toggleDevTools: () => {
        if (!useNativeWebview || !activeTabId) return;
        void toggleTabDevTools(tabWebviewLabel(activeTabId)).catch((e) =>
          console.error('browser_toggle_devtools failed:', e),
        );
      },
      togglePrivateMode: () => {
        privateMode = !privateMode;
        void savePrivacySettings();
      },
      onEscape: () => {
        showSettings = false;
        showDownloadsPanel = false;
        showMenu = false;
        showSearchResults = false;
        passwordSaveOffer = null;
        safeBrowsingOffer = null;
        tabGroupEditOffer = null;
        tabGroupDeleteOffer = null;
        cancelConfirmDialog();
        closeTabContextMenu();
        if (showFindBar) closeFindBar();
      },
    });

    let unlistenDlProgress: (() => void) | undefined;
    let unlistenDlDone: (() => void) | undefined;
    let unlistenDlError: (() => void) | undefined;
    let unlistenDlRequested: (() => void) | undefined;
    let unlistenNewWindow: (() => void) | undefined;
    let unlistenPopupBlocked: (() => void) | undefined;
    let unlayout: (() => void) | undefined;

    let unlistenExtTabs: (() => void) | undefined;
    let unlistenExtTabOps: (() => void) | undefined;
    let unlistenExtPerm: (() => void) | undefined;
    let unlistenExtNotif: (() => void) | undefined;
    let unlistenExtOpenPopup: (() => void) | undefined;
    let unlistenExtHost: (() => void) | undefined;
    let unlistenExtHostInstall: (() => void) | undefined;
    let unlistenBrowserSitePerm: (() => void) | undefined;
    let unlistenPrivateMode: (() => void) | undefined;

    (async () => {
      await loadSession();
      pushExtensionTabRegistry();

      unlistenPrivateMode = await listen<boolean>('exodus-private-mode-changed', (event) => {
        privateMode = event.payload;
        void recreateWebviewsForPrivateMode();
        void refreshSiteShieldForUrl(currentUrl);
      });

      unlistenExtTabOps = await listenExtensionTabOps((ops) => handleExtensionTabOps(ops));

      unlistenExtPerm = await listenExtensionPermissionRequests((req) => {
        const next = enqueuePrompt(extensionPermRequest, extensionPermQueue, req);
        extensionPermRequest = next.active;
        extensionPermQueue = next.queue;
      });

      unlistenExtHostInstall = await listenExtensionHostInstallRequests((req) => {
        const next = enqueuePrompt(extensionHostInstallRequest, extensionHostInstallQueue, req);
        extensionHostInstallRequest = next.active;
        extensionHostInstallQueue = next.queue;
      });

      unlistenBrowserSitePerm = await listenBrowserSitePermissionRequests((req) => {
        const next = enqueuePrompt(browserSitePermRequest, browserSitePermQueue, req);
        browserSitePermRequest = next.active;
        browserSitePermQueue = next.queue;
      });

      unlistenExtHost = await listenExtensionHostDenied((ev) => {
        showStatus(`Extension ${ev.extensionId} blocked: ${ev.url}`);
      });

      unlistenExtNotif = await listenExtensionNotifications((note) => {
        const title = note.title ?? 'Extension';
        const body = note.message ?? '';
        if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
          new Notification(title, { body });
        } else {
          showStatus(`${title}: ${body}`);
        }
      });

      unlistenExtOpenPopup = await listen<string>('exodus-extension-open-popup', (event) => {
        void invoke('extension_open_popup_window', { extensionId: event.payload }).catch((error) =>
          console.error('extension_open_popup_window failed:', error),
        );
      });

      unlistenExtTabs = await listenExtensionTabCreates(
        async (requests: ExtensionTabCreateRequest[]): Promise<TabCreateAck[]> => {
          const acks: TabCreateAck[] = [];
          for (const req of requests) {
            const newId = createTabId();
            tabs = [
              ...tabs,
              { id: newId, title: 'New Tab', url: req.url, webview: null },
            ];
            if (req.active) {
              await switchTab(newId);
            }
            if (useNativeWebview && contentHost) {
              const label = tabWebviewLabel(newId);
              const wv = await createTabWebview(contentHost, label, req.url);
              await setTabPopupBlocking(label, blockPopups);
              tabs = tabs.map((t) => (t.id === newId ? { ...t, webview: wv } : t));
            }
            const tabIndex = tabs.findIndex((t) => t.id === newId);
            acks.push({
              requestId: req.requestId,
              sourceWebviewLabel: req.sourceWebviewLabel ?? '',
              chromeTabId: tabIndex + 1,
              tabId: newId,
              url: req.url,
              title: tabs[tabIndex]?.title ?? 'New Tab',
            });
          }
          pushExtensionTabRegistry();
          return acks;
        },
      );
      try {
        const cfg = await loadAiConfig();
        aiPort = cfg.ai_port ?? 11435;
        aiModel = cfg.ai_model;
        embeddingModel = cfg.embedding_model || 'nomic-embed-text';
        homepageUrl = cfg.homepage_url || 'https://duckduckgo.com';
        searchEngineUrl = cfg.search_engine_url || 'https://duckduckgo.com/?q={query}';
        statusClearMs = cfg.status_clear_ms ?? STATUS_CLEAR_MS;
        spawnSidecar = cfg.spawn_sidecar ?? false;
        spawnAllama = cfg.spawn_allama ?? true;

        const normalized = await normalizeChatModelFromAllama(aiPort, aiModel);
        if (normalized.changed) {
          aiModel = normalized.model;
          await invoke('set_ai_config', {
            aiPort,
            aiModel,
            embeddingModel,
            homepageUrl,
            searchEngineUrl,
            statusClearMs,
            spawnSidecar,
            spawnAllama,
          });
        }

        logStartup('preloadWallpaperDataUrls begin');
        await preloadWallpaperDataUrls();
        logStartup('syncNtpWallpaperOnStartup begin');
        newTabWallpaperId = await syncNtpWallpaperOnStartup();
        logStartup('initNewTabPage', { wallpaperId: newTabWallpaperId });
        newTabPageUrl = initNewTabPage({
          aiOnline: false,
          aiModel,
          wallpaperId: newTabWallpaperId,
        });
        if (isNewTabUrl(tabs[0]?.url ?? '')) {
          tabs = tabs.map((t, i) =>
            i === 0 && isNewTabUrl(t.url) ? { ...t, url: newTabPageUrl } : t,
          );
          currentUrl = newTabPageUrl;
          urlInput = newTabPageUrl;
        }

        // Load privacy settings
        const privacy = await invoke<[boolean, boolean, boolean, boolean]>('get_privacy_settings');
        ({ httpsOnly, privateMode, blockPopups, sessionRestore } = parsePrivacyTuple(privacy));
        await refreshAiOnline();
        seedWelcomeAiChat();
        try {
          const tp = await loadTrackingProtectionSettings();
          trackingProtectionEnabled = tp.enabled;
        } catch {
          trackingProtectionEnabled = true;
        }
        void refreshNewTabTopSites();
        const persisted = await loadPersistedDownloads();
        if (persisted.length > 0) {
          downloads = persisted;
        }
        void refreshSiteShieldForUrl(currentUrl);
        void syncAllTabsLifecycle(
          tabs.map((t) => ({
            id: t.id,
            url: t.url,
            title: t.title,
            pinned: t.pinned,
          })),
          activeTabId,
        );
        await refreshSidecarStatus();
        void loadHistory().then((visits) => {
          void seedOmniboxFromVisits(
            visits.map((v) => ({ url: v.url, title: v.title })),
          );
        });
        void loadTabGroups();
        void refreshPrivacyStats();
        void loadVerticalTabSettings().then(applyVerticalTabLayout).catch((e) =>
          console.error('loadVerticalTabSettings failed:', e),
        );
      } catch (error) {
        logStartupError('loadAiConfig / wallpaper init failed', error);
      }

      logStartup('native webview init', { useNativeWebview, hasContentHost: !!contentHost });
      if (canUseNativeWebview() && contentHost) {
        try {
          await ensureActiveWebview();
          logStartup('ensureActiveWebview ok');
          await ensureExtensionBackgrounds(contentHost);
          for (const t of tabs) {
            if (t.webview) {
              await setTabPopupBlocking(tabWebviewLabel(t.id), blockPopups);
            }
            if (t.id !== activeTabId && t.webview) {
              await hideTabWebview(t.webview);
            }
          }
          unlayout = watchWebviewLayout(contentHost, () => activeTab()?.webview ?? null);
          const navPoll = window.setInterval(() => {
            if (useNativeWebview && activeTab()?.webview) {
              void refreshNavState();
            }
          }, 1500);
          const prevUnlayout = unlayout;
          unlayout = () => {
            clearInterval(navPoll);
            prevUnlayout?.();
          };
        } catch (e) {
          logStartupError('Native webview init failed', e);
          useNativeWebview = false;
        }
      }

      unlistenDlProgress = await listen<{
        id: string;
        url: string;
        filename: string;
        received: number;
        total: number;
        progress: number;
      }>('exodus-download-progress', (e) => {
        const p = e.payload;
        downloads = downloads.map((d) =>
          d.id === p.id
            ? {
                ...d,
                status: 'downloading',
                filename: p.filename,
                received: p.received,
                total: p.total,
                progress: p.progress,
              }
            : d,
        );
      });
      unlistenDlDone = await listen<{ id: string; path: string; filename: string }>(
        'exodus-download-done',
        (e) => {
          const p = e.payload;
          downloads = downloads.map((d) =>
            d.id === p.id
              ? { ...d, status: 'completed', progress: 100, path: p.path, filename: p.filename }
              : d,
          );
          showStatus(`Downloaded: ${p.filename}`);
        },
      );
      unlistenDlError = await listen<{ id: string; message: string }>('exodus-download-error', (e) => {
        const { id, message } = e.payload;
        downloads = downloads.map((d) => (d.id === id ? { ...d, status: 'failed' as const } : d));
        showStatus(`Download error: ${message}`);
      });

      unlistenNewWindow = await listen<{ url: string; opener_label: string }>(
        'exodus-new-window-requested',
        (e) => {
          void openUrlInNewTab(e.payload.url);
        },
      );
      unlistenPopupBlocked = await listen<{ url: string; opener_label: string }>(
        'exodus-popup-blocked',
        (e) => {
          showStatus(`Blocked popup: ${e.payload.url}`);
        },
      );

      unlistenDlRequested = await listen<{ label: string; url: string }>(
        'exodus-download-requested',
        (e) => {
          void startDownload(e.payload.url);
        },
      );

      await seedPresetBookmarksIfEmpty();
      await loadBookmarks();
      void refreshNavState();
      logStartup('+page onMount async init complete', {
        tabs: tabs.length,
        currentUrl: currentUrl.slice(0, 80),
        wallpaperId: newTabWallpaperId,
      });
    })().catch((e) => logStartupError('+page onMount async init failed', e));

    logStartup('+page onMount sync setup complete');
    void bindLifecycleRecovery();

    // Save session on window close and periodically (crash recovery)
    window.addEventListener('beforeunload', () => {
      void saveSession();
    });
    const sessionAutosaveTimer = window.setInterval(() => {
      if (shouldPersistSession(sessionRestore, privateMode)) void saveSession();
    }, 60_000);

    const extensionPumpTimer = window.setInterval(() => {
      if (useNativeWebview && activeTabId) {
        void pumpExtensionRuntime(tabWebviewLabel(activeTabId));
      }
    }, 2000);

    const privacyStatsTimer = window.setInterval(() => {
      void refreshPrivacyStats();
    }, 30_000);

    const tabLifecycleTimer = window.setInterval(() => {
      void runTabLifecycleMaintenance(
        activeTabId,
        tabs.map((t) => ({
          id: t.id,
          url: t.url,
          title: t.title,
          pinned: t.pinned,
        })),
        async (tabId, url) => {
          if (!contentHost || !useNativeWebview) return;
          const label = tabWebviewLabel(tabId);
          tabs = tabs.map((t) => (t.id === tabId ? { ...t, webview: null } : t));
          await discardTabWebview(label, url, contentHost);
        },
      ).then((stats) => {
        if (stats.newlyFrozen > 0) {
          showStatus(
            `Froze ${stats.newlyFrozen} inactive tab(s) · ${stats.totalFrozen} frozen total`,
          );
        }
        if (stats.sleepCandidates > 0) {
          showStatus(`Discarded ${stats.sleepCandidates} sleeping tab webview(s)`);
        }
      });
    }, 90_000);

    return () => {
      clearInterval(sessionAutosaveTimer);
      clearInterval(extensionPumpTimer);
      clearInterval(privacyStatsTimer);
      clearInterval(tabLifecycleTimer);
      document.removeEventListener('selectionchange', handleChromeSelection);
      unbindShortcuts();
      statusNotifier.dispose();
      unlistenDlProgress?.();
      unlistenDlDone?.();
      unlistenDlError?.();
      unlistenDlRequested?.();
      unlistenNewWindow?.();
      unlistenPopupBlocked?.();
      unlistenExtTabs?.();
      unlistenExtTabOps?.();
      unlistenExtPerm?.();
      unlistenExtNotif?.();
      unlistenExtOpenPopup?.();
      unlistenExtHost?.();
      unlistenExtHostInstall?.();
      unlistenBrowserSitePerm?.();
      unlistenPrivateMode?.();
      unlayout?.();
      if (autoIndexTimer) clearTimeout(autoIndexTimer);
    };
  });
</script>

<!-- Skip navigation link for keyboard users -->
<a
  href="#main-content"
  class="skip-link"
  onkeydown={(e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const target = document.getElementById('main-content');
      target?.focus();
    }
  }}
>
  Skip to main content
</a>

<ExtensionPermissionPrompt
  request={extensionPermRequest}
  onResolved={() => {
    extensionPermRequest = extensionPermQueue.shift() ?? null;
  }}
/>

<PasswordSavePrompt
  capture={passwordSaveOffer}
  busy={passwordSaveBusy}
  onSave={confirmPasswordSave}
  onDismiss={dismissPasswordSave}
  onNever={neverSavePasswordForSite}
/>

<TabGroupEditPrompt
  offer={tabGroupEditOffer}
  busy={tabGroupEditBusy}
  onSave={saveTabGroupEdit}
  onCancel={cancelTabGroupEdit}
/>

<TabGroupDeletePrompt
  groupTitle={tabGroupDeleteOffer?.title ?? null}
  busy={tabGroupDeleteBusy}
  onConfirm={confirmTabGroupDelete}
  onCancel={cancelTabGroupDelete}
/>

<ConfirmPrompt
  offer={confirmOffer}
  busy={confirmBusy}
  onConfirm={runConfirmDialog}
  onCancel={cancelConfirmDialog}
/>

<SafeBrowsingPrompt
  offer={safeBrowsingOffer}
  onProceed={() => void proceedSafeBrowsing()}
  onCancel={cancelSafeBrowsing}
/>

<ExtensionHostInstallPrompt
  request={extensionHostInstallRequest}
  onResolved={() => {
    extensionHostInstallRequest = extensionHostInstallQueue.shift() ?? null;
  }}
/>

<BrowserSitePermissionPrompt
  request={browserSitePermRequest}
  onResolved={() => {
    browserSitePermRequest = browserSitePermQueue.shift() ?? null;
  }}
/>

<main
  id="main-content"
  tabindex="-1"
  class="browser-container exodus-browser"
  class:vertical-tabs={verticalTabsOn}
  class:vt-right={verticalTabsOn && verticalTabsRight}
>
  <div class="browser-shell">
    {#if verticalTabsOn}
      <BrowserTabBar
        vertical
        verticalWidth={verticalTabWidth}
        verticalRight={verticalTabsRight}
        {tabs}
        {activeTabId}
        bind:tabBarEl
        tabContextMenu={tabContextMenu}
        sortedTabs={sortedTabs()}
        onSwitchTab={(id) => void switchTab(id)}
        onNewTab={newTab}
        onCloseTab={(id, force) => void closeTab(id, force)}
        onTabMouseDown={onTabMouseDown}
        onTabContextMenu={openTabContextMenu}
        onCloseContextMenu={closeTabContextMenu}
        onTogglePin={togglePinTab}
        onDuplicateTab={(id) => void duplicateTab(id)}
        {tabGroups}
        onNewTabGroup={(id) => void newTabGroupFromTab(id)}
        onAddTabToGroup={(tabId, groupId) => void addTabToExistingGroup(tabId, groupId)}
        onRemoveTabFromGroup={(id) => void removeTabGroupMembership(id)}
        onToggleGroupCollapse={(groupId, collapsed) => void toggleTabGroupCollapse(groupId, collapsed)}
        onRenameTabGroup={(groupId) => void renameTabGroupPrompt(groupId)}
        onCycleTabGroupColor={(groupId) => void cycleTabGroupColor(groupId)}
        onDeleteTabGroup={(groupId) => void deleteTabGroupById(groupId)}
      />
    {/if}

    <div class="browser-main">
      {#if !verticalTabsOn}
        <BrowserTabBar
          {tabs}
          {activeTabId}
          bind:tabBarEl
          tabContextMenu={tabContextMenu}
          sortedTabs={sortedTabs()}
          onSwitchTab={(id) => void switchTab(id)}
          onNewTab={newTab}
          onCloseTab={(id, force) => void closeTab(id, force)}
          onTabMouseDown={onTabMouseDown}
          onTabContextMenu={openTabContextMenu}
          onCloseContextMenu={closeTabContextMenu}
          onTogglePin={togglePinTab}
          onDuplicateTab={(id) => void duplicateTab(id)}
          {tabGroups}
          onNewTabGroup={(id) => void newTabGroupFromTab(id)}
          onAddTabToGroup={(tabId, groupId) => void addTabToExistingGroup(tabId, groupId)}
          onRemoveTabFromGroup={(id) => void removeTabGroupMembership(id)}
          onToggleGroupCollapse={(groupId, collapsed) => void toggleTabGroupCollapse(groupId, collapsed)}
          onRenameTabGroup={(groupId) => void renameTabGroupPrompt(groupId)}
          onCycleTabGroupColor={(groupId) => void cycleTabGroupColor(groupId)}
          onDeleteTabGroup={(groupId) => void deleteTabGroupById(groupId)}
        />
      {/if}

  <FindBar
    open={showFindBar}
    bind:findQuery
    {findResults}
    {currentFindIndex}
    onFindInput={() => {
      currentFindIndex = 0;
      void countFindMatches().then((n) => (findResults = n));
    }}
    onFind={findInPage}
    onClose={closeFindBar}
  />


  <ExtensionActionBar {contentHost} />
  <BookmarkBar
    visible={showBookmarkBar}
    barBookmarks={barBookmarks}
    folderNames={bookmarkFolders}
    {bookmarks}
    onNavigate={navigateToBookmark}
    onReorder={reorderBookmarkBar}
    onMoveToFolder={updateBookmarkFolder}
  />


  <AddressBar
    canGoBack={canGoBack}
    canGoForward={canGoForward}
    {currentUrl}
    bind:urlInput
    {isBookmarked}
    {showSearchResults}
    {showOmniboxSuggestions}
    {omniboxSuggestions}
    onUrlInput={() => {
      scheduleOmniboxSuggestions();
    }}
    onSelectOmniboxSuggestion={(url) => void selectOmniboxSuggestion(url)}
    {isSearching}
    searchResults={searchResults}
    showMenu={showMenu}
    aiSidebarOpen={aiSidebarOpen}
    {sidebarPanel}
    downloadsBadge={activeDownloadsCount()}
    closedTabsCount={closedTabs.length}
    onGoBack={goBack}
    onGoForward={goForward}
    onReload={reloadPage}
    onHome={goHome}
    onNavigate={navigate}
    onNavigateToResult={navigateToResult}
    onToggleBookmark={toggleBookmark}
    onOpenPanel={openPanel}
    onAnnounceSearchResult={(url, title) => void announceUrlToCdn(url, title)}
    cdnStatusLabel={cdnUrlStatusLabel(cdnPageStatus)}
    onCdnBadgeClick={() => openPanel('p2p')}
    onOpenDownloads={openDownloadsPanel}
    onToggleMenu={() => (showMenu = !showMenu)}
    onCloseMenu={closeMenu}
    onOpenBookmarksPanel={openBookmarksPanel}
    onRestoreClosedTab={() => { closeMenu(); void restoreClosedTab(); }}
    onOpenHistoryPanel={openHistoryPanel}
    onDownloadCurrentPage={downloadCurrentPage}
    onIndexPage={() => { closeMenu(); void capturePageContent(false); }}
    onPrint={printPage}
    onTranslatePage={() => void translateCurrentPage()}
    onToggleReadingMode={() => void toggleReadingMode()}
    onOpenSettings={() => {
      showSettings = true;
      showMenu = false;
      void refreshSidecarStatus();
      void refreshIndexedMemoryCount();
    }}
    shieldsCount={privacyStats?.trackers_blocked ?? 0}
    shieldsEnabled={trackingProtectionEnabled && !siteAllowTrackers}
    siteAllowTrackers={siteAllowTrackers}
    onOpenShields={openShieldsSettings}
    onToggleSiteShields={() => void toggleSiteShieldAllowTrackers()}
  />

  <SettingsModal
    open={showSettings}
    scrollToSection={settingsScrollSection}
    contentHost={contentHost}
    bind:aiPort
    bind:aiModel
    bind:embeddingModel
    bind:homepageUrl
    bind:searchEngineUrl
    bind:showBookmarkBar
    bind:statusClearMs
    bind:spawnSidecar
    bind:spawnAllama
    {sidecarStatus}
    {aiOnline}
    {embeddingsOnline}
    {autoIndexPages}
    {indexedPageCount}
    {indexedPagesLoading}
    {isDarkTheme}
    {zoomLevel}
    bind:httpsOnly
    bind:privateMode
    bind:blockPopups
    bind:sessionRestore
    onClose={() => {
      showSettings = false;
      settingsScrollSection = null;
    }}
    onSave={saveAiSettings}
    onRestartSidecar={restartSidecar}
    onRefreshSidecar={refreshSidecarStatus}
    onClearMemory={clearLocalMemory}
    onClearHistory={clearVisitHistory}
    onToggleTheme={toggleTheme}
    onZoomIn={zoomIn}
    onZoomOut={zoomOut}
    onZoomReset={resetZoom}
    onAutoIndexChange={(enabled) => {
      autoIndexPages = enabled;
      try {
        localStorage.setItem(AUTO_INDEX_KEY, enabled ? '1' : '0');
      } catch { /* ignore */ }
    }}
    onHttpsOnlyChange={savePrivacySettings}
    onPrivateModeChange={savePrivacySettings}
    onBlockPopupsChange={savePrivacySettings}
    onSessionRestoreChange={savePrivacySettings}
    onExportBookmarks={exportBookmarks}
    onImportBookmarks={importBookmarks}
    onExtensionStatus={showStatus}
    bind:p2pRoomId
    onVerticalTabsChange={applyVerticalTabLayout}
    onNewTabWallpaperChange={applyNewTabWallpaper}
  />

  <BrowserPanels
    showDownloads={showDownloadsPanel}
    {downloads}
    onCloseDownloads={closeDownloadsPanel}
    onOpenDownloadsDir={openDownloadsDir}
    onClearDownloads={clearDownloads}
  />


  <StatusBar
    message={statusMessage}
    {privateMode}
    {httpsOnly}
    {blockPopups}
    {privacyStats}
  />


  <BrowserContent
    bind:contentHost
    bind:webviewFrame
    {useNativeWebview}
    {blockPopups}
    {currentUrl}
    {aiSidebarOpen}
    showQuickLinks={isNewTabUrl(currentUrl)}
    newTabTopSites={newTabTopSites}
    newTabWallpaperId={newTabWallpaperId}
    aiOnline={aiOnline}
    aiModel={aiModel}
    onWallpaperChange={applyNewTabWallpaper}
    onFrameLoad={onFrameLoad}
    onContentMouseUp={handleContentMouseUp}
    onQuickLinkNavigate={navigateToResult}
  >
    {#snippet sidebar()}
    <BrowserSidebar
      open={aiSidebarOpen}
      {sidebarPanel}
      {agentPanelOpen}
      indexedMemoryGroups={indexedMemoryGroups()}
      indexedCount={indexedPages.length}
      historyGroups={historyGroups()}
      historyCount={historyPages.length}
      {bookmarks}
      {aiOnline}
      {isLoading}
      aiStreamMode={aiStreamMode}
      {aiResponse}
      {aiChatHistory}
      bind:aiChatInput
      {chatStreamBuffer}
      bind:agentCommand
      {agentLog}
      {isAgentExecuting}
      {agentDomSummary}
      onClose={toggleSidebar}
      onOpenPanel={openPanel}
      onLoadMemory={() => {
        void loadIndexedMemory();
        void loadHistory();
      }}
      onClearIndexed={() => void clearIndexedMemory()}
      onRemoveIndexedPage={(id: string) => void removeIndexedPage(id)}
      onClearHistory={clearVisitHistory}
      onNavigate={navigateToResult}
      onLoadBookmarks={loadBookmarks}
      onRemoveBookmark={removeBookmark}
      onUpdateBookmarkFolder={updateBookmarkFolder}
      onToggleAgent={toggleAgentPanel}
      onAgentExecute={executeAgentCommand}
      onAgentCompress={compressCurrentDom}
      onAgentBack={toggleAgentPanel}
      onAgentPreset={onAgentPreset}
      onAgentCommandChange={(v: string) => { agentCommand = v; }}
      onSendChat={sendAiChat}
      onCancelChat={cancelAiChat}
      onAgentAskAi={onAskAgentAi}
      onSearchBookmarks={searchBookmarks}
      onSearchMemory={searchMemoryPanel}
      bind:p2pRoomId
      currentPageUrl={currentUrl}
      currentPageTitle={activeTab()?.title ?? currentUrl}
      onAnnounceUrlToCdn={announceUrlToCdn}
      onAnnounceCurrentPageToCdn={announceCurrentPageToCdn}
      onP2pStatus={showStatus}
      aiCdnSuggestUrls={aiCdnSuggestUrls}
      onDismissAiCdnSuggest={() => {
        aiCdnSuggestUrls = [];
      }}
      onAnnounceAllAiCdnUrls={announceAllAiCdnUrls}
      {privacyStats}
      onOpenPrivacySettings={() => openSettingsSection('privacy')}
    />
    {/snippet}
  </BrowserContent>
  <SelectionPopup
    visible={showSelectionPopup}
    x={popupPosition.x}
    y={popupPosition.y}
    onSummarize={summarizeText}
  />
    </div>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  }

  .browser-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1a1a1a;
    color: #e0e0e0;
  }

  .browser-shell {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .browser-container.vertical-tabs .browser-shell {
    flex-direction: row;
  }

  .browser-container.vertical-tabs.vt-right .browser-shell {
    flex-direction: row-reverse;
  }

  .browser-main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }

  /* Accessibility: Visible focus indicators */
  :global(:focus-visible) {
    outline: 2px solid #007bff;
    outline-offset: 2px;
  }

  :global(button:focus-visible),
  :global(input:focus-visible),
  :global(a:focus-visible),
  :global([role="button"]:focus-visible) {
    outline: 2px solid #007bff;
    outline-offset: 2px;
  }

  /* Hide outline for mouse users, show for keyboard users */
  :global(:focus:not(:focus-visible)) {
    outline: none;
  }

  /* Skip navigation link */
  .skip-link {
    position: absolute;
    top: -40px;
    left: 0;
    background: #007bff;
    color: white;
    padding: 8px;
    z-index: 10000;
    text-decoration: none;
  }

  .skip-link:focus {
    top: 0;
  }

</style>
