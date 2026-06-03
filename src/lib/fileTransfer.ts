/**
 * Exodus Browser — ExodusWorkSpace + file transfer API (dashboard, resume, relay).
 */
import { invoke } from '@tauri-apps/api/core';

export type WorkspaceFileEntry = {
  name: string;
  relativePath: string;
  sizeBytes: number;
  contentHash?: string;
  addedAt: number;
};

export type ExodusWorkspaceInfo = {
  root: string;
  sharedDir: string;
  inboxDir: string;
  outboxDir: string;
  nodeId: string;
  roomId: string;
  meshHost?: string;
  meshPort?: number;
  fileCount: number;
};

export type FileTransferMetadata = {
  transferId: string;
  fileName: string;
  fileSize: number;
  fileType: string;
  blobHash: string;
  chunkCount: number;
  senderId: string;
  receiverId?: string;
  status: string;
  createdAt: number;
  completedAt?: number;
  shortCode?: string;
  retryCount: number;
  cdnContentHash?: string;
  cdnTicket?: string;
  workspaceRelPath?: string;
  roomId: string;
  localPath?: string;
  direction?: string;
  bytesDone?: number;
  progressPercent?: number;
  speedBps?: number;
  checksumVerified?: boolean;
  lastError?: string;
};

export type TransferEngineSettings = {
  throttleBytesPerSec: number;
  autoReconnect: boolean;
  backgroundJobs: boolean;
  workspaceWatchEnabled: boolean;
};

export type TransferDashboard = {
  transfers: FileTransferMetadata[];
  settings: TransferEngineSettings;
  relayEnabled: boolean;
  workspaceWatchActive: boolean;
  activeBackgroundJobs: number;
};

export type ChecksumReport = {
  algorithm: string;
  fileHash: string;
  fileSize: number;
  chunkCount: number;
  chunks: { index: number; hash: string; sizeBytes: number }[];
  destinationVerified: boolean;
  verifiedAt: number;
  mismatchChunks: number[];
};

export type TransferProgressEvent = {
  transferId: string;
  status: string;
  progressPercent: number;
  bytesDone: number;
  bytesTotal: number;
  speedBps: number;
  direction: string;
  checksumVerified: boolean;
  lastError?: string;
};

/** Start stack and return workspace info. */
export async function fileTransferServiceStart(): Promise<ExodusWorkspaceInfo> {
  return invoke<ExodusWorkspaceInfo>('file_transfer_service_start');
}

export async function exodusWorkspaceInfo(): Promise<ExodusWorkspaceInfo> {
  return invoke<ExodusWorkspaceInfo>('exodus_workspace_info');
}

export async function exodusWorkspaceList(): Promise<WorkspaceFileEntry[]> {
  return invoke<WorkspaceFileEntry[]>('exodus_workspace_list');
}

export async function fileTransferInitiate(
  filePath: string,
  receiverId?: string
): Promise<FileTransferMetadata> {
  return invoke<FileTransferMetadata>('file_transfer_initiate', {
    filePath,
    receiverId: receiverId ?? null,
  });
}

/** Native system file picker; returns absolute path or null. */
export async function fileTransferPickFile(): Promise<string | null> {
  return invoke<string | null>('file_transfer_pick_file');
}

export async function fileTransferList(): Promise<FileTransferMetadata[]> {
  return invoke<FileTransferMetadata[]>('file_transfer_list');
}

export async function fileTransferDashboard(): Promise<TransferDashboard> {
  return invoke<TransferDashboard>('file_transfer_dashboard');
}

export async function fileTransferSetThrottle(bytesPerSec: number): Promise<void> {
  return invoke('file_transfer_set_throttle', { bytesPerSec });
}

export async function fileTransferSetAutoReconnect(enabled: boolean): Promise<void> {
  return invoke('file_transfer_set_auto_reconnect', { enabled });
}

export async function fileTransferSetRelayConfig(
  enabled: boolean,
  relayBaseUrl?: string
): Promise<void> {
  return invoke('file_transfer_set_relay_config', {
    enabled,
    relayBaseUrl: relayBaseUrl ?? null,
  });
}

export type WanRelayServerInfo = {
  running: boolean;
  port: number;
  baseUrl: string;
  bindHost: string;
};

export async function wanRelayServerInfo(): Promise<WanRelayServerInfo> {
  return invoke<WanRelayServerInfo>('wan_relay_server_info');
}

export async function fileTransferSetRelayServe(
  serveEnabled: boolean,
  servePort?: number,
  serveBind?: string
): Promise<WanRelayServerInfo> {
  return invoke<WanRelayServerInfo>('file_transfer_set_relay_serve', {
    serveEnabled,
    servePort: servePort ?? null,
    serveBind: serveBind ?? null,
  });
}

/** Background download with resume + checksum (runs while Tauri app is open). */
export async function fileTransferStartBackgroundDownload(
  contentHash: string,
  fileName: string
): Promise<FileTransferMetadata> {
  return invoke<FileTransferMetadata>('file_transfer_start_background_download', {
    contentHash,
    fileName,
  });
}

export async function fileTransferVerifyChecksum(
  transferId: string
): Promise<ChecksumReport> {
  return invoke<ChecksumReport>('file_transfer_verify_checksum', { transferId });
}

export async function exodusWorkspaceWatchStart(): Promise<void> {
  return invoke('exodus_workspace_watch_start');
}

export async function exodusWorkspaceWatchStop(): Promise<void> {
  return invoke('exodus_workspace_watch_stop');
}

export async function fileTransferReceiveToInbox(
  contentHash: string,
  fileName: string
): Promise<string> {
  return invoke<string>('file_transfer_receive_to_inbox', { contentHash, fileName });
}
