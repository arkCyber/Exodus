<template>
  <div
    class="browser-page exodus-browser-page"
    :data-active-tab-id="activeTabId ?? ''"
    :data-active-tab-url="activeTab?.url ?? ''"
  >
    <!-- Quit Confirmation Dialog -->
    <Teleport to="body">
      <div v-if="showQuitConfirmDialog" class="quit-confirm-dialog-overlay" @click.self="cancelQuit">
        <div class="quit-confirm-dialog">
          <h3>Quit Exodus</h3>
          <p>Are you sure you want to quit Exodus?</p>
          <div class="quit-confirm-buttons">
            <button @click="cancelQuit" class="btn-cancel">Cancel</button>
            <button @click="confirmQuit" class="btn-confirm">Quit</button>
          </div>
        </div>
      </div>
    </Teleport>
    <div class="browser-shell">
      <div class="browser-main">
        <WindowTitleBar>
        <!-- Tab Bar -->
        <BrowserTabBar
          v-if="!sidebarPrefs.hideHorizontalTabBar.value"
          v-bind="tabBarBindProps"
          v-on="tabBarHandlers"
        />

        <!-- Toolbar / Address Bar -->
        <AddressBar
          :canGoBack="canGoBack"
          :canGoForward="canGoForward"
          :currentUrl="addressBarUrl"
          :isBookmarked="isBookmarked"
          :showMenu="showMenu"
          :sidebarOpen="sidebar.sidebarOpen.value"
          :sidebarPanel="sidebar.sidebarPanel.value"
          :showWebChatView="showWebChatView"
          @toggleSidebar="toggleSidebarSmart"
          :downloadsBadge="downloads.activeDownloadsCount.value"
          :closedTabsCount="closedTabsStack.closedTabsCount.value"
          :shieldsCount="privacyStats?.trackers_blocked ?? 0"
          :shieldsEnabled="shieldsEnabled"
          :siteAllowTrackers="siteShields.siteAllowTrackers.value"
          @goBack="goBack"
          @goForward="goForward"
          @reload="reload"
          @home="goHome"
          @navigate="(url) => void navigateToAddress(url)"
          @toggleBookmark="toggleBookmark"
          @toggleMenu="toggleMenu"
          @closeMenu="closeMenu"
          @openDownloads="downloads.openDownloadsPanel"
          @openPanel="openSidebarPanel"
          @toggleWebChat="toggleWebChat"
          @openPocketPanel="() => openSidebarPanel('pocket')"
          @openBookmarksPanel="() => openSidebarPanel('bookmarks')"
          @openHistoryPanel="() => openSidebarPanel('memory')"
          @openSettings="openSettings"
          @toggleSiteShields="toggleSiteShields"
          @print="() => void printPage()"
          @newTab="createNewTab"
          @newWindow="createNewWindow"
          @newIncognitoWindow="createNewIncognitoWindow"
          @zoomIn="() => void changeZoom(0.1)"
          @zoomOut="() => void changeZoom(-0.1)"
          @zoomReset="() => void changeZoom(0, true)"
          @cast="castPage"
          @find="() => { showFindBar.value = true; }"
          @moreTools="showMoreTools"
          @help="showHelp"
          @openProfile="openProfile"
          @exit="exitBrowser"
          @open-extensions="openExtensionsManager"
          @extension-popup-closed="onExtensionPopupClosed"
          @menuOpened="onMenuOpened"
          @menuClosed="onMenuClosed"
          :extensionsRefreshKey="extensionsRefreshKey"
          :recentHistory="recentHistoryItems"
          :bookmarkFolders="bookmarkFolderItems"
        />

        </WindowTitleBar>

        <!-- Bookmark Bar -->
        <BookmarkBar
          :visible="showBookmarkBar"
          :ui-locale="appUiLocale"
          :barBookmarks="barBookmarks"
          :folderNames="bookmarkFolders"
          :bookmarks="allBookmarks"
          :sidePanelOpen="sidebar.sidebarOpen.value"
          reorder-enabled
          @navigate="navigateToBookmark"
          @reorder="handleReorderBookmarkBar"
          @move-to-folder="onUpdateBookmarkFolder"
          @toggle-side-panel="toggleSidebarSmart"
          @open-apps="openBookmarkBarApps"
          @open-all-bookmarks="openAllBookmarksFromBar"
          @open-bookmarks-sidebar="openBookmarksSidebarFromBar"
          @open-in-new-tab="openUrlInNewTab"
          @open-in-new-window="openUrlInNewWindow"
          @open-in-incognito="openUrlInIncognito"
          @edit-bookmark="openBookmarkEditor"
          @remove-bookmark="onRemoveBookmark"
          @copy-url="onBookmarkUrlCopied"
          @add-bookmark="openAddBookmark"
          @group-created="onBookmarkGroupCreated"
          @toggle-bookmark-bar="toggleBookmarkBarFromMenu"
        />

        <!-- Content Area -->
        <div
          class="browser-body-row"
          :class="{ 'sidebar-position-left': sidebarPrefs.sidebarOnLeft.value, 'webchat-active': showWebChatView }"
        >
          <div
            class="browser-content"
            :class="{
              'webchat-content-host': showWebChatView,
              'browser-content--ntp': activeTab && isNewTabUrl(activeTab.url),
            }"
            ref="contentHost"
            @contextmenu="showPageContextMenu"
          >
            <!-- WebChat full-view in main content area (left of sidebar) -->
            <ImMessenger
              v-if="showWebChatView"
              full-width
              class="webchat-main-view"
              @status="onWebChatStatus"
            />

            <template v-else>
            <!-- Find Bar (inside content area) -->
            <FindBar
              :open="showFindBar"
              v-model:findQuery="findQuery"
              :findResults="findResults"
              :currentFindIndex="currentFindIndex"
              @find="handleFindInPage"
              @close="closeFindBar"
            />

            <!-- Page Content -->
            <NewTabPage
              v-if="activeTab && isNewTabUrl(activeTab.url)"
              :key="'ntp-' + activeTabId + '-' + activeNtpWallpaperId"
              :visible="true"
              :topSites="ntpTopSites"
              :pinnedTopSiteUrls="ntpPinnedSiteUrls"
              :links="ntpQuickLinks"
              :wallpaperId="activeNtpWallpaperId"
              :wallpaperDisplayUrl="activeNtpWallpaperUrl"
              :aiOnline="sidebar.aiOnline.value"
              :aiModel="browserConfig.aiModel.value"
              :onNavigate="navigateToAddress"
              :onOpenSettings="openSettings"
              @pin-site="handlePinSite"
              @unpin-site="handleUnpinSite"
              @remove-site="handleRemoveSite"
              @add-quick-link="handleAddQuickLink"
              @remove-quick-link="handleRemoveQuickLink"
              @add-top-site="handleAddTopSite"
            />
            <ChromeInternalView
              v-else-if="activeTab && isChromeInternalUrl(activeTab.url)"
              :url="activeTab.url"
              :content-host="contentHost"
              :p2p-room-id="sidebar.p2pRoomId.value"
              :ui-locale="appUiLocale"
              @navigate="navigateToAddress"
              @locale-change="onAppLocaleChange"
              @status="(msg: string) => { statusMessage = msg; }"
              @saved="onSettingsSaved"
              @extensions-changed="onExtensionsChanged"
              @tracking-changed="siteShields.loadTrackingProtection"
              @open-sidebar-customize="openSidebarCustomizeFromSettings"
              @ntp-layout-reset="handleNtpLayoutReset"
              @wallpaper-change="(id: string) => void applyNtpWallpaperChange(id)"
              @open-panel="onChromeInternalOpenPanel"
              @close="onChromeSettingsClose"
            />
            <div v-else-if="activeTab" class="webview-container">
              <!-- Screenshot placeholder when menu is open -->
              <div 
                v-if="showMenu && webviewScreenshot" 
                class="webview-screenshot-placeholder"
                :data-screenshot-length="webviewScreenshot?.length || 0"
              >
                <canvas ref="screenshotCanvas" class="screenshot-canvas" />
              </div>
              <!-- Fallback white background when menu is open but no screenshot -->
              <div v-else-if="showMenu && useNativeWebview && !webviewScreenshot" class="webview-white-placeholder">
                <div class="placeholder-text">{{ activeTab.title || 'Page' }}</div>
              </div>
              <div
                v-if="useNativeWebview"
                class="native-webview-host"
                aria-label="Native WebView"
              />
              <iframe
                v-else
                ref="webviewFrame"
                :src="contentFrameUrl"
                class="browser-webview"
                title="Browser Content"
                :sandbox="iframeSandbox"
                @load="onFrameLoad"
              />
            </div>
            <div v-else class="loading-placeholder">Loading...</div>
            </template>
          </div>

          <!-- Sidebar -->
          <BrowserSidebar
            :open="sidebar.sidebarOpen.value"
            :sidebar-panel="sidebar.sidebarPanel.value"
            :sidebar-position="sidebarPrefs.prefs.value.position"
            :icon-items="sidebarPrefs.iconItems.value"
            :sidebar-prefs="sidebarPrefs.prefs.value"
            :browser-tabs="tabs"
            :active-tab-id="activeTabId"
            :sorted-tabs="sortedTabsList as BrowserTab[]"
            :vertical-tab-strip-width="sidebarPrefs.prefs.value.verticalTabsInSidebar ? 280 : 220"
            :tab-context-menu="tabGroups.tabContextMenu.value"
            :tab-groups="tabGroups.tabGroups.value"
            :tab-bar-handlers="tabBarHandlers"
            :open-tabs-for-sync="openTabsForSync"
            :agentPanelOpen="sidebar.agentPanelOpen.value"
            :aiChatHistory="sidebar.aiChatHistory.value"
            :chatStreamBuffer="sidebar.chatStreamBuffer.value"
            :aiStreamMode="sidebar.aiStreamMode.value"
            :isLoading="sidebar.isLoading.value"
            :aiOnline="sidebar.aiOnline.value"
            :ai-chat-input="sidebar.aiChatInput.value"
            :agent-command="sidebar.agentCommand.value"
            :agentLog="sidebar.agentLog.value"
            :agentDomSummary="sidebar.agentDomSummary.value"
            :isAgentExecuting="sidebar.isAgentExecuting.value"
            :indexedMemoryGroups="sidebar.filteredIndexedMemoryGroups.value"
            :historyGroups="sidebar.filteredHistoryGroups.value"
            :memory-search-query="sidebar.memorySearchQuery.value"
            :bookmark-search-query="sidebar.bookmarkSearchQuery.value"
            :indexedCount="sidebar.indexedCount.value"
            :historyCount="sidebar.historyCount.value"
            :bookmarks="sidebar.filteredBookmarks.value"
            :p2pRoomId="sidebar.p2pRoomId.value"
            :canAnnouncePage="sidebar.canAnnouncePage.value"
            @close="sidebar.closeSidebar"
            @bookmark-search="(q) => { sidebar.bookmarkSearchQuery.value = q; }"
            @memory-search="(q) => { sidebar.memorySearchQuery.value = q; }"
            @open-panel="openSidebarPanel"
            @switch-tab="activateTab"
            @new-tab="createNewTab"
            @close-tab="closeTab"
            @sidebar-position-change="sidebarPrefs.setPosition"
            @vertical-tabs-in-sidebar-change="(v) => void onVerticalTabsInSidebarChange(v)"
            @toggle-sidebar-tool="onToggleSidebarTool"
            @navigate="navigateToAddress"
            @send-chat="sidebar.sendAiChat"
            @cancel-chat="sidebar.cancelChat"
            @toggle-agent="sidebar.toggleAgentPanel"
            @load-memory="sidebar.loadIndexedMemory"
            @remove-indexed="sidebar.removeIndexedPage"
            @clear-indexed="sidebar.clearIndexedMemory"
            @clear-history="sidebar.clearBrowsingHistory"
            @load-bookmarks="refreshBookmarkBar"
            @add-bookmark="openAddBookmark"
            @edit-bookmark="openBookmarkEditor"
            @open-in-new-tab="openUrlInNewTab"
            @reorder-bookmarks="handleReorderBookmarks"
            @remove-bookmark="onRemoveBookmark"
            @update-bookmark-folder="onUpdateBookmarkFolder"
            @agent-execute="sidebar.executeAgentCommand"
            @agent-compress="sidebar.compressCurrentDom"
            @agent-back="sidebar.agentBackToAi"
            @agent-preset="sidebar.onAgentPreset"
            @chat-input="(v: string) => { sidebar.aiChatInput.value = v; }"
            @agent-command-change="(v: string) => { sidebar.agentCommand.value = v; }"
            @agent-ask-ai="sidebar.sendAiChat"
            @agent-run-strategy="sidebar.runAgentHermesStrategy"
          />
        </div>
      </div>
    </div>

    <!-- Modals -->
    <DownloadPanel
      :showDownloads="downloads.showDownloadsPanel.value"
      :downloads="downloads.downloads.value"
      @close="downloads.closeDownloadsPanel"
      @open-folder="downloads.openDownloadsDir"
      @clear="downloads.clearDownloads"
      @open-file="downloads.openDownloadFile"
      @reveal-file="downloads.revealDownloadFile"
    />

    <TabGroupEditPrompt
      :offer="tabGroups.tabGroupEditOffer.value"
      :busy="tabGroups.tabGroupEditBusy.value"
      @save="(title, color) => tabGroups.saveTabGroupEdit(title, color)"
      @cancel="tabGroups.cancelTabGroupEdit"
    />
    <TabGroupDeletePrompt
      :group-title="tabGroups.tabGroupDeleteTitle.value"
      :busy="tabGroups.tabGroupDeleteBusy.value"
      @confirm="tabGroups.confirmTabGroupDelete"
      @cancel="tabGroups.cancelTabGroupDelete"
    />

    <ConfirmPrompt
      :offer="confirmDialog.confirmOffer.value"
      :busy="confirmDialog.confirmBusy.value"
      @confirm="confirmDialog.runConfirmDialog"
      @cancel="confirmDialog.cancelConfirmDialog"
    />

    <ExtensionPermissionPrompt :request="extensions.permRequest.value" />
    <ExtensionHostInstallPrompt :request="extensions.hostInstallRequest.value" />

    <BookmarkEditor
      :visible="showBookmarkEditor"
      :bookmark="editingBookmark"
      :draft="bookmarkDraft"
      :folders="bookmarkFolders"
      @close="onBookmarkEditorClose"
      @save="onBookmarkEditorSaved"
    />

    <StatusBar
      :message="statusMessage"
      :private-mode="browserConfig.privateMode.value"
      :https-only="browserConfig.httpsOnly.value"
      :block-popups="browserConfig.blockPopups.value"
      :privacy-stats="privacyStats"
      :is-agent-executing="sidebar.isAgentExecuting.value"
      :ai-model="browserConfig.aiModel.value"
      :is-online="isOnline"
      :agent-command="sidebar.agentCommand.value"
      :agent-log="sidebar.agentLog.value"
      :agent-dom-summary="sidebar.agentDomSummary.value"
    />

    <!-- Context Menu -->
    <ContextMenu
      :visible="showContextMenu"
      :x="contextMenuX"
      :y="contextMenuY"
      :items="contextMenuItems"
      @close="showContextMenu = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { startFrameGapMonitor } from '@/lib/perfLog';
