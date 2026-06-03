<script lang="ts">
  /**
   * Exodus Browser — Clipboard Sync UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface ClipboardItem {
    id: string;
    content: string;
    type: 'text' | 'image' | 'file';
    timestamp: number;
    source: string;
  }

  let clipboardHistory: ClipboardItem[] = [];
  let filteredHistory: ClipboardItem[] = [];
  let syncEnabled = true;
  let searchQuery = '';
  let showSettingsDialog = false;

  // Settings
  let syncInterval = 5;
  let maxHistorySize = 100;
  let autoSync = true;

  async function loadClipboardHistory() {
    // In a real implementation, this would load from the backend
    // For now, we'll use a placeholder
    clipboardHistory = [];
    filterHistory();
  }

  function filterHistory() {
    if (!searchQuery) {
      filteredHistory = clipboardHistory;
    } else {
      const query = searchQuery.toLowerCase();
      filteredHistory = clipboardHistory.filter(
        (item) =>
          item.content.toLowerCase().includes(query) ||
          item.source.toLowerCase().includes(query)
      );
    }
  }

  async function copyToClipboard(content: string) {
    try {
      await navigator.clipboard.writeText(content);
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  }

  async function deleteItem(id: string) {
    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting clipboard item:', id);
      clipboardHistory = clipboardHistory.filter((item) => item.id !== id);
      filterHistory();
    } catch (error) {
      console.error('Failed to delete item:', error);
    }
  }

  async function clearHistory() {
    if (!confirm('Are you sure you want to clear all clipboard history?')) return;

    try {
      // In a real implementation, this would clear via the backend
      console.log('Clearing clipboard history');
      clipboardHistory = [];
      filterHistory();
    } catch (error) {
      console.error('Failed to clear history:', error);
    }
  }

  async function toggleSync() {
    syncEnabled = !syncEnabled;

    try {
      // In a real implementation, this would enable/disable sync via the backend
      console.log('Sync enabled:', syncEnabled);
    } catch (error) {
      console.error('Failed to toggle sync:', error);
    }
  }

  async function saveSettings() {
    try {
      // In a real implementation, this would save settings via the backend
      console.log('Saving settings:', { syncInterval, maxHistorySize, autoSync });
      showSettingsDialog = false;
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }

  function getTypeIcon(type: string): string {
    switch (type) {
      case 'text':
        return '📝';
      case 'image':
        return '🖼️';
      case 'file':
        return '📁';
      default:
        return '📋';
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function truncateContent(content: string, maxLength: number = 100): string {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + '...';
  }

  // Load history on mount
  loadClipboardHistory();

  // Auto-sync simulation
  setInterval(() => {
    if (autoSync && syncEnabled) {
      // In a real implementation, this would sync with other devices
      console.log('Auto-syncing clipboard...');
    }
  }, syncInterval * 1000);

  $: searchQuery, filterHistory();
</script>

<div class="clipboard-sync">
  <div class="header">
    <h2>Clipboard Sync</h2>
    <div class="actions">
      <input
        type="text"
        placeholder="Search history..."
        bind:value={searchQuery}
        class="search-input"
      />
      <button class="btn btn-secondary" on:click={() => (showSettingsDialog = true)}>
        ⚙️
      </button>
      <button class="btn btn-danger" on:click={clearHistory}>
        Clear History
      </button>
    </div>
  </div>

  <div class="sync-status">
    <div class="status-indicator">
      <div class="indicator {syncEnabled ? 'enabled' : 'disabled'}"></div>
      <span>Sync {syncEnabled ? 'Enabled' : 'Disabled'}</span>
    </div>
    <button class="btn btn-primary" on:click={toggleSync}>
      {syncEnabled ? 'Disable Sync' : 'Enable Sync'}
    </button>
  </div>

  <div class="clipboard-history">
    <div class="history-header">
      <h3>Clipboard History</h3>
      <div class="history-count">{filteredHistory.length} items</div>
    </div>

    {#if filteredHistory.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📋</div>
        <p>No clipboard history</p>
      </div>
    {:else}
      <div class="history-list">
        {#each filteredHistory as item (item.id)}
          <div class="history-item">
            <div class="item-icon">{getTypeIcon(item.type)}</div>
            <div class="item-content">
              <div class="content">{truncateContent(item.content)}</div>
              <div class="meta">
                <span class="source">Source: {item.source}</span>
                <span class="timestamp">{formatDate(item.timestamp)}</span>
              </div>
            </div>
            <div class="item-actions">
              <button
                class="btn-icon"
                title="Copy"
                on:click={() => copyToClipboard(item.content)}
              >
                📋
              </button>
              <button
                class="btn-icon"
                title="Delete"
                on:click={() => deleteItem(item.id)}
              >
                🗑️
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Settings Dialog -->
  {#if showSettingsDialog}
    <div class="dialog-overlay" on:click={() => (showSettingsDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Clipboard Sync Settings</h3>
        <form on:submit|preventDefault={saveSettings}>
          <div class="form-group">
            <label>Sync Interval (seconds)</label>
            <input type="number" bind:value={syncInterval} min="1" max="60" />
          </div>
          <div class="form-group">
            <label>Max History Size</label>
            <input type="number" bind:value={maxHistorySize} min="10" max="1000" />
          </div>
          <div class="form-group">
            <label>
              <input type="checkbox" bind:checked={autoSync} />
              Auto-sync
            </label>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showSettingsDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .clipboard-sync {
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
  }

  .search-input {
    padding: 8px 12px;
    border: 1px solid #555;
    border-radius: 6px;
    background: #333;
    color: #eee;
  }

  .sync-status {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
    margin-bottom: 20px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .indicator.enabled {
    background: #059669;
    box-shadow: 0 0 8px #059669;
  }

  .indicator.disabled {
    background: #6b7280;
  }

  .clipboard-history {
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
    padding: 15px;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
  }

  .history-header h3 {
    margin: 0;
  }

  .history-count {
    color: #aaa;
    font-size: 14px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 20px;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 15px;
    background: #444;
    border-radius: 6px;
  }

  .item-icon {
    font-size: 24px;
  }

  .item-content {
    flex: 1;
  }

  .content {
    color: #eee;
    margin-bottom: 5px;
    font-family: monospace;
    font-size: 14px;
  }

  .meta {
    display: flex;
    gap: 15px;
    font-size: 12px;
    color: #888;
  }

  .item-actions {
    display: flex;
    gap: 5px;
  }

  .btn-icon {
    background: #555;
    border: 1px solid #666;
    color: #eee;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-icon:hover {
    background: #666;
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #333;
    border: 1px solid #555;
    border-radius: 8px;
    padding: 20px;
    min-width: 400px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    color: #aaa;
  }

  .form-group input[type='number'] {
    width: 100%;
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    border: none;
  }

  .btn-primary {
    background: #6366f1;
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }

  .btn-secondary {
    background: #444;
    color: #eee;
  }

  .btn-secondary:hover {
    background: #555;
  }

  .btn-danger {
    background: #dc2626;
    color: white;
  }

  .btn-danger:hover {
    background: #b91c1c;
  }
</style>
