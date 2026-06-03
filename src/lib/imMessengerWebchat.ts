/**
 * Exodus Browser — WebChat IM UI helpers (grid avatars, timeline dividers, mute).
 */
import type { GroupMessage } from '$lib/groupChat';

export const IM_MUTED_CHATS_KEY = 'exodus-im-muted-chats';

/** True when group id is a multi-user group (not a 1:1 DM room). */
export function isRealGroupChat(groupId: string): boolean {
  return !groupId.startsWith('dm-');
}

/** Stable conversation id for mute map. */
export function conversationIdForContact(nodeId: string): string {
  return `contact:${nodeId}`;
}

export function conversationIdForGroup(groupId: string): string {
  return `group:${groupId}`;
}

/** Load muted conversation ids from localStorage. */
export function loadMutedChatIds(): Set<string> {
  try {
    const raw = localStorage.getItem(IM_MUTED_CHATS_KEY);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set();
    return new Set(parsed.filter((id): id is string => typeof id === 'string'));
  } catch {
    return new Set();
  }
}

/** Persist muted conversation ids. */
export function saveMutedChatIds(ids: Set<string>): void {
  try {
    localStorage.setItem(IM_MUTED_CHATS_KEY, JSON.stringify([...ids]));
  } catch {
    /* storage unavailable */
  }
}

/** Pick up to 9 member node ids for a WebChat group avatar grid. */
export function pickGroupGridMemberIds(memberIds: string[], limit = 9): string[] {
  return memberIds.filter(Boolean).slice(0, limit);
}

