<script lang="ts">
  /**
   * Exodus Browser — right sidebar: AI chat, history, bookmarks, and agent panel.
   */
  import type { AiChatMessage, BookmarkItem, SidebarPanel } from '$lib/browserTypes';
  import type { HistoryGroup } from '$lib/historyGroups';
  import AgentPanel from '$lib/components/AgentPanel.svelte';
  import P2pSidebarPanel from '$lib/components/P2pSidebarPanel.svelte';
  import PocketPanel from '$lib/components/PocketPanel.svelte';
  import PrivacySummaryStrip from '$lib/components/PrivacySummaryStrip.svelte';
  import type { PrivacyStatsSummary } from '$lib/privacyStats';
  import {
    extractAnnounceableUrls,
    isLikelyLargeAssetUrl,
    suggestUrlsForCdnAnnounce,
  } from '$lib/p2p/cdnIntegrations';
  import { isNewTabUrl } from '$lib/newTabPage';

  type Props = {
    open: boolean;
    sidebarPanel: SidebarPanel;
    agentPanelOpen: boolean;
    indexedMemoryGroups: HistoryGroup[];
    indexedCount: number;
    historyGroups: HistoryGroup[];
    historyCount: number;
    bookmarks: BookmarkItem[];
    aiOnline: boolean;
    isLoading: boolean;
    aiStreamMode: 'none' | 'summary' | 'chat';
    aiResponse: string;
    aiChatHistory: AiChatMessage[];
    aiChatInput: string;
    chatStreamBuffer: string;
    agentCommand: string;
    agentLog: string[];
    isAgentExecuting: boolean;
    agentDomSummary: string;
    onClose: () => void;
    onOpenPanel: (panel: SidebarPanel) => void;
    onLoadMemory: () => void;
    onClearIndexed: () => void;
    onRemoveIndexedPage: (id: string) => void;
    onClearHistory: () => void;
    onNavigate: (url: string) => void;
    onLoadBookmarks: () => void;
    onRemoveBookmark: (id: string) => void;
    onUpdateBookmarkFolder: (id: string, folder: string) => void;
    onToggleAgent: () => void;
    onAgentExecute: () => void;
    onAgentCompress: () => void;
    onAgentBack: () => void;
    onAgentPreset: (actionJson: string) => void;
    onAgentCommandChange: (value: string) => void;
    onAgentAskAi: () => void;
    onSendChat: () => void;
    onCancelChat?: () => void;
    onSearchBookmarks?: (query: string) => void;
    onSearchMemory?: (query: string) => void;
    p2pRoomId?: string;
    currentPageUrl?: string;
    currentPageTitle?: string;
    onAnnounceUrlToCdn?: (url: string, title: string) => void | Promise<void>;
    onAnnounceCurrentPageToCdn?: () => void | Promise<void>;
    onP2pStatus?: (message: string) => void;
    /** After AI reply: large-file URLs to announce (one-click banner). */
    aiCdnSuggestUrls?: string[];
    onDismissAiCdnSuggest?: () => void;
    onAnnounceAllAiCdnUrls?: () => void | Promise<void>;
    privacyStats?: PrivacyStatsSummary | null;
    onOpenPrivacySettings?: () => void;
  };

  let {
    open,
    sidebarPanel,
    agentPanelOpen,
    indexedMemoryGroups,
    indexedCount,
    historyGroups,
    historyCount,
    bookmarks,
    aiOnline,
    isLoading,
    aiStreamMode,
    aiResponse,
    aiChatHistory,
    aiChatInput = $bindable(''),
    chatStreamBuffer,
    agentCommand = $bindable(''),
    agentLog,
    isAgentExecuting,
    agentDomSummary,
    onClose,
    onOpenPanel,
    onLoadMemory,
    onClearIndexed,
    onRemoveIndexedPage,
    onClearHistory,
    onNavigate,
    onLoadBookmarks,
    onRemoveBookmark,
    onUpdateBookmarkFolder,
    onToggleAgent,
    onAgentExecute,
    onAgentCompress,
    onAgentBack,
    onAgentPreset,
    onAgentCommandChange,
    onAgentAskAi,
    onSendChat,
    onCancelChat,
    onSearchBookmarks,
    onSearchMemory,
    p2pRoomId = $bindable('lobby'),
    currentPageUrl = '',
    currentPageTitle = '',
    onAnnounceUrlToCdn,
    onAnnounceCurrentPageToCdn,
    onP2pStatus = () => {},
    aiCdnSuggestUrls = [],
    onDismissAiCdnSuggest,
    onAnnounceAllAiCdnUrls,
    privacyStats = null,
    onOpenPrivacySettings,
  }: Props = $props();

  const streamLargeUrls = $derived(suggestUrlsForCdnAnnounce(chatStreamBuffer, 3));

  const canAnnouncePage = $derived(
    !isNewTabUrl(currentPageUrl) &&
      (currentPageUrl.startsWith('http://') || currentPageUrl.startsWith('https://')),
  );

  let bookmarkSearchQuery = $state('');
  let memorySearchQuery = $state('');
  let memorySearchTimer: ReturnType<typeof setTimeout> | undefined;
  let bookmarkSearchTimer: ReturnType<typeof setTimeout> | undefined;
  let focusedBookmarkIndex = $state(-1);
  let focusedHistoryIndex = $state(-1);

  /** Debounced memory panel search (indexed + visits). */
  function onMemorySearchInput(value: string) {
    memorySearchQuery = value;
    if (memorySearchTimer) clearTimeout(memorySearchTimer);
    memorySearchTimer = setTimeout(() => {
      onSearchMemory?.(value);
    }, 250);
  }

  /** Debounced bookmark search. */
  function onBookmarkSearchInput(value: string) {
    bookmarkSearchQuery = value;
    focusedBookmarkIndex = -1;
    if (bookmarkSearchTimer) clearTimeout(bookmarkSearchTimer);
    bookmarkSearchTimer = setTimeout(() => {
      onSearchBookmarks?.(value);
    }, 250);
  }

  /** Handle arrow key navigation in bookmark list. */
  function handleBookmarkKeydown(e: KeyboardEvent, index: number) {
    const filteredBookmarks = bookmarkSearchQuery
      ? bookmarks.filter(b => b.title.toLowerCase().includes(bookmarkSearchQuery.toLowerCase()) || b.url.toLowerCase().includes(bookmarkSearchQuery.toLowerCase()))
      : bookmarks;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusedBookmarkIndex = Math.min(focusedBookmarkIndex + 1, filteredBookmarks.length - 1);
      focusBookmark(focusedBookmarkIndex);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      focusedBookmarkIndex = Math.max(focusedBookmarkIndex - 1, 0);
      focusBookmark(focusedBookmarkIndex);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      onNavigate(filteredBookmarks[index].url);
    }
  }

  /** Focus a bookmark by index. */
  function focusBookmark(index: number) {
    const elements = document.querySelectorAll('[data-bookmark-index]');
    const element = elements[index] as HTMLElement;
    element?.focus();
  }

  /** Handle arrow key navigation in history list. */
  function handleHistoryKeydown(e: KeyboardEvent, index: number) {
    const allPages = indexedMemoryGroups.flatMap(g => g.pages).concat(historyGroups.flatMap(g => g.pages));

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusedHistoryIndex = Math.min(focusedHistoryIndex + 1, allPages.length - 1);
      focusHistory(focusedHistoryIndex);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      focusedHistoryIndex = Math.max(focusedHistoryIndex - 1, 0);
      focusHistory(focusedHistoryIndex);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      onNavigate(allPages[index].url);
    }
  }

  /** Focus a history item by index. */
  function focusHistory(index: number) {
    const elements = document.querySelectorAll('[data-history-index]');
    const element = elements[index] as HTMLElement;
    element?.focus();
  }