import { peekCachedWallpaperDisplayUrl, ensureWallpaperDataUrl, saveWallpaperIdAndSync, invalidateWallpaperCache, getWallpaperById, wallpaperAssetUrl, wallpaperAbsoluteAssetUrl, peekWallpaperForWindowSession, pickRandomWallpaperForNewTabAsync, pickWallpaperForWindowSession, readLaunchWallpaperId, resolveWallpaperDisplayUrl } from '@/lib/newTabWallpaper';
import { shellLog } from '@/lib/diagnosticLog';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { canInvokeTauri } from '@/lib/tauri';
import type { Webview } from '@tauri-apps/api/webview';
import BrowserTabBar from '@/components/BrowserTabBar.vue';
import TabGroupEditPrompt from '@/components/TabGroupEditPrompt.vue';
import TabGroupDeletePrompt from '@/components/TabGroupDeletePrompt.vue';
import AddressBar from '@/components/AddressBar.vue';
import WindowTitleBar from '@/components/WindowTitleBar.vue';
import BookmarkBar from '@/components/BookmarkBar.vue';
import BookmarkEditor from '@/components/BookmarkEditor.vue';
import FindBar from '@/components/FindBar.vue';
import NewTabPage from '@/components/NewTabPage.vue';
import ChromeInternalView from '@/components/ChromeInternalView.vue';
import BrowserSidebar from '@/components/BrowserSidebar.vue';
import ImMessenger from '@/components/ImMessenger.vue';
import DownloadPanel from '@/components/DownloadPanel.vue';
import ConfirmPrompt from '@/components/ConfirmPrompt.vue';
import ExtensionPermissionPrompt from '@/components/ExtensionPermissionPrompt.vue';
import ExtensionHostInstallPrompt from '@/components/ExtensionHostInstallPrompt.vue';
import StatusBar from '@/components/StatusBar.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import { bookmarksOnBar } from '@/lib/bookmarks';
import { mergeBookmarkFolderNames, reconcileBookmarkBarGroupsStorage, isReservedBookmarkGroupName } from '@/lib/bookmarkGroups';
import {
  persistBookmarkAddToBackend,
  persistBookmarkRemoveFromBackend,
  syncBookmarksFromBackendIfTauri,
} from '@/lib/bookmarkBackendSync';
import { seedPresetBookmarksIfEmpty } from '@/lib/presetBookmarks';
import { fetchPrivacyStats } from '@/lib/privacyStats';
import { initNewTabPage } from '@/lib/newTabPage';
import {
  createTabWebview,
  closeTabWebview,
  hideTabWebview,
  showTabWebview,
  navigateTab,
  reloadTab,
  tabWebviewLabel,
  setTabPopupBlocking,
  getTabNavState,
  goBackTab,
  goForwardTab,
  toggleTabDevTools,
  setTabZoom,
  getTabTitle,
  watchWebviewLayout,
  moveWebviewOffScreen,
  restoreWebviewPosition,
} from '@/lib/exodusBrowser';
import { pocketSaveArticle } from '@/lib/localPocket';
import type { OpenTabSnapshot } from '@/lib/syncedTabs';
import {
  NEWTAB_INTERNAL_URL,
  findReusableNewTab,
  isNewTabUrl,
  DEFAULT_NTP_TOP_SITES,
  DEFAULT_QUICK_LINKS,
} from '@/lib/newTabPage';
import {
  addNtpTopSite,
  buildNtpTopSitesGrid,
  isNtpTopSitesGridFull,
  listPinnedNtpSiteUrls,
  pinNtpTopSite,
  removeNtpTopSite,
  unpinNtpTopSite,
} from '@/lib/ntpTopSitesStore';
import {
  addNtpQuickLink,
  buildNtpQuickLinks,
  isNtpQuickLinksFull,
  removeNtpQuickLink,
} from '@/lib/ntpQuickLinksStore';
import {
  isChromeInternalUrl,
  parseChromeInternalUrl,
  normalizeChromeInternalUrl,
  chromeInternalTitle,
  chromeInternalRoutePath,
  chromeInternalUrlFromRouteParam,
  type ChromeInternalPage,
} from '@/lib/chromeInternal';
import { printActivePage } from '@/lib/printPage';
import { reorderTabsById } from '@/lib/tabReorder';
import { resolveOmniboxInput } from '@/lib/omnibox';
import { applyHttpsOnly, iframeSandboxAttr } from '@/lib/privacySettings';
import { navFlagsFromTrack, recordTabNavigation } from '@/lib/tabNavStack';
import { faviconUrlFor } from '@/lib/favicon';
import { useBookmarks } from '@/composables/useBookmarks';
import { useHistory } from '@/composables/useHistory';
import { useRagService } from '@/composables/useMicroservice';
import { useBrowserConfig } from '@/composables/useBrowserConfig';
import { isChromeSettingsSectionHop, isChromeSettingsUrl } from '@/lib/chromeSettingsNav';
import { resolveAppLocale, writeAppLocale, type AppLocale } from '@/lib/appLocale';
import {
  resolveSettingsCloseTarget,
  shouldRememberSettingsReturnUrl,
} from '@/lib/settingsCloseNavigation';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { CONFIRM_DIALOG_KEY } from '@/lib/confirm';
import { useClosedTabs } from '@/composables/useClosedTabs';
import { useBrowserSitePermissions } from '@/composables/useBrowserSitePermissions';
import { useSafeBrowsingNavigation } from '@/composables/useSafeBrowsingNavigation';
import { useSiteShields } from '@/composables/useSiteShields';
import { useBrowserDownloads } from '@/composables/useBrowserDownloads';
import { rescanExtensions } from '@/lib/extensions/api';
import { useExtensions } from '@/composables/useExtensions';
import { useBrowserTabGroups } from '@/composables/useBrowserTabGroups';
import { buildTabBarHandlers } from '@/composables/useBrowserTabBarHandlers';
import { usePasswordSaveOffer } from '@/composables/usePasswordSaveOffer';
import { useFindInPage } from '@/composables/useFindInPage';
import { useBrowserSidebar } from '@/composables/useBrowserSidebar';
import { useSidebarPreferences } from '@/composables/useSidebarPreferences';
import type { SidebarPanel, QuickLink, BrowserTab, BookmarkItem } from '@/lib/browserTypes';
import type { SidebarToolId } from '@/lib/sidebarPreferences';
import { useBrowserSession } from '@/composables/useBrowserSession';
import { useBrowserTabLifecycle } from '@/composables/useBrowserTabLifecycle';
import { bindLifecycleRecovery } from '@/lib/appLifecycle';
import { focusOmniboxInput, mountBrowserShortcuts } from '@/lib/browserShortcuts';
import { OPEN_WEBCHAT_EVENT } from '@/lib/imChat';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { writeShowBookmarkBar } from '@/lib/browserSettings';
import { provide } from 'vue';
import { useRoute, useRouter } from 'vue-router';

function createTabId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

// Core state — seed one tab so the tab bar never renders with activeTabId=null
const initialTabId = createTabId();
const tabs = ref<BrowserTab[]>([
  {
    id: initialTabId,
    title: 'New Tab',
    url: NEWTAB_INTERNAL_URL,
    webview: null,
  },
]);
const activeTabId = ref<string>(initialTabId);
const contentHost = ref<HTMLElement>();
const webviewFrame = ref<HTMLIFrameElement>();
const iframeNavStacks = new Map<string, { stack: string[]; index: number }>();
const canGoBack = ref(false);
const canGoForward = ref(false);
const isBookmarked = ref(false);
const showBookmarkBar = ref(true);
/** Shell UI language (settings → Appearance → Language). */
const appUiLocale = ref<AppLocale>(resolveAppLocale());
const showFindBar = ref(false);
const findQuery = ref('');
const findResults = ref(0);
const currentFindIndex = ref(0);
const zoomLevel = ref(100);
const statusMessage = ref('');
const privateMode = ref(false);
const httpsOnly = ref(false);
const blockPopups = ref(true);
const isOnline = ref(navigator.onLine);
const showMenu = ref(false);
const webviewScreenshot = ref<string | null>(null);
const screenshotCanvas = ref<HTMLCanvasElement | null>(null);
const showContextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const showBookmarkEditor = ref(false);
const showQuitConfirmDialog = ref(false);
const extensionsRefreshKey = ref(0);

// Online/offline handlers (defined outside onMounted to be accessible in onUnmounted)
const handleOnline = () => { isOnline.value = true; };
const handleOffline = () => { isOnline.value = false; };
const editingBookmark = ref<BookmarkItem | null>(null);
const bookmarkDraft = ref<{ title?: string; url?: string } | null>(null);
const contextMenuItems = ref<Array<{id: string; label: string; icon?: string; shortcut?: string; disabled?: boolean; separator?: boolean; action?: () => void}>>([]);
const privacyStats = ref<{
  trackers_blocked: number;
  trackers_allowed: number;
  fingerprinting_blocked: number;
  fingerprinting_allowed: number;
} | null>(null);
const barBookmarks = ref<BookmarkItem[]>([]);
const bookmarkFolders = ref<string[]>([]);
// Computed
const activeTab = computed(() => {
  return tabs.value.find(t => t.id === activeTabId.value) || null;
});

/** Omnibox display URL (empty on new-tab, like Chrome). */
const addressBarUrl = computed(() => {
  const url = activeTab.value?.url ?? '';
  return isNewTabUrl(url) ? '' : url;
});

/** iframe src for Vite / non-Tauri browsing (Chrome-like content area). */
const contentFrameUrl = computed(() => {
  const url = activeTab.value?.url ?? '';
  if (!url || isNewTabUrl(url) || isChromeInternalUrl(url)) return 'about:blank';
  return applyHttpsOnly(url, httpsOnly.value);
});

const route = useRoute();
const router = useRouter();

const iframeSandbox = computed(() => iframeSandboxAttr(blockPopups.value));

// Composables
const { addToHistory, loadHistory } = useHistory();
const {
  addBookmark,
  removeBookmark,
  isBookmarked: checkIsBookmarked,
  loadBookmarks,
  getBookmarks,
  updateBookmark,
  bookmarks: bookmarkStore,
} = useBookmarks();
const allBookmarks = ref<BookmarkItem[]>([]);
const ntpQuickLinks = ref<QuickLink[]>([...DEFAULT_QUICK_LINKS]);
const ntpTopSites = ref<QuickLink[]>([...DEFAULT_NTP_TOP_SITES]);
const ntpPinnedSiteUrls = ref<string[]>([]);

/** Recent history items for Chrome menu submenu */
const recentHistoryItems = computed(() => {
  // Return recent history items (simplified for now)
  return tabs.value
    .filter(t => t.url && !isNewTabUrl(t.url))
    .slice(0, 8)
    .map(t => ({
      url: t.url,
      title: t.title || t.url
    }));
});

/** Bookmark folder items for Chrome menu submenu */
const bookmarkFolderItems = computed(() => {
  return bookmarkFolders.value
    .filter(folder => folder && !isReservedBookmarkGroupName(folder))
    .map(folder => ({
      name: folder,
      count: allBookmarks.value.filter(b => b.folder === folder).length
    }));
});

/** Reload NTP quick-link chips from localStorage. */
function refreshNtpQuickLinks(): void {
  try {
    ntpQuickLinks.value = buildNtpQuickLinks();
  } catch (error) {
    shellLog.error('refresh NTP quick links failed', error);
    ntpQuickLinks.value = [...DEFAULT_QUICK_LINKS];
  }
}