/** WebChat centered divider label (absolute time, not relative). */
export function formatWebChatDividerTime(timestamp: number, now = new Date()): string {
  const date = new Date(timestamp);
  const timeStr = date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });

  if (date.toDateString() === now.toDateString()) {
    return timeStr;
  }

  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (date.toDateString() === yesterday.toDateString()) {
    return `昨天 ${timeStr}`;
  }

  const weekAgo = new Date(now);
  weekAgo.setDate(weekAgo.getDate() - 7);
  if (date > weekAgo) {
    const days = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    return `${days[date.getDay()]} ${timeStr}`;
  }

  if (date.getFullYear() === now.getFullYear()) {
    return `${date.getMonth() + 1}月${date.getDate()}日 ${timeStr}`;
  }

  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日 ${timeStr}`;
}

export type MessageTimelineItem =
  | { kind: 'divider'; key: string; label: string }
  | { kind: 'message'; key: string; message: GroupMessage };

/**
 * Insert centered time dividers between messages (WebChat).
 * @param gapMs minimum gap before showing a new divider (default 5 min)
 */
export function buildMessageTimelineItems(
  messages: GroupMessage[],
  gapMs = 5 * 60 * 1000,
  now = new Date(),
): MessageTimelineItem[] {
  const items: MessageTimelineItem[] = [];
  let prevTimestamp: number | undefined;

  for (const message of messages) {
    const ts = message.timestamp || 0;
    if (prevTimestamp === undefined || ts - prevTimestamp >= gapMs) {
      items.push({
        kind: 'divider',
        key: `divider-${message.messageId}`,
        label: formatWebChatDividerTime(ts, now),
      });
    }
    items.push({ kind: 'message', key: message.messageId, message });
    prevTimestamp = ts;
  }

  return items;
}

/** CSS grid class suffix for 1–9 avatar cells. */
export function groupGridCountClass(count: number): string {
  const n = Math.max(1, Math.min(count, 9));
  return `grid-count-${n}`;
}

export type ImNavId =
  | 'chats'
  | 'collections'
  | 'favorites'
  | 'contacts'
  | 'groups'
  | 'public_accounts'
  | 'timeline'
  | 'settings';

export type ImNavItemDef = {
  id: ImNavId;
  icon: 'chat' | 'collections' | 'starred' | 'contacts' | 'groups' | 'public' | 'timeline';
  titleEn: string;
  titleZh: string;
};

const ALL_NAV_ITEMS: ImNavItemDef[] = [
  { id: 'chats', icon: 'chat', titleEn: 'Chats', titleZh: '聊天' },
  { id: 'collections', icon: 'collections', titleEn: 'Collections', titleZh: '收藏' },
  { id: 'favorites', icon: 'starred', titleEn: 'Starred', titleZh: '星标' },
  { id: 'contacts', icon: 'contacts', titleEn: 'Contacts', titleZh: '通讯录' },
  { id: 'groups', icon: 'groups', titleEn: 'Groups', titleZh: '群聊' },
  { id: 'public_accounts', icon: 'public', titleEn: 'Public Accounts', titleZh: '公众号' },
  { id: 'timeline', icon: 'timeline', titleEn: 'Timeline', titleZh: '朋友圈' },
];

/** Primary nav order: WebChat desktop vs default sidebar. */
export function primaryNavItems(webchatDesktop: boolean): ImNavItemDef[] {
  if (webchatDesktop) {
    const order: ImNavId[] = [
      'chats',
      'contacts',
      'collections',
      'timeline',
      'favorites',
      'groups',
      'public_accounts',
    ];
    return order
      .map((id) => ALL_NAV_ITEMS.find((item) => item.id === id))
      .filter((item): item is ImNavItemDef => item != null);
  }
  return ALL_NAV_ITEMS.filter((item) => item.id !== 'settings');
}

export function navItemTitle(item: ImNavItemDef, webchatDesktop: boolean): string {
  return webchatDesktop ? item.titleZh : item.titleEn;
}

export function settingsNavTitle(webchatDesktop: boolean): string {
  return webchatDesktop ? '更多' : 'Settings';
}

export const IM_CONTACT_DIR_EXPANDED_KEY = 'exodus-im-contact-dir-expanded';

export type ContactDirectoryCategoryId =
  | 'new_friends'
  | 'group_chats'
  | 'official_accounts'
  | 'service_accounts'
  | 'wecom_contacts'
  | 'my_enterprises'
  | 'contacts';

export type ContactDirectoryCategoryDef = {
  id: ContactDirectoryCategoryId;
  titleZh: string;
  titleEn: string;
  showCount: boolean;
  expandable: boolean;
};

/** WebChat macOS-style collapsible address book categories. */
export const CONTACT_DIRECTORY_CATEGORIES: ContactDirectoryCategoryDef[] = [
  { id: 'new_friends', titleZh: '新的朋友', titleEn: 'New Friends', showCount: false, expandable: false },
  { id: 'group_chats', titleZh: '群聊', titleEn: 'Group Chats', showCount: true, expandable: true },
  { id: 'official_accounts', titleZh: '公众号', titleEn: 'Official Accounts', showCount: true, expandable: true },
  { id: 'service_accounts', titleZh: '服务号', titleEn: 'Service Accounts', showCount: true, expandable: true },
  { id: 'wecom_contacts', titleZh: '企业微信联系人', titleEn: 'WeCom Contacts', showCount: true, expandable: true },
  { id: 'my_enterprises', titleZh: '我的企业', titleEn: 'My Enterprises', showCount: true, expandable: true },
  { id: 'contacts', titleZh: '联系人', titleEn: 'Contacts', showCount: true, expandable: true },
];

export type ContactDirectoryCounts = {
  groupChats: number;
  officialAccounts: number;
  serviceAccounts: number;
  wecomContacts: number;
  myEnterprises: number;
  contacts: number;
};

export function contactDirectoryCategoryLabel(
  category: ContactDirectoryCategoryDef,
  webchatDesktop: boolean,
): string {
  return webchatDesktop ? category.titleZh : category.titleEn;
}

export function contactDirectoryCountForCategory(
  id: ContactDirectoryCategoryId,
  counts: ContactDirectoryCounts,
): number | null {
  const category = CONTACT_DIRECTORY_CATEGORIES.find((item) => item.id === id);
  if (!category?.showCount) return null;
  switch (id) {
    case 'group_chats':
      return counts.groupChats;
    case 'official_accounts':
      return counts.officialAccounts;
    case 'service_accounts':
      return counts.serviceAccounts;
    case 'wecom_contacts':
      return counts.wecomContacts;
    case 'my_enterprises':
      return counts.myEnterprises;
    case 'contacts':
      return counts.contacts;
    default:
      return null;
  }
}

/** Format count for right-aligned column (WebChat uses plain digits). */
export function formatContactDirectoryCount(count: number): string {
  return count.toLocaleString('zh-CN');
}

export function loadContactDirectoryExpanded(): Set<ContactDirectoryCategoryId> {
  try {
    const raw = localStorage.getItem(IM_CONTACT_DIR_EXPANDED_KEY);
    if (!raw) return new Set(['contacts']);
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set(['contacts']);
    const validIds = new Set(CONTACT_DIRECTORY_CATEGORIES.map((item) => item.id));
    const ids = parsed.filter(
      (id): id is ContactDirectoryCategoryId =>
        typeof id === 'string' && validIds.has(id as ContactDirectoryCategoryId),
    );
    return ids.length > 0 ? new Set(ids) : new Set(['contacts']);
  } catch {
    return new Set(['contacts']);
  }
}

export function saveContactDirectoryExpanded(ids: Set<ContactDirectoryCategoryId>): void {
  try {
    localStorage.setItem(IM_CONTACT_DIR_EXPANDED_KEY, JSON.stringify([...ids]));
  } catch {
    /* storage unavailable */
  }
}

export function toggleContactDirectoryExpanded(
  ids: Set<ContactDirectoryCategoryId>,
  id: ContactDirectoryCategoryId,
): Set<ContactDirectoryCategoryId> {
  const next = new Set(ids);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

/** Resolve quoted reply source from the loaded message list. */
export function findReplySourceMessage(
  messages: GroupMessage[],
  replyToId: string | null | undefined,
): GroupMessage | null {
  if (!replyToId) return null;
  return messages.find((message) => message.messageId === replyToId) ?? null;
}

/** Short preview line for a quoted reply bubble. */
export function replyQuotePreview(source: GroupMessage | null, fallback = '消息'): string {
  if (!source) return fallback;
  const text = source.content?.trim();
  if (text) return text.length > 80 ? `${text.slice(0, 80)}…` : text;
  if (source.attachments?.length) return '[文件]';
  return fallback;
}
