<script lang="ts">
  /**
   * Exodus Browser — settings modal (AI, browser, privacy, appearance).
   */
  import type { SidecarStatusDto } from '$lib/browserSettings';
  import { sidecarStateLabel } from '$lib/browserSettings';
  import ExtensionsSettings from '$lib/components/ExtensionsSettings.svelte';
  import BrowserSitePermissionsSettings from '$lib/components/BrowserSitePermissionsSettings.svelte';
  import P2pCdnPanel from '$lib/components/P2pCdnPanel.svelte';
  import GroupChatPanel from '$lib/components/GroupChatPanel.svelte';
  import PasswordManager from '$lib/components/PasswordManager.svelte';
  import CookieManager from '$lib/components/CookieManager.svelte';
  import { clearBrowsingData } from '$lib/browserIntegrations';
  import PrivacyShieldSettings from '$lib/components/PrivacyShieldSettings.svelte';
  import PasswordAutofillSettings from '$lib/components/PasswordAutofillSettings.svelte';
  import PrivacyDashboardPanel from '$lib/components/PrivacyDashboardPanel.svelte';
  import HistoryManagerPanel from '$lib/components/HistoryManagerPanel.svelte';
  import FormAutofillPanel from '$lib/components/FormAutofillPanel.svelte';
  import VerticalTabsSettings from '$lib/components/VerticalTabsSettings.svelte';
  import type { VerticalTabSettings } from '$lib/verticalTabs';
  import AllamaServiceSettings from '$lib/components/AllamaServiceSettings.svelte';
  import NewTabWallpaperSettings from '$lib/components/NewTabWallpaperSettings.svelte';

  export type SettingsScrollSection = 'privacy' | 'history' | 'autofill' | 'tabs' | 'allama';

  type Props = {
    open: boolean;
    scrollToSection?: SettingsScrollSection | null;
    aiPort: number;
    aiModel: string;
    embeddingModel: string;
    homepageUrl: string;
    searchEngineUrl: string;
    statusClearMs: number;
    spawnSidecar: boolean;
    spawnAllama: boolean;
    sidecarStatus: SidecarStatusDto | null;
    aiOnline: boolean;
    embeddingsOnline: boolean;
    autoIndexPages: boolean;
    showBookmarkBar: boolean;
    isDarkTheme: boolean;
    zoomLevel: number;
    httpsOnly: boolean;
    privateMode: boolean;
    blockPopups: boolean;
    sessionRestore: boolean;
    onClose: () => void;
    onSave: () => void;
    onRestartSidecar: () => void;
    onRefreshSidecar: () => void;
    indexedPageCount: number | null;
    indexedPagesLoading: boolean;
    onClearMemory: () => void;
    onClearHistory: () => void;
    onToggleTheme: () => void;
    onAutoIndexChange: (enabled: boolean) => void;
    onZoomIn: () => void;
    onZoomOut: () => void;
    onZoomReset: () => void;
    onHttpsOnlyChange: (enabled: boolean) => void;
    onPrivateModeChange: (enabled: boolean) => void;
    onBlockPopupsChange: (enabled: boolean) => void;
    onSessionRestoreChange: (enabled: boolean) => void;
    onExportBookmarks: () => void;
    onImportBookmarks: () => void;
    onExtensionStatus: (message: string) => void;
    contentHost?: HTMLElement;
    p2pRoomId?: string;
    onVerticalTabsChange?: (settings: VerticalTabSettings) => void;
    onNewTabWallpaperChange?: (id: string) => void;
  };

  let {
    open,
    scrollToSection = null,
    aiPort = $bindable(11435),
    aiModel = $bindable('exodus-default'),
    embeddingModel = $bindable('nomic-embed-text'),
    homepageUrl = $bindable(''),
    searchEngineUrl = $bindable(''),
    statusClearMs = $bindable(4000),
    spawnSidecar = $bindable(false),
    spawnAllama = $bindable(true),
    sidecarStatus,
    aiOnline,
    embeddingsOnline,
    autoIndexPages,
    showBookmarkBar = $bindable(true),
    isDarkTheme,
    zoomLevel,
    httpsOnly = $bindable(false),
    privateMode = $bindable(false),
    blockPopups = $bindable(true),
    sessionRestore = $bindable(true),
    onClose,
    onSave,
    onRestartSidecar,
    onRefreshSidecar,
    indexedPageCount = null,
    indexedPagesLoading = false,
    onClearMemory,
    onClearHistory,
    onToggleTheme,
    onAutoIndexChange,
    onZoomIn,
    onZoomOut,
    onZoomReset,
    onHttpsOnlyChange,
    onPrivateModeChange,
    onBlockPopupsChange,
    onSessionRestoreChange,
    onExportBookmarks,
    onImportBookmarks,
    onExtensionStatus,
    contentHost,
    p2pRoomId = $bindable('lobby'),
    onVerticalTabsChange,
    onNewTabWallpaperChange,
  }: Props = $props();

  let clearDataCache = $state(false);
  let clearDataCookies = $state(true);
  let clearDataLocalStorage = $state(false);
  let clearDataHistory = $state(false);
  let clearingData = $state(false);

  let settingsContentEl: HTMLDivElement | undefined = $state();

  /** Scroll settings content to a section when opened from chrome UI. */
  $effect(() => {
    const target = scrollToSection;
    if (!open || !target || !settingsContentEl) return;
    const id = `settings-section-${target}`;
    requestAnimationFrame(() => {
      const el = settingsContentEl?.querySelector(`#${id}`);
      el?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    });
  });

  async function handleClearBrowsingData() {
    if (!clearDataCache && !clearDataCookies && !clearDataLocalStorage && !clearDataHistory) {
      onExtensionStatus('Select at least one category to clear');
      return;
    }
    clearingData = true;
    try {
      const msg = await clearBrowsingData({
        clearCache: clearDataCache,
        clearCookies: clearDataCookies,
        clearLocalStorage: clearDataLocalStorage,
        clearHistory: clearDataHistory,
      });
      onExtensionStatus(msg);
    } catch (error) {
      console.error('clear_browsing_data failed:', error);
      onExtensionStatus('Failed to clear browsing data');
    } finally {
      clearingData = false;
    }
  }