/** Reload NTP top-site grid from localStorage. */
function refreshNtpTopSites(): void {
  try {
    ntpTopSites.value = buildNtpTopSitesGrid();
    ntpPinnedSiteUrls.value = listPinnedNtpSiteUrls();
  } catch (error) {
    shellLog.error('refresh NTP top sites failed', error);
    ntpTopSites.value = [...DEFAULT_NTP_TOP_SITES];
    ntpPinnedSiteUrls.value = [];
  }
}

/** Wallpaper id/url for the active new-tab page (each tab has its own). */
const activeNtpWallpaperId = computed(() => {
  const tab = activeTab.value;
  if (tab && isNewTabUrl(tab.url) && tab.ntpWallpaperId) {
    return tab.ntpWallpaperId;
  }
  return peekWallpaperForWindowSession();
});

const activeNtpWallpaperUrl = computed(() => {
  const tab = activeTab.value;
  if (tab?.ntpWallpaperUrl) return tab.ntpWallpaperUrl;
  const id = activeNtpWallpaperId.value;
  const entry = getWallpaperById(id);
  return (
    peekCachedWallpaperDisplayUrl(id) ||
    wallpaperAbsoluteAssetUrl(entry.file) ||
    wallpaperAssetUrl(entry.file)
  );
});

function ntpWallpaperIdsInUse(excludeTabId?: string): string[] {
  return tabs.value
    .filter(
      (tab) =>
        tab.id !== excludeTabId &&
        isNewTabUrl(tab.url) &&
        typeof tab.ntpWallpaperId === 'string' &&
        tab.ntpWallpaperId.length > 0,
    )
    .map((tab) => tab.ntpWallpaperId as string);
}

function setTabWallpaper(tabId: string, id: string, url?: string): void {
  tabs.value = tabs.value.map((tab) =>
    tab.id === tabId
      ? { ...tab, ntpWallpaperId: id, ntpWallpaperUrl: url ?? tab.ntpWallpaperUrl }
      : tab,
  );
}

/** In-flight wallpaper assignments (avoid duplicate picks on startup). */
const tabWallpaperInflight = new Map<string, Promise<void>>();

/** Assign a fresh random wallpaper to a new-tab page tab. */
async function assignRandomWallpaperToTab(tabId: string, force = false): Promise<void> {
  const tab = tabs.value.find((entry) => entry.id === tabId);
  if (!tab || !isNewTabUrl(tab.url)) return;
  if (tab.ntpWallpaperId && !force) return;

  const inflight = tabWallpaperInflight.get(tabId);
  if (inflight && !force) {
    await inflight;
    return;
  }

  const job = (async () => {
    const current = tabs.value.find((entry) => entry.id === tabId);
    if (!current || !isNewTabUrl(current.url)) return;
    if (current.ntpWallpaperId && !force) return;

    const used = ntpWallpaperIdsInUse(tabId);
    const id = await pickRandomWallpaperForNewTabAsync(used);
    const paintUrl = await resolveWallpaperDisplayUrl(id);
    setTabWallpaper(tabId, id, paintUrl);
    shellLog.info('tab wallpaper assigned', { tabId, id, usedCount: used.length });

    void ensureWallpaperDataUrl(id)
      .then((url) => {
        if (url) setTabWallpaper(tabId, id, url);
      })
      .catch((error) => {
        shellLog.warn('tab wallpaper upgrade failed', error);
      });
  })();

  tabWallpaperInflight.set(tabId, job);
  try {
    await job;
  } finally {
    tabWallpaperInflight.delete(tabId);
  }
}
const ragService = useRagService();
const browserConfig = useBrowserConfig();
const confirmDialog = useConfirmDialog();
provide(CONFIRM_DIALOG_KEY, confirmDialog);
const closedTabsStack = useClosedTabs();
const sitePermissions = useBrowserSitePermissions();
const safeBrowsing = useSafeBrowsingNavigation({
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
});
const siteShields = useSiteShields({
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
  reloadActiveTab: async () => {
    if (activeTabId.value && useNativeWebview) {
      await reloadTab(tabWebviewLabel(activeTabId.value));
    }
  },
});
const downloads = useBrowserDownloads({
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
});
const extensions = useExtensions({
  getTabs: () => tabs.value,
  getActiveTabId: () => activeTabId.value,
  contentHost,
  onTabOps: () => {},
  onTabCreates: async () => [],
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
});
const useNativeWebview = ref(extensions.useNativeWebview);
const tabGroups = useBrowserTabGroups({
  getTabs: () => tabs.value,
  getActiveTabId: () => activeTabId.value,
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
});
const passwordSave = usePasswordSaveOffer({
  getActiveTabLabel: () => (activeTabId.value ? tabWebviewLabel(activeTabId.value) : ''),
  useNativeWebview,
  privateMode: browserConfig.privateMode,
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
});
const sortedTabsList = tabGroups.sortedTabs;
const shieldsEnabled = computed(() => siteShields.shieldsEnabled());
const tabLifecycle = useBrowserTabLifecycle({
  getTabs: () =>
    tabs.value.map((t) => ({
      id: t.id,
      url: t.url,
      title: t.title,
      pinned: t.pinned,
    })),
  getActiveTabId: () => activeTabId.value,
  useNativeWebview,
});
const findInPage = useFindInPage({
  useNativeWebview,
  getActiveTabId: () => activeTabId.value,
  getContentDocument: () =>
    contentHost.value?.querySelector('iframe')?.contentDocument ?? undefined,
  findQuery,
  findResults,
  currentFindIndex,
});
const sidebar = useBrowserSidebar({
  getCurrentUrl: () => activeTab.value?.url ?? '',
  getActiveTabLabel: () => (activeTabId.value ? tabWebviewLabel(activeTabId.value) : ''),
  useNativeWebview,
  getContentDocument: () =>
    contentHost.value?.querySelector('iframe')?.contentDocument ?? undefined,
  navigate: (url) => navigateToAddress(url),
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
  aiPort: browserConfig.aiPort,
  aiModel: browserConfig.aiModel,
  loadBookmarks: async () => loadBookmarks(),
  getBookmarks: () => getBookmarks(),
  removeBookmark: async (id) => removeBookmark(id),
  updateBookmarkFolder: async (id, folder) => {
    if (!isTauri()) return;
    try {
      await invoke('update_bookmark_folder', { id, folder: folder.trim() });
      await loadBookmarks();
      allBookmarks.value = getBookmarks();
    } catch (error) {
      console.error('update_bookmark_folder failed:', error);
      statusMessage.value = 'Failed to update bookmark folder';
    }
  },
});

const sidebarPrefs = useSidebarPreferences();

/** When true, WebChat occupies the main browser content area (left pane). */
const webChatFullViewOpen = ref(false);

const showWebChatView = computed(() => webChatFullViewOpen.value);

function onWebChatStatus(msg: string): void {
  statusMessage.value = msg;
}

/** Hide native webview while WebChat full-view is shown. */
async function hideWebviewForWebChat(): Promise<void> {
  const tab = activeTab.value;
  if (!useNativeWebview || !tab?.webview) return;
  await hideTabWebview(tab.webview);
}

/** Restore native webview after leaving WebChat full-view. */
async function restoreWebviewAfterWebChat(): Promise<void> {
  await nextTick();
  const tab = activeTab.value;
  if (!useNativeWebview || !tab?.webview || !contentHost.value) return;
  if (isNewTabUrl(tab.url) || isChromeInternalUrl(tab.url)) return;
  await showTabWebview(tab.webview, contentHost.value);
}

/** Hide native webview while Vue renders NTP or chrome:// internal overlays. */
async function hideWebviewForOverlayPage(): Promise<void> {
  const tab = activeTab.value;
  if (!useNativeWebview || !tab?.webview) return;
  try {
    await hideTabWebview(tab.webview);
  } catch (error) {
    console.error('hideWebviewForOverlayPage failed:', error);
  }
}

/** Toggle WebChat full-view vs normal browser content. */
async function toggleWebChat(forceOpen = false): Promise<void> {
  if (!forceOpen && webChatFullViewOpen.value) {
    webChatFullViewOpen.value = false;
    await restoreWebviewAfterWebChat();
    return;
  }
  if (webChatFullViewOpen.value && forceOpen) return;

  if (sidebar.sidebarOpen.value && sidebar.sidebarPanel.value === 'p2p') {
    sidebar.closeSidebar();
  }

  await hideWebviewForWebChat();
  webChatFullViewOpen.value = true;
}

/** Open sidebar panel respecting Customize tool visibility (Firefox-style). */
function openSidebarPanel(panel: SidebarPanel): void {
  const resolved = sidebarPrefs.resolvePanel(panel);
  
  // Lazy init: history when opening history panel
  if (resolved === 'memory') {
    void ensureHistoryLoaded();
  }
  
  // Lazy init: extensions when opening settings or extensions panel
  if (resolved === 'settings' || resolved === 'extensions') {
    void ensureExtensionsBackgroundsEnsured();
  }
  
  sidebar.openPanel(resolved);
}

/** Chrome bookmark bar — open apps page (chrome://apps → extensions grid). */
function openBookmarkBarApps(): void {
  void navigateToAddress('chrome://apps');
}

/** Chrome toolbar puzzle — open extensions manager (chrome://extensions). */
function openExtensionsManager(): void {
  void navigateToAddress('chrome://extensions');
}

/** Restore main tab webview after an embedded extension popup closes. */
function onExtensionPopupClosed(): void {
  void ensureActiveWebview();
}

/** Render screenshot to canvas with original resolution */
function renderScreenshotToCanvas(screenshotData: string): void {
  const renderStart = performance.now();
  
  // Use requestAnimationFrame for optimal rendering timing
  requestAnimationFrame(() => {
    const canvas = screenshotCanvas.value;
    if (!canvas) {
      // Fallback to nextTick if canvas not available
      nextTick(() => {
        const canvas = screenshotCanvas.value;
        if (!canvas) {
          console.warn('[BrowserPage] Canvas not available even after nextTick');
          return;
        }
        doRenderCanvas(canvas, screenshotData, renderStart);
      });
      return;
    }
    
    doRenderCanvas(canvas, screenshotData, renderStart);
  });
}

/** Actual canvas rendering logic */
function doRenderCanvas(canvas: HTMLCanvasElement, screenshotData: string, renderStart: number): void {
    
    const ctx = canvas.getContext('2d', { 
      alpha: false, // No transparency for better performance
      desynchronized: true, // Allow async rendering
      willReadFrequently: false // Optimize for write-only
    });
    if (!ctx) {
      console.error('[BrowserPage] Failed to get canvas context');
      return;
    }
    
    try {
      const parseStart = performance.now();
      
      // Parse JSON response with dimensions and base64 data
      const parsed = JSON.parse(screenshotData);
      const { width, height, data } = parsed;
      
      // Removed detailed logging for performance
      
      // Get the container's actual size
      const container = canvas.parentElement;
      const containerWidth = container?.clientWidth || 0;
      const containerHeight = container?.clientHeight || 0;
      
      // Set canvas resolution to match the original screenshot (high resolution)
      // This preserves the high-quality image data
      canvas.width = width;
      canvas.height = height;
      
      // Set CSS size to match container exactly
      // The browser will handle the scaling from high-res to display size
      canvas.style.width = `${containerWidth}px`;
      canvas.style.height = `${containerHeight}px`;
      
      const decodeStart = performance.now();
      
      // Decode base64 RGBA data
      const binaryString = atob(data);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      
      // Verify data size matches dimensions
      const expectedSize = width * height * 4; // RGBA = 4 bytes per pixel
      if (bytes.length !== expectedSize) {
        console.error('[BrowserPage] Data size mismatch:', bytes.length, 'vs expected', expectedSize);
        return;
      }
      
      // Directly put image data - browser CSS will handle scaling
      const imageData = new ImageData(new Uint8ClampedArray(bytes), width, height);
      ctx.putImageData(imageData, 0, 0);
    } catch (err) {
      console.error('[BrowserPage] Failed to render screenshot:', err);
    }
}

/** Menu opened - capture screenshot and move webview off-screen */
async function onMenuOpened(): Promise<void> {
  const tab = activeTab.value;
  console.log('[BrowserPage] onMenuOpened called - useNativeWebview:', useNativeWebview.value, 'tab:', tab?.id, 'has webview:', !!tab?.webview);
  
  if (!useNativeWebview.value || !tab?.webview) {
    console.log('[BrowserPage] Skipping menu open - no native webview');
    return;
  }
  
  try {
    // IMPORTANT: Capture screenshot BEFORE moving webview off-screen
    // Otherwise we'll capture empty/blank content
    console.log('[BrowserPage] Checking screenshot conditions - isTauri:', isTauri(), 'url:', tab.url);
    console.log('[BrowserPage] isNewTabUrl:', isNewTabUrl(tab.url), 'isChromeInternalUrl:', isChromeInternalUrl(tab.url));
    
    if (isTauri() && !isNewTabUrl(tab.url) && !isChromeInternalUrl(tab.url)) {
      console.log('[BrowserPage] Capturing screenshot for tab:', tab.id);
      try {
        const screenshotData = await invoke<string>('browser_capture_screenshot', { 
          label: tabWebviewLabel(tab.id) 
        });
        console.log('[BrowserPage] Screenshot captured, length:', screenshotData?.length || 0);
        
        if (screenshotData && screenshotData.length > 0) {
          webviewScreenshot.value = screenshotData;
          // Render immediately
          renderScreenshotToCanvas(screenshotData);
          console.log('[BrowserPage] Screenshot rendered, now moving webview');
        } else {
          console.warn('[BrowserPage] Screenshot is empty');
        }
      } catch (err) {
        console.error('[BrowserPage] Screenshot capture failed:', err);
      }
    } else {
      console.log('[BrowserPage] Skipping screenshot - new tab or chrome URL');
    }
    
    // Move webview off-screen AFTER screenshot is captured and rendered
    await moveWebviewOffScreen(tab.webview);
    console.log('[BrowserPage] Webview moved off-screen for menu');
  } catch (error) {
    console.error('onMenuOpened failed:', error);
  }
}

