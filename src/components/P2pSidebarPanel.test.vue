/**
 * Exodus Browser — P2pSidebarPanel component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import P2pSidebarPanel from './P2pSidebarPanel.vue';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => vi.fn())
}));

vi.mock('@/components/ImMessenger.vue', () => ({
  default: {
    name: 'ImMessenger',
    template: '<div class="im-messenger"><slot /></div>',
    emits: ['status']
  }
}));

vi.mock('@/components/ContactDirectoryPanel.vue', () => ({
  default: {
    name: 'ContactDirectoryPanel',
    template: '<div class="contact-directory"><slot /></div>',
    emits: ['status']
  }
}));

vi.mock('@/components/settings/GroupChatSettings.vue', () => ({
  default: {
    name: 'GroupChatSettings',
    template: '<div class="group-chat-settings"><slot /></div>',
    props: ['groupId'],
    emits: ['status']
  }
}));

vi.mock('@/components/settings/P2pCdnSettings.vue', () => ({
  default: {
    name: 'P2pCdnSettings',
    template: '<div class="p2p-cdn-settings"><slot /></div>'
  }
}));

vi.mock('@/components/FileTransfer.vue', () => ({
  default: {
    name: 'FileTransfer',
    template: '<div class="file-transfer"><slot /></div>',
    emits: ['status']
  }
}));

vi.mock('@/components/CollaborativeEditing.vue', () => ({
  default: {
    name: 'CollaborativeEditing',
    template: '<div class="collaborative-editing"><slot /></div>',
    emits: ['status']
  }
}));

vi.mock('@/components/VideoCall.vue', () => ({
  default: {
    name: 'VideoCall',
    template: '<div class="video-call"><slot /></div>',
    emits: ['status']
  }
}));

vi.mock('@/components/MeetingRoom.vue', () => ({
  default: {
    name: 'MeetingRoom',
    template: '<div class="meeting-room"><slot /></div>',
    emits: ['status']
  }
}));

describe('P2pSidebarPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockProps = {
    roomId: 'test-room'
  };

  it('renders p2p sidebar', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.p2p-sidebar').exists()).toBe(true);
  });

  it('renders sub tabs', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.sub-tabs').exists()).toBe(true);
  });

  it('has correct ARIA role on sub tabs', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.sub-tabs').attributes('role')).toBe('tablist');
  });

  it('renders all tab buttons', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons.length).toBe(8);
  });

  it('has correct ARIA role on tab buttons', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.sub-tabs button');
    buttons.forEach(button => {
      expect(button.attributes('role')).toBe('tab');
    });
  });

  it('displays correct tab labels', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons[0].text()).toBe('WebChat');
    expect(buttons[1].text()).toBe('Contacts');
    expect(buttons[2].text()).toBe('Group');
    expect(buttons[3].text()).toBe('CDN');
    expect(buttons[4].text()).toBe('WorkSpace');
    expect(buttons[5].text()).toBe('Collab');
    expect(buttons[6].text()).toBe('Call');
    expect(buttons[7].text()).toBe('Meeting');
  });

  it('applies active class to current tab', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons[0].classes()).toContain('active');
  });

  it('does not apply active class to other tabs', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons[1].classes()).not.toContain('active');
  });

  it('switches tab on click', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons[1].classes()).toContain('active');
    expect(buttons[0].classes()).not.toContain('active');
  });

  it('renders panel body', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.panel-body').exists()).toBe(true);
  });

  it('renders ImMessenger for IM tab', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    expect(wrapper.findComponent({ name: 'ImMessenger' }).exists()).toBe(true);
  });

  it('renders ContactDirectoryPanel for contacts tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'ContactDirectoryPanel' }).exists()).toBe(true);
  });

  it('renders GroupChatSettings for chat tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[2].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'GroupChatSettings' }).exists()).toBe(true);
  });

  it('passes roomId to GroupChatSettings', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[2].trigger('click');
    await wrapper.vm.$nextTick();
    
    const groupSettings = wrapper.findComponent({ name: 'GroupChatSettings' });
    expect(groupSettings.props('groupId')).toBe('test-room');
  });

  it('renders P2pCdnSettings for CDN tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[3].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'P2pCdnSettings' }).exists()).toBe(true);
  });

  it('renders FileTransfer for workspace tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[4].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'FileTransfer' }).exists()).toBe(true);
  });

  it('renders CollaborativeEditing for collab tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[5].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'CollaborativeEditing' }).exists()).toBe(true);
  });

  it('renders VideoCall for call tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[6].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'VideoCall' }).exists()).toBe(true);
  });

  it('renders MeetingRoom for meeting tab', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[7].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findComponent({ name: 'MeetingRoom' }).exists()).toBe(true);
  });

  it('forwards status events from child components', async () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findComponent({ name: 'ImMessenger' }).vm.$emit('status', 'Test status');
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Test status']);
  });

  it('uses default roomId when not provided', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: {}
    });
    
    expect(wrapper.vm.roomId).toBe('lobby');
  });

  it('uses provided roomId', () => {
    const wrapper = mount(P2pSidebarPanel, {
      props: { roomId: 'custom-room' }
    });
    
    expect(wrapper.vm.roomId).toBe('custom-room');
  });

  it('sets up event listeners on mount', () => {
    const { listen } = require('@tauri-apps/api/event');
    listen.mockResolvedValue(vi.fn());
    
    mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    expect(listen).toHaveBeenCalledWith('exodus-focus-im', expect.any(Function));
    expect(listen).toHaveBeenCalledWith('exodus-focus-workspace', expect.any(Function));
  });

  it('switches to IM tab on exodus-focus-im event', async () => {
    const { listen } = require('@tauri-apps/api/event');
    let imCallback: Function;
    listen.mockImplementation(async (event: string, callback: Function) => {
      if (event === 'exodus-focus-im') imCallback = callback;
      return vi.fn();
    });
    
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.sub-tabs button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    if (imCallback) imCallback();
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons[0].classes()).toContain('active');
  });

  it('switches to workspace tab on exodus-focus-workspace event', async () => {
    const { listen } = require('@tauri-apps/api/event');
    let workspaceCallback: Function;
    listen.mockImplementation(async (event: string, callback: Function) => {
      if (event === 'exodus-focus-workspace') workspaceCallback = callback;
      return vi.fn();
    });
    
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    if (workspaceCallback) workspaceCallback();
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.sub-tabs button');
    expect(buttons[4].classes()).toContain('active');
  });

  it('cleans up event listeners on unmount', () => {
    const { listen } = require('@tauri-apps/api/event');
    const unlisten = vi.fn();
    listen.mockResolvedValue(unlisten);
    
    const wrapper = mount(P2pSidebarPanel, {
      props: mockProps
    });
    
    wrapper.unmount();
    
    expect(unlisten).toHaveBeenCalled();
  });
});
