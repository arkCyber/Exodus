/**
 * Exodus Browser — GroupChatSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import GroupChatSettings from './GroupChatSettings.vue';

vi.mock('$lib/groupChat', () => ({
  groupChatServiceStart: vi.fn(),
  groupGetMessages: vi.fn()
}));

vi.mock('$lib/p2p/cdnIntegrations', () => ({
  sendGroupMessageWithCdn: vi.fn()
}));

describe('GroupChatSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('h3').text()).toBe('Group chat');
  });

  it('renders group ID label', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('label').text()).toContain('Group ID');
  });

  it('renders group ID input', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('input[type="text"]').exists()).toBe(true);
  });

  it('uses default group ID when not provided', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('input[type="text"]').element.value).toBe('lobby');
  });

  it('uses provided group ID prop', () => {
    const wrapper = mount(GroupChatSettings, {
      props: { groupId: 'custom-group' }
    });
    
    expect(wrapper.find('input[type="text"]').element.value).toBe('custom-group');
  });

  it('renders messages container', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('.messages').exists()).toBe(true);
  });

  it('shows empty state when no messages', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([]);
    
    const wrapper = mount(GroupChatSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.hint').text()).toBe('No messages yet.');
  });

  it('renders messages when available', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([
      { messageId: '1', senderId: 'user-123', content: 'Hello', messageType: 'text', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(GroupChatSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const messages = wrapper.findAll('.msg');
    expect(messages.length).toBe(1);
  });

  it('displays truncated sender ID', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([
      { messageId: '1', senderId: 'user-12345678', content: 'Hello', messageType: 'text', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(GroupChatSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.msg strong').text()).toBe('user-123');
  });

  it('displays message content', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([
      { messageId: '1', senderId: 'user-123', content: 'Hello world', messageType: 'text', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(GroupChatSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.msg').text()).toContain('Hello world');
  });

  it('renders toolbar', () => {
    const wrapper = mount(GroupChatSettings);
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders message input', () => {
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    expect(inputs[1].exists()).toBe(true);
  });

  it('has correct placeholder on message input', () => {
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    expect(inputs[1].attributes('placeholder')).toBe('Message…');
  });

  it('renders send button', () => {
    const wrapper = mount(GroupChatSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Send');
  });

  it('disables send button when draft is empty', () => {
    const wrapper = mount(GroupChatSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].attributes('disabled')).toBeDefined();
  });

  it('enables send button when draft has text', async () => {
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[1].setValue('Hello');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].attributes('disabled')).toBeUndefined();
  });

  it('renders refresh button', () => {
    const wrapper = mount(GroupChatSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].text()).toBe('Refresh');
  });

  it('sends message on send button click', async () => {
    const { sendGroupMessageWithCdn } = require('$lib/p2p/cdnIntegrations');
    sendGroupMessageWithCdn.mockResolvedValue(undefined);
    
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[1].setValue('Hello');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(sendGroupMessageWithCdn).toHaveBeenCalled();
  });

  it('sends message on enter key', async () => {
    const { sendGroupMessageWithCdn } = require('$lib/p2p/cdnIntegrations');
    sendGroupMessageWithCdn.mockResolvedValue(undefined);
    
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[1].setValue('Hello');
    await inputs[1].trigger('keydown.enter');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(sendGroupMessageWithCdn).toHaveBeenCalled();
  });

  it('clears draft after sending', async () => {
    const { sendGroupMessageWithCdn } = require('$lib/p2p/cdnIntegrations');
    sendGroupMessageWithCdn.mockResolvedValue(undefined);
    
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[1].setValue('Hello');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(inputs[1].element.value).toBe('');
  });

  it('emits status on successful send', async () => {
    const { sendGroupMessageWithCdn } = require('$lib/p2p/cdnIntegrations');
    sendGroupMessageWithCdn.mockResolvedValue(undefined);
    
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[1].setValue('Hello');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Message sent']);
  });

  it('emits status on failed send', async () => {
    const { sendGroupMessageWithCdn } = require('$lib/p2p/cdnIntegrations');
    sendGroupMessageWithCdn.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[1].setValue('Hello');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Send failed']);
  });

  it('does not send empty message', async () => {
    const { sendGroupMessageWithCdn } = require('$lib/p2p/cdnIntegrations');
    
    const wrapper = mount(GroupChatSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    
    expect(sendGroupMessageWithCdn).not.toHaveBeenCalled();
  });

  it('reloads messages on refresh button click', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([]);
    
    const wrapper = mount(GroupChatSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(groupGetMessages).toHaveBeenCalled();
  });

  it('reloads messages on group ID change', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([]);
    
    const wrapper = mount(GroupChatSettings);
    
    const inputs = wrapper.findAll('input[type="text"]');
    await inputs[0].setValue('new-group');
    await inputs[0].trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(groupGetMessages).toHaveBeenCalledWith('new-group');
  });

  it('loads messages on mount', async () => {
    const { groupGetMessages } = require('$lib/groupChat');
    groupGetMessages.mockResolvedValue([]);
    
    mount(GroupChatSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(groupGetMessages).toHaveBeenCalled();
  });

  it('ensures service is started before loading messages', async () => {
    const { groupChatServiceStart, groupGetMessages } = require('$lib/groupChat');
    groupChatServiceStart.mockResolvedValue(undefined);
    groupGetMessages.mockResolvedValue([]);
    
    mount(GroupChatSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(groupChatServiceStart).toHaveBeenCalled();
  });
});