/** Menu closed - restore webview position and clear screenshot */
async function onMenuClosed(): Promise<void> {
  await nextTick();
  const tab = activeTab.value;
  if (!useNativeWebview.value || !tab?.webview || !contentHost.value) return;
  try {
    await restoreWebviewPosition(tab.webview, contentHost.value);
    // Clear screenshot after webview is restored
    setTimeout(() => {
      webviewScreenshot.value = null;
    }, 50);
    console.log('[BrowserPage] Webview position restored after menu');
  } catch (error) {
    console.error('onMenuClosed failed:', error);
  }
}

function bumpExtensionsToolbarRefresh(): void {
  extensionsRefreshKey.value += 1;
}

/** Chrome bookmark bar — right-pinned “All bookmarks” opens sidebar panel. */
function openAllBookmarksFromBar(): void {
  openSidebarPanel('bookmarks');
  sidebar.sidebarOpen.value = true;
}

/** Open URL in a new tab (bookmark bar middle-click / context menu). */
async function openUrlInNewTab(url: string, title?: string): Promise<void> {
  const raw = url.trim();
  if (!raw) return;

  const targetUrl = isChromeInternalUrl(raw)
    ? normalizeChromeInternalUrl(raw)
    : applyHttpsOnly(
        raw.startsWith('http://') || raw.startsWith('https://') ? raw : `https://${raw}`,
        browserConfig.httpsOnly.value,
      );

  const internalPage = isChromeInternalUrl(targetUrl) ? parseChromeInternalUrl(targetUrl) : null;
  const tabTitle =
    title?.trim() || (internalPage && internalPage !== 'unknown' ? chromeInternalTitle(internalPage) : targetUrl);

  const newTab: BrowserTab = {
    id: createTabId(),
    title: tabTitle,
    url: targetUrl,
    webview: null,
    pinned: false,
    favicon: faviconUrlFor(targetUrl) ?? undefined,
  };
  tabs.value = [...tabs.value, newTab];
  await activateTab(newTab.id);

  if (isNewTabUrl(targetUrl)) {
    await assignRandomWallpaperToTab(newTab.id, true);
  } else if (
    useNativeWebview &&
    contentHost.value &&
    !isChromeInternalUrl(targetUrl)
  ) {
    const label = tabWebviewLabel(newTab.id);
    const wv = await createTabWebview(contentHost.value, label, targetUrl);
    await setTabPopupBlocking(label, blockPopups.value);
    tabs.value = tabs.value.map((t) => (t.id === newTab.id ? { ...t, webview: wv } : t));
    for (const t of tabs.value) {
      if (t.id !== newTab.id && t.webview) await hideTabWebview(t.webview);
    }
  }

  if (internalPage && internalPage !== 'unknown' && internalPage !== 'newtab') {
    const routePath = chromeInternalRoutePath(internalPage);
    if (routePath && route.path !== routePath) {
      try {
        await router.replace(routePath);
      } catch (error) {
        console.error('router.replace failed:', error);
      }
    }
  }

  extensions.syncRegistry();
  void browserSession.saveSession();
  void tabLifecycle.registerTab({ id: newTab.id, url: targetUrl, title: tabTitle });
}

/** Open URL in a new window (bookmark bar context menu). */
async function openUrlInNewWindow(url: string, title?: string): Promise<void> {
  const raw = url.trim();
  if (!raw) return;

  const targetUrl = isChromeInternalUrl(raw)
    ? normalizeChromeInternalUrl(raw)
    : applyHttpsOnly(
        raw.startsWith('http://') || raw.startsWith('https://') ? raw : `https://${raw}`,
        browserConfig.httpsOnly.value,
      );

  if (isTauri()) {
    try {
      await invoke('open_new_window_from_dock', { url: targetUrl });
      statusMessage.value = 'Opened in new window';
    } catch (error) {
      console.error('openUrlInNewWindow failed:', error);
      statusMessage.value = 'Failed to open new window';
    }
  } else {
    // Fallback to new tab in web mode
    await openUrlInNewTab(targetUrl, title);
    statusMessage.value = 'Opened in new tab (new window only available in desktop app)';
  }
}

/** Open URL in incognito window (bookmark bar context menu). */
async function openUrlInIncognito(url: string, title?: string): Promise<void> {
  const raw = url.trim();
  if (!raw) return;

  const targetUrl = isChromeInternalUrl(raw)
    ? normalizeChromeInternalUrl(raw)
    : applyHttpsOnly(
        raw.startsWith('http://') || raw.startsWith('https://') ? raw : `https://${raw}`,
        browserConfig.httpsOnly.value,
      );

  // Toggle private mode and open in new tab
  const wasPrivate = privateMode.value;
  privateMode.value = true;
  await openUrlInNewTab(targetUrl, title);
  if (!wasPrivate) {
    statusMessage.value = 'Opened in private mode';
  }
}

function toggleBookmarkBarFromMenu(): void {
  showBookmarkBar.value = !showBookmarkBar.value;
  writeShowBookmarkBar(showBookmarkBar.value);
  statusMessage.value = showBookmarkBar.value ? 'Bookmarks bar shown' : 'Bookmarks bar hidden';
}

function openBookmarksSidebarFromBar(): void {
  openSidebarPanel('bookmarks');
  sidebar.sidebarOpen.value = true;
}

function openBookmarkEditor(bookmark: BookmarkItem): void {
  bookmarkDraft.value = null;
  editingBookmark.value = bookmark;
  showBookmarkEditor.value = true;
}

function openAddBookmark(): void {
  editingBookmark.value = null;
  const tab = activeTab.value;
  if (tab && !isNewTabUrl(tab.url) && !isChromeInternalUrl(tab.url) && tab.url) {
    bookmarkDraft.value = {
      title: tab.title?.trim() || tab.url,
      url: tab.url,
    };
  } else {
    bookmarkDraft.value = null;
  }
  showBookmarkEditor.value = true;
}

function onBookmarkEditorClose(): void {
  showBookmarkEditor.value = false;
  editingBookmark.value = null;
  bookmarkDraft.value = null;
}

async function onBookmarkEditorSaved(saved: BookmarkItem): Promise<void> {
  if (isTauri()) {
    await persistBookmarkAddToBackend(saved.url, saved.title, saved.folder);
  }
  await refreshBookmarkBar();
  statusMessage.value = 'Bookmark saved';
  onBookmarkEditorClose();
}

function onBookmarkUrlCopied(url: string): void {
  statusMessage.value = `Copied: ${url}`;
}

/** Firefox Customize: disable current tool → switch to a valid panel. */
function onToggleSidebarTool(tool: SidebarToolId): void {
  sidebarPrefs.toggleTool(tool);
  const current = sidebar.sidebarPanel.value;
  if (
    current !== 'customize' &&
    !sidebarPrefs.prefs.value.enabledTools.includes(current as SidebarToolId)
  ) {
    openSidebarPanel(sidebarPrefs.defaultPanel());
  }
}

/** Enable vertical tabs in sidebar and show Tabs panel. */
async function onVerticalTabsInSidebarChange(enabled: boolean): Promise<void> {
  await sidebarPrefs.setVerticalTabsInSidebar(enabled);
  if (enabled) {
    openSidebarPanel('tabs');
    sidebar.sidebarOpen.value = true;
  } else if (sidebar.sidebarPanel.value === 'tabs') {
    openSidebarPanel(sidebarPrefs.defaultPanel());
  }
}

/** Open/close sidebar; when opening with vertical tabs, land on Tabs panel. */
function toggleSidebarSmart(): void {
  if (sidebar.sidebarOpen.value) {
    sidebar.closeSidebar();
    return;
  }
  if (sidebarPrefs.prefs.value.verticalTabsInSidebar) {
    openSidebarPanel('tabs');
    return;
  }
  sidebar.toggleSidebar();
}

/** After prefs load, ensure current panel is still enabled. */
const tabBarBindProps = computed(() => ({
  tabs: tabs.value,
  activeTabId: activeTabId.value,
  sortedTabs: sortedTabsList.value,
  tabContextMenu: tabGroups.tabContextMenu.value,
  tabGroups: tabGroups.tabGroups.value,
}));

const openTabsForSync = computed((): OpenTabSnapshot[] =>
  tabs.value.map((t) => ({ id: t.id, title: t.title, url: t.url })),
);

function toggleTabPin(tabId: string): void {
  tabs.value = tabs.value.map((t) => (t.id === tabId ? { ...t, pinned: !t.pinned } : t));
  void browserSession.saveSession();
  extensions.syncRegistry();
}

async function duplicateTabById(tabId: string): Promise<void> {
  const source = tabs.value.find((t) => t.id === tabId);
  if (!source) return;
  const newTab: BrowserTab = {
    id: createTabId(),
    title: source.title,
    url: source.url,
    webview: null,
    pinned: false,
    favicon: source.favicon,
  };
  tabs.value = [...tabs.value, newTab];
  await activateTab(newTab.id);
  if (isNewTabUrl(newTab.url)) {
    await assignRandomWallpaperToTab(newTab.id, true);
  }
  if (useNativeWebview && contentHost.value && !isNewTabUrl(newTab.url)) {
    const label = tabWebviewLabel(newTab.id);
    const wv = await createTabWebview(contentHost.value, label, newTab.url);
    await setTabPopupBlocking(label, blockPopups.value);
    tabs.value = tabs.value.map((t) => (t.id === newTab.id ? { ...t, webview: wv } : t));
  }
  extensions.syncRegistry();
  void browserSession.saveSession();
  void tabLifecycle.registerTab({ id: newTab.id, url: newTab.url, title: newTab.title });
}

function handleReorderTabs(fromId: string, toId: string): void {
  tabs.value = reorderTabsById(tabs.value, fromId, toId);
  extensions.syncRegistry();
  void browserSession.saveSession();
}

const tabBarHandlers = buildTabBarHandlers({
  activateTab,
  createNewTab,
  closeTab,
  toggleTabPin,
  duplicateTab: duplicateTabById,
  reorderTabs: handleReorderTabs,
  tabGroups,
});

function openSidebarCustomizeFromSettings(): void {
  openSidebarPanel('customize');
  sidebar.sidebarOpen.value = true;
}

async function savePageToReadingList(): Promise<void> {
  const tab = getActiveTab();
  const url = tab?.url ?? '';
  if (!url || isNewTabUrl(url)) {
    statusMessage.value = 'Cannot save this page to reading list';
    return;
  }
  try {
    await pocketSaveArticle({
      url,
      title: tab?.title || url,
      content: '',
      author: null,
      tags: ['reading-list'],
    });
    statusMessage.value = 'Saved to reading list';
    openSidebarPanel('reading');
    sidebar.sidebarOpen.value = true;
  } catch (error) {
    console.error('savePageToReadingList failed:', error);
    statusMessage.value = 'Failed to save to reading list';
  }
}


// Cleanup functions
let unbeforeunload: (() => void) | undefined;
let unbindShortcuts: (() => void) | undefined;
let unbindLifecycle: (() => void) | undefined;
let unlistenWebChat: UnlistenFn | undefined;
let unlistenHistory: UnlistenFn | undefined;

// Store all event listener cleanup functions
const eventListeners: UnlistenFn[] = [];
const watchStoppers: Array<() => void> = [];
let unlistenMenuEvents: UnlistenFn | undefined;
let onOpenWebChatUi: (() => void) | undefined;

// Event-driven lazy initialization flags
let newTabPageInitialized = false;
let historyLoaded = false;
let privacyStatsRefreshed = false;
let extensionsBackgroundsEnsured = false;
let ntpWallpaperSynced = false;

// Event-driven lazy initialization functions
async function ensureNewTabPageInitialized(): Promise<void> {
  if (newTabPageInitialized) return;
  newTabPageInitialized = true;
  shellLog.timeStart('ensureNewTabPageInitialized');
  try {
    shellLog.info('lazy init: new tab page');
    await initNewTabPage();
    await ensureNtpWallpaperSynced();
    shellLog.info('lazy init: new tab page complete');
  } catch (e) {
    shellLog.error('init new tab page failed', e);
  } finally {
    shellLog.timeEnd('ensureNewTabPageInitialized');
  }
}

async function ensureHistoryLoaded(): Promise<void> {
  if (historyLoaded) return;
  historyLoaded = true;
  try {
    console.log('[BrowserPage] Lazy init: history');
    await loadHistory();
  } catch (e) {
    console.error('[BrowserPage] Failed to load history:', e);
  }
}

