/**
 * Exodus Browser — ImMessenger WebChat desktop layout tests.
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

vi.mock('$lib/contactDirectory', () => ({
  buildHumanContact: vi.fn(),
  contactAdd: vi.fn(),
  contactAddFriendByDigit: vi.fn(),
  contactGetLocalDigit: vi.fn(async () => '123456789012'),
  contactDirectoryServiceStart: vi.fn(),
  contactList: vi.fn(async () => []),
  contactRemove: vi.fn(),
  contactToggleFavorite: vi.fn(),
  contactUpdate: vi.fn(),
  touchContactLastContacted: vi.fn(),
}));

vi.mock('$lib/imChat', () => ({
  IM_OPEN_CONTACT_EVENT: 'exodus-open-im',
  IM_NEW_MESSAGE_EVENT: 'exodus-im-new-message',
  dmRoomId: vi.fn((a: string, b: string) => `dm-${a}-${b}`),
  ensureDmGroup: vi.fn(),
  loadDmMessages: vi.fn(async () => []),
  notifyImNewMessage: vi.fn(),
  openImChat: vi.fn(),
  sendDmText: vi.fn(),
  startCallFromUi: vi.fn(),
  applyMessageCacheUpdate: vi.fn(),
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
  groupListUser: vi.fn(async () => [
    {
      groupId: 'team-alpha',
      name: 'Project Team',
      description: 'Daily sync',
      ownerId: 'user-1',
      memberIds: ['node-me', 'node-a', 'node-b', 'node-c'],
      adminIds: ['user-1'],
      isPrivate: false,
      createdAt: 1_700_000_000_000,
      lastActivity: 1_700_000_100_000,
      messageCount: 3,
    },
  ]),
  groupRemoveMember: vi.fn(),
}));

vi.mock('$lib/p2p/cdnIntegrations', () => ({
  sendGroupMessageWithCdn: vi.fn(),
  prepareBrowserFileAttachment: vi.fn(),
}));

vi.mock('$lib/imSession', () => ({
  resolveLocalIdentity: vi.fn(async () => ({
    userId: 'user-1',
    displayName: 'Me',
    nodeId: 'node-me',
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
  chatCollectionDelete: vi.fn(),
  chatCollectionList: vi.fn(async () => []),
  chatCollectionSave: vi.fn(),
  chatCollectionIsSaved: vi.fn(async () => false),
  collectionItemPreview: vi.fn(),
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

describe('ImMessenger WebChat desktop layout', () => {
  beforeEach(() => {
    resetImStoreForTests();
    localStorage.clear();
  });

  it('applies webchat-desktop and full-width classes when fullWidth prop set', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    const root = wrapper.find('.im-messenger');
    expect(root.classes()).toContain('webchat-desktop');
    expect(root.classes()).toContain('full-width');
    expect(root.classes()).toContain('dark-mode');
  });

  it('renders WebChat search toolbar with 搜索 placeholder', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    expect(wrapper.find('.webchat-list-toolbar').exists()).toBe(true);
    expect(wrapper.find('.webchat-search input').attributes('placeholder')).toBe('搜索');
    expect(wrapper.find('.webchat-toolbar-btn .im-icon--plus').exists()).toBe(true);
  });

  it('places settings nav item in footer menu with Chinese title', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    expect(wrapper.find('.nav-menu-footer .nav-item[title="更多"]').exists()).toBe(true);
    expect(wrapper.find('.nav-menu-primary .nav-item[title="Settings"]').exists()).toBe(false);
  });

  it('uses unified ImMessengerIcon set in primary nav', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    expect(wrapper.find('.nav-menu-primary .im-icon--chat').exists()).toBe(true);
    expect(wrapper.find('.nav-menu-primary .im-icon--contacts').exists()).toBe(true);
    expect(wrapper.find('.nav-menu-footer .im-icon--menu').exists()).toBe(true);
  });

  it('shows WebChat empty state copy when no chat selected', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    expect(wrapper.find('.webchat-empty-main').exists()).toBe(true);
    expect(wrapper.find('.webchat-empty-main .im-icon--webchat-logo').exists()).toBe(true);
    expect(wrapper.text()).toContain('WebChat');
    expect(wrapper.text()).toContain('选择一个聊天开始对话');
  });

  it('uses compact chat header without avatar in WebChat mode', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    expect(wrapper.find('.chat-window-header.webchat-header').exists()).toBe(false);
    expect(wrapper.find('.webchat-empty-main').exists()).toBe(true);
  });

  it('renders group grid avatar and mute indicator in unified chat list', async () => {
    const { saveMutedChatIds } = await import('$lib/imMessengerWebchat');
    saveMutedChatIds(new Set(['group:team-alpha']));

    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    expect(wrapper.find('.group-grid-avatar.grid-count-4').exists()).toBe(true);
    expect(wrapper.find('.mute-indicator').exists()).toBe(true);
    expect(wrapper.text()).toContain('Project Team');
  });

  it('shows WebChat macOS address book directory in contacts nav', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    const contactsNav = wrapper.find('.nav-menu-primary .nav-item[title="通讯录"]');
    expect(contactsNav.exists()).toBe(true);
    await contactsNav.trigger('click');
    await flushPromises();

    expect(wrapper.find('.contact-manage-btn').text()).toContain('通讯录管理');
    expect(wrapper.find('.contact-category-row').exists()).toBe(true);
    expect(wrapper.text()).toContain('新的朋友');
    expect(wrapper.text()).toContain('群聊');
    expect(wrapper.text()).toContain('公众号');
    expect(wrapper.text()).toContain('服务号');
    expect(wrapper.text()).toContain('企业微信联系人');
    expect(wrapper.text()).toContain('我的企业');
    expect(wrapper.text()).toContain('联系人');
    expect(wrapper.find('.category-count').exists()).toBe(true);
    expect(wrapper.find('.category-chevron').exists()).toBe(true);
    expect(wrapper.find('.empty-state').exists()).toBe(false);
  });

  it('expands group chats inline from address book category row', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    await wrapper.find('.nav-menu-primary .nav-item[title="通讯录"]').trigger('click');
    await flushPromises();

    const groupRow = wrapper.findAll('.contact-category-row').find((row) => row.text().includes('群聊'));
    expect(groupRow).toBeTruthy();
    expect(groupRow!.text()).toContain('1');
    await groupRow!.trigger('click');
    await flushPromises();

    expect(wrapper.find('.contact-nested-item .nested-item-name').text()).toBe('Project Team');
  });

  it('opens add friend dialog from 新的朋友 category row', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    await wrapper.find('.nav-menu-primary .nav-item[title="通讯录"]').trigger('click');
    await flushPromises();

    const newFriendsRow = wrapper.findAll('.contact-category-row').find((row) => row.text().includes('新的朋友'));
    await newFriendsRow!.trigger('click');
    await flushPromises();

    expect(wrapper.find('.modal-content h3').text()).toMatch(/Add Contact|添加/);
  });

  it('shows contact manage export/import actions', async () => {
    const wrapper = mount(ImMessenger, { props: { fullWidth: true } });
    await flushPromises();

    await wrapper.find('.nav-menu-primary .nav-item[title="通讯录"]').trigger('click');
    await flushPromises();
    await wrapper.find('.contact-manage-btn').trigger('click');

    expect(wrapper.find('.contact-manage-actions').exists()).toBe(true);
    expect(wrapper.text()).toContain('导出通讯录');
    expect(wrapper.text()).toContain('导入通讯录');
  });
});
