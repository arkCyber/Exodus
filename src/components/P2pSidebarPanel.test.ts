import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('@/components/ImMessenger.vue', () => ({
  default: { name: 'ImMessenger', template: '<div class="im-stub" />' },
}));
vi.mock('@/components/ContactDirectoryPanel.vue', () => ({
  default: { name: 'ContactDirectoryPanel', template: '<div class="contacts-stub" />' },
}));
vi.mock('@/components/settings/GroupChatSettings.vue', () => ({
  default: { name: 'GroupChatSettings', template: '<div class="chat-stub" />' },
}));
vi.mock('@/components/settings/P2pCdnSettings.vue', () => ({
  default: { name: 'P2pCdnSettings', template: '<div class="cdn-stub" />' },
}));
vi.mock('@/components/FileTransfer.vue', () => ({
  default: { name: 'FileTransfer', template: '<div class="ft-stub" />' },
}));
vi.mock('@/components/CollaborativeEditing.vue', () => ({
  default: { name: 'CollaborativeEditing', template: '<div class="collab-stub" />' },
}));
vi.mock('@/components/VideoCall.vue', () => ({
  default: { name: 'VideoCall', template: '<div class="call-stub" />' },
}));
vi.mock('@/components/MeetingRoom.vue', () => ({
  default: { name: 'MeetingRoom', template: '<div class="meeting-stub" />' },
}));

import P2pSidebarPanel from './P2pSidebarPanel.vue';

describe('P2pSidebarPanel', () => {
  it('shows all P2P sub-tabs', () => {
    const wrapper = mount(P2pSidebarPanel);
    expect(wrapper.text()).toContain('WebChat');
    expect(wrapper.text()).toContain('WorkSpace');
    expect(wrapper.text()).toContain('Collab');
    expect(wrapper.text()).toContain('Call');
    expect(wrapper.text()).toContain('Meeting');
  });

  it('switches to workspace panel', async () => {
    const wrapper = mount(P2pSidebarPanel);
    const btn = wrapper.findAll('button').find((b) => b.text() === 'WorkSpace');
    expect(btn).toBeTruthy();
    await btn!.trigger('click');
    expect(wrapper.find('.ft-stub').exists()).toBe(true);
  });

  it('switches to WebChat tab on exodus-p2p-tab event', async () => {
    const { P2P_TAB_EVENT } = await import('$lib/imChat');
    const wrapper = mount(P2pSidebarPanel);
    await wrapper.findAll('button').find((b) => b.text() === 'WorkSpace')!.trigger('click');
    expect(wrapper.find('.im-stub').exists()).toBe(false);

    window.dispatchEvent(new CustomEvent(P2P_TAB_EVENT, { detail: 'im' }));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('.im-stub').exists()).toBe(true);
  });
});