async function ensurePrivacyStatsRefreshed(): Promise<void> {
  if (privacyStatsRefreshed) return;
  privacyStatsRefreshed = true;
  try {
    console.log('[BrowserPage] Lazy init: privacy stats');
    await refreshPrivacyStats();
  } catch (e) {
    console.error('[BrowserPage] Failed to refresh privacy stats:', e);
  }
}

async function ensureExtensionsBackgroundsEnsured(): Promise<void> {
  if (extensionsBackgroundsEnsured) return;
  extensionsBackgroundsEnsured = true;
  try {
    console.log('[BrowserPage] Lazy init: extension backgrounds');
    await extensions.ensureBackgroundsLazy();
  } catch (e) {
    console.error('[BrowserPage] Failed to ensure extension backgrounds:', e);
  }
}

async function ensureNtpWallpaperSynced(): Promise<void> {
  if (ntpWallpaperSynced) return;
  ntpWallpaperSynced = true;
  shellLog.timeStart('ensureNtpWallpaperSynced');
  try {
    shellLog.info('lazy init: NTP wallpaper');
    const tabId = activeTabId.value;
    const tab = tabs.value.find((entry) => entry.id === tabId);
    if (tab && isNewTabUrl(tab.url) && !tab.ntpWallpaperId) {
      if (readLaunchWallpaperId()) {
        const id = await pickWallpaperForWindowSession();
        const url = await resolveWallpaperDisplayUrl(id);
        setTabWallpaper(tabId, id, url);
      } else {
        await assignRandomWallpaperToTab(tabId, true);
      }
    }
    shellLog.info('NTP wallpaper ready', {
      tabId,
      id: activeNtpWallpaperId.value,
      hasUrl: !!activeNtpWallpaperUrl.value,
    });
  } catch (e) {
    shellLog.error('sync NTP wallpaper failed', e);
    if (activeTabId.value) {
      await assignRandomWallpaperToTab(activeTabId.value, true);
    }
  } finally {
    shellLog.timeEnd('ensureNtpWallpaperSynced');
  }
}

/** Apply wallpaper picked in Settings → Appearance to the live new tab overlay. */
async function applyNtpWallpaperChange(id: string): Promise<void> {
  if (!id) return;
  const tabId = activeTabId.value;
  if (!tabId) return;
  try {
    const tab = tabs.value.find((entry) => entry.id === tabId);
    if (tab?.ntpWallpaperId) invalidateWallpaperCache(tab.ntpWallpaperId);
    const url = await ensureWallpaperDataUrl(id);
    setTabWallpaper(tabId, id, url);
    await saveWallpaperIdAndSync(id);
  } catch (error) {
    console.error('[BrowserPage] applyNtpWallpaperChange failed:', error);
  }
}

function getActiveTab(): BrowserTab | undefined {
  return tabs.value.find((t) => t.id === activeTabId.value);
}

async function refreshBookmarkBar(): Promise<void> {
  await syncBookmarksFromBackendIfTauri();
  loadBookmarks();
  const fullList = bookmarkStore.value.map((bookmark) => ({
    id: bookmark.id,
    url: bookmark.url,
    title: bookmark.title,
    created_at: bookmark.created_at,
    folder: bookmark.folder,
    favicon: bookmark.favicon,
    bar_order: bookmark.bar_order,
  }));
  allBookmarks.value = fullList;
  barBookmarks.value = bookmarksOnBar(fullList);
  reconcileBookmarkBarGroupsStorage(fullList);
  bookmarkFolders.value = mergeBookmarkFolderNames(fullList);
}

function onBookmarkGroupCreated(name: string, color: string): void {
  shellLog.info('bookmark group created', { name, color });
  void refreshBookmarkBar();
}

async function handleReorderBookmarkBar(orderedIds: string[]): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('reorder_bookmarks_bar', { orderedIds });
      await loadBookmarks();
    } else {
      orderedIds.forEach((id, index) => {
        updateBookmark(id, { bar_order: index });
      });
    }
    await refreshBookmarkBar();
  } catch (error) {
    shellLog.error('reorder bookmark bar failed', error);
    statusMessage.value = 'Failed to reorder bookmarks';
  }
}

async function handleReorderBookmarks(orderedIds: string[]): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('reorder_bookmarks', { orderedIds });
      await loadBookmarks();
    } else {
      orderedIds.forEach((id, index) => {
        updateBookmark(id, { order: index });
      });
    }
    await refreshBookmarkBar();
  } catch (error) {
    shellLog.error('reorder bookmarks failed', error);
    statusMessage.value = 'Failed to reorder bookmarks';
  }
}

async function onRemoveBookmark(id: string): Promise<void> {
  if (isTauri()) {
    await persistBookmarkRemoveFromBackend(id);
  } else {
    await removeBookmark(id);
  }
  await refreshBookmarkBar();
}

async function onUpdateBookmarkFolder(id: string, folder: string): Promise<void> {
  try {
    if (isTauri()) {
      await invoke('update_bookmark_folder', { id, folder: folder.trim() });
      await loadBookmarks();
    } else {
      updateBookmark(id, { folder: folder.trim() || undefined });
    }
    await refreshBookmarkBar();
  } catch (error) {
    console.error('update_bookmark_folder failed:', error);
    statusMessage.value = 'Failed to update bookmark folder';
  }
}

async function refreshPrivacyStats(): Promise<void> {
  const stats = await fetchPrivacyStats();
  if (stats) {
    privacyStats.value = {
      trackers_blocked: stats.trackers_blocked,
      trackers_allowed: 0,
      fingerprinting_blocked: stats.fingerprinting_blocked,
      fingerprinting_allowed: 0,
    };
  }
}

function goHome(): void {
  void navigateToAddress(browserConfig.homepageUrl.value);
}

function toggleMenu(): void {
  console.log('=== toggleMenu called ===', showMenu.value);
  showMenu.value = !showMenu.value;
  console.log('=== toggleMenu new value ===', showMenu.value);

  // Position dropdown using JavaScript when opening
  if (showMenu.value) {
    nextTick(() => {
      const button = document.querySelector('.chrome-menu-btn') as HTMLElement;
      const dropdown = document.querySelector('.chrome-menu-dropdown') as HTMLElement;
      if (button && dropdown) {
        const buttonRect = button.getBoundingClientRect();
        dropdown.style.position = 'fixed';
        dropdown.style.top = `${buttonRect.bottom + 4}px`;
        dropdown.style.right = 'auto';
        dropdown.style.left = `${buttonRect.left}px`;
        // Ensure dropdown doesn't overflow viewport
        const dropdownRect = dropdown.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        if (dropdownRect.right > viewportWidth) {
          dropdown.style.left = `${viewportWidth - dropdownRect.width - 8}px`;
        }
      }
    });
  }
}

function closeMenu(): void {
  showMenu.value = false;
}

function applyIframeNavFlags(tabId: string): void {
  const flags = navFlagsFromTrack(iframeNavStacks.get(tabId));
  canGoBack.value = flags.canGoBack;
  canGoForward.value = flags.canGoForward;
}

function onFrameLoad(): void {
  const tabId = activeTabId.value;
  const tab = getActiveTab();
  if (!tabId || !tab || isNewTabUrl(tab.url)) return;
  recordTabNavigation(iframeNavStacks, tabId, tab.url);
  applyIframeNavFlags(tabId);
  void refreshActiveTabTitle();
}

async function refreshActiveTabTitle(): Promise<void> {
  const tab = getActiveTab();
  if (!tab || isNewTabUrl(tab.url)) {
    document.title = 'New Tab - Exodus Browser';
    return;
  }
  try {
    let title = '';
    if (useNativeWebview && activeTabId.value) {
      title = (await getTabTitle(tabWebviewLabel(activeTabId.value))).trim();
    } else if (webviewFrame.value?.contentDocument) {
      title = webviewFrame.value.contentDocument.title.trim();
    }
    if (title) {
      const icon = faviconUrlFor(tab.url) ?? undefined;
      tabs.value = tabs.value.map((t) =>
        t.id === tab.id ? { ...t, title, favicon: icon ?? t.favicon } : t,
      );
      document.title = `${title} - Exodus Browser`;
    }
  } catch (error) {
    console.error('refreshActiveTabTitle failed:', error);
  }
}

/** Save the current page as a file (Chrome ⌘S / menu Save as). */
async function savePage(): Promise<void> {
  const tab = getActiveTab();
  if (!tab?.url) {
    statusMessage.value = 'Cannot save this page';
    return;
  }
  if (isNewTabUrl(tab.url) || isChromeInternalUrl(tab.url)) {
    statusMessage.value = 'Cannot save this page';
    return;
  }
  const filename = tab.title?.replace(/[^a-z0-9]/gi, '_') || 'page';
  await downloads.startDownload(tab.url, `${filename}.html`);
  statusMessage.value = 'Downloading page...';
}

/** Open print dialog for the active tab (Chrome ⌘P / menu Print). */
async function printPage(): Promise<void> {
  const tab = getActiveTab();
  const ok = await printActivePage({
    useNativeWebview: useNativeWebview.value,
    activeTabId: activeTabId.value,
    tabUrl: tab?.url ?? '',
    iframe: webviewFrame.value,
  });
  statusMessage.value = ok ? 'Print dialog opened' : 'Print not available for this page';
}

/** Open sidebar/panel for chrome:// hub pages (history, bookmarks, downloads). */
function onChromeInternalOpenPanel(panel: 'memory' | 'bookmarks' | 'downloads'): void {
  if (panel === 'downloads') {
    downloads.openDownloadsPanel();
    return;
  }
  openSidebarPanel(panel);
  sidebar.sidebarOpen.value = true;
}

/** Apply chrome:// side effects when a tab shows an internal page. */
function applyChromeInternalSideEffects(page: ChromeInternalPage): void {
  if (page === 'history') {
    openSidebarPanel('memory');
    sidebar.sidebarOpen.value = true;
    return;
  }
  if (page === 'bookmarks') {
    openSidebarPanel('bookmarks');
    sidebar.sidebarOpen.value = true;
    return;
  }
  if (page === 'downloads') {
    downloads.openDownloadsPanel();
  }
}

/** Navigate active tab to a chrome:// URL and sync hash route. */
async function commitChromeInternalNavigation(targetUrl: string): Promise<void> {
  const page = parseChromeInternalUrl(targetUrl);
  if (!page) return;
  if (page === 'newtab') {
    await commitNavigation(NEWTAB_INTERNAL_URL);
    return;
  }
  const normalized = normalizeChromeInternalUrl(targetUrl);
  const tabId = activeTabId.value;
  if (!tabId) return;

  const idx = tabs.value.findIndex((t) => t.id === tabId);
  if (idx < 0) return;

  const previousUrl = tabs.value[idx]?.url ?? '';
  const settingsSectionHop = isChromeSettingsSectionHop(previousUrl, normalized);

  const rememberReturn =
    (page === 'settings' || page === 'extensions') &&
    shouldRememberSettingsReturnUrl(previousUrl, settingsSectionHop);
  const nextTabs = [...tabs.value];
  nextTabs[idx] = {
    ...nextTabs[idx],
    url: normalized,
    title: chromeInternalTitle(page),
    favicon: faviconUrlFor(normalized) ?? undefined,
    ...(rememberReturn ? { settingsReturnUrl: previousUrl } : {}),
  };
  tabs.value = nextTabs;
  statusMessage.value = chromeInternalTitle(page);
  applyChromeInternalSideEffects(page);
  if (useNativeWebview) {
    await hideWebviewForOverlayPage();
  }
  if (!settingsSectionHop) {
    extensions.syncRegistry();
  }
  void browserSession.saveSession();

  const routePath = chromeInternalRoutePath(page);
  if (routePath && route.path !== routePath) {
    try {
      await router.replace(routePath);
    } catch (error) {
      console.error('router.replace failed:', error);
    }
  }
}

function openSettings(): void {
  showMenu.value = false;
  void navigateToAddress('chrome://settings');
}

/** Close settings and restore the page that was open before chrome://settings. */
async function onChromeSettingsClose(): Promise<void> {
  const tab = getActiveTab();
  const tabId = tab?.id;
  const targetUrl = resolveSettingsCloseTarget(tab ?? null);
  if (tabId) {
    tabs.value = tabs.value.map((t) =>
      t.id === tabId ? { ...t, settingsReturnUrl: undefined } : t,
    );
  }
  await navigateToAddress(targetUrl);
}

