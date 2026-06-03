<script lang="ts">
  /**
   * Exodus Browser — ExodusWorkSpace file transfer UI (P2P CDN + transfer registry).
   */

  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    exodusWorkspaceInfo,
    exodusWorkspaceList,
    exodusWorkspaceWatchStart,
    exodusWorkspaceWatchStop,
    fileTransferDashboard,
    fileTransferInitiate,
    fileTransferList,
    fileTransferPickFile,
    fileTransferReceiveToInbox,
    fileTransferServiceStart,
    fileTransferSetAutoReconnect,
    fileTransferSetRelayConfig,
    fileTransferSetRelayServe,
    fileTransferSetThrottle,
    wanRelayServerInfo,
    fileTransferStartBackgroundDownload,
    fileTransferVerifyChecksum,
    type ExodusWorkspaceInfo,
    type FileTransferMetadata,
    type TransferDashboard,
    type TransferProgressEvent,
    type WorkspaceFileEntry,
  } from '$lib/fileTransfer';
  import { invoke } from '@tauri-apps/api/core';

  interface TransferItem {
    id: string;
    name: string;
    size: number;
    direction: 'upload' | 'download';
    status: 'pending' | 'transferring' | 'completed' | 'failed' | 'cancelled' | 'paused';
    progress: number;
    speed: number;
    peer: string;
    created_at: number;
    shortCode?: string;
    cdnHash?: string;
  }

  let transfers: TransferItem[] = [];
  let filteredTransfers: TransferItem[] = [];
  let workspace: ExodusWorkspaceInfo | null = null;
  let workspaceFiles: WorkspaceFileEntry[] = [];
  let statusMessage = '';
  let showUploadDialog = false;
  let showDownloadDialog = false;
  let searchQuery = '';
  let filterDirection: 'all' | 'upload' | 'download' = 'all';
  let activeTab: 'transfers' | 'workspace' = 'transfers';

  let uploadPath = '';
  let uploadPeer = '';
  let downloadHash = '';
  let downloadFileName = 'received.bin';
  let dashboard: TransferDashboard | null = null;
  let throttleMbps = 0;
  let autoReconnect = true;
  let relayEnabled = false;
  let relayBaseUrl = '';
  let relayServerRunning = false;
  let relayServerPort = 8790;
  let workspaceWatch = false;
  let backgroundJobs = 0;
  let unlistenProgress: UnlistenFn | null = null;

  function mapMeta(m: FileTransferMetadata): TransferItem {
    const dir =
      m.direction === 'download' ? 'download' : ('upload' as TransferItem['direction']);
    const progress =
      m.progressPercent ??
      (m.status === 'completed' ? 100 : m.status === 'pending' ? 0 : 10);
    return {
      id: m.transferId,
      name: m.fileName,
      size: m.fileSize,
      direction: dir,
      status: m.status as TransferItem['status'],
      progress,
      speed: m.speedBps ?? 0,
      peer: m.receiverId ?? m.senderId,
      created_at: m.createdAt,
      shortCode: m.shortCode,
      cdnHash: m.cdnContentHash,
    };
  }

  function applyProgress(ev: TransferProgressEvent) {
    transfers = transfers.map((t) =>
      t.id === ev.transferId
        ? {
            ...t,
            status: ev.status as TransferItem['status'],
            progress: ev.progressPercent,
            speed: ev.speedBps,
            direction: ev.direction === 'download' ? 'download' : 'upload',
          }
        : t
    );
    filterTransfers();
  }

  async function loadDashboard() {
    try {
      dashboard = await fileTransferDashboard();
      transfers = dashboard.transfers.map(mapMeta);
      throttleMbps = Math.round((dashboard.settings.throttleBytesPerSec / (1024 * 1024)) * 10) / 10;
      autoReconnect = dashboard.settings.autoReconnect;
      relayEnabled = dashboard.relayEnabled;
      workspaceWatch = dashboard.workspaceWatchActive;
      backgroundJobs = dashboard.activeBackgroundJobs;
      filterTransfers();
    } catch (e) {
      statusMessage = String(e);
      await loadTransfersFallback();
    }
  }

  async function loadTransfersFallback() {
    try {
      const list = await fileTransferList();
      transfers = list.map(mapMeta);
      filterTransfers();
    } catch (e) {
      statusMessage = String(e);
      transfers = [];
      filterTransfers();
    }
  }

  async function refreshWorkspace() {
    try {
      workspace = await exodusWorkspaceInfo();
      statusMessage = `WorkSpace ${workspace.roomId} · ${workspace.fileCount} file(s)`;
    } catch {
      workspace = await fileTransferServiceStart();
      statusMessage = `Started · mesh ${workspace.meshHost ?? '?'}:${workspace.meshPort ?? '?'}`;
    }
  }

  async function loadWorkspaceFiles() {
    try {
      workspaceFiles = await exodusWorkspaceList();
    } catch (e) {
      statusMessage = String(e);
      workspaceFiles = [];
    }
  }

  function filterTransfers() {
    filteredTransfers = transfers.filter((t) => {
      if (filterDirection !== 'all' && t.direction !== filterDirection) {
        return false;
      }
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return (
          t.name.toLowerCase().includes(query) ||
          t.peer.toLowerCase().includes(query) ||
          (t.shortCode ?? '').includes(query)
        );
      }
      return true;
    });
  }

  async function pickFileAndUpload() {
    try {
      const path = await fileTransferPickFile();
      if (path) {
        uploadPath = path;
        showUploadDialog = true;
      }
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function applyThrottle() {
    const bps = Math.max(0, Math.round(throttleMbps * 1024 * 1024));
    try {
      await fileTransferSetThrottle(bps);
      statusMessage = bps === 0 ? 'Throttle: unlimited' : `Throttle: ${formatSpeed(bps)}`;
      await loadDashboard();
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function toggleAutoReconnect() {
    try {
      await fileTransferSetAutoReconnect(autoReconnect);
      statusMessage = autoReconnect ? 'Auto-reconnect on' : 'Auto-reconnect off';
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function saveRelayConfig() {
    try {
      await fileTransferSetRelayConfig(relayEnabled, relayBaseUrl.trim() || undefined);
      const info = await fileTransferSetRelayServe(true, relayServerPort, '127.0.0.1');
      relayServerRunning = info.running;
      relayServerPort = info.port;
      if (!relayBaseUrl.trim() && info.baseUrl) {
        relayBaseUrl = info.baseUrl;
      }
      statusMessage = relayEnabled
        ? `WAN relay client on · server ${info.running ? info.baseUrl : 'off'}`
        : 'WAN relay client disabled';
      await loadDashboard();
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function refreshRelayServer() {
    try {
      const info = await wanRelayServerInfo();
      relayServerRunning = info.running;
      relayServerPort = info.port;
      if (info.baseUrl && !relayBaseUrl.trim()) {
        relayBaseUrl = info.baseUrl;
      }
    } catch {
      relayServerRunning = false;
    }
  }

  async function toggleWorkspaceWatch() {
    try {
      if (workspaceWatch) {
        await exodusWorkspaceWatchStop();
        workspaceWatch = false;
        statusMessage = 'Workspace watch stopped';
      } else {
        await exodusWorkspaceWatchStart();
        workspaceWatch = true;
        statusMessage = 'Watching shared/ for new files';
      }
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function startUpload() {
    if (!uploadPath.trim()) return;
    try {
      await fileTransferInitiate(uploadPath.trim(), uploadPeer.trim() || undefined);
      showUploadDialog = false;
      uploadPath = '';
      uploadPeer = '';
      await refreshWorkspace();
      await loadDashboard();
      statusMessage = 'File published to ExodusWorkSpace + CDN';
    } catch (error) {
      statusMessage = String(error);
      console.error('Failed to start upload:', error);
    }
  }

  async function startDownload() {
    if (!downloadHash.trim()) return;
    try {
      const name = downloadFileName.trim() || 'received.bin';
      await fileTransferStartBackgroundDownload(downloadHash.trim(), name);
      showDownloadDialog = false;
      downloadHash = '';
      statusMessage = `Background download started: ${name}`;
      await loadDashboard();
    } catch (error) {
      statusMessage = String(error);
      console.error('Failed to start download:', error);
    }
  }

  async function verifyTransfer(id: string) {
    try {
      const report = await fileTransferVerifyChecksum(id);
      statusMessage = report.destinationVerified
        ? 'Checksum verified'
        : `Checksum mismatch (${report.mismatchChunks.length} chunks)`;
      await loadDashboard();
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function cancelTransfer(id: string) {
    try {
      await invoke('file_transfer_cancel', { transferId: id });
      await loadDashboard();
    } catch (error) {
      statusMessage = String(error);
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'completed':
        return '#059669';
      case 'transferring':
        return '#2563eb';
      case 'failed':
        return '#dc2626';
      case 'cancelled':
        return '#6b7280';
      default:
        return '#d97706';
    }
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }

  function formatSpeed(bytesPerSecond: number): string {
    return formatSize(bytesPerSecond) + '/s';
  }

  onMount(() => {
    void (async () => {
      await refreshWorkspace();
      await loadDashboard();
      await refreshRelayServer();
      unlistenProgress = await listen<TransferProgressEvent>(
        'file-transfer-progress',
        (e) => applyProgress(e.payload)
      );
    })();
  });

  onDestroy(() => {
    unlistenProgress?.();
  });

  $: filterDirection, filterTransfers();
  $: searchQuery, filterTransfers();
  $: if (activeTab === 'workspace') {
    void loadWorkspaceFiles();
  }
</script>

<div class="file-transfer">
  <div class="header">
    <h2>ExodusWorkSpace</h2>
    {#if statusMessage}
      <p class="ws-status">{statusMessage}</p>
    {/if}
    {#if workspace}
      <p class="ws-path" title={workspace.sharedDir}>Shared: {workspace.sharedDir}</p>
    {/if}
    <div class="dashboard-bar">
      <label class="dash-item">
        <span>Throttle (MB/s, 0=unlimited)</span>
        <input type="number" min="0" step="0.5" bind:value={throttleMbps} />
        <button type="button" class="btn btn-secondary" on:click={applyThrottle}>Apply</button>
      </label>
      <label class="dash-item">
        <input type="checkbox" bind:checked={autoReconnect} on:change={toggleAutoReconnect} />
        Auto-reconnect
      </label>
      <label class="dash-item">
        <input type="checkbox" bind:checked={workspaceWatch} on:change={toggleWorkspaceWatch} />
        Watch shared/
      </label>
      {#if backgroundJobs > 0}
        <span class="dash-badge">{backgroundJobs} background job(s)</span>
      {/if}
    </div>
    <div class="relay-bar">
      <label class="dash-item">
        <input type="checkbox" bind:checked={relayEnabled} />
        WAN relay
      </label>
      <input
        type="text"
        class="relay-input"
        placeholder="https://relay.example.com"
        bind:value={relayBaseUrl}
      />
      <button type="button" class="btn btn-secondary" on:click={saveRelayConfig}>Save relay</button>
      {#if relayServerRunning}
        <span class="dash-badge">Relay server :{relayServerPort}</span>
      {/if}
    </div>
    <p class="tray-hint">Close window hides to tray — transfers keep running in background.</p>
    <div class="tabs">
      <button 
        class="tab-btn" 
        class:active={activeTab === 'transfers'} 
        on:click={() => (activeTab = 'transfers')}
      >
        Transfers
      </button>
      <button 
        class="tab-btn" 
        class:active={activeTab === 'workspace'} 
        on:click={() => (activeTab = 'workspace')}
      >
        Workspace Files ({workspaceFiles.length})
      </button>
    </div>
    {#if activeTab === 'transfers'}
      <div class="actions">
        <input
          type="text"
          placeholder="Search transfers..."
          bind:value={searchQuery}
          class="search-input"
        />
        <select bind:value={filterDirection} class="filter-select">
          <option value="all">All</option>
          <option value="upload">Uploads</option>
          <option value="download">Downloads</option>
        </select>
        <button class="btn btn-primary" on:click={() => (showUploadDialog = true)}>
          Upload File
        </button>
        <button class="btn btn-secondary" on:click={() => (showDownloadDialog = true)}>
          Download from Peer
        </button>
      </div>
    {/if}
  </div>

  {#if activeTab === 'transfers'}
    <div class="transfer-list">
      {#if filteredTransfers.length === 0}
        <div class="empty-state">
          <p>No transfers found</p>
        </div>
      {:else}
        {#each filteredTransfers as transfer (transfer.id)}
          <div class="transfer-item">
            <div class="transfer-icon">
              {transfer.direction === 'upload' ? '⬆️' : '⬇️'}
            </div>
            <div class="transfer-info">
              <div class="name">{transfer.name}</div>
              <div class="peer">Peer: {transfer.peer}</div>
              <div class="size">{formatSize(transfer.size)}</div>
              {#if transfer.speed > 0}
                <div class="speed">Speed: {formatSpeed(transfer.speed)}</div>
              {/if}
              <div class="status" style="color: {getStatusColor(transfer.status)}">
                {transfer.status}
              </div>
            </div>
            <div class="transfer-progress">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  style="width: {transfer.progress}%; background: {getStatusColor(transfer.status)}"
                ></div>
              </div>
              <div class="progress-text">{Math.round(transfer.progress)}%</div>
            </div>
            <div class="transfer-actions">
              {#if transfer.status === 'transferring' || transfer.status === 'paused'}
                <button
                  class="btn-icon"
                  title="Cancel"
                  on:click={() => cancelTransfer(transfer.id)}
                >
                  ❌
                </button>
              {/if}
              {#if transfer.status === 'completed'}
                <button
                  class="btn-icon"
                  title="Verify checksum"
                  on:click={() => verifyTransfer(transfer.id)}
                >
                  ✓
                </button>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {:else}
    <div class="workspace-files">
      {#if workspaceFiles.length === 0}
        <div class="empty-state">
          <p>No files in workspace</p>
        </div>
      {:else}
        {#each workspaceFiles as file (file.name + file.relativePath)}
          <div class="file-item">
            <div class="file-icon">📄</div>
            <div class="file-info">
              <div class="name">{file.name}</div>
              <div class="path">{file.relativePath}</div>
              <div class="size">{formatSize(file.sizeBytes)}</div>
              {#if file.contentHash}
                <div class="hash">Hash: {file.contentHash.slice(0, 16)}...</div>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}

  <!-- Upload Dialog -->
  {#if showUploadDialog}
    <div class="dialog-overlay" on:click={() => (showUploadDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Upload File</h3>
        <form on:submit|preventDefault={startUpload}>
          <div class="form-group">
            <button type="button" class="btn btn-secondary" on:click={pickFileAndUpload}>
              Pick File
            </button>
          </div>
          <div class="form-group">
            <label>Local file path</label>
            <input type="text" bind:value={uploadPath} placeholder="/path/to/file" required />
          </div>
          <div class="form-group">
            <label>Receiver id (optional)</label>
            <input type="text" bind:value={uploadPeer} placeholder="peer node id" />
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showUploadDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Upload</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Download Dialog -->
  {#if showDownloadDialog}
    <div class="dialog-overlay" on:click={() => (showDownloadDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Background download (resume + checksum)</h3>
        <form on:submit|preventDefault={startDownload}>
          <div class="form-group">
            <label>CDN content hash (BLAKE3)</label>
            <input type="text" bind:value={downloadHash} placeholder="hex hash" required />
          </div>
          <div class="form-group">
            <label>Save as</label>
            <input type="text" bind:value={downloadFileName} placeholder="filename" />
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showDownloadDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Download</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .file-transfer {
    padding: 20px;
  }

  .ws-status {
    font-size: 12px;
    color: #9ca3af;
    margin: 4px 0 0;
  }

  .tray-hint {
    font-size: 11px;
    color: #6b7280;
    margin: 0;
  }

  .dashboard-bar,
  .relay-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    font-size: 12px;
  }

  .dash-item {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #aaa;
  }

  .dash-item input[type='number'] {
    width: 72px;
    padding: 4px 6px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .dash-badge {
    padding: 4px 8px;
    background: #1e3a5f;
    border-radius: 4px;
    color: #93c5fd;
  }

  .relay-input {
    flex: 1;
    min-width: 180px;
    padding: 6px 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .ws-path {
    font-size: 11px;
    color: #6b7280;
    margin: 2px 0 0;
    max-width: 420px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .header {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
  }

  .tabs {
    display: flex;
    gap: 8px;
  }

  .tab-btn {
    padding: 8px 16px;
    background: #333;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    background: #444;
  }

  .tab-btn.active {
    background: #6366f1;
    border-color: #6366f1;
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

  .filter-select {
    padding: 8px 12px;
    border: 1px solid #555;
    border-radius: 6px;
    background: #333;
    color: #eee;
  }

  .transfer-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .workspace-files {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .file-icon {
    font-size: 24px;
  }

  .file-info .path {
    color: #888;
    font-size: 12px;
    margin-bottom: 3px;
  }

  .file-info .hash {
    color: #666;
    font-size: 11px;
    font-family: monospace;
    margin-top: 3px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .transfer-item {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .transfer-icon {
    font-size: 24px;
  }

  .transfer-info {
    flex: 1;
  }

  .name {
    font-weight: bold;
    color: #eee;
    margin-bottom: 3px;
  }

  .peer {
    color: #888;
    font-size: 12px;
    margin-bottom: 3px;
  }

  .size {
    color: #aaa;
    font-size: 14px;
    margin-bottom: 3px;
  }

  .speed {
    color: #888;
    font-size: 12px;
    margin-bottom: 3px;
  }

  .status {
    font-weight: bold;
    text-transform: capitalize;
  }

  .transfer-progress {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 150px;
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

  .transfer-actions {
    display: flex;
    gap: 5px;
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
    max-width: 500px;
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

  .form-group input[type='text'],
  .form-group input[type='file'] {
    width: 100%;
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .file-info {
    margin: 15px 0;
    padding: 15px;
    background: #444;
    border-radius: 4px;
  }

  .file-info div {
    margin-bottom: 5px;
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

  .dashboard-bar {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
    padding: 10px;
    background: #2a2a2a;
    border-radius: 6px;
    border: 1px solid #444;
  }

  .dash-item {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 12px;
    color: #aaa;
  }

  .dash-item input[type='number'] {
    width: 70px;
    padding: 4px 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .dash-item input[type='checkbox'] {
    cursor: pointer;
  }

  .dash-badge {
    padding: 4px 8px;
    background: #6366f1;
    border-radius: 4px;
    font-size: 11px;
    color: white;
  }

  .relay-bar {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px;
    background: #2a2a2a;
    border-radius: 6px;
    border: 1px solid #444;
  }

  .relay-input {
    flex: 1;
    padding: 6px 10px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }
</style>
