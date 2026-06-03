/**
 * Exodus Browser — WebChat Collections (WebChat 收藏) API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  deriveCollectionContentType,
  collectionItemPreview,
  buildSaveChatItemRequest,
  chatCollectionSave,
  chatCollectionList,
  chatCollectionSearch,
  chatCollectionDelete,
  chatCollectionIsSaved,
} from './chatCollections';
import type { GroupMessage } from './groupChat';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

const sampleMessage = (): GroupMessage => ({
  messageId: 'msg-1',
  groupId: 'room-1',
  senderId: 'user-a',
  senderName: 'Alice',
  content: 'See https://example.com',
  messageType: 'text',
  attachments: [],
  replyTo: null,
  mentions: [],
  timestamp: 1_700_000_000_000,
  isEdited: false,
  editedAt: null,
});

describe('chatCollections', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('derives link content type from URL', () => {
    expect(deriveCollectionContentType('visit https://exodus.dev', 'text', [])).toBe('link');
  });

  it('derives image content type from attachments', () => {
    expect(
      deriveCollectionContentType('', 'text', [
        {
          attachmentId: 'a1',
          fileType: 'image/png',
          blobHash: 'hash',
          fileName: 'photo.png',
          fileSize: 100,
        },
      ]),
    ).toBe('image');
  });

  it('builds preview text for saved items', () => {
    expect(
      collectionItemPreview({
        id: '1',
        user_id: 'u1',
        source_message_id: 'm1',
        conversation_id: 'r1',
        conversation_type: 'dm',
        conversation_title: 'Alice',
        sender_id: 'a',
        sender_name: 'Alice',
        content_type: 'text',
        content: 'Hello',
        message_type: 'text',
        attachments: [],
        original_timestamp: 1,
        saved_at: 2,
      }),
    ).toBe('Hello');
  });

  it('builds save request from group message', () => {
    const req = buildSaveChatItemRequest({
      userId: 'user-1',
      message: sampleMessage(),
      conversationType: 'dm',
      conversationTitle: 'Alice',
    });
    expect(req.sourceMessageId).toBe('msg-1');
    expect(req.conversationTitle).toBe('Alice');
  });

  it('saves collection item via invoke', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const saved = {
      id: 'saved-1',
      user_id: 'user-1',
      source_message_id: 'msg-1',
      conversation_id: 'room-1',
      conversation_type: 'dm',
      conversation_title: 'Alice',
      sender_id: 'user-a',
      sender_name: 'Alice',
      content_type: 'link',
      content: 'See https://example.com',
      message_type: 'text',
      attachments: [],
      original_timestamp: 1_700_000_000_000,
      saved_at: 1_700_000_000_100,
    };
    vi.mocked(invoke).mockResolvedValue(saved);

    const result = await chatCollectionSave(
      buildSaveChatItemRequest({
        userId: 'user-1',
        message: sampleMessage(),
        conversationType: 'dm',
        conversationTitle: 'Alice',
      }),
    );

    expect(result).toEqual(saved);
    expect(invoke).toHaveBeenCalledWith('chat_collection_save', expect.any(Object));
  });

  it('lists collection items', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue([]);
    await chatCollectionList('user-1');
    expect(invoke).toHaveBeenCalledWith('chat_collection_list', { userId: 'user-1' });
  });

  it('searches collection items', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue([]);
    await chatCollectionSearch('user-1', 'hello');
    expect(invoke).toHaveBeenCalledWith('chat_collection_search', { userId: 'user-1', query: 'hello' });
  });

  it('deletes collection item', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);
    const ok = await chatCollectionDelete('saved-1', 'user-1');
    expect(ok).toBe(true);
    expect(invoke).toHaveBeenCalledWith('chat_collection_delete', { id: 'saved-1', userId: 'user-1' });
  });

  it('checks if message is saved', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);
    const saved = await chatCollectionIsSaved('user-1', 'msg-1');
    expect(saved).toBe(true);
    expect(invoke).toHaveBeenCalledWith('chat_collection_is_saved', {
      userId: 'user-1',
      sourceMessageId: 'msg-1',
    });
  });
});