function showPageContextMenu(event: MouseEvent): void {
  event.preventDefault();
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  
  const canBack = canGoBack.value;
  const canForward = canGoForward.value;
  const currentUrl = activeTab.value?.url;
  const canSave = currentUrl && !isNewTabUrl(currentUrl) && !isChromeInternalUrl(currentUrl);
  const canPrint = canSave;
  
  contextMenuItems.value = [
    { id: 'back', label: 'Back', icon: '←', shortcut: '⌘+[', disabled: !canBack, action: () => void goBack() },
    { id: 'forward', label: 'Forward', icon: '→', shortcut: '⌘+]', disabled: !canForward, action: () => void goForward() },
    { id: 'reload', label: 'Reload', icon: '↻', shortcut: '⌘+R', action: () => void reload() },
    { id: 'separator1', label: '', separator: true },
    { id: 'save', label: 'Save as...', icon: '💾', shortcut: '⌘+S', disabled: !canSave, action: () => void savePage() },
    { id: 'print', label: 'Print...', icon: '🖨', shortcut: '⌘+P', disabled: !canPrint, action: () => void printPage() },
    { id: 'separator2', label: '', separator: true },
    { id: 'bookmark', label: isBookmarked.value ? 'Remove bookmark' : 'Bookmark this tab', icon: isBookmarked.value ? '★' : '☆', shortcut: '⌘+D', action: () => void toggleBookmark() },
    { id: 'separator3', label: '', separator: true },
    { id: 'new-tab', label: 'New tab', icon: '+', shortcut: '⌘+T', action: () => void createNewTab() },
    { id: 'close-tab', label: 'Close tab', icon: '×', shortcut: '⌘+W', action: () => activeTabId.value && void closeTab(activeTabId.value) },
    { id: 'separator4', label: '', separator: true },
    { id: 'zoom-in', label: 'Zoom in', icon: '🔍+', shortcut: '⌘++', action: () => void changeZoom(0.1) },
    { id: 'zoom-out', label: 'Zoom out', icon: '🔍-', shortcut: '⌘+-', action: () => void changeZoom(-0.1) },
    { id: 'zoom-reset', label: 'Reset zoom', icon: '🔍', shortcut: '⌘+0', action: () => void changeZoom(0, true) },
    { id: 'separator5', label: '', separator: true },
    { id: 'find', label: 'Find...', icon: '🔍', shortcut: '⌘+F', action: () => { showFindBar.value = true; } },
    { id: 'separator6', label: '', separator: true },
    { id: 'settings', label: 'Settings', icon: '⚙', action: () => openSettings() },
  ];
  
  showContextMenu.value = true;
}

async function onSettingsSaved(): Promise<void> {
  showBookmarkBar.value = browserConfig.showBookmarkBar.value;
  httpsOnly.value = browserConfig.httpsOnly.value;
  privateMode.value = browserConfig.privateMode.value;
  blockPopups.value = browserConfig.blockPopups.value;
}

/** Apply UI language chosen in settings (Appearance). */
function onAppLocaleChange(locale: AppLocale): void {
  appUiLocale.value = locale;
  writeAppLocale(locale);
}

function onExtensionsChanged(): void {
  extensions.syncRegistry();
  bumpExtensionsToolbarRefresh();
  window.dispatchEvent(new CustomEvent('exodus-extensions-changed'));
}

// New tab page site management handlers
function handlePinSite(site: QuickLink): void {
  try {
    if (pinNtpTopSite(site)) {
      refreshNtpTopSites();
      statusMessage.value = `Pinned ${site.title || site.url} to front`;
      return;
    }
    statusMessage.value = isNtpTopSitesGridFull()
      ? 'Top sites grid is full — remove a site first'
      : 'Could not pin top site';
  } catch (error) {
    shellLog.error('pin NTP top site failed', error);
    statusMessage.value = 'Failed to pin top site';
  }
}

function handleUnpinSite(site: QuickLink): void {
  try {
    unpinNtpTopSite(site);
    refreshNtpTopSites();
    statusMessage.value = `Unpinned ${site.title || site.url}`;
  } catch (error) {
    shellLog.error('unpin NTP top site failed', error);
    statusMessage.value = 'Failed to unpin top site';
  }
}

function handleRemoveSite(site: QuickLink): void {
  try {
    removeNtpTopSite(site);
    refreshNtpTopSites();
    statusMessage.value = `Removed ${site.title || site.url} from top sites`;
  } catch (error) {
    shellLog.error('remove NTP top site failed', error);
    statusMessage.value = 'Failed to remove top site';
  }
}

function handleAddQuickLink(link: QuickLink): void {
  try {
    if (addNtpQuickLink(link)) {
      refreshNtpQuickLinks();
      statusMessage.value = `Added ${link.title || link.url} to quick links`;
      return;
    }
    statusMessage.value = isNtpQuickLinksFull()
      ? 'Quick links row is full — remove a link first'
      : 'Could not add quick link';
  } catch (error) {
    shellLog.error('add NTP quick link failed', error);
    statusMessage.value = 'Failed to add quick link';
  }
}

function handleRemoveQuickLink(link: QuickLink): void {
  try {
    removeNtpQuickLink(link);
    refreshNtpQuickLinks();
    statusMessage.value = `Removed ${link.title || link.url} from quick links`;
  } catch (error) {
    shellLog.error('remove NTP quick link failed', error);
    statusMessage.value = 'Failed to remove quick link';
  }
}

function handleAddTopSite(site: QuickLink): void {
  try {
    if (addNtpTopSite(site)) {
      refreshNtpTopSites();
      statusMessage.value = `Added ${site.title || site.url} to top sites`;
      return;
    }
    statusMessage.value = isNtpTopSitesGridFull()
      ? 'Top sites grid is full — remove a site first'
      : 'Could not add top site';
  } catch (error) {
    shellLog.error('add NTP top site failed', error);
    statusMessage.value = 'Failed to add top site';
  }
}

/** Refresh NTP grid/chips after settings factory reset. */
function handleNtpLayoutReset(): void {
  try {
    refreshNtpTopSites();
    refreshNtpQuickLinks();
  } catch (error) {
    shellLog.error('refresh NTP layout after reset failed', error);
    statusMessage.value = 'Failed to refresh new tab page layout';
  }
}

async function toggleSiteShields(): Promise<void> {
  const url = activeTab.value?.url ?? '';
  await siteShields.toggleSiteShieldAllowTrackers(url);
}

async function ensureActiveWebview(): Promise<Webview | null> {
  if (!useNativeWebview || !contentHost.value) return null;
  const tab = getActiveTab();
  if (!tab) return null;

  if (!tab.url || isNewTabUrl(tab.url) || isChromeInternalUrl(tab.url)) {
    return null;
  }

  if (tab.webview) {
    await showTabWebview(tab.webview, contentHost.value);
    return tab.webview;
  }

  const label = tabWebviewLabel(tab.id);
  const wv = await createTabWebview(contentHost.value, label, tab.url);
  await setTabPopupBlocking(label, blockPopups.value);
  tabs.value = tabs.value.map((t) => (t.id === tab.id ? { ...t, webview: wv } : t));
  return wv;
}

async function refreshNavState(): Promise<void> {
  if (!useNativeWebview || !activeTabId.value) return;
  try {
    const state = await getTabNavState(tabWebviewLabel(activeTabId.value));
    canGoBack.value = state.can_go_back;
    canGoForward.value = state.can_go_forward;
    if (state.url && activeTabId.value) {
      tabs.value = tabs.value.map((t) =>
        t.id === activeTabId.value ? { ...t, url: state.url } : t,
      );
    }
  } catch (error) {
    console.error('refreshNavState failed:', error);
  }
}

async function activateTab(tabId: string) {
  if (tabId === activeTabId.value) return;

  const prev = tabs.value.find((t) => t.id === activeTabId.value);
  if (prev?.webview && useNativeWebview) {
    await hideTabWebview(prev.webview);
  }

  activeTabId.value = tabId;
  const tab = tabs.value.find((t) => t.id === tabId);
  if (tab) {
    statusMessage.value = `Loaded: ${tab.title}`;
    isBookmarked.value = tab.url ? checkIsBookmarked(tab.url) : false;
    void siteShields.refreshSiteShieldForUrl(tab.url);

    if (useNativeWebview && contentHost.value) {
      for (const t of tabs.value) {
        if (t.id !== tabId && t.webview) {
          await hideTabWebview(t.webview);
        }
      }
      await ensureActiveWebview();
      await refreshNavState();
    } else {
      applyIframeNavFlags(tabId);
    }
    const page = tab?.url ? parseChromeInternalUrl(tab.url) : null;
    if (page) applyChromeInternalSideEffects(page);
  }
  extensions.syncRegistry();
  if (tab) {
    void tabLifecycle.markTabActive(tabId);
  }
}

const browserSession = useBrowserSession({
  getTabs: () => tabs.value,
  getActiveTabId: () => activeTabId.value,
  getSortedTabs: () => sortedTabsList.value,
  sessionRestore: browserConfig.sessionRestore,
  privateMode: browserConfig.privateMode,
  createTabId,
  activateTab,
  onStatus: (msg) => {
    statusMessage.value = msg;
  },
  newTabPageUrl: NEWTAB_INTERNAL_URL,
});

async function createNewTab() {
  console.log('=== createNewTab called ===');
  console.log('Current tabs count:', tabs.value.length);

  // Always create a new tab instead of reusing existing new tab pages
  // This allows users to have multiple new tab pages open simultaneously
  console.log('Creating new tab');
  const newTab: BrowserTab = {
    id: createTabId(),
    title: 'New Tab',
    url: NEWTAB_INTERNAL_URL,
    webview: null,
    favicon: undefined,
  };
  console.log('New tab created with ID:', newTab.id);
  tabs.value = [...tabs.value, newTab];
  console.log('Tabs after adding:', tabs.value.length);
  await activateTab(newTab.id);
  await assignRandomWallpaperToTab(newTab.id, true);

  // Lazy init: new tab page options when creating a new tab
  void ensureNewTabPageInitialized();

  // New-tab uses the Vue overlay; skip native WebView (huge data: URL freezes the cursor).
  if (useNativeWebview && contentHost.value && !isNewTabUrl(newTab.url)) {
    const label = tabWebviewLabel(newTab.id);
    const wv = await createTabWebview(contentHost.value, label, newTab.url);
    await setTabPopupBlocking(label, blockPopups.value);
    tabs.value = tabs.value.map((t) => (t.id === newTab.id ? { ...t, webview: wv } : t));
    for (const t of tabs.value) {
      if (t.id !== newTab.id && t.webview) await hideTabWebview(t.webview);
    }
  }
  extensions.syncRegistry();
  void browserSession.saveSession();
  void tabLifecycle.registerTab({
    id: newTab.id,
    url: newTab.url,
    title: newTab.title,
  });
  console.log('=== createNewTab complete ===');
}

async function createNewWindow(): Promise<void> {
  console.log('=== createNewWindow called ===');
  console.log('isTauri():', isTauri());
  if (isTauri()) {
    try {
      console.log('Calling invoke with open_new_window_from_dock');
      await invoke('open_new_window_from_dock', { url: null });
      console.log('invoke succeeded');
      statusMessage.value = 'New window opened';
    } catch (error) {
      console.error('createNewWindow failed:', error);
      statusMessage.value = 'Failed to open new window';
    }
  } else {
    console.log('Not running in Tauri environment');
    statusMessage.value = 'New window only available in desktop app';
  }
}

async function createNewIncognitoWindow(): Promise<void> {
  if (isTauri()) {
    try {
      await invoke('open_new_window_from_dock', { url: null });
      statusMessage.value = 'Incognito window opened (note: full incognito mode not yet implemented)';
    } catch (error) {
      console.error('createNewIncognitoWindow failed:', error);
      statusMessage.value = 'Failed to open incognito window';
    }
  } else {
    statusMessage.value = 'Incognito window only available in desktop app';
  }
}

function castPage(): void {
  statusMessage.value = 'Cast feature requires WebRTC support (not yet implemented)';
}

function showMoreTools(): void {
  statusMessage.value = 'More tools submenu not yet implemented';
}

function showHelp(): void {
  window.open('https://github.com/exodus-browser/exodus', '_blank');
}

function openProfile(): void {
  // For now, open settings as profile management
  void navigateToAddress('chrome://settings');
}

function exitBrowser(): void {
  if (isTauri()) {
    invoke('close_window').catch(console.error);
  } else {
    statusMessage.value = 'Exit only available in desktop app';
  }
}

function cancelQuit(): void {
  showQuitConfirmDialog.value = false;
  shellLog.info('User cancelled quit');
}

async function confirmQuit(): Promise<void> {
  showQuitConfirmDialog.value = false;
  shellLog.info('User confirmed quit');
  if (isTauri()) {
    await invoke('quit_app');
  }
}

async function closeTab(tabId: string, force = false) {
  if (tabs.value.length <= 1 && !force) return;
  const tab = tabs.value.find((t) => t.id === tabId);
  if (tab?.pinned && !force) return;

  if (tab) {
    closedTabsStack.recordClosedTab({
      title: tab.title,
      url: tab.url,
      pinned: tab.pinned,
    });
  }

  if (tab?.webview && useNativeWebview) {
    await closeTabWebview(tabWebviewLabel(tabId));
  }
  void tabLifecycle.unregisterTab(tabId);

  const index = tabs.value.findIndex((t) => t.id === tabId);
  const wasActive = activeTabId.value === tabId;
  tabs.value = tabs.value.filter((t) => t.id !== tabId);

  if (wasActive && tabs.value.length > 0) {
    await activateTab(tabs.value[Math.max(0, index - 1)].id);
  } else if (tabs.value.length === 0) {
    // Create a new tab when all tabs are closed
    await createNewTab();
  }
  extensions.syncRegistry();
  void browserSession.saveSession();
}

