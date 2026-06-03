/**
 * Exodus Browser — WebChat Collections (WebChat 收藏).
 * Saved message snapshots via Tauri chat_collection_* commands.
 */
import { invoke } from '@tauri-apps/api/core';
import type { GroupMessage } from '$lib/groupChat';

export type SavedChatItem = {
  id: string;
  user_id: string;
  source_message_id: string;
  conversation_id: string;
  conversation_type: 'dm' | 'group' | string;
  conversation_title: string;
  sender_id: string;
  sender_name: string;
  content_type: 'text' | 'link' | 'image' | 'file' | 'mixed' | string;
  content: string;
  message_type: string;
  attachments: GroupMessage['attachments'];
  original_timestamp: number;
  saved_at: number;
};

export type SaveChatItemRequest = {
  userId: string;
  sourceMessageId: string;
  conversationId: string;
  conversationType: 'dm' | 'group';
  conversationTitle: string;
  senderId: string;
  senderName: string;
  content: string;
  messageType: string;
  attachments: GroupMessage['attachments'];
  originalTimestamp: number;
};

const URL_PATTERN = /https?:\/\/[^\s]+/i;

/** Derive display content type (mirrors Rust derive_content_type). */
export function deriveCollectionContentType(
  content: string,
  messageType: string,
  attachments: GroupMessage['attachments'],
): SavedChatItem['content_type'] {
  if (attachments.length > 0) {
    if (attachments.some((a) => a.fileType.startsWith('image'))) return 'image';
    if (attachments.length > 1) return 'mixed';
    return 'file';
  }
  if (messageType === 'image') return 'image';
  if (messageType === 'file') return 'file';
  if (URL_PATTERN.test(content)) return 'link';
  return 'text';
}

/** Preview label for a saved item row. */
export function collectionItemPreview(item: SavedChatItem): string {
  if (item.content.trim()) return item.content.trim();
  if (item.content_type === 'image') return '[Image]';
  if (item.content_type === 'file') return `[File] ${item.attachments[0]?.fileName ?? ''}`.trim();
  if (item.content_type === 'mixed') return '[Attachments]';
  return '[Message]';
}

/** Save a chat message snapshot to Collections. */
export async function chatCollectionSave(request: SaveChatItemRequest): Promise<SavedChatItem> {
  return invoke<SavedChatItem>('chat_collection_save', {
    request: {
      user_id: request.userId,
      source_message_id: request.sourceMessageId,
      conversation_id: request.conversationId,
      conversation_type: request.conversationType,
      conversation_title: request.conversationTitle,
      sender_id: request.senderId,
      sender_name: request.senderName,
      content: request.content,
      message_type: request.messageType,
      attachments: request.attachments.map((a) => ({
        attachment_id: a.attachmentId,
        file_type: a.fileType,
        blob_hash: a.blobHash,
        file_name: a.fileName,
        file_size: a.fileSize,
        thumbnail_hash: a.thumbnailHash ?? null,
      })),
      original_timestamp: request.originalTimestamp,
    },
  });
}

/** List all saved items for the current user. */
export async function chatCollectionList(userId: string): Promise<SavedChatItem[]> {
  return invoke<SavedChatItem[]>('chat_collection_list', { userId });
}

/** Search saved items. */
export async function chatCollectionSearch(userId: string, query: string): Promise<SavedChatItem[]> {
  return invoke<SavedChatItem[]>('chat_collection_search', { userId, query });
}

/** Delete a saved item. */
export async function chatCollectionDelete(id: string, userId: string): Promise<boolean> {
  return invoke<boolean>('chat_collection_delete', { id, userId });
}

/** Whether a source message is already in Collections. */
export async function chatCollectionIsSaved(userId: string, sourceMessageId: string): Promise<boolean> {
  return invoke<boolean>('chat_collection_is_saved', { userId, sourceMessageId });
}

/** Build save payload from an in-chat message. */
export function buildSaveChatItemRequest(params: {
  userId: string;
  message: GroupMessage;
  conversationType: 'dm' | 'group';
  conversationTitle: string;
}): SaveChatItemRequest {
  return {
    userId: params.userId,
    sourceMessageId: params.message.messageId,
    conversationId: params.message.groupId,
    conversationType: params.conversationType,
    conversationTitle: params.conversationTitle,
    senderId: params.message.senderId,
    senderName: params.message.senderName,
    content: params.message.content,
    messageType: params.message.messageType,
    attachments: params.message.attachments,
    originalTimestamp: params.message.timestamp,
  };
}
