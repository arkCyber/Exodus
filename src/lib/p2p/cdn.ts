/**
 * Exodus Browser — AI-driven P2P CDN (iroh-blobs content addressing).
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type CdnContentKind =
  | 'article'
  | 'ai_model'
  | 'video_model'
  | 'dataset'
  | 'generic_file';

export type CdnAsset = {
  contentHash: string;
  title: string;
  kind: CdnContentKind;
  sizeBytes: number;
  mimeType?: string;
  sourceUrl?: string;
  roomId: string;
  announcerNodeId: string;
  announcedAt: number;
};

export type CdnPeerSource = {
  nodeId: string;
  contentHash: string;
  ticket?: string;
  lastSeen: number;
  rttMs?: number;
};

export type CdnRoomFeed = {
  roomId: string;
  assets: CdnAsset[];
  peerMap: Record<string, CdnPeerSource[]>;
};

export type CdnDownloadJob = {
  jobId: string;
  contentHash: string;
  title: string;
  status: string;
  progressPercent: number;
  bytesDone: number;
  bytesTotal: number;
  source: string;
  peerCount: number;
  localPath?: string;
  error?: string;
};

export type CdnDownloadProgress = {
  jobId: string;
  contentHash: string;
  progressPercent: number;
  bytesDone: number;
  bytesTotal: number;
  source: string;
  peerCount: number;
};

export type CdnNodeInfo = {
  nodeId: string;
  joinedRooms: string[];
  meshHost?: string | null;
  meshPort?: number | null;
};

/** P2P swarm status for a URL in a room (discovery hash = BLAKE3(url)). */
export type CdnUrlStatus = {
  url: string;
  discoveryHash: string;
  announced: boolean;
  peerCount: number;
  localComplete: boolean;
  title?: string | null;
};

/** Query CDN room for URL announce / peer availability (address bar). */
export async function p2pCdnUrlStatus(roomId: string, url: string): Promise<CdnUrlStatus> {
  return invoke<CdnUrlStatus>('p2p_cdn_url_status', { roomId, url });
}

/** Start HTTP mesh server (peer seeding). */
export async function p2pCdnStartMesh(): Promise<CdnNodeInfo> {
  return invoke<CdnNodeInfo>('p2p_cdn_start_mesh');
}

/** Join a group chat room or global lobby for P2P CDN gossip. */
export async function p2pCdnJoinRoom(roomId: string): Promise<void> {
  await invoke('p2p_cdn_join_room', { roomId });
}

/** Leave a CDN room topic. */
export async function p2pCdnLeaveRoom(roomId: string): Promise<void> {
  await invoke('p2p_cdn_leave_room', { roomId });
}

/** Local node id and joined rooms. */
export async function p2pCdnNodeInfo(): Promise<CdnNodeInfo> {
  return invoke<CdnNodeInfo>('p2p_cdn_node_info');
}

/**
 * Group chat / lobby: announce hot content (optionally seed from local file).
 */
export async function p2pCdnAnnounceGroupHot(params: {
  groupId: string;
  title: string;
  contentHash: string;
  kind: CdnContentKind;
  sizeBytes: number;
  sourceUrl?: string;
  localPath?: string;
}): Promise<void> {
  await invoke('p2p_cdn_announce_group_hot', {
    groupId: params.groupId,
    title: params.title,
    contentHash: params.contentHash,
    kind: params.kind,
    sizeBytes: params.sizeBytes,
    sourceUrl: params.sourceUrl ?? null,
    localPath: params.localPath ?? null,
  });
}

/** Trending / announced assets in a room with peer availability. */
export async function p2pCdnRoomFeed(roomId: string): Promise<CdnRoomFeed> {
  return invoke<CdnRoomFeed>('p2p_cdn_room_feed', { roomId });
}

/** Pull gossip from in-process bus and optional `p2p_gossip` microservice. */
export async function p2pCdnSyncGossip(roomId: string): Promise<number> {
  return invoke<number>('p2p_cdn_sync_gossip', { roomId });
}

/** Announce a hot URL (discovery hash = BLAKE3(url); file hash verified on download). */
export async function p2pCdnAnnounceUrlHot(params: {
  roomId: string;
  title: string;
  url: string;
  kind: CdnContentKind;
  sizeBytes?: number;
}): Promise<void> {
  await invoke('p2p_cdn_announce_url_hot', {
    roomId: params.roomId,
    title: params.title,
    url: params.url,
    kind: params.kind,
    sizeBytes: params.sizeBytes ?? null,
  });
}

/** Group message + P2P CDN attachment announce (payload uses Rust snake_case fields). */
export async function p2pCdnGroupSendMessage(message: Record<string, unknown>): Promise<string> {
  return invoke<string>('p2p_cdn_group_send_message', { message });
}

/** BLAKE3 hash + byte size for a local file path. */
export async function p2pCdnHashFile(
  localPath: string,
): Promise<{ contentHash: string; sizeBytes: number }> {
  return invoke('p2p_cdn_hash_file', { localPath });
}

/** Peers that can serve a content hash. */
export async function p2pCdnListPeers(contentHash: string): Promise<CdnPeerSource[]> {
  return invoke<CdnPeerSource[]>('p2p_cdn_list_peers', { contentHash });
}

/**
 * AI recommendation → gossip announce (e.g. hot article / 5GB model in lobby).
 */
export async function p2pCdnAnnounceFromAi(
  roomId: string,
  payload: {
    contentHash: string;
    title: string;
    kind: CdnContentKind;
    sizeBytes: number;
    sourceUrl?: string;
    mimeType?: string;
  },
): Promise<void> {
  await invoke('p2p_cdn_announce_asset', { roomId, payload });
}

/** After HTTP download, seed the swarm from a local file path. */
export async function p2pCdnRegisterLocalSeed(
  roomId: string,
  localPath: string,
  title: string,
  kind: CdnContentKind,
  sourceUrl?: string,
): Promise<void> {
  await invoke('p2p_cdn_register_local_seed', {
    roomId,
    localPath,
    title,
    kind,
    sourceUrl: sourceUrl ?? null,
  });
}

/**
 * Smart download: local cache → parallel peers → HTTP fallback, then re-seed.
 */
export async function p2pCdnDownload(params: {
  roomId: string;
  contentHash: string;
  title: string;
  kind: CdnContentKind;
  httpUrl?: string;
}): Promise<CdnDownloadJob> {
  return invoke<CdnDownloadJob>('p2p_cdn_download', {
    roomId: params.roomId,
    contentHash: params.contentHash,
    title: params.title,
    kind: params.kind,
    httpUrl: params.httpUrl ?? null,
  });
}

/** Listen for CDN download progress (P2P vs HTTP source label). */
export function listenP2pCdnProgress(
  onProgress: (p: CdnDownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<CdnDownloadProgress>('exodus-p2p-cdn-progress', (e) => {
    onProgress(e.payload);
  });
}

/** Human-readable source label for UI. */
export function cdnSourceLabel(source: string, peerCount: number): string {
  switch (source) {
    case 'local_cache':
      return 'Local cache';
    case 'p2p_peers':
      return `P2P · ${peerCount} peer(s)`;
    case 'http_origin':
      return 'Origin HTTP';
    case 'http_then_seed':
      return 'HTTP → now seeding swarm';
    default:
      return source;
  }
}
