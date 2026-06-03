/**
 * Exodus Browser — BrowserSidebar component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import BrowserSidebar from './BrowserSidebar.vue';

vi.mock('@/components/AgentPanel.vue', () => ({
  default: {
    name: 'AgentPanel',
    template: '<div class="agent-panel"><slot /></div>',
    props: ['command', 'log', 'executing', 'dom-summary'],
    emits: ['execute', 'compress', 'back', 'preset', 'command-change', 'ask-ai', 'run-strategy', 'strategy-saved']
  }
}));

vi.mock('@/components/PocketPanel.vue', () => ({
  default: {
    name: 'PocketPanel',
    template: '<div class="pocket-panel"><slot /></div>',
    emits: ['status']
  }
}));

vi.mock('@/components/P2pSidebarPanel.vue', () => ({
  default: {
    name: 'P2pSidebarPanel',
    template: '<div class="p2p-panel"><slot /></div>',
    props: ['roomId'],
    emits: ['status']
  }
}));

vi.mock('@/components/sidebar/SidebarAiPanel.vue', () => ({
  default: {
    name: 'SidebarAiPanel',
    template: '<div class="ai-panel"><slot /></div>',
    props: ['aiChatHistory', 'chatStreamBuffer', 'aiStreamMode', 'isLoading', 'aiOnline', 'aiChatInput', 'canAnnouncePage'],
    emits: ['navigate', 'send-chat', 'cancel-chat', 'toggle-agent', 'open-p2p', 'chat-input']
  }
}));

vi.mock('@/components/sidebar/SidebarMemoryPanel.vue', () => ({
  default: {
    name: 'SidebarMemoryPanel',
    template: '<div class="memory-panel"><slot /></div>',
    props: ['indexedMemoryGroups', 'historyGroups', 'indexedCount', 'historyCount'],
    emits: ['navigate', 'load-memory', 'remove-indexed', 'clear-indexed', 'clear-history']
  }
}));

describe('BrowserSidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockProps = {
    open: true,
    sidebarPanel: 'ai' as const,
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
    p2pRoomId: '',
    canAnnouncePage: false
  };

  it('does not render when open is false', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, open: false }
    });
    
    expect(wrapper.find('.ai-sidebar').exists()).toBe(false);
  });

  it('renders when open is true', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.ai-sidebar').exists()).toBe(true);
  });

  it('has correct ARIA label', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.ai-sidebar').attributes('aria-label')).toBe('Exodus sidebar');
  });

  it('renders icon bar', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.sidebar-icon-bar').exists()).toBe(true);
  });

  it('renders all icon buttons', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    const iconButtons = wrapper.findAll('.icon-list .sidebar-icon-btn');
    expect(iconButtons.length).toBe(5);
  });

  it('renders close button', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.close-btn').exists()).toBe(true);
    expect(wrapper.find('.close-btn').text()).toBe('×');
  });

  it('has aria-label on close button', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.close-btn').attributes('aria-label')).toBe('Close sidebar');
  });

  it('emits close when close button is clicked', async () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    await wrapper.find('.close-btn').trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('applies active class to current panel icon', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    const iconButtons = wrapper.findAll('.icon-list .sidebar-icon-btn');
    expect(iconButtons[0].classes()).toContain('active');
  });

  it('does not apply active class to other panel icons', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    const iconButtons = wrapper.findAll('.icon-list .sidebar-icon-btn');
    expect(iconButtons[1].classes()).not.toContain('active');
  });

  it('emits open-panel when icon button is clicked', async () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    await wrapper.findAll('.icon-list .sidebar-icon-btn')[1].trigger('click');
    
    expect(wrapper.emitted('open-panel')).toBeTruthy();
    expect(wrapper.emitted('open-panel')?.[0]).toEqual(['memory']);
  });

  it('renders sidebar content', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.sidebar-content').exists()).toBe(true);
  });

  it('renders sidebar header', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('.sidebar-header').exists()).toBe(true);
  });

  it('displays correct panel title for AI panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.find('h3').text()).toBe('AI Chat');
  });

  it('displays correct panel title for memory panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'memory' as const }
    });
    
    expect(wrapper.find('h3').text()).toBe('Memory & History');
  });

  it('displays correct panel title for bookmarks panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const }
    });
    
    expect(wrapper.find('h3').text()).toBe('Bookmarks');
  });

  it('displays correct panel title for pocket panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'pocket' as const }
    });
    
    expect(wrapper.find('h3').text()).toBe('Pocket');
  });

  it('displays correct panel title for p2p panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'p2p' as const }
    });
    
    expect(wrapper.find('h3').text()).toBe('P2P CDN');
  });

  it('displays Agent title when agent panel is open', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, agentPanelOpen: true }
    });
    
    expect(wrapper.find('h3').text()).toBe('Agent');
  });

  it('renders SidebarAiPanel for AI panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    expect(wrapper.findComponent({ name: 'SidebarAiPanel' }).exists()).toBe(true);
  });

  it('renders SidebarMemoryPanel for memory panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'memory' as const }
    });
    
    expect(wrapper.findComponent({ name: 'SidebarMemoryPanel' }).exists()).toBe(true);
  });

  it('renders bookmarks panel for bookmarks panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const }
    });
    
    expect(wrapper.find('.list-panel').exists()).toBe(true);
  });

  it('renders P2pSidebarPanel for p2p panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'p2p' as const }
    });
    
    expect(wrapper.findComponent({ name: 'P2pSidebarPanel' }).exists()).toBe(true);
  });

  it('renders PocketPanel for pocket panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'pocket' as const }
    });
    
    expect(wrapper.findComponent({ name: 'PocketPanel' }).exists()).toBe(true);
  });

  it('renders AgentPanel when agent panel is open', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, agentPanelOpen: true }
    });
    
    expect(wrapper.findComponent({ name: 'AgentPanel' }).exists()).toBe(true);
  });

  it('renders bookmark search input in bookmarks panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const }
    });
    
    expect(wrapper.find('.search-input').exists()).toBe(true);
  });

  it('emits bookmark-search on input change', async () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const }
    });
    
    await wrapper.find('.search-input').setValue('test');
    
    expect(wrapper.emitted('bookmark-search')).toBeTruthy();
    expect(wrapper.emitted('bookmark-search')?.[0]).toEqual(['test']);
  });

  it('renders refresh button in bookmarks panel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const }
    });
    
    expect(wrapper.find('.nav-button').exists()).toBe(true);
    expect(wrapper.find('.nav-button').text()).toBe('Refresh');
  });

  it('emits load-bookmarks when refresh is clicked', async () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const }
    });
    
    await wrapper.find('.nav-button').trigger('click');
    
    expect(wrapper.emitted('load-bookmarks')).toBeTruthy();
  });

  it('renders bookmark items', () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    expect(wrapper.findAll('.list-item').length).toBe(1);
  });

  it('displays bookmark title', () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    expect(wrapper.find('.list-title').text()).toBe('Google');
  });

  it('displays bookmark URL', () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    expect(wrapper.find('.list-sub').text()).toBe('https://google.com');
  });

  it('displays folder input with folder value', () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    expect(wrapper.find('.folder-input').element.value).toBe('Work');
  });

  it('displays empty folder value when folder is null', () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: null }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    expect(wrapper.find('.folder-input').element.value).toBe('');
  });

  it('emits update-bookmark-folder when folder input changes', async () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    await wrapper.find('.folder-input').setValue('Personal');
    await wrapper.find('.folder-input').trigger('change');
    
    expect(wrapper.emitted('update-bookmark-folder')).toBeTruthy();
    expect(wrapper.emitted('update-bookmark-folder')?.[0]).toEqual(['1', 'Personal']);
  });

  it('emits navigate when bookmark item is clicked', async () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    await wrapper.find('.list-grow').trigger('click');
    
    expect(wrapper.emitted('navigate')).toBeTruthy();
    expect(wrapper.emitted('navigate')?.[0]).toEqual(['https://google.com']);
  });

  it('emits remove-bookmark when close button is clicked', async () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    await wrapper.find('.tab-close').trigger('click');
    
    expect(wrapper.emitted('remove-bookmark')).toBeTruthy();
    expect(wrapper.emitted('remove-bookmark')?.[0]).toEqual(['1']);
  });

  it('has aria-label on bookmark close button', () => {
    const bookmarks = [
      { id: '1', title: 'Google', url: 'https://google.com', folder: 'Work' }
    ];
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks }
    });
    
    expect(wrapper.find('.tab-close').attributes('aria-label')).toBe('Remove bookmark');
  });

  it('shows no bookmarks message when bookmarks is empty', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'bookmarks' as const, bookmarks: [] }
    });
    
    expect(wrapper.find('.muted').exists()).toBe(true);
    expect(wrapper.find('.muted').text()).toBe('No bookmarks. Use ★ in the toolbar.');
  });

  it('passes correct props to SidebarMemoryPanel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'memory' as const }
    });
    
    const memoryPanel = wrapper.findComponent({ name: 'SidebarMemoryPanel' });
    expect(memoryPanel.props('indexedMemoryGroups')).toEqual([]);
    expect(memoryPanel.props('historyGroups')).toEqual([]);
    expect(memoryPanel.props('indexedCount')).toBe(0);
    expect(memoryPanel.props('historyCount')).toBe(0);
  });

  it('passes correct props to P2pSidebarPanel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, sidebarPanel: 'p2p' as const, p2pRoomId: 'room-123' }
    });
    
    const p2pPanel = wrapper.findComponent({ name: 'P2pSidebarPanel' });
    expect(p2pPanel.props('roomId')).toBe('room-123');
  });

  it('passes correct props to AgentPanel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, agentPanelOpen: true, agentCommand: 'test command' }
    });
    
    const agentPanel = wrapper.findComponent({ name: 'AgentPanel' });
    expect(agentPanel.props('command')).toBe('test command');
  });

  it('passes correct props to SidebarAiPanel', () => {
    const wrapper = mount(BrowserSidebar, {
      props: { ...mockProps, aiChatInput: 'test input' }
    });
    
    const aiPanel = wrapper.findComponent({ name: 'SidebarAiPanel' });
    expect(aiPanel.props('aiChatInput')).toBe('test input');
  });

  it('has title attribute on icon buttons', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    const iconButtons = wrapper.findAll('.icon-list .sidebar-icon-btn');
    expect(iconButtons[0].attributes('title')).toBe('AI Chat');
    expect(iconButtons[1].attributes('title')).toBe('Memory & history');
  });

  it('displays icon glyphs', () => {
    const wrapper = mount(BrowserSidebar, {
      props: mockProps
    });
    
    const iconGlyphs = wrapper.findAll('.icon-glyph');
    expect(iconGlyphs[0].text()).toBe('AI');
    expect(iconGlyphs[1].text()).toBe('⏱');
  });
});
