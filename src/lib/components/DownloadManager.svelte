<script lang="ts">
  /**
   * Exodus Browser — Download Manager UI
   */

  import { openPath } from '@tauri-apps/plugin-opener';
  import { invoke } from '@tauri-apps/api/core';

  interface DownloadItem {
    id: string;
    url: string;
    filename: string;
    file_path: string;
    total_size: number;
    downloaded_size: number;
    status: 'pending' | 'downloading' | 'completed' | 'failed' | 'cancelled';
    error: string | null;
    created_at: number;
  }

  let downloads: DownloadItem[] = [];
  let filteredDownloads: DownloadItem[] = [];
  let showCompletedOnly = false;
  let searchQuery = '';

  async function loadDownloads() {
    // In a real implementation, this would load downloads from the backend
    // For now, we'll use a placeholder
    downloads = [];
    filterDownloads();
  }

  function filterDownloads() {
    filteredDownloads = downloads.filter((d) => {
      if (showCompletedOnly && d.status !== 'completed') {
        return false;
      }
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return (
          d.filename.toLowerCase().includes(query) ||
          d.url.toLowerCase().includes(query)
        );
      }
      return true;
    });
  }

  function getProgress(item: DownloadItem): number {
    if (item.total_size === 0) return 0;
    return (item.downloaded_size / item.total_size) * 100;
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'completed':
        return '#059669';
      case 'downloading':
        return '#2563eb';
      case 'failed':
        return '#dc2626';
      case 'cancelled':
        return '#6b7280';
      default:
        return '#d97706';
    }
  }

  async function openFile(filePath: string) {
    try {
      await openPath(filePath);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  }

  async function openDownloadsFolder() {
    try {
      await invoke('open_downloads_folder');
    } catch (error) {
      console.error('Failed to open downloads folder:', error);
    }
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }

  // Load downloads on mount
  loadDownloads();

  $: showCompletedOnly, filterDownloads();
  $: searchQuery, filterDownloads();
</script>

<div class="download-manager">
  <div class="header">
    <h2>Download Manager</h2>
    <div class="actions">
      <input
        type="text"
        placeholder="Search downloads..."
        bind:value={searchQuery}
        class="search-input"
      />
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={showCompletedOnly} />
        Show completed only
      </label>
      <button class="btn btn-secondary" on:click={openDownloadsFolder}>
        Open Downloads Folder
      </button>
    </div>
  </div>

  <div class="download-list">
    {#if filteredDownloads.length === 0}
      <div class="empty-state">
        <p>No downloads found</p>
      </div>
    {:else}
      {#each filteredDownloads as download (download.id)}
        <div class="download-item">
          <div class="download-info">
            <div class="filename">{download.filename}</div>
            <div class="url">{download.url}</div>
            <div class="size">
              {formatSize(download.downloaded_size)} / {formatSize(download.total_size)}
            </div>
            <div class="status" style="color: {getStatusColor(download.status)}">
              {download.status}
            </div>
            {#if download.error}
              <div class="error">{download.error}</div>
            {/if}
          </div>
          <div class="download-progress">
            <div class="progress-bar">
              <div
                class="progress-fill"
                style="width: {getProgress(download)}%; background: {getStatusColor(download.status)}"
              ></div>
            </div>
            <div class="progress-text">{Math.round(getProgress(download))}%</div>
          </div>
          <div class="download-actions">
            {#if download.status === 'completed'}
              <button
                class="btn-icon"
                title="Open file"
                on:click={() => openFile(download.file_path)}
              >
                📂
              </button>
            {/if}
            <button
              class="btn-icon"
              title="Delete"
            >
              🗑️
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .download-manager {
    padding: 20px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .search-input {
    padding: 8px 12px;
    border: 1px solid #555;
    border-radius: 6px;
    background: #333;
    color: #eee;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 5px;
    color: #aaa;
  }

  .download-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .download-item {
    display: flex;
    flex-direction: column;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .download-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .filename {
    font-weight: bold;
    color: #eee;
  }

  .url {
    color: #888;
    font-size: 12px;
    margin-bottom: 3px;
  }

  .size {
    color: #aaa;
    font-size: 14px;
  }

  .status {
    font-weight: bold;
    text-transform: capitalize;
  }

  .error {
    color: #dc2626;
    font-size: 12px;
  }

  .download-progress {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
  }

  .progress-bar {
    flex: 1;
    height: 8px;
    background: #444;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 12px;
    color: #aaa;
    min-width: 40px;
    text-align: right;
  }

  .download-actions {
    display: flex;
    gap: 5px;
    justify-content: flex-end;
  }

  .btn-icon {
    background: #444;
    border: 1px solid #555;
    color: #eee;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-icon:hover {
    background: #555;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    border: none;
  }

  .btn-secondary {
    background: #444;
    color: #eee;
  }

  .btn-secondary:hover {
    background: #555;
  }
</style>