async function commitNavigation(targetUrl: string): Promise<void> {
  if (isChromeInternalUrl(targetUrl)) {
    await commitChromeInternalNavigation(targetUrl);
    return;
  }

  const tab = getActiveTab();
  if (!tab) return;

  if (route.path.startsWith('/chrome/')) {
    await router.replace('/');
  }

  tabs.value = tabs.value.map((t) =>
    t.id === tab.id
      ? {
          ...t,
          url: targetUrl,
          title: isNewTabUrl(targetUrl) ? 'New Tab' : targetUrl,
          favicon: faviconUrlFor(targetUrl) ?? undefined,
          settingsReturnUrl: undefined,
        }
      : t,
  );
  statusMessage.value = isNewTabUrl(targetUrl) ? 'New Tab' : `Navigating to: ${targetUrl}`;
  if (!isNewTabUrl(targetUrl)) {
    addToHistory(targetUrl, targetUrl);
    ragService.recordVisit(targetUrl, targetUrl).catch((e) => {
      console.warn('Failed to record visit in RAG service:', e);
    });
  }

  if (useNativeWebview && contentHost.value) {
    if (isNewTabUrl(targetUrl)) {
      await hideWebviewForOverlayPage();
    } else {
      const wv = await ensureActiveWebview();
      if (wv) {
        await navigateTab(tabWebviewLabel(tab.id), targetUrl);
        await refreshNavState();
        void passwordSave.runPasswordAutofillHooks(targetUrl);
      }
    }
  } else {
    recordTabNavigation(iframeNavStacks, tab.id, targetUrl);
    applyIframeNavFlags(tab.id);
    if (!isNewTabUrl(targetUrl) && webviewFrame.value && webviewFrame.value.src !== targetUrl) {
      webviewFrame.value.src = targetUrl;
    }
  }
  extensions.syncRegistry();
  void browserSession.saveSession();
  if (!isNewTabUrl(targetUrl)) {
    void siteShields.refreshSiteShieldForUrl(targetUrl);
    void ensurePrivacyStatsRefreshed();
  }
}

async function navigateToAddress(url: string) {
  // Clean URL: remove any surrounding quotes and embedded quotes
  let cleanedUrl = url.trim().replace(/^["']|["']$/g, '');
  
  // Handle malformed URLs like: https://"https://example.com"
  // This pattern has double protocol with embedded quotes
  if (cleanedUrl.includes('https://"https://')) {
    cleanedUrl = cleanedUrl.replace('https://"https://', 'https://');
  } else if (cleanedUrl.includes('http://"http://')) {
    cleanedUrl = cleanedUrl.replace('http://"http://', 'http://');
  } else if (cleanedUrl.startsWith('https://"')) {
    cleanedUrl = cleanedUrl.replace('https://"', 'https://');
  } else if (cleanedUrl.startsWith('http://"')) {
    cleanedUrl = cleanedUrl.replace('http://"', 'http://');
  }
  
  // Remove any remaining quotes
  cleanedUrl = cleanedUrl.replace(/"/g, '');
  
  const resolved = resolveOmniboxInput(cleanedUrl, browserConfig.searchEngineUrl.value);
  if (!resolved) return;
  const rawUrl =
    resolved.kind === 'navigate'
      ? resolved.url
      : browserConfig.searchEngineUrl.value.replace('{query}', encodeURIComponent(resolved.query));
  const finalUrl = applyHttpsOnly(rawUrl, browserConfig.httpsOnly.value);
  if (isChromeInternalUrl(finalUrl)) {
    await commitChromeInternalNavigation(finalUrl);
    return;
  }
  if (!(await safeBrowsing.ensureNavigationAllowed(finalUrl))) {
    return;
  }
  await commitNavigation(finalUrl);
}

if (typeof window !== 'undefined') {
  (window as Window & { __exodusNavigate?: (target: string) => Promise<void> }).__exodusNavigate =
    navigateToAddress;
}

async function goBack() {
  if (!activeTabId.value) return;
  if (useNativeWebview) {
    await goBackTab(tabWebviewLabel(activeTabId.value));
    await refreshNavState();
    statusMessage.value = 'Going back...';
    return;
  }
  const tabId = activeTabId.value;
  const track = iframeNavStacks.get(tabId);
  if (track && track.index > 0) {
    track.index -= 1;
    const url = track.stack[track.index];
    tabs.value = tabs.value.map((t) => (t.id === tabId ? { ...t, url } : t));
    applyIframeNavFlags(tabId);
    if (webviewFrame.value) webviewFrame.value.src = url;
    statusMessage.value = 'Going back...';
    return;
  }
  try {
    webviewFrame.value?.contentWindow?.history.back();
    applyIframeNavFlags(tabId);
    statusMessage.value = 'Going back...';
  } catch {
    /* cross-origin */
  }
}

async function goForward() {
  if (!activeTabId.value) return;
  if (useNativeWebview) {
    await goForwardTab(tabWebviewLabel(activeTabId.value));
    await refreshNavState();
    statusMessage.value = 'Going forward...';
    return;
  }
  const tabId = activeTabId.value;
  const track = iframeNavStacks.get(tabId);
  if (track && track.index + 1 < track.stack.length) {
    track.index += 1;
    const url = track.stack[track.index];
    tabs.value = tabs.value.map((t) => (t.id === tabId ? { ...t, url } : t));
    applyIframeNavFlags(tabId);
    if (webviewFrame.value) webviewFrame.value.src = url;
    statusMessage.value = 'Going forward...';
    return;
  }
  try {
    webviewFrame.value?.contentWindow?.history.forward();
    applyIframeNavFlags(tabId);
    statusMessage.value = 'Going forward...';
  } catch {
    /* cross-origin */
  }
}

async function reload() {
  if (!activeTabId.value) return;
  if (useNativeWebview) {
    await reloadTab(tabWebviewLabel(activeTabId.value));
    statusMessage.value = 'Reloading page...';
    return;
  }
  const tab = getActiveTab();
  if (webviewFrame.value && tab?.url && !isNewTabUrl(tab.url)) {
    webviewFrame.value.src = applyHttpsOnly(tab.url, httpsOnly.value);
    statusMessage.value = 'Reloading page...';
  }
}

async function toggleBookmark(): Promise<void> {
  if (!activeTab.value?.url) return;

  const url = activeTab.value.url;
  const title = activeTab.value.title || url;

  if (checkIsBookmarked(url)) {
    const bookmark = getBookmarks().find((b) => b.url === url);
    if (bookmark) {
      if (isTauri()) {
        await persistBookmarkRemoveFromBackend(bookmark.id);
      } else {
        removeBookmark(bookmark.id);
      }
      isBookmarked.value = false;
      statusMessage.value = 'Bookmark removed';
    }
  } else {
    if (isTauri()) {
      await persistBookmarkAddToBackend(url, title, '');
    } else {
      addBookmark(title, url, undefined, activeTab.value.favicon ?? undefined);
    }
    isBookmarked.value = true;
    statusMessage.value = 'Bookmark added';
  }
  await refreshBookmarkBar();
}

function closeFindBar() {
  showFindBar.value = false;
  findQuery.value = '';
  findResults.value = 0;
  currentFindIndex.value = 0;
}

function navigateToBookmark(url: string) {
  if (!url) return;
  navigateToAddress(url);
}

function handleFindInPage() {
  // Find in page logic handled by findInPage composable
  findInPage.findInPage('next');
}

async function changeZoom(delta: number, reset = false): Promise<void> {
  if (!activeTabId.value || !useNativeWebview) return;
  zoomLevel.value = reset ? 100 : Math.min(500, Math.max(25, zoomLevel.value + delta));
  try {
    await setTabZoom(tabWebviewLabel(activeTabId.value), zoomLevel.value / 100);
    statusMessage.value = reset ? 'Zoom reset' : `Zoom ${zoomLevel.value}%`;
  } catch (error) {
    console.error('setTabZoom failed:', error);
  }
}

async function openDevTools() {
  if (!activeTabId.value || !useNativeWebview || !canInvokeTauri()) {
    statusMessage.value = 'Developer tools are not available';
    return;
  }
  try {
    const opened = await toggleTabDevTools(tabWebviewLabel(activeTabId.value));
    statusMessage.value = opened ? 'Developer tools toggled' : 'Developer tools unavailable';
  } catch (e) {
    console.error('Failed to open dev tools:', e);
    statusMessage.value = 'Failed to open developer tools';
  }
}

async function restoreClosedTab() {
  const snap = closedTabsStack.popClosedTab();
  if (!snap) {
    statusMessage.value = 'No recently closed tabs';
    return;
  }
  const newId = createTabId();
  const restored: BrowserTab = {
    id: newId,
    title: snap.title,
    url: snap.url,
    webview: null,
    pinned: snap.pinned,
    favicon: faviconUrlFor(snap.url) ?? undefined,
  };
  tabs.value = [...tabs.value, restored];
  await activateTab(newId);
  if (useNativeWebview && contentHost.value && !isNewTabUrl(snap.url)) {
    const wv = await createTabWebview(contentHost.value, tabWebviewLabel(newId), snap.url);
    tabs.value = tabs.value.map((t) => (t.id === newId ? { ...t, webview: wv } : t));
  }
  isBookmarked.value = snap.url ? checkIsBookmarked(snap.url) : false;
  if (isNewTabUrl(snap.url)) {
    await assignRandomWallpaperToTab(newId, true);
  }
  statusMessage.value = 'Restored closed tab';
  extensions.syncRegistry();
  void browserSession.saveSession();
  void tabLifecycle.registerTab({
    id: newId,
    url: snap.url,
    title: snap.title,
    pinned: snap.pinned,
  });
}


let stopFrameGapMonitor: (() => void) | undefined;
let unwatchWebviewLayout: (() => void) | undefined;

function bindWebviewLayoutWatch(): void {
  unwatchWebviewLayout?.();
  if (!contentHost.value) return;
  unwatchWebviewLayout = watchWebviewLayout(contentHost.value, () => getActiveTab()?.webview ?? null);
}

watchStoppers.push(watch(
  [contentHost, () => sidebar.sidebarOpen.value, () => sidebarPrefs.prefs.value.position],
  () => {
    void nextTick(() => bindWebviewLayoutWatch());
  },
));

watchStoppers.push(watch(
  () => [route.path, activeTabId.value] as const,
  ([path, tabId]) => {
    if (!path.startsWith('/chrome/') || !tabId) return;
    const segment = path.slice('/chrome/'.length).split('/')[0];
    if (!segment) return;
    const routeUrl = chromeInternalUrlFromRouteParam(segment);
    const tab = getActiveTab();
    if (!tab) return;
    // Hash route is /chrome/settings only — keep chrome://settings/{section} from omnibox/deep links.
    if (
      segment === 'settings' &&
      isChromeSettingsUrl(tab.url) &&
      tab.url.length > routeUrl.length
    ) {
      return;
    }
    if (tab.url === routeUrl) return;
    void commitChromeInternalNavigation(routeUrl);
  },
  { immediate: true },
));

watchStoppers.push(watch(activeTabId, (tabId) => {
  const tab = tabs.value.find((entry) => entry.id === tabId);
  if (tab && isNewTabUrl(tab.url) && !tab.ntpWallpaperId) {
    void assignRandomWallpaperToTab(tabId, false);
  }
}));

onMounted(() => {
  shellLog.info('BrowserPage onMounted', {
    activeTabId: activeTabId.value,
    tabCount: tabs.value.length,
    url: activeTab.value?.url,
    isNewTab: activeTab.value ? isNewTabUrl(activeTab.value.url) : false,
  });

  try {
    if (isTauri()) {
      void seedPresetBookmarksIfEmpty();
    }
    void refreshBookmarkBar();
    refreshNtpTopSites();
    refreshNtpQuickLinks();
  } catch (e) {
    shellLog.error('bookmarks init failed', e);
  }

  try {
    showBookmarkBar.value = browserConfig.showBookmarkBar.value;
    httpsOnly.value = browserConfig.httpsOnly.value;
    privateMode.value = browserConfig.privateMode.value;
    blockPopups.value = browserConfig.blockPopups.value;
  } catch (e) {
    shellLog.error('browser config init failed', e);
  }

  // Track online/offline status
  window.addEventListener('online', handleOnline);
  window.addEventListener('offline', handleOffline);

  unbeforeunload = () => {
    void browserSession.saveSession();
  };
  window.addEventListener('beforeunload', unbeforeunload);

  try {
    unbindShortcuts = mountBrowserShortcuts({
      focusOmnibox: focusOmniboxInput,
      reload,
      goBack,
      goForward,
      newTab: () => void createNewTab(),
      restoreClosedTab: () => void restoreClosedTab(),
      closeActiveTab: () => {
        if (activeTabId.value) void closeTab(activeTabId.value);
      },
      zoomIn: () => void changeZoom(10),
      zoomOut: () => void changeZoom(-10),
      resetZoom: () => void changeZoom(0, true),
      toggleBookmark,
      toggleBookmarkBar: () => {
        showBookmarkBar.value = !showBookmarkBar.value;
        writeShowBookmarkBar(showBookmarkBar.value);
      },
      openHistory: () => openSidebarPanel('memory'),
      openBookmarksPanel: () => openSidebarPanel('bookmarks'),
      toggleWebChat: () => toggleWebChat(),
      toggleSidebar: toggleSidebarSmart,
      toggleFindBar: () => {
        showFindBar.value = !showFindBar.value;
      },
      print: () => void printPage(),
      switchToTabIndex: (idx) => {
        const ordered = sortedTabsList.value;
        if (ordered[idx]) void activateTab(ordered[idx].id);
      },
      tabIdsInOrder: () => sortedTabsList.value.map((t) => t.id),
      toggleDevTools: () => void openDevTools(),
      togglePrivateMode: () => {
        browserConfig.privateMode.value = !browserConfig.privateMode.value;
        privateMode.value = browserConfig.privateMode.value;
      },
      onEscape: () => {
        if (webChatFullViewOpen.value) {
          webChatFullViewOpen.value = false;
          void restoreWebviewAfterWebChat();
          return;
        }
        downloads.closeDownloadsPanel();
        showMenu.value = false;
        showContextMenu.value = false;
        if (showFindBar.value) closeFindBar();
      },
    });
  } catch (e) {
    shellLog.error('shortcuts mount failed', e);
  }

  onOpenWebChatUi = () => toggleWebChat(true);
  window.addEventListener(OPEN_WEBCHAT_EVENT, onOpenWebChatUi);
  if (isTauri()) {
    void listen('exodus-open-webchat', () => toggleWebChat(true)).then((unlisten) => {
      unlistenWebChat = unlisten;
    }).catch((e) => {
      shellLog.error('exodus-open-webchat listener failed', e);
    });
    void listen('exodus-open-history', () => openSidebarPanel('memory')).then((unlisten) => {
      unlistenHistory = unlisten;
    }).catch((e) => {
      shellLog.error('exodus-open-history listener failed', e);
    });

    // Listen for menu events
    void listen('exodus-new-tab', () => createNewTab()).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-new-tab listener failed', e);
    });
    void listen('exodus-new-incognito-window', () => createNewIncognitoWindow()).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-new-incognito-window listener failed', e);
    });
    void listen('exodus-open-settings', () => void navigateToAddress('chrome://settings')).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-open-settings listener failed', e);
    });
    void listen('exodus-open-bookmarks', () => openSidebarPanel('bookmarks')).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-open-bookmarks listener failed', e);
    });
    void listen('exodus-bookmark-this-page', () => toggleBookmark()).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-bookmark-this-page listener failed', e);
    });
    void listen('exodus-open-downloads', () => openSidebarPanel('downloads')).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-open-downloads listener failed', e);
    });
    void listen('exodus-open-profile-settings', () => void navigateToAddress('chrome://settings')).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-open-profile-settings listener failed', e);
    });
    void listen('exodus-print', () => printPage()).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-print listener failed', e);
    });
    void listen('exodus-find', () => { showFindBar.value = true; }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-find listener failed', e);
    });
    void listen('exodus-zoom-in', () => void changeZoom(0.1)).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-zoom-in listener failed', e);
    });
    void listen('exodus-zoom-out', () => void changeZoom(-0.1)).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-zoom-out listener failed', e);
    });
    void listen('exodus-zoom-reset', () => void changeZoom(0, true)).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-zoom-reset listener failed', e);
    });
    void listen('exodus-quit', () => exitBrowser()).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-quit listener failed', e);
    });
    shellLog.info('Registering exodus-quit-request listener');
    void listen('exodus-quit-request', async () => {
      shellLog.info('exodus-quit-request event received');
      console.log('[Shell] exodus-quit-request event received');
      showQuitConfirmDialog.value = true;
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-quit-request listener failed', e);
      console.error('[Shell] exodus-quit-request listener failed', e);
    });
    shellLog.info('exodus-quit-request listener registered successfully');

    // Cleanup resources before quit
    void listen('exodus-cleanup-resources', async () => {
      shellLog.info('Cleaning up resources before quit...');
      
      // Close all webviews
      try {
        for (const tab of tabs.value) {
          if (tab.webview && useNativeWebview) {
            await closeTabWebview(tabWebviewLabel(tab.id));
          }
        }
        shellLog.info('All webviews closed');
      } catch (e) {
        shellLog.error('Failed to close webviews', e);
      }
      
      // Save session
      try {
        await browserSession.saveSession();
        shellLog.info('Session saved');
      } catch (e) {
        shellLog.error('Failed to save session', e);
      }
      
      // Clear caches
      try {
        invalidateWallpaperCache();
        shellLog.info('Caches cleared');
      } catch (e) {
        shellLog.error('Failed to clear caches', e);
      }
      
      shellLog.info('Resource cleanup complete');
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-cleanup-resources listener failed', e);
    });
    void listen('exodus-navigate', (url: string) => void navigateToAddress(url)).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-navigate listener failed', e);
    });

    // File menu events
    void listen('exodus-reopen-closed-tab', () => restoreClosedTab()).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-reopen-closed-tab listener failed', e);
    });
    void listen('exodus-open-file', () => {
      statusMessage.value = 'Open file dialog not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-open-file listener failed', e);
    });
    void listen('exodus-focus-address-bar', () => {
      const addressBar = document.querySelector('.url-input, #exodus-omnibox-input') as HTMLInputElement;
      if (addressBar) addressBar.focus();
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-focus-address-bar listener failed', e);
    });
    void listen('exodus-close-tab', () => {
      if (activeTabId.value) void closeTab(activeTabId.value);
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-close-tab listener failed', e);
    });
    void listen('exodus-close-window', () => {
      statusMessage.value = 'Close window not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-close-window listener failed', e);
    });
    void listen('exodus-save-page', () => {
      statusMessage.value = 'Save page not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-save-page listener failed', e);
    });

    // Edit menu events
    void listen('exodus-find-next', () => {
      statusMessage.value = 'Find next not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-find-next listener failed', e);
    });
    void listen('exodus-find-previous', () => {
      statusMessage.value = 'Find previous not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-find-previous listener failed', e);
    });

    // View menu events
    void listen('exodus-reload', () => {
      if (activeTab.value) void reloadTab(activeTabId.value);
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-reload listener failed', e);
    });
    void listen('exodus-force-reload', () => {
      if (activeTab.value) void reloadTab(activeTabId.value, true);
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-force-reload listener failed', e);
    });
    void listen('exodus-developer-tools', () => {
      statusMessage.value = 'Developer tools not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-developer-tools listener failed', e);
    });

    // Window menu events
    void listen('exodus-select-next-tab', () => {
      const currentIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
      if (currentIndex >= 0 && currentIndex < tabs.value.length - 1) {
        void activateTab(tabs.value[currentIndex + 1].id);
      }
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-select-next-tab listener failed', e);
    });
    void listen('exodus-select-previous-tab', () => {
      const currentIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
      if (currentIndex > 0) {
        void activateTab(tabs.value[currentIndex - 1].id);
      }
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-select-previous-tab listener failed', e);
    });
    void listen('exodus-move-tab-to-new-window', () => {
      statusMessage.value = 'Move tab to new window not yet implemented';
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-move-tab-to-new-window listener failed', e);
    });

    // Help menu events
    void listen('exodus-help-docs', () => {
      void navigateToAddress('https://github.com/arksong/Exodus');
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-help-docs listener failed', e);
    });
    void listen('exodus-report-issue', () => {
      void navigateToAddress('https://github.com/arksong/Exodus/issues');
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-report-issue listener failed', e);
    });
    void listen('exodus-search', () => {
      const addressBar = document.querySelector('.url-input, #exodus-omnibox-input') as HTMLInputElement;
      if (addressBar) {
        addressBar.focus();
        addressBar.select();
      }
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-search listener failed', e);
    });
    void listen('exodus-about', () => {
      void navigateToAddress('chrome://about');
    }).then((unlisten) => {
      eventListeners.push(unlisten);
    }).catch((e) => {
      shellLog.error('exodus-about listener failed', e);
    });

    // Update menus when data changes
    watchStoppers.push(watch(() => recentHistoryItems.value, (items) => {
      if (isTauri()) {
        void invoke('update_history_menu', { items });
      }
    }, { deep: true }));

    watchStoppers.push(watch(() => bookmarkFolderItems.value, (items) => {
      if (isTauri()) {
        const bookmarkItems = items.map(folder => ({
          title: folder.name,
          url: `chrome://bookmarks?folder=${encodeURIComponent(folder.name)}`
        }));
        void invoke('update_bookmarks_menu', { items: bookmarkItems });
      }
    }, { deep: true }));

    void extensions.setup().then(async () => {
      try {
        await rescanExtensions();
      } catch (error) {
        shellLog.error('extension_rescan failed', error);
      }
      bumpExtensionsToolbarRefresh();
      window.dispatchEvent(new CustomEvent('exodus-extensions-changed'));
    }).catch((e) => {
      shellLog.error('extensions.setup failed', e);
    });
  }

  void bindLifecycleRecovery().then((unbind) => {
    unbindLifecycle = unbind;
  }).catch((e) => {
    shellLog.error('lifecycle recovery bind failed', e);
  });

  try {
    stopFrameGapMonitor = startFrameGapMonitor();
  } catch (e) {
    shellLog.warn('frame gap monitor failed', e);
  }

  void ensureNewTabPageInitialized();

  const path = route.path;
  if (path.startsWith('/chrome/')) {
    const segment = path.slice('/chrome/'.length).split('/')[0];
    if (segment) {
      void commitChromeInternalNavigation(chromeInternalUrlFromRouteParam(segment));
    }
  }

  shellLog.info('BrowserPage onMounted complete');
});

