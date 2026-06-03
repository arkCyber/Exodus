<!--
  Exodus Browser — ExodusWorkSpace file transfer UI (P2P CDN + transfer registry).
-->
<template>
  <div class="file-transfer">
    <div class="header">
      <h2>ExodusWorkSpace</h2>
      <p v-if="statusMessage" class="ws-status">{{ statusMessage }}</p>
      <p v-if="workspace" class="ws-path" :title="workspace.sharedDir">
        Shared: {{ workspace.sharedDir }}
      </p>

      <div class="dashboard-bar">
        <label class="dash-item">
          <span>Throttle (MB/s, 0=unlimited)</span>
          <input v-model.number="throttleMbps" type="number" min="0" step="0.5" />
          <button type="button" class="btn btn-secondary" @click="() => void applyThrottle()">Apply</button>
        </label>
        <label class="dash-item">
          <input v-model="autoReconnect" type="checkbox" @change="() => void toggleAutoReconnect()" />
          Auto-reconnect
        </label>
        <label class="dash-item">
          <input v-model="workspaceWatch" type="checkbox" @change="onWorkspaceWatchChange" />
          Watch shared/
        </label>
        <span v-if="backgroundJobs > 0" class="dash-badge">{{ backgroundJobs }} background job(s)</span>
      </div>

      <div class="relay-bar">
        <label class="dash-item">
          <input v-model="relayEnabled" type="checkbox" />
          WAN relay
        </label>
        <input v-model="relayBaseUrl" type="text" class="relay-input" placeholder="https://relay.example.com" />
        <button type="button" class="btn btn-secondary" @click="() => void saveRelayConfig()">Save relay</button>
        <span v-if="relayServerRunning" class="dash-badge">Relay server :{{ relayServerPort }}</span>
      </div>

      <div class="tabs">
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'transfers' }"
          @click="activeTab = 'transfers'"
        >
          Transfers
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'workspace' }"
          @click="switchToWorkspace"
        >
          Workspace Files ({{ workspaceFiles.length }})
        </button>
      </div>

      <div v-if="activeTab === 'transfers'" class="actions">
        <input v-model="searchQuery" type="text" placeholder="Search transfers..." class="search-input" />
        <select v-model="filterDirection" class="filter-select">
          <option value="all">All</option>
          <option value="upload">Uploads</option>
          <option value="download">Downloads</option>
        </select>
        <button type="button" class="btn btn-primary" @click="showUploadDialog = true">Upload File</button>
        <button type="button" class="btn btn-secondary" @click="showDownloadDialog = true">
          Download from Peer
        </button>
      </div>
    </div>

    <div v-if="activeTab === 'transfers'" class="transfer-list">
      <div v-if="filteredTransfers.length === 0" class="empty-state">
        <p>No transfers found</p>
      </div>
      <div v-for="transfer in filteredTransfers" :key="transfer.id" class="transfer-item">
        <div class="transfer-icon">{{ transfer.direction === 'upload' ? '⬆️' : '⬇️' }}</div>
        <div class="transfer-info">
          <div class="name">{{ transfer.name }}</div>
          <div class="peer">Peer: {{ transfer.peer }}</div>
          <div class="size">{{ formatSize(transfer.size) }}</div>
          <div v-if="transfer.speed > 0" class="speed">Speed: {{ formatSpeed(transfer.speed) }}</div>
          <div class="status" :style="{ color: getStatusColor(transfer.status) }">
            {{ transfer.status }}
          </div>
        </div>
        <div class="transfer-progress">
          <div class="progress-bar">
            <div
              class="progress-fill"
              :style="{
                width: `${transfer.progress}%`,
                background: getStatusColor(transfer.status),
              }"
            />
          </div>
          <div class="progress-text">{{ Math.round(transfer.progress) }}%</div>
        </div>
        <div class="transfer-actions">
          <button
            v-if="transfer.status === 'transferring' || transfer.status === 'paused'"
            type="button"
            class="btn-icon"
            title="Cancel"
            @click="() => void cancelTransfer(transfer.id)"
          >
            ❌
          </button>
          <button
            v-if="transfer.status === 'completed'"
            type="button"
            class="btn-icon"
            title="Verify checksum"
            @click="() => void verifyTransfer(transfer.id)"
          >
            ✓
          </button>
        </div>
      </div>
    </div>

    <div v-else class="workspace-files">
      <div v-if="workspaceFiles.length === 0" class="empty-state">
        <p>No files in workspace</p>
      </div>
      <div v-for="file in workspaceFiles" :key="file.name + file.relativePath" class="file-item">
        <div class="file-icon">📄</div>
        <div class="file-info">
          <div class="name">{{ file.name }}</div>
          <div class="path">{{ file.relativePath }}</div>
          <div class="size">{{ formatSize(file.sizeBytes) }}</div>
          <div v-if="file.contentHash" class="hash">Hash: {{ file.contentHash.slice(0, 16) }}…</div>
        </div>
      </div>
    </div>

    <div v-if="showUploadDialog" class="dialog-overlay" @click.self="showUploadDialog = false">
      <div class="dialog" role="dialog" aria-modal="true" @click.stop>
        <h3>Upload File</h3>
        <form @submit.prevent="() => void startUpload()">
          <button type="button" class="btn btn-secondary" @click="() => void pickFileAndUpload()">Pick File</button>
          <label>
            Local file path
            <input v-model="uploadPath" type="text" required placeholder="/path/to/file" />
          </label>
          <label>
            Receiver id (optional)
            <input v-model="uploadPeer" type="text" placeholder="peer node id" />
          </label>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" @click="showUploadDialog = false">Cancel</button>
            <button type="submit" class="btn btn-primary">Upload</button>
          </div>
        </form>
      </div>
    </div>

    <div v-if="showDownloadDialog" class="dialog-overlay" @click.self="showDownloadDialog = false">
      <div class="dialog" role="dialog" aria-modal="true" @click.stop>
        <h3>Background download (resume + checksum)</h3>
        <form @submit.prevent="() => void startDownload()">
          <label>
            CDN content hash (BLAKE3)
            <input v-model="downloadHash" type="text" required placeholder="hex hash" />
          </label>
          <label>
            Save as
            <input v-model="downloadFileName" type="text" placeholder="filename" />
          </label>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" @click="showDownloadDialog = false">Cancel</button>
            <button type="submit" class="btn btn-primary">Download</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
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
  fileTransferServiceStart,
  fileTransferSetAutoReconnect,
  fileTransferSetRelayConfig,
  fileTransferSetRelayServe,
  fileTransferSetThrottle,
  fileTransferStartBackgroundDownload,
  fileTransferVerifyChecksum,
  wanRelayServerInfo,
  type ExodusWorkspaceInfo,
  type FileTransferMetadata,
  type TransferDashboard,
  type TransferProgressEvent,
  type WorkspaceFileEntry,
} from '$lib/fileTransfer';