</script>

{#if open}
  <button type="button" class="settings-backdrop" aria-label="Close settings" onclick={onClose}></button>
  <div class="settings-modal" role="dialog" aria-labelledby="settings-title">
    <div class="settings-header">
      <h2 id="settings-title">Settings</h2>
      <button type="button" class="close-btn" onclick={onClose}>×</button>
    </div>
    <div class="settings-content" bind:this={settingsContentEl}>
      <div class="settings-section">
        <h3>AI Configuration</h3>
        <label>
          AI port
          <input type="number" bind:value={aiPort} min="1" max="65535" />
        </label>
        <label>
          Chat model
          <input type="text" bind:value={aiModel} />
        </label>
        <label>
          Embedding model
          <input type="text" bind:value={embeddingModel} placeholder="nomic-embed-text" />
        </label>
        <button type="button" class="nav-button" onclick={onSave}>Save</button>
        <span class="ai-status" class:online={aiOnline}>{aiOnline ? 'Chat API online' : 'Chat API offline'}</span>
        <span class="ai-status embed-status" class:online={embeddingsOnline}>
          {embeddingsOnline ? 'Vector search ready' : 'Vector search offline (keyword fallback)'}
        </span>
        <AllamaServiceSettings
          {spawnAllama}
          {aiPort}
          onStatus={onExtensionStatus}
          onSpawnAllamaChange={(v) => (spawnAllama = v)}
          onAiPortChange={(v) => (aiPort = v)}
        />
      </div>
      <div class="settings-section">
        <h3>exodus-core sidecar (legacy)</h3>
        <p class="settings-hint">
          Optional legacy sidecar. Allama on port 11435 replaces Ollama; enable only if you still need exodus-core.
        </p>
        <label class="checkbox-row">
          <input
            type="checkbox"
            id="spawnSidecar"
            bind:checked={spawnSidecar}
          />
          <span>Start exodus-core sidecar with the app</span>
        </label>
        {#if sidecarStatus}
          <div class="sidecar-status-box">
            <span class="sidecar-state" class:online={sidecarStatus.state === 'running'}>
              {sidecarStateLabel(sidecarStatus.state)} · port {sidecarStatus.port}
            </span>
            <span class="settings-hint">{sidecarStatus.detail}</span>
            <span class="ai-status" class:online={sidecarStatus.endpointOnline}>
              {sidecarStatus.endpointOnline
                ? 'Inference API responding (/v1/models)'
                : 'Inference API not reachable on this port'}
            </span>
          </div>
        {/if}
        <div class="sidecar-actions">
          <button type="button" class="nav-button secondary" onclick={onRefreshSidecar}>Refresh status</button>
          <button type="button" class="nav-button secondary" onclick={onRestartSidecar}>Restart sidecar</button>
        </div>
      </div>
      <div class="settings-section">
        <h3>Browser</h3>
        <label for="homepageUrl">
          Homepage URL
          <input type="text" id="homepageUrl" bind:value={homepageUrl} placeholder="https://duckduckgo.com" />
        </label>
        <label for="searchEngineUrl">
          Search engine (use {'{query}'} for terms)
          <input type="text" id="searchEngineUrl" bind:value={searchEngineUrl} placeholder={'https://duckduckgo.com/?q={query}'} />
        </label>
        <VerticalTabsSettings
          onStatus={onExtensionStatus}
          onLayoutChange={onVerticalTabsChange}
        />
      </div>

      <ExtensionsSettings onStatus={onExtensionStatus} {contentHost} />

      <GroupChatPanel bind:groupId={p2pRoomId} onStatus={onExtensionStatus} />
      <P2pCdnPanel roomId={p2pRoomId} onStatus={onExtensionStatus} />

      <div class="settings-section" id="settings-section-privacy">
        <h3>Privacy & memory</h3>
        <label class="checkbox-row">
          <input
            type="checkbox"
            id="autoIndexPages"
            checked={autoIndexPages}
            onchange={(e) => onAutoIndexChange(e.currentTarget.checked)}
          />
          <span>Auto-index pages for /ask search (after each visit)</span>
        </label>
        <label class="checkbox-row">
          <input
            type="checkbox"
            id="httpsOnly"
            bind:checked={httpsOnly}
            onchange={(e) => onHttpsOnlyChange(e.currentTarget.checked)}
          />
          <span>HTTPS-only mode (upgrade HTTP to HTTPS)</span>
        </label>
        <label class="checkbox-row">
          <input
            type="checkbox"
            id="privateMode"
            bind:checked={privateMode}
            onchange={(e) => onPrivateModeChange(e.currentTarget.checked)}
          />
          <span>Private mode (no history, no cookies)</span>
        </label>
        <label class="checkbox-row">
          <input
            type="checkbox"
            id="blockPopups"
            bind:checked={blockPopups}
            onchange={(e) => onBlockPopupsChange(e.currentTarget.checked)}
          />
          <span>Block popups (window.open and new windows)</span>
        </label>
        <label class="checkbox-row">
          <input
            type="checkbox"
            id="sessionRestore"
            bind:checked={sessionRestore}
            onchange={(e) => onSessionRestoreChange(e.currentTarget.checked)}
          />
          <span>Restore tabs on startup (disabled in private mode)</span>
        </label>

        <BrowserSitePermissionsSettings onStatus={onExtensionStatus} />

        <PrivacyShieldSettings onStatus={onExtensionStatus} />

        <PrivacyDashboardPanel onStatus={onExtensionStatus} />

        <p class="settings-hint memory-count" class:loading={indexedPagesLoading}>
          {#if indexedPagesLoading}
            Loading indexed pages…
          {:else if indexedPageCount !== null}
            {indexedPageCount} page{indexedPageCount === 1 ? '' : 's'} in local memory (RAG index)
          {:else}
            Indexed page count unavailable
          {/if}
        </p>
        <button type="button" class="nav-button secondary full danger" onclick={onClearMemory}>
          Clear local memory (RAG index)
        </button>
        <button type="button" class="nav-button secondary full danger" onclick={onClearHistory}>
          Clear browsing history
        </button>
        <div class="clear-data-box">
          <p class="settings-hint">Clear browsing data (Chrome-style)</p>
          <label class="checkbox-row">
            <input type="checkbox" bind:checked={clearDataCache} />
            <span>Cached images and files</span>
          </label>
          <label class="checkbox-row">
            <input type="checkbox" bind:checked={clearDataCookies} />
            <span>Cookies and site data</span>
          </label>
          <label class="checkbox-row">
            <input type="checkbox" bind:checked={clearDataLocalStorage} />
            <span>Local memory (RAG index)</span>
          </label>
          <label class="checkbox-row">
            <input type="checkbox" bind:checked={clearDataHistory} />
            <span>Visit history</span>
          </label>
          <button
            type="button"
            class="nav-button secondary full danger"
            disabled={clearingData}
            onclick={() => void handleClearBrowsingData()}
          >
            {clearingData ? 'Clearing…' : 'Clear selected data'}
          </button>
        </div>
        <div class="bookmarks-actions">
          <button type="button" class="nav-button secondary full" onclick={onExportBookmarks}>
            Export bookmarks
          </button>
          <button type="button" class="nav-button secondary full" onclick={onImportBookmarks}>
            Import bookmarks
          </button>
        </div>
      </div>

      <HistoryManagerPanel onStatus={onExtensionStatus} />

      <FormAutofillPanel onStatus={onExtensionStatus} />

      <div class="settings-section">
        <PasswordAutofillSettings onStatus={onExtensionStatus} />
      </div>

      <div class="settings-section settings-panel-embed">
        <h3>Passwords</h3>
        <PasswordManager />
      </div>

      <div class="settings-section settings-panel-embed">
        <h3>Cookies</h3>
        <CookieManager />
      </div>

      <div class="settings-section">
        <h3>Appearance</h3>
        <label class="checkbox-row">
          <input type="checkbox" id="showBookmarkBar" bind:checked={showBookmarkBar} />
          <span>Show bookmark bar</span>
        </label>
        <label class="checkbox-row">
          <input type="checkbox" id="darkTheme" checked={isDarkTheme} onchange={onToggleTheme} />
          <span>Dark theme</span>
        </label>
        <label>
          Status bar duration (ms)
          <input type="number" bind:value={statusClearMs} min="1000" max="60000" step="500" />
        </label>
        <p class="settings-hint">How long status messages stay visible before clearing.</p>
        <NewTabWallpaperSettings
          onStatus={onExtensionStatus}
          onWallpaperChange={onNewTabWallpaperChange}
        />
        <div class="zoom-controls">
          <span class="zoom-label">Zoom: {zoomLevel}%</span>
          <div class="zoom-buttons">
            <button type="button" class="zoom-btn" onclick={onZoomOut} title="Zoom out">−</button>
            <button type="button" class="zoom-btn" onclick={onZoomReset} title="Reset zoom">Reset</button>
            <button type="button" class="zoom-btn" onclick={onZoomIn} title="Zoom in">+</button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1999;
    background: rgba(0, 0, 0, 0.45);
    border: none;
    cursor: default;
  }

  .settings-modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: #2d2d2d;
    border: 1px solid #404040;
    border-radius: 12px;
    z-index: 2000;
    width: 90%;
    max-width: 640px;
    max-height: 85vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #404040;
  }

  .settings-header h2 {
    margin: 0;
    font-size: 18px;
    color: #e0e0e0;
  }

  .close-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 24px;
    cursor: pointer;
    line-height: 1;
  }

  .settings-content {
    padding: 16px 20px;
    overflow-y: auto;
  }

  .settings-section {
    margin-bottom: 20px;
  }

  .settings-section h3 {
    margin: 0 0 12px;
    font-size: 14px;
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .settings-section label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
    font-size: 13px;
    color: #ccc;
  }

  .settings-section label.checkbox-row {
    flex-direction: row;
    align-items: center;
    gap: 8px;
  }

  .settings-section label input[type='text'],
  .settings-section label input[type='number'] {
    background: #1a1a1a;
    border: 1px solid #404040;
    color: #e0e0e0;
    padding: 8px;
    border-radius: 6px;
  }

  .nav-button {
    background: #2563eb;
    border: none;
    color: #fff;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }

  .nav-button.secondary {
    background: #404040;
    display: block;
    width: 100%;
    margin-top: 8px;
  }

  .nav-button.danger {
    background: #7f1d1d;
  }

  .ai-status {
    display: block;
    margin-top: 8px;
    font-size: 12px;
    color: #888;
  }

  .ai-status.online {
    color: #22c55e;
  }

  .ai-status.embed-status {
    margin-top: 4px;
  }

  .memory-count.loading {
    opacity: 0.75;
  }

  .settings-hint {
    font-size: 12px;
    color: #888;
    margin: 0 0 10px;
    line-height: 1.4;
  }

  .sidecar-status-box {
    background: #1a1a1a;
    border: 1px solid #404040;
    border-radius: 8px;
    padding: 10px 12px;
    margin-bottom: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .sidecar-state {
    font-size: 13px;
    font-weight: 500;
    color: #fbbf24;
  }

  .sidecar-state.online {
    color: #22c55e;
  }

  .sidecar-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .sidecar-actions .nav-button.secondary {
    flex: 1;
    min-width: 120px;
    margin-top: 0;
  }

  .zoom-controls {
    margin-top: 8px;
  }

  .zoom-label {
    font-size: 13px;
    color: #aaa;
    display: block;
    margin-bottom: 8px;
  }

  .zoom-buttons {
    display: flex;
    gap: 8px;
  }

  .zoom-btn {
    flex: 1;
    background: #404040;
    border: none;
    color: #e0e0e0;
    padding: 6px;
    border-radius: 6px;
    cursor: pointer;
  }

  .clear-data-box {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid #404040;
  }

  .settings-panel-embed {
    max-height: 320px;
    overflow-y: auto;
  }

  .settings-panel-embed :global(.password-manager),
  .settings-panel-embed :global(.cookie-manager) {
    max-height: 260px;
    overflow-y: auto;
  }
</style>
