/**
 * Exodus Browser — ImMessenger favorites (WebChat sidebar 收藏) tests.
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

vi.mock('$lib/contactDirectory', () => ({
  buildHumanContact: vi.fn((data: Record<string, unknown>) => ({
  contact_id: 'new-id',
  name: data.name,
  node_id: data.nodeId,
  contact_type: 'human',
  agent_ids: [],
  groups: ['friends'],
  tags: [],
  notes: '',
  is_favorite: false,
  is_blocked: false,
  created_at: 0,
  last_contacted: 0,
  contact_count: 0,
  public_account_id: null,
  })),
  contactAdd: vi.fn(),
  contactAddFriendByDigit: vi.fn(),
  contactGetLocalDigit: vi.fn(async () => '123456789012'),
  contactDirectoryServiceStart: vi.fn(),
  contactList: vi.fn(),
  contactToggleFavorite: vi.fn(),
  touchContactLastContacted: vi.fn(),
}));

vi.mock('$lib/imChat', () => ({
  IM_OPEN_CONTACT_EVENT: 'im-open-contact',
  IM_NEW_MESSAGE_EVENT: 'exodus-im-new-message',
  dmRoomId: vi.fn((a: string, b: string) => `dm-${a}-${b}`),
  ensureDmGroup: vi.fn(),
  loadDmMessages: vi.fn(async () => []),
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

vi.mock('$lib/chatCollections', () => ({
  buildSaveChatItemRequest: vi.fn(),
  chatCollectionSave: vi.fn(),
  chatCollectionList: vi.fn(async () => []),
  chatCollectionIsSaved: vi.fn(async () => false),
  chatCollectionDelete: vi.fn(),
  collectionItemPreview: vi.fn((item: { content: string }) => item.content),
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

async function mountMessenger(contacts: Contact[] = []) {
  const { contactDirectoryServiceStart, contactList } = await import('$lib/contactDirectory');
  const { groupChatServiceStart, groupListUser } = await import('$lib/groupChat');
  vi.mocked(contactDirectoryServiceStart).mockResolvedValue({
    storageDir: '/tmp',
    nodeId: 'node-local',
    inProcess: true,
  });
  vi.mocked(contactList).mockResolvedValue(contacts);
  vi.mocked(groupChatServiceStart).mockResolvedValue(undefined);
  vi.mocked(groupListUser).mockResolvedValue([]);

  const wrapper = mount(ImMessenger, {
    global: {
      stubs: {
        SocialTimeline: { template: '<div class="social-timeline-stub" />' },
      },
    },
  });
  await flushPromises();
  return wrapper;
}

describe('ImMessenger favorites', () => {
  beforeEach(() => {
    resetImStoreForTests();
    vi.clearAllMocks();
  });

  it('renders Starred navigation item', async () => {
    const wrapper = await mountMessenger();
    const starredNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    expect(starredNav).toBeTruthy();
  });

  it('shows favorite count badge when contacts are starred', async () => {
    const wrapper = await mountMessenger([
      mockContact({ contact_id: 'c1', is_favorite: true }),
      mockContact({ contact_id: 'c2', name: 'Bob', node_id: 'node-bob', is_favorite: true }),
      mockContact({ contact_id: 'c3', name: 'Carol', node_id: 'node-carol', is_favorite: false }),
    ]);

    const favoritesNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    expect(favoritesNav?.find('.nav-badge-muted')?.text()).toBe('2');
  });

  it('sorts favorited contacts to the top in Chats list', async () => {
    const wrapper = await mountMessenger([
      mockContact({ contact_id: 'c1', name: 'Regular', node_id: 'node-regular', is_favorite: false, last_contacted: 500 }),
      mockContact({ contact_id: 'c2', name: 'Starred', node_id: 'node-starred', is_favorite: true, last_contacted: 100 }),
    ]);

    const names = wrapper.findAll('.chat-item .chat-name').map((el) => el.text());
    expect(names[0]).toBe('Starred');
    expect(names[1]).toBe('Regular');
  });

  it('shows only starred contacts in Starred tab', async () => {
    const wrapper = await mountMessenger([
      mockContact({ contact_id: 'c1', name: 'Starred', is_favorite: true }),
      mockContact({ contact_id: 'c2', name: 'Regular', node_id: 'node-bob', is_favorite: false }),
    ]);

    const favoritesNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    await favoritesNav!.trigger('click');
    await flushPromises();

    expect(wrapper.find('.sidebar-title').text()).toBe('Starred');
    const names = wrapper.findAll('.chat-item .chat-name').map((el) => el.text());
    expect(names).toEqual(['Starred']);
  });

  it('shows empty state on Favorites tab when none starred', async () => {
    const wrapper = await mountMessenger([mockContact({ is_favorite: false })]);

    const favoritesNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    await favoritesNav!.trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('No starred contacts yet');
    expect(wrapper.text()).toContain('Star a contact from Chats to add them here.');
  });

  it('toggles favorite via star button and emits status', async () => {
    const { contactToggleFavorite } = await import('$lib/contactDirectory');
    vi.mocked(contactToggleFavorite).mockResolvedValue(true);

    const wrapper = await mountMessenger([mockContact()]);
    const starBtn = wrapper.find('.favorite-btn');
    expect(starBtn.text()).toBe('☆');

    await starBtn.trigger('click');
    await flushPromises();

    expect(contactToggleFavorite).toHaveBeenCalledWith('contact-1');
    expect(wrapper.emitted('status')?.[0]).toEqual(['Alice added to Starred']);
    expect(wrapper.find('.favorite-btn.active').exists()).toBe(true);
  });

  it('removes favorite and updates badge count', async () => {
    const { contactToggleFavorite } = await import('$lib/contactDirectory');
    vi.mocked(contactToggleFavorite).mockResolvedValue(false);

    const wrapper = await mountMessenger([mockContact({ is_favorite: true })]);
    await wrapper.find('.favorite-btn.active').trigger('click');
    await flushPromises();

    expect(wrapper.emitted('status')?.[0]).toEqual(['Alice removed from Starred']);
    const favoritesNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    expect(favoritesNav?.find('.nav-badge-muted')?.exists()).toBe(false);
  });

  it('filters favorites by search query', async () => {
    const wrapper = await mountMessenger([
      mockContact({ contact_id: 'c1', name: 'Alice Star', is_favorite: true }),
      mockContact({ contact_id: 'c2', name: 'Bob Star', node_id: 'node-bob', is_favorite: true }),
    ]);

    const favoritesNav = wrapper.findAll('.nav-item').find((btn) => btn.attributes('title') === 'Starred');
    await favoritesNav!.trigger('click');
    await wrapper.find('.search-input').setValue('bob');
    await flushPromises();

    const names = wrapper.findAll('.chat-item .chat-name').map((el) => el.text());
    expect(names).toEqual(['Bob Star']);
  });

  it('emits error status when toggle favorite fails', async () => {
    const { contactToggleFavorite } = await import('$lib/contactDirectory');
    vi.mocked(contactToggleFavorite).mockRejectedValue(new Error('Directory offline'));

    const wrapper = await mountMessenger([mockContact()]);
    await wrapper.find('.favorite-btn').trigger('click');
    await flushPromises();

    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to update favorite: Directory offline']);
  });

  it('applies favorited class to starred chat rows', async () => {
    const wrapper = await mountMessenger([mockContact({ is_favorite: true })]);
    expect(wrapper.find('.chat-item.favorited').exists()).toBe(true);
  });

  it('adds friend by 12-digit ID from add contact dialog', async () => {
    const { contactAddFriendByDigit } = await import('$lib/contactDirectory');
    const newContact = mockContact({ contact_id: 'new-friend', name: 'Bob', node_id: 'node-bob' });
    vi.mocked(contactAddFriendByDigit).mockResolvedValue(newContact);

    const wrapper = await mountMessenger([]);
    await wrapper.find('[title="Add Contact"]').trigger('click');
    await wrapper.find('input[placeholder="Enter 12-digit ID"]').setValue('987654321098');
    await wrapper.find('.primary-button').trigger('click');
    await flushPromises();

    expect(contactAddFriendByDigit).toHaveBeenCalledWith('987654321098', 'Friend 987654321098', 'user-1');
    expect(wrapper.emitted('status')?.some((entry) => entry[0] === 'Friend added')).toBe(true);
  });
});