const emit = defineEmits<{ status: [message: string] }>();

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

const transfers = ref<TransferItem[]>([]);
const workspace = ref<ExodusWorkspaceInfo | null>(null);
const workspaceFiles = ref<WorkspaceFileEntry[]>([]);
const statusMessage = ref('');
const showUploadDialog = ref(false);
const showDownloadDialog = ref(false);
const searchQuery = ref('');
const filterDirection = ref<'all' | 'upload' | 'download'>('all');
const activeTab = ref<'transfers' | 'workspace'>('transfers');

const uploadPath = ref('');
const uploadPeer = ref('');
const downloadHash = ref('');
const downloadFileName = ref('received.bin');
const throttleMbps = ref(0);
const autoReconnect = ref(true);
const relayEnabled = ref(false);
const relayBaseUrl = ref('');
const relayServerRunning = ref(false);
const relayServerPort = ref(8790);
const workspaceWatch = ref(false);
const backgroundJobs = ref(0);

let unlistenProgress: UnlistenFn | null = null;

const filteredTransfers = computed(() => {
  const q = searchQuery.value.toLowerCase();
  return transfers.value.filter((t) => {
    if (filterDirection.value !== 'all' && t.direction !== filterDirection.value) return false;
    if (!q) return true;
    return (
      t.name.toLowerCase().includes(q) ||
      t.peer.toLowerCase().includes(q) ||
      (t.shortCode ?? '').includes(q)
    );
  });
});

