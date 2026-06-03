/**
 * Exodus Browser — ImMessenger Collections (WebChat 收藏) tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('./SocialTimeline.vue', () => ({
  default: {
    name: 'SocialTimeline',
    template: '<div class="social-timeline-stub" />',
  },
}));

import { mount, flushPromises } from '@vue/test-utils';
import ImMessenger from './ImMessenger.vue';
import { resetImStoreForTests } from '$lib/imStore';
import type { Contact } from '$lib/contactDirectory';
import type { SavedChatItem } from '$lib/chatCollections';

const mockContact = (overrides: Partial<Contact> = {}): Contact => ({
  contact_id: overrides.contact_id ?? 'contact-1',
  name: overrides.name ?? 'Alice',
  contact_type: 'human',
  agent_deployment_type: null,
  agent_ids: [],
  node_id: overrides.node_id ?? 'node-alice',
  groups: [],
  tags: [],
  notes: '',
  is_favorite: overrides.is_favorite ?? false,
  is_blocked: false,
  created_at: 100,
  last_contacted: overrides.last_contacted ?? 200,
  contact_count: 0,
  public_account_id: null,
});

const mockSavedItem = (overrides: Partial<SavedChatItem> = {}): SavedChatItem => ({
  id: overrides.id ?? 'saved-1',
  user_id: 'user-1',
  source_message_id: overrides.source_message_id ?? 'msg-1',
  conversation_id: 'room-1',
  conversation_type: 'dm',
  conversation_title: 'Alice',
  sender_id: 'user-a',
  sender_name: 'Alice',
  content_type: 'text',
  content: overrides.content ?? 'Saved hello',
  message_type: 'text',
  attachments: [],
  original_timestamp: 1_700_000_000_000,
  saved_at: 1_700_000_000_100,
});

vi.mock('$lib/contactDirectory', () => ({
  buildHumanContact: vi.fn(),
  contactAdd: vi.fn(),
  contactAddFriendByDigit: vi.fn(),
  contactGetLocalDigit: vi.fn(async () => '123456789012'),
  contactDirectoryServiceStart: vi.fn(),
  contactList: vi.fn(),
  contactRemove: vi.fn(),
  contactToggleFavorite: vi.fn(),
  contactUpdate: vi.fn(),
  touchContactLastContacted: vi.fn(),
}));

vi.mock('$lib/chatCollections', () => ({
  buildSaveChatItemRequest: vi.fn((params) => ({
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
  })),
  chatCollectionSave: vi.fn(),
  chatCollectionList: vi.fn(),
  chatCollectionDelete: vi.fn(),
  collectionItemPreview: vi.fn((item: SavedChatItem) => item.content),
}));

vi.mock('$lib/imChat', () => ({
  IM_OPEN_CONTACT_EVENT: 'im-open-contact',
  IM_NEW_MESSAGE_EVENT: 'exodus-im-new-message',
  dmRoomId: vi.fn((a: string, b: string) => `dm-${a}-${b}`),
  ensureDmGroup: vi.fn(),
  loadDmMessages: vi.fn(),
  notifyImNewMessage: vi.fn(),
  sendDmText: vi.fn(),
  startCallFromUi: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => false,
}));

vi.mock('$lib/groupChat', () => ({
  buildGroupPayload: vi.fn(),
  groupChatServiceStart: vi.fn(),
  groupCreate: vi.fn(),
  groupDeleteMessage: vi.fn(),
  groupEditMessage: vi.fn(),
  groupGet: vi.fn(),
  groupGetMembers: vi.fn(),
  groupGetMessages: vi.fn(),
  groupListUser: vi.fn(),
  groupRemoveMember: vi.fn(),
}));

vi.mock('$lib/p2p/cdnIntegrations', () => ({
  sendGroupMessageWithCdn: vi.fn(),
  prepareBrowserFileAttachment: vi.fn(),
}));

vi.mock('$lib/groupMentions', () => ({
  extractMentionNodeIds: vi.fn(() => []),
}));

vi.mock('$lib/imSession', () => ({
  resolveLocalIdentity: vi.fn(async () => ({
    userId: 'user-1',
    displayName: 'Test User',
    nodeId: 'node-local',
  })),
}));

vi.mock('$lib/presence', () => ({
  startPresenceHeartbeat: vi.fn(),
  stopPresenceHeartbeat: vi.fn(),
  fetchOnlinePeers: vi.fn(async () => new Map()),
  isNodeOnline: vi.fn(() => false),
}));

vi.mock('$lib/publicAccount', () => ({
  publicAccountServiceStart: vi.fn(),
  publicAccountList: vi.fn(async () => []),
  publicAccountSubscribe: vi.fn(),
  publicAccountUnsubscribe: vi.fn(),
  publicAccountGetSubscriptions: vi.fn(async () => []),
  publicAccountListArticles: vi.fn(async () => []),
  publicAccountSearch: vi.fn(async () => []),
}));

vi.mock('$lib/imMessageSync', () => ({
  ensureImMessageSync: vi.fn(),
}));

vi.mock('@/lib/logger', () => ({
  logInfo: vi.fn(),
  logDebug: vi.fn(),
  logWarn: vi.fn(),
  logError: vi.fn(),
}));

async function mountMessenger() {
  const { contactDirectoryServiceStart, contactList } = await import('$lib/contactDirectory');
  const { groupChatServiceStart, groupListUser } = await import('$lib/groupChat');
  const { chatCollectionList } = await import('$lib/chatCollections');
  vi.mocked(contactDirectoryServiceStart).mockResolvedValue({
    storageDir: '/tmp',
    nodeId: 'node-local',
    inProcess: true,
  });
  vi.mocked(contactList).mockResolvedValue([mockContact()]);
  vi.mocked(groupChatServiceStart).mockResolvedValue(undefined);
  vi.mocked(groupListUser).mockResolvedValue([]);
  vi.mocked(chatCollectionList).mockResolvedValue([mockSavedItem()]);

  const wrapper = mount(ImMessenger, {
    global: {
      stubs: { SocialTimeline: { template: '<div />' } },
    },
  });
  await flushPromises();
  return wrapper;
}

describe('ImMessenger collections', () => {
  beforeEach(() => {
    resetImStoreForTests();
    vi.clearAllMocks();
  });

  it('renders Collections navigation item', async () => {
    const wrapper = await mountMessenger();
    const nav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Collections');
    expect(nav).toBeTruthy();
  });

  it('shows saved items in Collections tab', async () => {
    const wrapper = await mountMessenger();
    const collectionsNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Collections');
    await collectionsNav!.trigger('click');
    await flushPromises();

    expect(wrapper.find('.sidebar-title').text()).toBe('Collections');
    expect(wrapper.find('.collection-item').exists()).toBe(true);
    expect(wrapper.text()).toContain('Saved hello');
  });

  it('shows collection count badge', async () => {
    const wrapper = await mountMessenger();
    const collectionsNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Collections');
    expect(collectionsNav?.find('.nav-badge-muted')?.text()).toBe('1');
  });

  it('renames contact favorites nav to Starred', async () => {
    const wrapper = await mountMessenger();
    const starredNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    expect(starredNav).toBeTruthy();
  });
});
