<script lang="ts">
  /**
   * Exodus Browser — main content row: webview host, quick links, and sidebar slot.
   */
  import type { Snippet } from 'svelte';
  import type { QuickLink } from '$lib/browserTypes';
  import NewTabPage from '$lib/components/NewTabPage.svelte';

  import { iframeSandboxAttr } from '$lib/privacySettings';

  type Props = {
    contentHost: HTMLDivElement | undefined;
    webviewFrame: HTMLIFrameElement | undefined;
    useNativeWebview: boolean;
    blockPopups?: boolean;
    currentUrl: string;
    aiSidebarOpen: boolean;
    showQuickLinks: boolean;
    newTabTopSites?: QuickLink[];
    newTabWallpaperId?: string;
    aiOnline?: boolean;
    aiModel?: string;
    onWallpaperChange?: (id: string) => void;
    onFrameLoad: () => void;
    onContentMouseUp: () => void;
    onQuickLinkNavigate: (url: string) => void;
    sidebar: Snippet;
  };

  let {
    contentHost = $bindable(),
    webviewFrame = $bindable(),
    useNativeWebview,
    blockPopups = false,
    currentUrl,
    aiSidebarOpen,
    showQuickLinks,
    newTabTopSites = [],
    newTabWallpaperId = 'nebula',
    aiOnline = false,
    aiModel = 'gemma4-e2b',
    onWallpaperChange,
    onFrameLoad,
    onContentMouseUp,
    onQuickLinkNavigate,
    sidebar,
  }: Props = $props();
</script>

<div class="content-area exodus-content">
  <div
    class="webview-container"
    class:sidebar-closed={!aiSidebarOpen}
    bind:this={contentHost}
    role="region"
    aria-label="Browser content"
    tabindex="-1"
    onpointerup={onContentMouseUp}
  >
    {#if !useNativeWebview}
      <iframe
        bind:this={webviewFrame}
        src={currentUrl}
        class="browser-webview"
        title="Browser Content"
        sandbox={iframeSandboxAttr(blockPopups)}
        onload={onFrameLoad}
      ></iframe>
    {:else}
      <div class="native-webview-host" aria-label="Native WebView"></div>
    {/if}
    <NewTabPage
      visible={showQuickLinks}
      wallpaperId={newTabWallpaperId}
      topSites={newTabTopSites}
      {aiOnline}
      {aiModel}
      onNavigate={onQuickLinkNavigate}
      onWallpaperChange={(id) => onWallpaperChange?.(id)}
    />
  </div>

  {@render sidebar()}
</div>

<style>
  .content-area {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .webview-container {
    flex: 1;
    position: relative;
    min-height: 0;
  }

  .browser-webview,
  .native-webview-host {
    width: 100%;
    height: 100%;
    border: none;
    background: #111;
  }
</style>