onUnmounted(async () => {
  console.log('[BrowserPage] onUnmounted - starting cleanup');
  
  // Clean up all event listeners
  console.log(`[BrowserPage] Cleaning up ${eventListeners.length} event listeners`);
  for (const unlisten of eventListeners) {
    try {
      unlisten();
    } catch (e) {
      console.error('[BrowserPage] Failed to unlisten event:', e);
    }
  }
  eventListeners.length = 0; // Clear the array
  
  // Clean up all watch stoppers
  console.log(`[BrowserPage] Stopping ${watchStoppers.length} watch listeners`);
  for (const stop of watchStoppers) {
    try {
      stop();
    } catch (e) {
      console.error('[BrowserPage] Failed to stop watch:', e);
    }
  }
  watchStoppers.length = 0;
  
  // Clean up specific listeners
  stopFrameGapMonitor?.();
  unwatchWebviewLayout?.();
  unbindShortcuts?.();
  unbindLifecycle?.();
  
  if (onOpenWebChatUi) {
    window.removeEventListener(OPEN_WEBCHAT_EVENT, onOpenWebChatUi);
    onOpenWebChatUi = undefined;
  }
  
  unlistenWebChat?.();
  unlistenWebChat = undefined;
  unlistenHistory?.();
  unlistenHistory = undefined;
  
  if (unbeforeunload) {
    window.removeEventListener('beforeunload', unbeforeunload);
  }
  
  window.removeEventListener('online', handleOnline);
  window.removeEventListener('offline', handleOffline);
  
  // Clean up webviews
  if (useNativeWebview) {
    console.log(`[BrowserPage] Closing ${tabs.value.length} webviews`);
    for (const tab of tabs.value) {
      if (tab.webview) {
        try {
          await closeTabWebview(tabWebviewLabel(tab.id));
        } catch (e) {
          console.error(`[BrowserPage] Failed to close webview ${tab.id}:`, e);
        }
      }
    }
  }
  
  // Teardown services
  sitePermissions.teardownSitePermissionListener();
  extensions.teardown();
  
  console.log('[BrowserPage] onUnmounted - cleanup complete');
});

defineExpose({
  tabs,
  activeTab,
  activeTabId,
  showFindBar,
  showWebChatView,
  webChatFullViewOpen,
  toggleWebChat,
  createNewTab,
  activateTab,
  restoreClosedTab,
  useNativeWebview,
});
</script>

<style scoped>
.quit-confirm-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.quit-confirm-dialog {
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 24px;
  min-width: 320px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.quit-confirm-dialog h3 {
  margin: 0 0 12px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.quit-confirm-dialog p {
  margin: 0 0 20px 0;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.quit-confirm-buttons {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.quit-confirm-buttons button {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background 0.2s;
}

.btn-cancel {
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
}

.btn-cancel:hover {
  background: var(--color-bg-tertiary);
}

.btn-confirm {
  background: #ef4444;
  color: white;
}

.btn-confirm:hover {
  background: #dc2626;
}

.browser-page {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--color-bg-primary);
  cursor: default;
  overflow: hidden;
}

.browser-shell {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.browser-main {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.browser-body-row {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

.browser-body-row.sidebar-position-left {
  flex-direction: row-reverse;
}

.browser-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--color-bg-primary);
  position: relative;
  display: flex;
  flex-direction: column;
}

.browser-content--ntp {
  background: #0a0a0f;
}

.webchat-content-host {
  background: #f5f5f5;
}

.webchat-main-view {
  flex: 1;
  min-width: 0;
  min-height: 0;
  width: 100%;
}

.browser-body-row.webchat-active .browser-content {
  flex: 1;
}

.webview-container {
  width: 100%;
  height: 100%;
  position: relative;
  background: #fff;
  flex: 1;
  min-height: 0;
}

.browser-webview,
.native-webview-host {
  width: 100%;
  height: 100%;
  border: none;
  display: block;
  background: #111;
}

.webview-screenshot-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
  background: #fff;
  pointer-events: none;
}

.screenshot-canvas {
  /* Size will be set by JavaScript based on DPR */
  /* Do not use percentage-based sizing to avoid distortion */
  display: block;
  position: absolute;
  top: 0;
  left: 0;
  /* Use default browser rendering for best quality */
  image-rendering: auto;
}

.webview-white-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
  background: #ffffff;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.webview-white-placeholder .placeholder-text {
  color: #666;
  font-size: 18px;
  font-weight: 500;
}

.loading-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  font-size: 1rem;
}
</style>