</script>

{#if open}
  <aside class="ai-sidebar exodus-sidebar" aria-label="Exodus sidebar">
    <div class="sidebar-header">
      <h3>Exodus</h3>
      <div class="sidebar-tabs">
        <button type="button" class="sidebar-tab-icon" class:active={sidebarPanel === 'ai'} onclick={() => onOpenPanel('ai')} title="AI">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3a7 7 0 0 0-4 12.7V21l4-2 4 2v-5.3A7 7 0 0 0 12 3z"/></svg>
        </button>
        <button type="button" class="sidebar-tab-icon" class:active={sidebarPanel === 'memory'} onclick={() => onOpenPanel('memory')} title="Memory & history">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>
        </button>
        <button type="button" class="sidebar-tab-icon" class:active={sidebarPanel === 'bookmarks'} onclick={() => onOpenPanel('bookmarks')} title="Bookmarks">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15z"/></svg>
        </button>
        <button type="button" class="sidebar-tab-icon" class:active={sidebarPanel === 'pocket'} onclick={() => onOpenPanel('pocket')} title="Pocket">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15z"/><path d="M12 7v10"/><path d="M9 10l3-3 3 3"/></svg>
        </button>
        <button type="button" class="sidebar-tab-icon" class:active={sidebarPanel === 'p2p'} onclick={() => onOpenPanel('p2p')} title="P2P CDN & group chat">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="12" r="2"/><circle cx="18" cy="7" r="2"/><circle cx="18" cy="17" r="2"/><path d="M8 12h5M13 10l3-3M13 14l3 3"/></svg>
        </button>
      </div>
      <button type="button" class="close-button" onclick={onClose} aria-label="Close sidebar">×</button>
    </div>
    <div class="sidebar-content">
      <PrivacySummaryStrip stats={privacyStats} onOpenSettings={onOpenPrivacySettings} />
      {#if sidebarPanel === 'memory'}
        <div class="list-panel">
          <input
            type="text"
            class="search-input"
            placeholder="Search indexed pages and history..."
            value={memorySearchQuery}
            oninput={(e) => onMemorySearchInput(e.currentTarget.value)}
          />
          <div class="history-panel-actions">
              <button type="button" class="nav-button secondary" onclick={onLoadMemory}>Refresh</button>
          </div>

          <h4 class="memory-section-title">Indexed memory ({indexedCount})</h4>
            <p class="muted memory-section-hint">Pages saved for /ask search. Use menu → Index page.</p>
            {#if indexedCount > 0}
              {#each indexedMemoryGroups as group (group.label)}
                <p class="history-group-label">{group.label}</p>
                {#each group.pages as page, pageIndex (page.id)}
                  <div class="list-item row">
                    <div
                      class="list-grow"
                      onclick={() => onNavigate(page.url)}
                      onkeydown={(e) => handleHistoryKeydown(e, pageIndex)}
                      role="link"
                      tabindex={focusedHistoryIndex === pageIndex ? 0 : -1}
                      data-history-index={pageIndex}
                    >
                      <div class="list-title">{page.title || page.url}</div>
                      <div class="list-sub">{page.url}</div>
                    </div>
                    <button
                      type="button"
                      class="tab-close"
                      aria-label="Remove from indexed memory"
                      onclick={(e) => {
                        e.stopPropagation();
                        onRemoveIndexedPage(page.id);
                      }}
                    >×</button>
                  </div>
                {/each}
              {/each}
              <button type="button" class="nav-button secondary danger full memory-clear-btn" onclick={onClearIndexed}>
                Clear indexed memory
              </button>
            {:else}
              <p class="muted">No indexed pages yet.</p>
            {/if}

            <h4 class="memory-section-title">Browsing history ({historyCount})</h4>
            {#if historyCount > 0}
              <div>
                {#each historyGroups as group (group.label)}
                  <p class="history-group-label">{group.label}</p>
                  {#each group.pages as page, historyIndex (page.id)}
                    <div
                      class="list-item"
                      onclick={() => onNavigate(page.url)}
                      onkeydown={(e) => handleHistoryKeydown(e, historyIndex)}
                      role="link"
                      tabindex={focusedHistoryIndex === historyIndex ? 0 : -1}
                      data-history-index={historyIndex}
                    >
                      <div class="list-title">{page.title || page.url}</div>
                      <div class="list-sub">
                        {page.url}
                        {#if page.visit_count && page.visit_count > 1}
                          · {page.visit_count} visits
                        {/if}
                      </div>
                    </div>
                  {/each}
                {/each}
                <button type="button" class="nav-button secondary danger full memory-clear-btn" onclick={onClearHistory}>
                  Clear browsing history
                </button>
              </div>
            {:else}
              <p class="muted">No visits recorded yet.</p>
            {/if}
        </div>
      {:else if sidebarPanel === 'bookmarks'}
        <div class="list-panel">
          <input
            type="text"
            class="search-input"
            placeholder="Search bookmarks..."
            bind:value={bookmarkSearchQuery}
            oninput={(e) => onBookmarkSearchInput(e.currentTarget.value)}
          />
          <button type="button" class="nav-button secondary full" onclick={onLoadBookmarks}>Refresh</button>
          {#each bookmarks as bm, index (bm.id)}
            <div class="list-item row">
              <div
                class="list-grow"
                onclick={() => onNavigate(bm.url)}
                onkeydown={(e) => handleBookmarkKeydown(e, index)}
                role="link"
                tabindex={focusedBookmarkIndex === index ? 0 : -1}
                data-bookmark-index={index}
              >
                <div class="list-title">{bm.title}</div>
                <div class="list-sub">{bm.url}</div>
                <input
                  type="text"
                  class="folder-input"
                  value={bm.folder || ''}
                  placeholder="Folder (empty = bar)"
                  onclick={(e) => e.stopPropagation()}
                  onkeydown={(e) => e.stopPropagation()}
                  onchange={(e) => onUpdateBookmarkFolder(bm.id, e.currentTarget.value)}
                />
              </div>
              <button type="button" class="tab-close" onclick={() => onRemoveBookmark(bm.id)} aria-label="Remove bookmark">×</button>
            </div>
          {:else}
            <p class="muted">No bookmarks. Use ★ in the toolbar.</p>
          {/each}
        </div>
      {:else if sidebarPanel === 'p2p'}
        <P2pSidebarPanel bind:roomId={p2pRoomId} onStatus={onP2pStatus} />
      {:else if sidebarPanel === 'pocket'}
        <PocketPanel onStatus={onP2pStatus} />
      {:else if agentPanelOpen}
        <AgentPanel
          bind:command={agentCommand}
          log={agentLog}
          executing={isAgentExecuting}
          domSummary={agentDomSummary}
          onExecute={onAgentExecute}
          onCompress={onAgentCompress}
          onBack={onAgentBack}
          onPreset={onAgentPreset}
          onCommandChange={onAgentCommandChange}
          onAskAi={onAgentAskAi}
        />
      {:else}
        <div class="ai-chat-panel">
          {#if aiCdnSuggestUrls.length > 0 && onAnnounceAllAiCdnUrls}
            <div class="cdn-suggest-banner">
              <p>AI shared {aiCdnSuggestUrls.length} large-file link(s) for P2P CDN.</p>
              <div class="cdn-suggest-actions">
                <button type="button" class="nav-button secondary cdn-btn" onclick={() => void onAnnounceAllAiCdnUrls()}>
                  Announce all to room
                </button>
                <button type="button" class="nav-button secondary cdn-btn" onclick={() => onDismissAiCdnSuggest?.()}>
                  Dismiss
                </button>
              </div>
            </div>
          {/if}
          {#if canAnnouncePage && onAnnounceCurrentPageToCdn}
            <div class="cdn-quick-actions">
              <button type="button" class="nav-button secondary cdn-btn" onclick={() => void onAnnounceCurrentPageToCdn()}>
                Share page to P2P CDN
              </button>
              <button type="button" class="nav-button secondary cdn-btn" onclick={() => onOpenPanel('p2p')}>
                Open P2P room
              </button>
            </div>
          {/if}
          {#if aiChatHistory.length > 0 || chatStreamBuffer}
            <div class="ai-chat-messages">
              {#each aiChatHistory as msg, i (i)}
                <div class="chat-bubble" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
                  {msg.content}
                  {#if msg.role === 'assistant' && onAnnounceUrlToCdn}
                    {@const urls = extractAnnounceableUrls(msg.content)}
                    {#if urls.length > 0}
                      <div class="cdn-url-chips">
                        {#each urls as url (url)}
                          <button
                            type="button"
                            class="cdn-chip"
                            onclick={() => void onAnnounceUrlToCdn(url, url)}
                          >
                            P2P · {url.slice(0, 36)}{url.length > 36 ? '…' : ''}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  {/if}
                </div>
              {/each}
              {#if isLoading && aiStreamMode === 'chat' && chatStreamBuffer}
                <div class="chat-bubble assistant streaming">
                  {chatStreamBuffer}
                  {#if onAnnounceUrlToCdn && streamLargeUrls.length > 0}
                    <div class="cdn-url-chips">
                      {#each streamLargeUrls as url (url)}
                        <button
                          type="button"
                          class="cdn-chip highlight"
                          onclick={() => void onAnnounceUrlToCdn(url, url)}
                        >
                          P2P seed · {url.slice(0, 32)}…
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
          {#if isLoading && aiStreamMode === 'summary'}
            <div class="loading-indicator">
              <div class="spinner"></div>
              <p>Generating summary...</p>
            </div>
          {:else if aiResponse}
            <div class="ai-response">
              <p class="ai-section-label">Summary</p>
              {#each aiResponse.split('\n') as line}
                <p class="ai-line">{line}</p>
              {/each}
            </div>
          {:else if aiChatHistory.length === 0 && !isLoading}
            <div class="sidebar-placeholder">
              <p>Ask Exodus anything, or select text on the page for AI Summary.</p>
              <button type="button" class="nav-button secondary full" onclick={onToggleAgent}>Open Agent</button>
            </div>
          {/if}
          <form
            class="ai-chat-form"
            onsubmit={(e) => {
              e.preventDefault();
              onSendChat();
            }}
          >
            <input
              type="text"
              class="ai-chat-input"
              bind:value={aiChatInput}
              placeholder={aiOnline ? 'Ask Exodus…' : 'AI offline — check settings'}
              disabled={isLoading}
            />
            {#if isLoading && onCancelChat}
              <button type="button" class="nav-button secondary" onclick={onCancelChat}>Stop</button>
            {:else}
              <button type="submit" class="nav-button" disabled={isLoading || !aiChatInput.trim()}>Send</button>
            {/if}
          </form>
        </div>
      {/if}
    </div>
  </aside>
{/if}

<style>
  .ai-sidebar {
    width: 380px;
    background: #252525;
    border-left: 1px solid #404040;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid #404040;
  }

  .sidebar-header h3 {
    margin: 0;
    font-size: 15px;
    flex-shrink: 0;
    color: #e0e0e0;
  }

  .sidebar-tabs {
    display: flex;
    gap: 4px;
    flex: 1;
  }

  .sidebar-tab-icon {
    background: #404040;
    border: none;
    color: #ccc;
    width: 32px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sidebar-tab-icon.active {
    background: #6366f1;
    color: #fff;
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    min-height: 0;
  }

  .tab-close {
    background: transparent;
    border: none;
    color: #888;
    font-size: 18px;
    cursor: pointer;
    padding: 0 6px;
  }

  .tab-close:hover {
    color: #f87171;
  }

  .sidebar-placeholder {
    text-align: center;
    color: #888;
    padding: 32px 12px;
  }

  .ai-chat-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .ai-chat-messages {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 12px;
  }

  .chat-bubble {
    padding: 8px 12px;
    border-radius: 10px;
    font-size: 13px;
    line-height: 1.5;
    max-width: 95%;
    white-space: pre-wrap;
  }

  .chat-bubble.user {
    align-self: flex-end;
    background: #6366f1;
    color: #fff;
  }

  .chat-bubble.assistant {
    align-self: flex-start;
    background: #333;
    color: #eee;
  }

  .chat-bubble.streaming {
    opacity: 0.85;
  }

  .ai-section-label {
    font-size: 11px;
    text-transform: uppercase;
    color: #888;
    margin: 0 0 8px;
  }

  .ai-chat-form {
    display: flex;
    gap: 8px;
    margin-top: auto;
    padding-top: 8px;
    border-top: 1px solid #404040;
  }

  .cdn-quick-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 10px;
  }

  .cdn-btn {
    font-size: 11px;
    padding: 4px 8px;
  }

  .cdn-url-chips {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
  }

  .cdn-chip {
    font-size: 10px;
    text-align: left;
    padding: 4px 6px;
    background: #1e3a2f;
    border: 1px solid #4ade80;
    color: #bbf7d0;
    border-radius: 4px;
    cursor: pointer;
  }

  .cdn-chip:hover {
    background: #14532d;
  }

  .cdn-chip.highlight {
    border-color: #fbbf24;
    background: #422006;
    color: #fde68a;
  }

  .cdn-suggest-banner {
    padding: 10px;
    margin-bottom: 10px;
    background: #2a2520;
    border: 1px solid #fbbf24;
    border-radius: 8px;
    font-size: 12px;
    color: #fde68a;
  }

  .cdn-suggest-banner p {
    margin: 0 0 8px;
  }

  .cdn-suggest-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .ai-chat-input {
    flex: 1;
    background: #1a1a1a;
    border: 1px solid #404040;
    color: #eee;
    padding: 8px 10px;
    border-radius: 6px;
    font-size: 13px;
  }

  .ai-response {
    line-height: 1.6;
    font-size: 14px;
    color: #e0e0e0;
  }

  .ai-line {
    margin: 0 0 0.5em;
    white-space: pre-wrap;
  }

  .loading-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 40px;
  }
</style>