function mapMeta(m: FileTransferMetadata): TransferItem {
  const dir = m.direction === 'download' ? 'download' : 'upload';
  const progress =
    m.progressPercent ?? (m.status === 'completed' ? 100 : m.status === 'pending' ? 0 : 10);
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

function applyProgress(ev: TransferProgressEvent): void {
  transfers.value = transfers.value.map((t) =>
    t.id === ev.transferId
      ? {
          ...t,
          status: ev.status as TransferItem['status'],
          progress: ev.progressPercent,
          speed: ev.speedBps,
          direction: ev.direction === 'download' ? 'download' : 'upload',
        }
      : t,
  );
}

async function loadDashboard(): Promise<void> {
  try {
    const dashboard: TransferDashboard = await fileTransferDashboard();
    transfers.value = dashboard.transfers.map(mapMeta);
    throttleMbps.value =
      Math.round((dashboard.settings.throttleBytesPerSec / (1024 * 1024)) * 10) / 10;
    autoReconnect.value = dashboard.settings.autoReconnect;
    relayEnabled.value = dashboard.relayEnabled;
    workspaceWatch.value = dashboard.workspaceWatchActive;
    backgroundJobs.value = dashboard.activeBackgroundJobs;
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
    await loadTransfersFallback();
  }
}

async function loadTransfersFallback(): Promise<void> {
  try {
    transfers.value = (await fileTransferList()).map(mapMeta);
  } catch (e) {
    statusMessage.value = String(e);
    transfers.value = [];
  }
}

async function refreshWorkspace(): Promise<void> {
  try {
    workspace.value = await exodusWorkspaceInfo();
    statusMessage.value = `WorkSpace ${workspace.value.roomId} · ${workspace.value.fileCount} file(s)`;
  } catch {
    workspace.value = await fileTransferServiceStart();
    statusMessage.value = `Started · mesh ${workspace.value.meshHost ?? '?'}:${workspace.value.meshPort ?? '?'}`;
  }
  emit('status', statusMessage.value);
}

async function loadWorkspaceFiles(): Promise<void> {
  try {
    workspaceFiles.value = await exodusWorkspaceList();
  } catch (e) {
    statusMessage.value = String(e);
    workspaceFiles.value = [];
  }
}

function switchToWorkspace(): void {
  activeTab.value = 'workspace';
  void loadWorkspaceFiles();
}

async function pickFileAndUpload(): Promise<void> {
  try {
    const path = await fileTransferPickFile();
    if (path) {
      uploadPath.value = path;
      showUploadDialog.value = true;
    }
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function applyThrottle(): Promise<void> {
  const bps = Math.max(0, Math.round(throttleMbps.value * 1024 * 1024));
  try {
    await fileTransferSetThrottle(bps);
    statusMessage.value = bps === 0 ? 'Throttle: unlimited' : `Throttle: ${formatSpeed(bps)}`;
    await loadDashboard();
    emit('status', statusMessage.value);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function toggleAutoReconnect(): Promise<void> {
  try {
    await fileTransferSetAutoReconnect(autoReconnect.value);
    statusMessage.value = autoReconnect.value ? 'Auto-reconnect on' : 'Auto-reconnect off';
    emit('status', statusMessage.value);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function saveRelayConfig(): Promise<void> {
  try {
    await fileTransferSetRelayConfig(relayEnabled.value, relayBaseUrl.value.trim() || undefined);
    const info = await fileTransferSetRelayServe(true, relayServerPort.value, '127.0.0.1');
    relayServerRunning.value = info.running;
    relayServerPort.value = info.port;
    if (!relayBaseUrl.value.trim() && info.baseUrl) relayBaseUrl.value = info.baseUrl;
    statusMessage.value = relayEnabled.value
      ? `WAN relay client on · server ${info.running ? info.baseUrl : 'off'}`
      : 'WAN relay client disabled';
    await loadDashboard();
    emit('status', statusMessage.value);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function refreshRelayServer(): Promise<void> {
  try {
    const info = await wanRelayServerInfo();
    relayServerRunning.value = info.running;
    relayServerPort.value = info.port;
    if (info.baseUrl && !relayBaseUrl.value.trim()) relayBaseUrl.value = info.baseUrl;
  } catch {
    relayServerRunning.value = false;
  }
}

/** Apply workspace folder watch when the checkbox changes. */
async function onWorkspaceWatchChange(): Promise<void> {
  try {
    if (workspaceWatch.value) {
      await exodusWorkspaceWatchStart();
      statusMessage.value = 'Watching shared/ for new files';
    } else {
      await exodusWorkspaceWatchStop();
      statusMessage.value = 'Workspace watch stopped';
    }
    emit('status', statusMessage.value);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function startUpload(): Promise<void> {
  if (!uploadPath.value.trim()) return;
  try {
    await fileTransferInitiate(uploadPath.value.trim(), uploadPeer.value.trim() || undefined);
    showUploadDialog.value = false;
    uploadPath.value = '';
    uploadPeer.value = '';
    await refreshWorkspace();
    await loadDashboard();
    statusMessage.value = 'File published to ExodusWorkSpace + CDN';
    emit('status', statusMessage.value);
  } catch (error) {
    statusMessage.value = String(error);
    emit('status', statusMessage.value);
  }
}

async function startDownload(): Promise<void> {
  if (!downloadHash.value.trim()) return;
  try {
    const name = downloadFileName.value.trim() || 'received.bin';
    await fileTransferStartBackgroundDownload(downloadHash.value.trim(), name);
    showDownloadDialog.value = false;
    downloadHash.value = '';
    statusMessage.value = `Background download started: ${name}`;
    await loadDashboard();
    emit('status', statusMessage.value);
  } catch (error) {
    statusMessage.value = String(error);
    emit('status', statusMessage.value);
  }
}

async function verifyTransfer(id: string): Promise<void> {
  try {
    const report = await fileTransferVerifyChecksum(id);
    statusMessage.value = report.destinationVerified
      ? 'Checksum verified'
      : `Checksum mismatch (${report.mismatchChunks.length} chunks)`;
    await loadDashboard();
    emit('status', statusMessage.value);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function cancelTransfer(id: string): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke('file_transfer_cancel', { transferId: id });
    await loadDashboard();
  } catch (error) {
    statusMessage.value = String(error);
    emit('status', statusMessage.value);
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
  return `${formatSize(bytesPerSecond)}/s`;
}

onMounted(() => {
  void (async () => {
    await refreshWorkspace();
    await loadDashboard();
    await refreshRelayServer();
    unlistenProgress = await listen<TransferProgressEvent>('file-transfer-progress', (e) =>
      applyProgress(e.payload),
    );
  })();
});

onUnmounted(() => {
  unlistenProgress?.();
});
</script>

<style scoped>
.file-transfer {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 12px;
  min-height: 0;
  height: 100%;
  overflow-y: auto;
}

.header h2 {
  margin: 0 0 6px;
  font-size: 15px;
}

.ws-status,
.ws-path {
  font-size: 11px;
  color: #9ca3af;
  margin: 2px 0;
}

.dashboard-bar,
.relay-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin: 6px 0;
}

.dash-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
}

.dash-badge {
  background: #333;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
}

.relay-input {
  flex: 1;
  min-width: 120px;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #e0e0e0;
}

.tabs {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.tab-btn {
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid #444;
  background: #333;
  color: #ccc;
  cursor: pointer;
}

.tab-btn.active {
  background: #6366f1;
  border-color: #6366f1;
  color: #fff;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.search-input,
.filter-select {
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #e0e0e0;
}

.transfer-item,
.file-item {
  display: flex;
  gap: 8px;
  padding: 8px;
  border-bottom: 1px solid #333;
  align-items: center;
}

.transfer-info,
.file-info {
  flex: 1;
  min-width: 0;
}

.progress-bar {
  height: 4px;
  background: #333;
  border-radius: 2px;
  overflow: hidden;
  width: 80px;
}

.progress-fill {
  height: 100%;
}

.btn {
  padding: 4px 10px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 12px;
}

.btn-primary {
  background: #6366f1;
  color: #fff;
}

.btn-secondary {
  background: #444;
  color: #e0e0e0;
}

.btn-icon {
  background: transparent;
  border: none;
  cursor: pointer;
}

.empty-state {
  text-align: center;
  color: #888;
  padding: 24px;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.dialog {
  background: #292a2d;
  padding: 16px;
  border-radius: 8px;
  min-width: 280px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dialog label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.dialog input {
  padding: 6px 8px;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #e0e0e0;
}

.form-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
