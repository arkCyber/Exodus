<script lang="ts">
  /**
   * Exodus Browser — toolbar: navigation, omnibox, bookmarks, and chrome menu.
   */
  import { isSecureUrl } from '$lib/favicon';
  import { isNewTabUrl } from '$lib/newTabPage';
  import { OMNIBOX_INPUT_ID } from '$lib/browserShortcuts';
  import type { SearchHit, SidebarPanel } from '$lib/browserTypes';
  import { omniboxSuggestionTypeLabel } from '$lib/browserIntegrations';
  import type { OmniboxSuggestion } from '$lib/browserIntegrations';

  type Props = {
    canGoBack: boolean;
    canGoForward: boolean;
    currentUrl: string;
    urlInput: string;
    isBookmarked: boolean;
    showSearchResults: boolean;
    showOmniboxSuggestions?: boolean;
    omniboxSuggestions?: OmniboxSuggestion[];
    onUrlInput?: () => void;
    onSelectOmniboxSuggestion?: (url: string) => void;
    isSearching: boolean;
    searchResults: SearchHit[];
    showMenu: boolean;
    aiSidebarOpen: boolean;
    sidebarPanel: SidebarPanel;
    downloadsBadge: number;
    closedTabsCount: number;
    onGoBack: () => void;
    onGoForward: () => void;
    onReload: () => void;
    onHome: () => void;
    onNavigate: () => void;
    onNavigateToResult: (url: string) => void;
    onToggleBookmark: () => void;
    onOpenPanel: (panel: SidebarPanel) => void;
    onOpenDownloads: () => void;
    onToggleMenu: () => void;
    onCloseMenu: () => void;
    onOpenBookmarksPanel: () => void;
    onRestoreClosedTab: () => void;
    onOpenHistoryPanel: () => void;
    onDownloadCurrentPage: () => void;
    onIndexPage: () => void;
    onPrint: () => void;
    onTranslatePage?: () => void;
    onToggleReadingMode?: () => void;
    onOpenSettings: () => void;
    onAnnounceSearchResult?: (url: string, title: string) => void | Promise<void>;
    /** Omnibox badge e.g. "P2P · 2" when peers seed this URL. */
    cdnStatusLabel?: string | null;
    onCdnBadgeClick?: () => void;
    /** Brave-style shields: trackers blocked this session. */
    shieldsCount?: number;
    shieldsEnabled?: boolean;
    siteAllowTrackers?: boolean;
    onOpenShields?: () => void;
    onToggleSiteShields?: () => void;
  };

  let {
    canGoBack,
    canGoForward,
    currentUrl,
    urlInput = $bindable(''),
    isBookmarked,
    showSearchResults,
    showOmniboxSuggestions = false,
    omniboxSuggestions = [],
    onUrlInput,
    onSelectOmniboxSuggestion,
    isSearching,
    searchResults,
    showMenu,
    aiSidebarOpen,
    sidebarPanel,
    downloadsBadge,
    closedTabsCount,
    onGoBack,
    onGoForward,
    onReload,
    onHome,
    onNavigate,
    onNavigateToResult,
    onToggleBookmark,
    onOpenPanel,
    onOpenDownloads,
    onToggleMenu,
    onCloseMenu,
    onOpenBookmarksPanel,
    onRestoreClosedTab,
    onOpenHistoryPanel,
    onDownloadCurrentPage,
    onIndexPage,
    onPrint,
    onTranslatePage,
    onToggleReadingMode,
    onOpenSettings,
    onAnnounceSearchResult,
    cdnStatusLabel = null,
    onCdnBadgeClick,
    shieldsCount = 0,
    shieldsEnabled = true,
    siteAllowTrackers = false,
    onOpenShields,
    onToggleSiteShields,
  }: Props = $props();
</script>

