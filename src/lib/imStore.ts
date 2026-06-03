/**
 * Exodus Browser — Shared IM / WebChat state for full-width and sidebar instances.
 * Keeps active conversation, messages, drafts, and preview cache in sync.
 */

import { reactive } from 'vue';
import type { GroupMessage } from '$lib/groupChat';
import type { MessageCacheEntry } from '$lib/imChat';
import type { ImNavId } from '$lib/imMessengerWebchat';

/** Conversation id: group id or DM room id. */
export type ImConversationKey = string;

export type ImSharedState = {
  activeNav: ImNavId;
  activeContactNodeId: string | null;
  activeGroupId: string | null;
  messagesByKey: Record<ImConversationKey, GroupMessage[]>;
  draftsByKey: Record<ImConversationKey, string>;
  messageCache: Record<string, MessageCacheEntry>;
  /** Incremented when any shared field changes (cross-instance sync). */
  revision: number;
};

export const imStore = reactive<ImSharedState>({
  activeNav: 'chats',
  activeContactNodeId: null,
  activeGroupId: null,
  messagesByKey: {},
  draftsByKey: {},
  messageCache: {},
  revision: 0,
});

/** Notify all ImMessenger instances that shared state changed. */
export function bumpImStore(): void {
  imStore.revision += 1;
}

export function setActiveContactNode(nodeId: string | null): void {
  imStore.activeContactNodeId = nodeId;
  if (nodeId) imStore.activeGroupId = null;
  bumpImStore();
}

export function setActiveGroupId(groupId: string | null): void {
  imStore.activeGroupId = groupId;
  if (groupId) imStore.activeContactNodeId = null;
  bumpImStore();
}

export function setActiveNav(nav: ImNavId): void {
  imStore.activeNav = nav;
  bumpImStore();
}

export function getStoreMessages(key: ImConversationKey): GroupMessage[] {
  return imStore.messagesByKey[key] ?? [];
}

export function setStoreMessages(key: ImConversationKey, messages: GroupMessage[]): void {
  imStore.messagesByKey[key] = messages;
  bumpImStore();
}

export function getStoreDraft(key: ImConversationKey): string {
  return imStore.draftsByKey[key] ?? '';
}

export function setStoreDraft(key: ImConversationKey, draft: string): void {
  imStore.draftsByKey[key] = draft;
  bumpImStore();
}

export function getMessageCacheEntry(nodeId: string): MessageCacheEntry {
  return (
    imStore.messageCache[nodeId] ?? {
      lastMessage: '',
      lastTime: 0,
      unread: 0,
      lastSeenTime: 0,
    }
  );
}

export function setMessageCacheEntry(nodeId: string, entry: MessageCacheEntry): void {
  imStore.messageCache[nodeId] = entry;
  bumpImStore();
}

export function activeConversationKey(): ImConversationKey | null {
  if (imStore.activeGroupId) return imStore.activeGroupId;
  return null;
}

/** Reset shared store between unit tests. */
export function resetImStoreForTests(): void {
  imStore.activeNav = 'chats';
  imStore.activeContactNodeId = null;
  imStore.activeGroupId = null;
  imStore.messagesByKey = {};
  imStore.draftsByKey = {};
  imStore.messageCache = {};
  imStore.revision = 0;
}
