/**
 * Exodus Browser — ContactDirectoryPanel component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import ContactDirectoryPanel from './ContactDirectoryPanel.vue';

vi.mock('$lib/contactDirectory', () => ({
  buildHumanContact: vi.fn((data) => ({ ...data, id: 'test-id', is_blocked: false, notes: '' })),
  contactAdd: vi.fn(),
  contactDirectoryServiceStart: vi.fn(),
  contactList: vi.fn()
}));

vi.mock('$lib/imChat', () => ({
  openImChat: vi.fn(),
  startCallFromUi: vi.fn()
}));

vi.mock('$lib/imSession', () => ({
  resolveLocalIdentity: vi.fn(async () => ({ nodeId: 'local-node-123' }))
}));

vi.mock('$lib/presence', () => ({
  fetchOnlinePeers: vi.fn(async () => new Map()),
  isNodeOnline: vi.fn(() => false)
}));

describe('ContactDirectoryPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders contact directory', () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    expect(wrapper.find('.contact-directory').exists()).toBe(true);
  });

  it('renders toolbar', () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders search input', () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    expect(wrapper.find('.field').exists()).toBe(true);
    expect(wrapper.find('.field').attributes('placeholder')).toBe('Search contacts…');
  });

  it('renders add button', () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    const buttons = wrapper.findAll('.btn-primary');
    expect(buttons.length).toBe(1);
    expect(buttons[0].text()).toBe('Add');
  });

  it('shows add dialog when add button is clicked', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('hides add dialog when overlay is clicked', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog-overlay').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('does not hide dialog when dialog content is clicked', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('renders dialog title', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h3').text()).toBe('Add contact');
  });

  it('renders name input in dialog', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    expect(inputs[0].attributes('placeholder')).toBe('Name');
  });

  it('renders node ID input in dialog', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    expect(inputs[1].attributes('placeholder')).toBe('Node ID');
  });

  it('renders cancel button in dialog', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.dialog .btn-secondary');
    expect(buttons[0].text()).toBe('Cancel');
  });

  it('hides dialog on cancel button click', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.dialog .btn-secondary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('renders save button in dialog', async () => {
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog .btn-primary').text()).toBe('Save');
  });

  it('renders contact list', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.contact-list').exists()).toBe(true);
  });

  it('renders contact items', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1234567890123456', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const contacts = wrapper.findAll('.contact-row');
    expect(contacts.length).toBe(1);
  });

  it('displays contact name', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.info strong').text()).toBe('Alice');
  });

  it('displays truncated node ID', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1234567890123456', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.node').text()).toBe('node-123456789012…');
  });

  it('displays online status when node is online', async () => {
    const { contactList } = require('$lib/contactDirectory');
    const { isNodeOnline } = require('$lib/presence');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    isNodeOnline.mockReturnValue(true);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.online').exists()).toBe(true);
    expect(wrapper.find('.online').text()).toBe('online');
  });

  it('does not display online status when node is offline', async () => {
    const { contactList } = require('$lib/contactDirectory');
    const { isNodeOnline } = require('$lib/presence');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    isNodeOnline.mockReturnValue(false);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.online').exists()).toBe(false);
  });

  it('renders chat button', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.actions .btn-secondary');
    expect(buttons[0].text()).toBe('Chat');
  });

  it('renders call button', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.actions .btn-secondary');
    expect(buttons[1].text()).toBe('Call');
  });

  it('emits status and opens chat on chat button click', async () => {
    const { contactList } = require('$lib/contactDirectory');
    const { openImChat } = require('$lib/imChat');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.actions .btn-secondary')[0].trigger('click');
    
    expect(openImChat).toHaveBeenCalled();
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('emits status and starts call on call button click', async () => {
    const { contactList } = require('$lib/contactDirectory');
    const { startCallFromUi } = require('$lib/imChat');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.actions .btn-secondary')[1].trigger('click');
    
    expect(startCallFromUi).toHaveBeenCalled();
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('opens chat when contact info is clicked', async () => {
    const { contactList } = require('$lib/contactDirectory');
    const { openImChat } = require('$lib/imChat');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.info').trigger('click');
    
    expect(openImChat).toHaveBeenCalled();
  });

  it('filters contacts by name', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' },
      { id: '2', name: 'Bob', node_id: 'node-2', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.toolbar .field').setValue('alice');
    await wrapper.vm.$nextTick();
    
    const contacts = wrapper.findAll('.contact-row');
    expect(contacts.length).toBe(1);
    expect(contacts[0].text()).toContain('Alice');
  });

  it('filters contacts by node ID', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' },
      { id: '2', name: 'Bob', node_id: 'node-2', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.toolbar .field').setValue('node-1');
    await wrapper.vm.$nextTick();
    
    const contacts = wrapper.findAll('.contact-row');
    expect(contacts.length).toBe(1);
  });

  it('filters contacts by notes', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: 'friend' },
      { id: '2', name: 'Bob', node_id: 'node-2', is_blocked: false, notes: 'work' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.toolbar .field').setValue('friend');
    await wrapper.vm.$nextTick();
    
    const contacts = wrapper.findAll('.contact-row');
    expect(contacts.length).toBe(1);
  });

  it('excludes blocked contacts from filtered results', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' },
      { id: '2', name: 'Bob', node_id: 'node-2', is_blocked: true, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const contacts = wrapper.findAll('.contact-row');
    expect(contacts.length).toBe(1);
  });

  it('shows no contacts message when empty', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.muted').text()).toBe('No contacts');
  });

  it('adds contact when save is clicked', async () => {
    const { contactAdd } = require('$lib/contactDirectory');
    const { contactList } = require('$lib/contactDirectory');
    contactAdd.mockResolvedValue();
    contactList.mockResolvedValue([]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Test Contact');
    await inputs[1].setValue('node-123');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(contactAdd).toHaveBeenCalled();
  });

  it('does not add contact with empty name', async () => {
    const { contactAdd } = require('$lib/contactDirectory');
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('');
    await inputs[1].setValue('node-123');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    
    expect(contactAdd).not.toHaveBeenCalled();
  });

  it('does not add contact with empty node ID', async () => {
    const { contactAdd } = require('$lib/contactDirectory');
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Test Contact');
    await inputs[1].setValue('');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    
    expect(contactAdd).not.toHaveBeenCalled();
  });

  it('emits status on successful add', async () => {
    const { contactAdd } = require('$lib/contactDirectory');
    const { contactList } = require('$lib/contactDirectory');
    contactAdd.mockResolvedValue();
    contactList.mockResolvedValue([]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Test Contact');
    await inputs[1].setValue('node-123');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Contact added']);
  });

  it('emits status on add error', async () => {
    const { contactAdd } = require('$lib/contactDirectory');
    contactAdd.mockRejectedValue(new Error('Add failed'));
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Test Contact');
    await inputs[1].setValue('node-123');
    await wrapper.find('.dialog .btn-primary').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('loads contacts on mount', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([]);
    
    mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(contactList).toHaveBeenCalled();
  });

  it('resolves local identity on mount', async () => {
    const { resolveLocalIdentity } = require('$lib/imSession');
    
    mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(resolveLocalIdentity).toHaveBeenCalled();
  });

  it('displays status message', async () => {
    const { contactList } = require('$lib/contactDirectory');
    contactList.mockResolvedValue([
      { id: '1', name: 'Alice', node_id: 'node-1', is_blocked: false, notes: '' }
    ]);
    
    const wrapper = mount(ContactDirectoryPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.hint').exists()).toBe(true);
  });
});