<div class="address-bar exodus-address-bar">
  <div class="nav-controls">
    <button type="button" class="nav-icon-btn" disabled={!canGoBack} onclick={onGoBack} title="Back" aria-label="Back">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
    </button>
    <button type="button" class="nav-icon-btn" disabled={!canGoForward} onclick={onGoForward} title="Forward" aria-label="Forward">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
    </button>
    <button type="button" class="nav-icon-btn" onclick={onReload} title="Reload (⌘R)" aria-label="Reload">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4v6h6M20 20v-6h-6"/><path d="M20 9A8 8 0 0 0 6.7 6.7L4 10M4 15a8 8 0 0 0 13.3 2.3L20 14"/></svg>
    </button>
    <button type="button" class="nav-icon-btn" onclick={onHome} title="Home" aria-label="Home">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 11l9-8 9 8"/><path d="M5 10v10h14V10"/></svg>
    </button>
  </div>
  <div class="url-input-wrapper omnibox">
    {#if !isNewTabUrl(currentUrl)}
      <span
        class="site-indicator"
        class:secure={isSecureUrl(currentUrl)}
        title={isSecureUrl(currentUrl) ? 'Connection is secure' : 'Not secure'}
      >
        {#if isSecureUrl(currentUrl)}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 1a5 5 0 0 0-5 5v3H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V11a2 2 0 0 0-2-2h-1V6a5 5 0 0 0-5-5zm0 2a3 3 0 0 1 3 3v3H9V6a3 3 0 0 1 3-3z"/></svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg>
        {/if}
      </span>
    {/if}
    <input
      id={OMNIBOX_INPUT_ID}
      type="text"
      bind:value={urlInput}
      oninput={() => onUrlInput?.()}
      onfocus={() => onUrlInput?.()}
      onkeydown={(e) => e.key === 'Enter' && onNavigate()}
      placeholder="Search or type a URL (/ask for memory)"
      class="url-input"
      class:has-cdn-badge={!!cdnStatusLabel}
    />
    {#if cdnStatusLabel}
      <button
        type="button"
        class="cdn-omnibox-badge"
        title="P2P CDN status for this page"
        onclick={() => onCdnBadgeClick?.()}
      >
        {cdnStatusLabel}
      </button>
    {/if}
    {#if showOmniboxSuggestions && omniboxSuggestions.length > 0}
      <div class="search-results-dropdown omnibox-suggestions">
        {#each omniboxSuggestions as row (row.id)}
          <div
            class="search-result-item"
            onclick={() => onSelectOmniboxSuggestion?.(row.url)}
            onkeydown={(e) => e.key === 'Enter' && onSelectOmniboxSuggestion?.(row.url)}
            role="link"
            tabindex="0"
          >
            <div class="result-title">{row.text}</div>
            <div class="result-url">{row.url}</div>
            <div class="result-meta">
              <span class="suggestion-type">{omniboxSuggestionTypeLabel(row.suggestion_type)}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else if showSearchResults}
      <div class="search-results-dropdown">
        {#if isSearching}
          <div class="search-loading">
            <div class="spinner-small"></div>
            <span>Searching local memory...</span>
          </div>
        {:else if searchResults.length > 0}
          {#each searchResults as result}
            <div
              class="search-result-item"
              onclick={() => onNavigateToResult(result.page.url)}
              onkeydown={(e) => e.key === 'Enter' && onNavigateToResult(result.page.url)}
              role="link"
              tabindex="0"
            >
              <div class="result-title">{result.page.title}</div>
              <div class="result-url">{result.page.url}</div>
              <div class="result-meta">
                <span class="result-score">{(result.score * 100).toFixed(0)}%</span>
                {#if onAnnounceSearchResult}
                  <button
                    type="button"
                    class="search-p2p-btn"
                    title="Announce to P2P CDN lobby"
                    onclick={(e) => {
                      e.stopPropagation();
                      void onAnnounceSearchResult(result.page.url, result.page.title);
                    }}
                  >
                    P2P
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        {:else}
          <div class="search-empty">No results in local memory</div>
        {/if}
      </div>
    {/if}
  </div>
  <button
    type="button"
    class="toolbar-icon-btn"
    class:bookmarked={isBookmarked}
    onclick={onToggleBookmark}
    title={isBookmarked ? 'Remove bookmark' : 'Add bookmark'}
  >
    <svg width="18" height="18" viewBox="0 0 24 24" fill={isBookmarked ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01L12 2z"/>
    </svg>
  </button>
  {#if !isNewTabUrl(currentUrl)}
    <button
      type="button"
      class="toolbar-icon-btn shields-btn"
      class:shields-off={!shieldsEnabled}
      onclick={(e) => {
        if (e.shiftKey) onToggleSiteShields?.();
        else onOpenShields?.();
      }}
      title={siteAllowTrackers
        ? 'Trackers allowed on this site (Shift+click to block again)'
        : shieldsEnabled
          ? `Shields up · ${shieldsCount} blocked · Shift+click allow on this site`
          : 'Shields off · click Privacy · Shift+click per-site allow'}
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2l8 4v6c0 5-3.5 9.5-8 10-4.5-.5-8-5-8-10V6l8-4z"/>
      </svg>
      {#if shieldsCount > 0}
        <span class="toolbar-badge shields-badge">{shieldsCount > 99 ? '99+' : shieldsCount}</span>
      {/if}
    </button>
  {/if}
  <div class="toolbar-actions">
    <button
      type="button"
      class="toolbar-icon-btn"
      class:active={aiSidebarOpen && sidebarPanel === 'ai'}
      onclick={() => onOpenPanel('ai')}
      title="AI assistant"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3a7 7 0 0 0-4 12.7V21l4-2 4 2v-5.3A7 7 0 0 0 12 3z"/></svg>
    </button>
    <button
      type="button"
      class="toolbar-icon-btn"
      class:active={aiSidebarOpen && sidebarPanel === 'memory'}
      onclick={() => onOpenPanel('memory')}
      title="Memory & history"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>
    </button>
    <button
      type="button"
      class="toolbar-icon-btn"
      class:active={aiSidebarOpen && sidebarPanel === 'bookmarks'}
      onclick={() => onOpenPanel('bookmarks')}
      title="Bookmarks"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15z"/></svg>
    </button>
    <button
      type="button"
      class="toolbar-icon-btn"
      class:active={aiSidebarOpen && sidebarPanel === 'p2p'}
      onclick={() => onOpenPanel('p2p')}
      title="P2P CDN & group chat"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="12" r="2"/><circle cx="18" cy="7" r="2"/><circle cx="18" cy="17" r="2"/><path d="M8 12h5M13 10l3-3M13 14l3 3"/></svg>
    </button>
    <button type="button" class="toolbar-icon-btn" onclick={onOpenDownloads} title="Downloads">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v12"/><path d="M8 11l4 4 4-4"/><path d="M4 20h16"/></svg>
      {#if downloadsBadge > 0}
        <span class="toolbar-badge">{downloadsBadge}</span>
      {/if}
    </button>
  </div>
  <button type="button" class="chrome-menu-btn" title="Menu" onclick={onToggleMenu}>
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
      <circle cx="12" cy="5" r="2"/>
      <circle cx="12" cy="12" r="2"/>
      <circle cx="12" cy="19" r="2"/>
    </svg>
  </button>
  {#if showMenu}
    <button type="button" class="menu-backdrop" aria-label="Close menu" onclick={onCloseMenu}></button>
    <div class="chrome-menu-dropdown">
      <button type="button" class="menu-item" onclick={onToggleBookmark}>
        <span>⭐</span>
        <span>Bookmark this page</span>
      </button>
      <button type="button" class="menu-item" onclick={onOpenBookmarksPanel}>
        <span>📑</span>
        <span>Bookmarks</span>
      </button>
      <button type="button" class="menu-item" onclick={onRestoreClosedTab} disabled={closedTabsCount === 0}>
        <span>↩</span>
        <span>Reopen closed tab (⌘⇧T){closedTabsCount > 0 ? ` (${closedTabsCount})` : ''}</span>
      </button>
      <button type="button" class="menu-item" onclick={onOpenHistoryPanel}>
        <span>🕐</span>
        <span>Memory & history</span>
      </button>
      <button type="button" class="menu-item" onclick={onOpenDownloads}>
        <span>⬇</span>
        <span>Downloads</span>
      </button>
      <button type="button" class="menu-item" onclick={onDownloadCurrentPage}>
        <span>💾</span>
        <span>Save page as download</span>
      </button>
      <button type="button" class="menu-item" onclick={onIndexPage}>
        <span>📇</span>
        <span>Index page to memory</span>
      </button>
      {#if onTranslatePage}
        <button type="button" class="menu-item" onclick={onTranslatePage}>
          <span>🌐</span>
          <span>Translate page</span>
        </button>
      {/if}
      {#if onToggleReadingMode}
        <button type="button" class="menu-item" onclick={onToggleReadingMode}>
          <span>📖</span>
          <span>Toggle reading mode</span>
        </button>
      {/if}
      <button type="button" class="menu-item" onclick={onPrint}>
        <span>🖨</span>
        <span>Print</span>
      </button>
      <div class="menu-divider"></div>
      <button type="button" class="menu-item" onclick={onOpenSettings}>
        <span>⚙</span>
        <span>Settings</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .address-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #404040;
    position: relative;
  }

  .nav-controls {
    display: flex;
    gap: 4px;
  }

  .nav-icon-btn {
    background: transparent;
    border: none;
    color: #e0e0e0;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .nav-icon-btn:hover:not(:disabled) {
    background: #505050;
  }

  .nav-icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .url-input-wrapper.omnibox {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    background: #1a1a1a;
    border: 1px solid #404040;
    border-radius: 20px;
    padding: 0 12px;
  }

  .url-input-wrapper.omnibox .url-input {
    border: none;
    background: transparent;
    padding: 8px 0;
    flex: 1;
    color: #e0e0e0;
    font-size: 14px;
    outline: none;
  }

  .url-input.has-cdn-badge {
    padding-right: 4px;
  }

  .cdn-omnibox-badge {
    flex-shrink: 0;
    font-size: 10px;
    padding: 3px 8px;
    border-radius: 10px;
    background: #1e3a2f;
    border: 1px solid #4ade80;
    color: #bbf7d0;
    cursor: pointer;
    white-space: nowrap;
  }

  .cdn-omnibox-badge:hover {
    background: #14532d;
  }

  .url-input-wrapper.omnibox:focus-within {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.25);
  }

  .site-indicator {
    display: flex;
    color: #888;
    flex-shrink: 0;
  }

  .site-indicator.secure {
    color: #4ade80;
  }

  .search-results-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #2d2d2d;
    border: 1px solid #404040;
    border-radius: 8px;
    margin-top: 4px;
    max-height: 360px;
    overflow-y: auto;
    z-index: 100;
  }

  .search-loading,
  .search-empty {
    padding: 16px;
    color: #888;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .search-result-item {
    padding: 12px 16px;
    border-bottom: 1px solid #404040;
    cursor: pointer;
  }

  .search-result-item:hover {
    background: #3d3d3d;
  }

  .result-title {
    font-weight: 500;
    margin-bottom: 4px;
  }

  .result-url {
    font-size: 12px;
    color: #888;
    word-break: break-all;
  }

  .result-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 4px;
  }

  .search-p2p-btn {
    flex-shrink: 0;
    font-size: 10px;
    padding: 2px 8px;
    background: #1e3a2f;
    border: 1px solid #4ade80;
    color: #bbf7d0;
    border-radius: 4px;
    cursor: pointer;
  }

  .search-p2p-btn:hover {
    background: #14532d;
  }

  .suggestion-type {
    font-size: 11px;
    color: #9cdcfe;
    background: #3a3a3a;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .result-score {
    font-size: 12px;
    color: #6366f1;
  }

  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: 4px;
  }

  .toolbar-icon-btn {
    position: relative;
    background: transparent;
    border: none;
    color: #ccc;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .toolbar-icon-btn:hover {
    background: #404040;
    color: #fff;
  }

  .toolbar-icon-btn.active {
    background: #6366f1;
    color: #fff;
  }

  .toolbar-icon-btn.bookmarked {
    color: #fef08a;
  }

  .shields-btn {
    color: #a5b4fc;
  }

  .shields-btn.shields-off {
    color: #9aa0a6;
    opacity: 0.75;
  }

  .shields-badge {
    background: #22c55e;
  }

  .toolbar-badge {
    position: absolute;
    top: 2px;
    right: 2px;
    min-width: 14px;
    height: 14px;
    padding: 0 4px;
    font-size: 10px;
    line-height: 14px;
    text-align: center;
    background: #6366f1;
    color: #fff;
    border-radius: 8px;
  }

  .chrome-menu-btn {
    background: transparent;
    border: none;
    color: #ccc;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .chrome-menu-btn:hover {
    background: #404040;
  }

  .chrome-menu-dropdown {
    position: absolute;
    right: 16px;
    top: 100%;
    margin-top: 8px;
    background: #292a2d;
    border: 1px solid #5f6368;
    border-radius: 8px;
    padding: 8px 0;
    min-width: 200px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
    z-index: 200;
  }
</style>
