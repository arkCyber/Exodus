/**
 * Exodus Browser — IM direct-message rooms (1:1 chat tied to P2P node ids).
 */
import {
  buildGroupPayload,
  groupChatServiceStart,
  groupCreate,
  groupGetMessages,
  groupListUser,
  type GroupMessage,
} from '$lib/groupChat';
import { sendGroupMessageWithCdn, type GroupChatMessage } from '$lib/p2p/cdnIntegrations';
import { p2pCdnJoinRoom } from '$lib/p2p/cdn';

/** Stable DM room id for two node ids (both peers use the same room). */
export function dmRoomId(localNode: string, remoteNode: string): string {
  const ids = [localNode, remoteNode].sort();
  const raw = `dm-${ids[0]}-${ids[1]}`;
  return raw.length > 96 ? raw.slice(0, 96) : raw;
}

export type ImOpenContactDetail = {
  contactId: string;
  name: string;
  nodeId: string;
};

export const IM_OPEN_CONTACT_EVENT = 'exodus-open-im';
export const IM_START_CALL_EVENT = 'exodus-start-call';
export const P2P_TAB_EVENT = 'exodus-p2p-tab';
/** Tray / sidebar: focus WorkSpace file transfer tab. */
export const FOCUS_WORKSPACE_EVENT = 'exodus-focus-workspace';
/** Tray / sidebar: focus IM tab. */
export const FOCUS_IM_EVENT = 'exodus-focus-im';

export type P2pSidebarTab = 'im' | 'contacts' | 'chat' | 'cdn' | 'workspace' | 'call' | 'meeting';

/** Switch P2P sidebar tab (e.g. open IM before chat). */
export function openP2pTab(tab: P2pSidebarTab): void {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(new CustomEvent(P2P_TAB_EVENT, { detail: tab }));
}

export type ImStartCallDetail = {
  nodeId: string;
  name: string;
  video: boolean;
  audio: boolean;
};

/** Ask IM panel to open a chat with this contact. */
export function openImChat(contact: ImOpenContactDetail): void {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(new CustomEvent(IM_OPEN_CONTACT_EVENT, { detail: contact }));
}

/** Start voice/video call from any UI (IM will show overlay). */
export function startCallFromUi(detail: ImStartCallDetail): void {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(new CustomEvent(IM_START_CALL_EVENT, { detail }));
}

/** Ensure DM group exists in group-chat service. */
export async function ensureDmGroup(
  roomId: string,
  localUserId: string,
  localName: string,
  remoteNode: string,
  remoteName: string
): Promise<void> {
  await groupChatServiceStart();
  const groups = await groupListUser(localUserId);
  if (groups.some((g) => g.groupId === roomId)) return;
  await groupCreate(
    buildGroupPayload({
      groupId: roomId,
      name: `Chat · ${remoteName}`,
      description: `DM ${localName} ↔ ${remoteName}`,
      ownerId: localUserId,
      memberIds: [localUserId, remoteNode],
    })
  );
}

export async function loadDmMessages(roomId: string, limit = 80): Promise<GroupMessage[]> {
  await p2pCdnJoinRoom(roomId);
  return groupGetMessages(roomId, limit);
}

export async function sendDmText(
  roomId: string,
  localUserId: string,
  localName: string,
  text: string,
  mentions: string[] = []
): Promise<void> {
  const msg: GroupChatMessage = {
    messageId: `msg-${Date.now()}`,
    groupId: roomId,
    senderId: localUserId,
    senderName: localName,
    content: text,
    messageType: 'text',
    attachments: [],
    mentions,
    timestamp: Date.now(),
    isEdited: false,
  };
  await sendGroupMessageWithCdn(msg);
}
