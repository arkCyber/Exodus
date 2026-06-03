/**
 * Exodus Browser — Group Chat microservice API (Unix socket JSON-RPC via Tauri).
 * Includes both group chat and 1-to-1 direct messaging.
 */

import { invoke } from '@tauri-apps/api/core';

export type GroupChat = {
  groupId: string;
  name: string;
  description: string;
  avatarUrl?: string | null;
  ownerId: string;
  memberIds: string[];
  adminIds: string[];
  isPrivate: boolean;
  createdAt: number;
  lastActivity: number;
  messageCount: number;
};

// 1-to-1 Direct Chat Types
export type DirectChat = {
  chatId: string;
  userA: string;
  userB: string;
  createdAt: number;
  lastActivity: number;
  messageCount: number;
};

export type DirectMessage = {
  messageId: string;
  chatId: string;
  senderId: string;
  receiverId: string;
  content: string;
  messageType: string;
  attachments: MessageAttachment[];
  replyTo?: string | null;
  sequence: number;
  timestamp: number;
  integrityHash?: string | null;
  isEdited: boolean;
  editedAt?: number | null;
};

export type MessageAttachment = {
  attachmentId: string;
  fileType: string;
  blobHash: string;
  fileName: string;
  fileSize: number;
  thumbnailHash?: string | null;
};

export type MessageReceipt = {
  receiptId: string;
  messageId: string;
  receiverId: string;
  sequence: number;
  receivedAt: number;
};

export type UserSequence = {
  userId: string;
  senderId: string;
  lastSequence: number;
  updatedAt: number;
};

export type GroupMember = {
  agentId: string;
  agentName: string;
  role: string;
  joinedAt: number;
  lastSeen: number;
  isOnline: boolean;
  nickname?: string | null;
};

export type GroupMessage = {
  messageId: string;
  groupId: string;
  senderId: string;
  senderName: string;
  content: string;
  messageType: string;
  attachments: Array<{
    attachmentId: string;
    fileType: string;
    blobHash: string;
    fileName: string;
    fileSize: number;
    thumbnailHash?: string | null;
  }>;
  replyTo?: string | null;
  mentions: string[];
  timestamp: number;
  isEdited: boolean;
  editedAt?: number | null;
};

/** Start the group chat microservice. */
export async function groupChatServiceStart(): Promise<void> {
  await invoke('group_chat_service_start');
}

/** Create a group (returns group id). */
export async function groupCreate(groupPayload: Record<string, unknown>): Promise<string> {
  return invoke<string>('group_create', { group: groupPayload });
}

/** List groups for a user id. */
export async function groupListUser(userId: string): Promise<GroupChat[]> {
  const raw = await invoke<Record<string, unknown>[]>('group_list_user', { userId });
  return raw.map(mapGroup);
}

/** Fetch one group by id. */
export async function groupGet(groupId: string): Promise<GroupChat | null> {
  try {
    const raw = await invoke<Record<string, unknown>>('group_get', { groupId });
    return mapGroup(raw);
  } catch {
    return null;
  }
}

/** Members of a group (for @mention roster). */
export async function groupGetMembers(groupId: string): Promise<GroupMember[]> {
  const raw = await invoke<Record<string, unknown>[]>('group_get_members', { groupId });
  return raw.map((m) => ({
    agentId: String(m.agent_id ?? m.agentId ?? ''),
    agentName: String(m.agent_name ?? m.agentName ?? ''),
    role: String(m.role ?? 'member'),
    joinedAt: Number(m.joined_at ?? m.joinedAt ?? 0),
    lastSeen: Number(m.last_seen ?? m.lastSeen ?? 0),
    isOnline: Boolean(m.is_online ?? m.isOnline),
    nickname: (m.nickname as string | null) ?? null,
  }));
}

/** Fetch messages for a group. */
export async function groupGetMessages(
  groupId: string,
  limit?: number,
): Promise<GroupMessage[]> {
  const raw = await invoke<Record<string, unknown>[]>('group_get_messages', {
    groupId,
    limit: limit ?? 50,
  });
  return raw.map(mapGroupMessage);
}

