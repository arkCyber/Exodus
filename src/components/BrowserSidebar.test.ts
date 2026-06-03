import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';

vi.mock('@/components/P2pSidebarPanel.vue', () => ({
  default: { name: 'P2pSidebarPanel', template: '<div class="p2p-mock" />' },
}));

vi.mock('@/components/PocketPanel.vue', () => ({
  default: { name: 'PocketPanel', template: '<div class="pocket-mock" />' },
}));

vi.mock('@/components/sidebar/SidebarVerticalTabsPanel.vue', () => ({
  default: { name: 'SidebarVerticalTabsPanel', template: '<div class="vtabs-mock" />' },
}));

vi.mock('@/components/sidebar/SidebarSyncedTabsPanel.vue', () => ({
  default: { name: 'SidebarSyncedTabsPanel', template: '<div class="synced-mock" />' },
}));

vi.mock('@/components/sidebar/SidebarReadingListPanel.vue', () => ({
  default: { name: 'SidebarReadingListPanel', template: '<div class="reading-mock" />' },
}));

vi.mock('@/components/sidebar/SidebarCustomizePanel.vue', () => ({
  default: { name: 'SidebarCustomizePanel', template: '<div class="customize-mock" />' },
}));

import BrowserSidebar from './BrowserSidebar.vue';
import { loadSidebarPreferences } from '$lib/sidebarPreferences';
import { sidebarIconItemsFromPrefs } from '$lib/sidebarIcons';

describe('BrowserSidebar', () => {
  const sidebarPrefs = loadSidebarPreferences();

  const baseProps = {
    open: true,
    sidebarPanel: 'ai' as const,
    sidebarPosition: 'right' as const,
    iconItems: sidebarIconItemsFromPrefs(sidebarPrefs),
    sidebarPrefs,
    agentPanelOpen: false,
    aiChatHistory: [],
    chatStreamBuffer: '',
    aiStreamMode: 'none' as const,
    isLoading: false,
    aiOnline: true,
    aiChatInput: '',
    agentCommand: '',
    agentLog: [],
    agentDomSummary: '',
    isAgentExecuting: false,
    indexedMemoryGroups: [],
    historyGroups: [],
    indexedCount: 0,
    historyCount: 0,
    bookmarks: [],
    p2pRoomId: 'lobby',
    canAnnouncePage: false,
    tabContextMenu: null,
    tabGroups: [],
    tabBarHandlers: undefined,
    openTabsForSync: [],
  };

  it('renders AI panel by default', () => {
    const wrapper = mount(BrowserSidebar, { props: baseProps });
    expect(wrapper.text()).toContain('Ask Exodus');
  });

  it('emits open-panel for history icon', async () => {
    const wrapper = mount(BrowserSidebar, { props: baseProps });
    const buttons = wrapper.findAll('.sidebar-icon-bar .icon-list .sidebar-icon-btn');
    const memoryIdx = sidebarPrefs.enabledTools.indexOf('memory');
    await buttons[memoryIdx].trigger('click');
    expect(wrapper.emitted('open-panel')?.[0]).toEqual(['memory']);
  });

  it('renders Firefox-style SVG icons in the rail', () => {
    const wrapper = mount(BrowserSidebar, { props: baseProps });
    expect(wrapper.findAll('.icon-svg').length).toBeGreaterThanOrEqual(5);
  });

  it('emits close on close button', async () => {
    const wrapper = mount(BrowserSidebar, { props: baseProps });
    const closeBtn = wrapper.find('.close-btn');
    await closeBtn.trigger('click');
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('renders customize panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...baseProps, sidebarPanel: 'customize' },
    });
    expect(wrapper.find('.customize-mock').exists()).toBe(true);
  });

  it('collapses content panel when collapse control is used', async () => {
    const wrapper = mount(BrowserSidebar, { props: baseProps });
    expect(wrapper.find('.sidebar-content').isVisible()).toBe(true);
    const footerBtns = wrapper.findAll('.icon-footer .sidebar-icon-btn');
    await footerBtns[1].trigger('click');
    expect(wrapper.classes()).toContain('exodus-sidebar--collapsed');
  });
});
