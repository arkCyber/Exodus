/**
 * Exodus Browser — ContactDirectoryPanel tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import ContactDirectoryPanel from './ContactDirectoryPanel.vue';

vi.mock('$lib/contactDirectory', () => ({
  buildHumanContact: vi.fn((p) => ({ ...p, contact_id: 'new-id' })),
  contactAdd: vi.fn(),
  contactDirectoryServiceStart: vi.fn(),
  contactList: vi.fn(async () => [
    {
      contact_id: 'c-1',
      name: 'Alice',
      contact_type: 'human',
      agent_ids: [],
      node_id: 'node-alice',
      groups: [],
      tags: [],
      notes: '',
      is_favorite: false,
      is_blocked: false,
      created_at: 0,
      last_contacted: 0,
      contact_count: 0,
    },
  ]),
}));

vi.mock('$lib/imChat', () => ({
  openImChat: vi.fn(),
  openP2pTab: vi.fn(),
  openWebChat: vi.fn(),
  startCallFromUi: vi.fn(),
}));

vi.mock('$lib/imSession', () => ({
  resolveLocalIdentity: vi.fn(async () => ({
    userId: 'user-1',
    displayName: 'Me',
    nodeId: 'node-local',
  })),
}));

vi.mock('$lib/presence', () => ({
  fetchOnlinePeers: vi.fn(async () => new Map()),
  isNodeOnline: vi.fn(() => false),
}));

describe('ContactDirectoryPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('opens chat with contact_id not id', async () => {
    const { openImChat, openP2pTab, openWebChat } = await import('$lib/imChat');
    const wrapper = mount(ContactDirectoryPanel);
    await flushPromises();

    const chatBtn = wrapper.findAll('button').find((b) => b.text() === 'Chat');
    expect(chatBtn).toBeTruthy();
    await chatBtn!.trigger('click');

    expect(openP2pTab).toHaveBeenCalledWith('im');
    expect(openWebChat).toHaveBeenCalled();
    expect(openImChat).toHaveBeenCalledWith({
      contactId: 'c-1',
      name: 'Alice',
      nodeId: 'node-alice',
    });
  });
});
