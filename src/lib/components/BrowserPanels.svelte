<script lang="ts">
  /**
   * Exodus Browser — downloads overlay panel (Chrome-style actions per item).
   */
  import { invoke } from '@tauri-apps/api/core';
  import type { DownloadRecord } from '$lib/browserTypes';
  import BrowserPanel from '$lib/components/BrowserPanel.svelte';

  type Props = {
    showDownloads: boolean;
    downloads: DownloadRecord[];
    onCloseDownloads: () => void;
    onOpenDownloadsDir: () => void;
    onClearDownloads: () => void;
  };

  let {
    showDownloads,
    downloads,
    onCloseDownloads,
    onOpenDownloadsDir,
    onClearDownloads,
  }: Props = $props();

  async function openDownloadFile(path: string) {
    try {
      await invoke('open_download', { path });
    } catch (error) {
      console.error('open_download failed:', error);
    }
  }

  async function revealDownloadFile(path: string) {
    try {
      await invoke('reveal_download', { path });
    } catch (error) {
      console.error('reveal_download failed:', error);
    }
  }
</script>

<BrowserPanel open={showDownloads} title="Downloads" onClose={onCloseDownloads}>
  {#if downloads.length > 0}
    <div class="downloads-list">
      {#each downloads as download (download.id)}
        <div class="download-item">
          <div class="download-row">
            <span class="download-name">{download.filename}</span>
            <span
              class="download-status"
              class:done={download.status === 'completed'}
              class:failed={download.status === 'failed'}
            >
              {download.status}
              {#if download.status === 'downloading' && download.total > 0}
                · {download.progress.toFixed(0)}%
              {/if}
            </span>
          </div>
          {#if download.status === 'downloading' || download.status === 'pending'}
            <div class="download-progress-track">
              <div
                class="download-progress-bar"
                style="width: {Math.max(download.progress, 2)}%"
              ></div>
            </div>
          {/if}
          {#if download.path && download.status === 'completed'}
            <div class="download-item-actions">
              <button type="button" class="download-action" onclick={() => openDownloadFile(download.path!)}>
                Open
              </button>
              <button type="button" class="download-action secondary" onclick={() => revealDownloadFile(download.path!)}>
                Show in folder
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
    <div class="download-actions">
      <button type="button" class="nav-button secondary" onclick={onOpenDownloadsDir}>Open folder</button>
      <button type="button" class="nav-button secondary" onclick={onClearDownloads}>Clear list</button>
    </div>
  {:else}
    <div class="empty-state">No downloads yet. Use menu → Save page as download.</div>
  {/if}
</BrowserPanel>

<style>
  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #9aa0a6;
    font-size: 14px;
  }

  .downloads-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .download-item {
    padding: 8px 0;
    border-bottom: 1px solid #404040;
  }

  .download-row {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 13px;
  }

  .download-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .download-progress-track {
    height: 4px;
    background: #404040;
    border-radius: 2px;
    margin-top: 6px;
    overflow: hidden;
  }

  .download-progress-bar {
    height: 100%;
    background: #6366f1;
    transition: width 0.2s ease;
  }

  .download-status {
    color: #9aa0a6;
    font-size: 12px;
    padding: 4px 8px;
    background: #2d2d30;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .download-status.done {
    color: #4ade80;
  }

  .download-status.failed {
    color: #f87171;
  }

  .download-item-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .download-action {
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid #6366f1;
    background: rgba(99, 102, 241, 0.2);
    color: #e9e9ff;
    cursor: pointer;
  }

  .download-action.secondary {
    border-color: #555;
    background: transparent;
    color: #c4c4cc;
  }

  .download-action:hover {
    background: rgba(99, 102, 241, 0.4);
  }

  .download-actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }
</style>
