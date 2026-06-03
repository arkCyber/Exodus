/**
 * Exodus Browser — Shared IM store tests.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import {
  imStore,
  setActiveContactNode,
  setActiveGroupId,
  setStoreMessages,
  getStoreMessages,
  setStoreDraft,
  getStoreDraft,
  bumpImStore,
} from './imStore';

describe('imStore', () => {
  beforeEach(() => {
    imStore.activeNav = 'chats';
    imStore.activeContactNodeId = null;
    imStore.activeGroupId = null;
    imStore.messagesByKey = {};
    imStore.draftsByKey = {};
    imStore.messageCache = {};
    imStore.revision = 0;
  });

  it('selecting contact clears active group', () => {
    setActiveGroupId('group-1');
    setActiveContactNode('node-a');
    expect(imStore.activeGroupId).toBeNull();
    expect(imStore.activeContactNodeId).toBe('node-a');
  });

  it('selecting group clears active contact', () => {
    setActiveContactNode('node-a');
    setActiveGroupId('group-1');
    expect(imStore.activeContactNodeId).toBeNull();
    expect(imStore.activeGroupId).toBe('group-1');
  });

  it('stores messages and drafts per conversation key', () => {
    setStoreMessages('room-1', [
      {
        messageId: 'm1',
        groupId: 'room-1',
        senderId: 'u1',
        senderName: 'A',
        content: 'hi',
        messageType: 'text',
        attachments: [],
        replyTo: null,
        mentions: [],
        timestamp: 1,
        isEdited: false,
      },
    ]);
    setStoreDraft('room-1', 'draft text');
    expect(getStoreMessages('room-1')).toHaveLength(1);
    expect(getStoreDraft('room-1')).toBe('draft text');
  });

  it('bumps revision on shared updates', () => {
    const before = imStore.revision;
    bumpImStore();
    expect(imStore.revision).toBe(before + 1);
  });
});