// Helper functions
function mapMessageAttachment(raw: Record<string, unknown>): MessageAttachment {
  return {
    attachmentId: String(raw.attachmentId ?? raw.attachment_id ?? ''),
    fileType: String(raw.fileType ?? raw.file_type ?? ''),
    blobHash: String(raw.blobHash ?? raw.blob_hash ?? ''),
    fileName: String(raw.fileName ?? raw.file_name ?? ''),
    fileSize: Number(raw.fileSize ?? raw.file_size ?? 0),
    thumbnailHash: (raw.thumbnailHash ?? raw.thumbnail_hash) as string | null | undefined,
  };
}

function mapGroupMessage(raw: Record<string, unknown>): GroupMessage {
  return {
    messageId: String(raw.messageId ?? raw.message_id ?? ''),
    groupId: String(raw.groupId ?? raw.group_id ?? ''),
    senderId: String(raw.senderId ?? raw.sender_id ?? ''),
    senderName: String(raw.senderName ?? raw.sender_name ?? ''),
    content: String(raw.content ?? ''),
    messageType: String(raw.messageType ?? raw.message_type ?? 'text'),
    attachments: Array.isArray(raw.attachments) ? raw.attachments.map(mapMessageAttachment) : [],
    replyTo: (raw.replyTo ?? raw.reply_to) as string | null ?? null,
    mentions: (raw.mentions ?? []) as string[],
    timestamp: Number(raw.timestamp ?? 0),
    isEdited: Boolean(raw.isEdited ?? raw.is_edited),
    editedAt: (raw.editedAt ?? raw.edited_at) as number | null ?? null,
  };
}

function mapGroup(g: Record<string, unknown>): GroupChat {
  return {
    groupId: String(g.group_id ?? g.groupId ?? ''),
    name: String(g.name ?? ''),
    description: String(g.description ?? ''),
    avatarUrl: (g.avatar_url ?? g.avatarUrl) as string | null | undefined,
    ownerId: String(g.owner_id ?? g.ownerId ?? ''),
    memberIds: (g.member_ids ?? g.memberIds ?? []) as string[],
    adminIds: (g.admin_ids ?? g.adminIds ?? []) as string[],
    isPrivate: Boolean(g.is_private ?? g.isPrivate),
    createdAt: Number(g.created_at ?? g.createdAt ?? 0),
    lastActivity: Number(g.last_activity ?? g.lastActivity ?? 0),
    messageCount: Number(g.message_count ?? g.messageCount ?? 0),
  };
}

function mapMessage(m: Record<string, unknown>): GroupMessage {
  const attachments = (m.attachments as Record<string, unknown>[] | undefined) ?? [];
  return {
    messageId: String(m.message_id ?? m.messageId ?? ''),
    groupId: String(m.group_id ?? m.groupId ?? ''),
    senderId: String(m.sender_id ?? m.senderId ?? ''),
    senderName: String(m.sender_name ?? m.senderName ?? ''),
    content: String(m.content ?? ''),
    messageType: String(m.message_type ?? m.messageType ?? 'text'),
    attachments: attachments.map((a) => ({
      attachmentId: String(a.attachment_id ?? a.attachmentId ?? ''),
      fileType: String(a.file_type ?? a.fileType ?? 'file'),
      blobHash: String(a.blob_hash ?? a.blobHash ?? ''),
      fileName: String(a.file_name ?? a.fileName ?? ''),
      fileSize: Number(a.file_size ?? a.fileSize ?? 0),
      thumbnailHash: (a.thumbnail_hash ?? a.thumbnailHash) as string | null | undefined,
    })),
    replyTo: (m.reply_to ?? m.replyTo) as string | null | undefined,
    mentions: (m.mentions as string[]) ?? [],
    timestamp: Number(m.timestamp ?? 0),
    isEdited: Boolean(m.is_edited ?? m.isEdited),
    editedAt: (m.edited_at ?? m.editedAt) as number | null | undefined,
  };
}

/** Build a snake_case group payload for `group_create`. */
export function buildGroupPayload(params: {
  groupId: string;
  name: string;
  description: string;
  ownerId: string;
  memberIds: string[];
}): Record<string, unknown> {
  const now = Date.now();
  return {
    group_id: params.groupId,
    name: params.name,
    description: params.description,
    avatar_url: null,
    owner_id: params.ownerId,
    member_ids: params.memberIds,
    admin_ids: [params.ownerId],
    is_private: false,
    created_at: now,
    last_activity: now,
    message_count: 0,
  };
}
