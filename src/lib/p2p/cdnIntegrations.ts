/**
 * Exodus Browser — P2P CDN integrations (RAG indexing, group attachments).
 */

import {
  p2pCdnAnnounceFromAi,
  p2pCdnAnnounceGroupHot,
  p2pCdnAnnounceUrlHot,
  p2pCdnGroupSendMessage,
  p2pCdnHashFile,
  p2pCdnJoinRoom,
  type CdnContentKind,
} from './cdn';

/** Minimum indexed text length before auto-announcing to the P2P CDN lobby. */
export const CDN_AUTO_ANNOUNCE_MIN_BYTES = 500_000;

/**
 * Infer CDN content kind from URL or file name extension.
 */
export function cdnKindFromName(nameOrUrl: string): CdnContentKind {
  const lower = nameOrUrl.toLowerCase();
  if (/\.(gguf|safetensors|onnx|pt|pth|bin|model)(?:\?|$)/.test(lower)) return 'ai_model';
  if (/\.(mp4|webm|mov|mkv)(?:\?|$)/.test(lower)) return 'video_model';
  if (/\.(csv|jsonl|parquet|arrow)(?:\?|$)/.test(lower)) return 'dataset';
  if (/\.(html?|md|pdf)(?:\?|$)/.test(lower) || lower.includes('article')) return 'article';
  return 'generic_file';
}

/**
 * After RAG capture: announce large pages to the lobby for P2P discovery.
 */
export async function maybeAnnounceIndexedPage(
  url: string,
  title: string,
  contentByteLength: number,
  roomId = 'lobby',
): Promise<boolean> {
  if (contentByteLength < CDN_AUTO_ANNOUNCE_MIN_BYTES) return false;
  if (!url.startsWith('http://') && !url.startsWith('https://')) return false;
  try {
    await p2pCdnJoinRoom(roomId);
    await p2pCdnAnnounceUrlHot({
      roomId,
      title,
      url,
      kind: cdnKindFromName(url),
      sizeBytes: contentByteLength,
    });
    return true;
  } catch (error) {
    console.error('maybeAnnounceIndexedPage failed:', error);
    return false;
  }
}

/** File extensions / patterns that suggest a large downloadable asset. */
const LARGE_ASSET_URL =
  /\.(gguf|safetensors|onnx|pt|pth|bin|model|mp4|webm|mov|mkv|zip|tar|gz|7z|parquet|arrow)(?:\?|$)/i;

/** Whether a URL likely points at a large model / media / archive file. */
export function isLikelyLargeAssetUrl(url: string): boolean {
  return LARGE_ASSET_URL.test(url);
}

/** Extract http(s) URLs from AI or user text for CDN announce chips. */
export function extractAnnounceableUrls(text: string, limit = 5): string[] {
  const re = /https?:\/\/[^\s<>"')\]]+/gi;
  const seen = new Set<string>();
  const out: string[] = [];
  for (const match of text.matchAll(re)) {
    const url = match[0].replace(/[.,;:!?]+$/, '');
    if (!seen.has(url)) {
      seen.add(url);
      out.push(url);
      if (out.length >= limit) break;
    }
  }
  return out;
}

/** URLs in AI text that look like hot CDN candidates (models, video, archives). */
export function suggestUrlsForCdnAnnounce(text: string, limit = 3): string[] {
  return extractAnnounceableUrls(text, limit * 2)
    .filter(isLikelyLargeAssetUrl)
    .slice(0, limit);
}

/**
 * Announce the current browser page (or any URL) to a P2P CDN room.
 */
export async function announcePageUrlToCdn(
  url: string,
  title: string,
  roomId = 'lobby',
  sizeBytes = 0,
): Promise<void> {
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    throw new Error('Only http(s) URLs can be announced');
  }
  await p2pCdnJoinRoom(roomId);
  await p2pCdnAnnounceUrlHot({
    roomId,
    title: title.trim() || url,
    url,
    kind: cdnKindFromName(url),
    sizeBytes: sizeBytes > 0 ? sizeBytes : undefined,
  });
}

export async function announceAiRecommendation(
  roomId: string,
  title: string,
  contentHash: string,
  kind: CdnContentKind,
  sizeBytes: number,
  sourceUrl?: string,
): Promise<void> {
  await p2pCdnAnnounceFromAi(roomId, {
    contentHash,
    title,
    kind,
    sizeBytes,
    sourceUrl,
  });
}

export type GroupMessageAttachment = {
  attachmentId: string;
  fileType: string;
  blobHash: string;
  fileName: string;
  fileSize: number;
  thumbnailHash?: string | null;
};

export type GroupChatMessage = {
  messageId: string;
  groupId: string;
  senderId: string;
  senderName: string;
  content: string;
  messageType: string;
  attachments: GroupMessageAttachment[];
  replyTo?: string | null;
  mentions: string[];
  timestamp: number;
  isEdited: boolean;
  editedAt?: number | null;
};

/**
 * Send a group message and announce file attachments to the group's CDN room.
 */
export async function sendGroupMessageWithCdn(message: GroupChatMessage): Promise<string> {
  return p2pCdnGroupSendMessage({
    message_id: message.messageId,
    group_id: message.groupId,
    sender_id: message.senderId,
    sender_name: message.senderName,
    content: message.content,
    message_type: message.messageType,
    attachments: message.attachments.map((a) => ({
      attachment_id: a.attachmentId,
      file_type: a.fileType,
      blob_hash: a.blobHash,
      file_name: a.fileName,
      file_size: a.fileSize,
      thumbnail_hash: a.thumbnailHash ?? null,
    })),
    reply_to: message.replyTo ?? null,
    mentions: message.mentions,
    timestamp: message.timestamp,
    is_edited: message.isEdited,
    edited_at: message.editedAt ?? null,
  });
}

/**
 * Hash + seed a local file, then return attachment metadata for group_send.
 */
export async function prepareGroupFileAttachment(
  groupId: string,
  localPath: string,
  fileName?: string,
): Promise<GroupMessageAttachment> {
  const { contentHash, sizeBytes } = await p2pCdnHashFile(localPath);
  const name = fileName ?? localPath.split(/[/\\]/).pop() ?? 'file';
  const fileType = cdnKindFromName(name) === 'ai_model' ? 'model' : 'file';
  await p2pCdnAnnounceGroupHot({
    groupId,
    title: name,
    contentHash,
    kind: cdnKindFromName(name),
    sizeBytes,
    localPath,
  });
  return {
    attachmentId: `att-${Date.now()}`,
    fileType,
    blobHash: contentHash,
    fileName: name,
    fileSize: sizeBytes,
    thumbnailHash: null,
  };
}
