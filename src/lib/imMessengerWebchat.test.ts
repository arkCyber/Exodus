/**
 * Exodus Browser — WebChat IM UI helper tests.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import {
  IM_MUTED_CHATS_KEY,
  buildMessageTimelineItems,
  conversationIdForContact,
  conversationIdForGroup,
  formatWebChatDividerTime,
  groupGridCountClass,
  isRealGroupChat,
  loadMutedChatIds,
  pickGroupGridMemberIds,
  primaryNavItems,
  saveMutedChatIds,
  settingsNavTitle,
  navItemTitle,
  CONTACT_DIRECTORY_CATEGORIES,
  contactDirectoryCategoryLabel,
  contactDirectoryCountForCategory,
  formatContactDirectoryCount,
  loadContactDirectoryExpanded,
  saveContactDirectoryExpanded,
  toggleContactDirectoryExpanded,
  IM_CONTACT_DIR_EXPANDED_KEY,
  findReplySourceMessage,
  replyQuotePreview,
} from './imMessengerWebchat';
import type { GroupMessage } from './groupChat';

const mockMessage = (id: string, timestamp: number): GroupMessage => ({
  messageId: id,
  groupId: 'g1',
  senderId: 'u1',
  senderName: 'Alice',
  content: 'hi',
  messageType: 'text',
  attachments: [],
  replyTo: null,
  mentions: [],
  timestamp,
  isEdited: false,
});

describe('imMessengerWebchat', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('detects real groups vs dm rooms', () => {
    expect(isRealGroupChat('team-alpha')).toBe(true);
    expect(isRealGroupChat('dm-node-a-node-b')).toBe(false);
  });

  it('builds conversation ids for mute map', () => {
    expect(conversationIdForContact('node-1')).toBe('contact:node-1');
    expect(conversationIdForGroup('team-alpha')).toBe('group:team-alpha');
  });

  it('persists muted chat ids', () => {
    const ids = new Set(['contact:a', 'group:g1']);
    saveMutedChatIds(ids);
    expect(loadMutedChatIds()).toEqual(ids);
    expect(JSON.parse(localStorage.getItem(IM_MUTED_CHATS_KEY) || '[]')).toEqual([
      'contact:a',
      'group:g1',
    ]);
  });

  it('picks up to nine member ids for grid avatar', () => {
    const members = Array.from({ length: 12 }, (_, i) => `node-${i}`);
    expect(pickGroupGridMemberIds(members)).toHaveLength(9);
    expect(pickGroupGridMemberIds(members, 4)).toHaveLength(4);
  });

  it('formats divider time for same day as HH:mm', () => {
    const now = new Date('2026-05-24T12:00:00');
    const ts = new Date('2026-05-24T10:25:00').getTime();
    expect(formatWebChatDividerTime(ts, now)).toBe('10:25');
  });

  it('formats divider time for yesterday', () => {
    const now = new Date('2026-05-24T12:00:00');
    const ts = new Date('2026-05-23T09:15:00').getTime();
    expect(formatWebChatDividerTime(ts, now)).toBe('昨天 09:15');
  });

  it('builds timeline with dividers when gap exceeds threshold', () => {
    const t0 = 1_700_000_000_000;
    const items = buildMessageTimelineItems(
      [
        mockMessage('m1', t0),
        mockMessage('m2', t0 + 60_000),
        mockMessage('m3', t0 + 6 * 60_000),
      ],
      5 * 60_000,
      new Date(t0 + 7 * 60_000),
    );

    expect(items.filter((i) => i.kind === 'divider')).toHaveLength(2);
    expect(items.filter((i) => i.kind === 'message')).toHaveLength(3);
  });

  it('maps grid count to css class suffix', () => {
    expect(groupGridCountClass(0)).toBe('grid-count-1');
    expect(groupGridCountClass(5)).toBe('grid-count-5');
    expect(groupGridCountClass(12)).toBe('grid-count-9');
  });

  it('orders primary nav for WebChat desktop vs default', () => {
    const desktop = primaryNavItems(true).map((item) => item.id);
    const sidebar = primaryNavItems(false).map((item) => item.id);

    expect(desktop[0]).toBe('chats');
    expect(desktop[1]).toBe('contacts');
    expect(desktop).toContain('timeline');
    expect(sidebar[0]).toBe('chats');
    expect(sidebar[1]).toBe('collections');
  });

  it('returns localized nav titles', () => {
    const chats = primaryNavItems(true)[0];
    expect(navItemTitle(chats, true)).toBe('聊天');
    expect(navItemTitle(chats, false)).toBe('Chats');
    expect(settingsNavTitle(true)).toBe('更多');
    expect(settingsNavTitle(false)).toBe('Settings');
  });

  it('defines WebChat macOS address book categories with counts', () => {
    expect(CONTACT_DIRECTORY_CATEGORIES).toHaveLength(7);
    expect(CONTACT_DIRECTORY_CATEGORIES.map((item) => item.id)).toEqual([
      'new_friends',
      'group_chats',
      'official_accounts',
      'service_accounts',
      'wecom_contacts',
      'my_enterprises',
      'contacts',
    ]);
    expect(contactDirectoryCategoryLabel(CONTACT_DIRECTORY_CATEGORIES[0], true)).toBe('新的朋友');
    expect(contactDirectoryCategoryLabel(CONTACT_DIRECTORY_CATEGORIES[6], true)).toBe('联系人');
  });

  it('formats category counts and resolves count by category id', () => {
    const counts = {
      groupChats: 22,
      officialAccounts: 163,
      serviceAccounts: 72,
      wecomContacts: 38,
      myEnterprises: 9,
      contacts: 7710,
    };
    expect(formatContactDirectoryCount(7710)).toBe('7,710');
    expect(contactDirectoryCountForCategory('new_friends', counts)).toBeNull();
    expect(contactDirectoryCountForCategory('group_chats', counts)).toBe(22);
    expect(contactDirectoryCountForCategory('contacts', counts)).toBe(7710);
  });

  it('persists expanded contact directory categories', () => {
    const expanded = toggleContactDirectoryExpanded(new Set(['contacts']), 'group_chats');
    saveContactDirectoryExpanded(expanded);
    expect(loadContactDirectoryExpanded()).toEqual(new Set(['contacts', 'group_chats']));
    expect(JSON.parse(localStorage.getItem(IM_CONTACT_DIR_EXPANDED_KEY) || '[]')).toEqual([
      'contacts',
      'group_chats',
    ]);
  });

  it('builds reply quote preview from source message', () => {
    const messages = [
      {
        messageId: 'm1',
        groupId: 'g1',
        senderId: 'u1',
        senderName: 'Alice',
        content: 'Original message body',
        messageType: 'text' as const,
        attachments: [],
        replyTo: null,
        mentions: [],
        timestamp: 1,
        isEdited: false,
      },
    ];
    expect(findReplySourceMessage(messages, 'm1')?.senderName).toBe('Alice');
    expect(replyQuotePreview(findReplySourceMessage(messages, 'm1'))).toBe('Original message body');
    expect(replyQuotePreview(null)).toBe('消息');
  });
});
