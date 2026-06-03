/**
 * Exodus Browser — SidebarAiPanel component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import SidebarAiPanel from './SidebarAiPanel.vue';

describe('SidebarAiPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders AI chat panel', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-panel').exists()).toBe(true);
  });

  it('renders placeholder when no history and not loading', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.sidebar-placeholder').exists()).toBe(true);
  });

  it('displays placeholder text', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.sidebar-placeholder p').text()).toContain('Ask Exodus anything');
  });

  it('renders open agent button', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.sidebar-placeholder .nav-button').text()).toBe('Open Agent');
  });

  it('emits toggle-agent on open agent button click', async () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    await wrapper.find('.sidebar-placeholder .nav-button').trigger('click');
    
    expect(wrapper.emitted('toggle-agent')).toBeTruthy();
  });

  it('renders open P2P room button when can announce', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: true
      }
    });
    
    const buttons = wrapper.findAll('.sidebar-placeholder .nav-button');
    expect(buttons[1].text()).toBe('Open P2P room');
  });

  it('does not render P2P button when cannot announce', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    const buttons = wrapper.findAll('.sidebar-placeholder .nav-button');
    expect(buttons.length).toBe(1);
  });

  it('emits open-p2p on P2P button click', async () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: true
      }
    });
    
    const buttons = wrapper.findAll('.sidebar-placeholder .nav-button');
    await buttons[1].trigger('click');
    
    expect(wrapper.emitted('open-p2p')).toBeTruthy();
  });

  it('renders chat messages when history exists', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [{ role: 'user', content: 'Hello' }],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-messages').exists()).toBe(true);
  });

  it('renders user message bubble', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [{ role: 'user', content: 'Hello' }],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.chat-bubble.user').text()).toBe('Hello');
  });

  it('renders assistant message bubble', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [{ role: 'assistant', content: 'Hi there' }],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.chat-bubble.assistant').text()).toBe('Hi there');
  });

  it('renders streaming buffer when loading', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: 'Thinking...',
        aiStreamMode: 'chat',
        isLoading: true,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.chat-bubble.streaming').text()).toBe('Thinking...');
  });

  it('does not render streaming buffer when not in chat mode', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: 'Thinking...',
        aiStreamMode: 'summary',
        isLoading: true,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.chat-bubble.streaming').exists()).toBe(false);
  });

  it('renders chat form', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-form').exists()).toBe(true);
  });

  it('renders chat input', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-input').exists()).toBe(true);
  });

  it('has correct placeholder when AI online', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-input').attributes('placeholder')).toBe('Ask Exodus…');
  });

  it('has correct placeholder when AI offline', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: false,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-input').attributes('placeholder')).toBe('AI offline — check settings');
  });

  it('disables input when loading', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: true,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    expect(wrapper.find('.ai-chat-input').attributes('disabled')).toBeDefined();
  });

  it('emits chat-input on input change', async () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    await wrapper.find('.ai-chat-input').setValue('Hello');
    
    expect(wrapper.emitted('chat-input')).toBeTruthy();
    expect(wrapper.emitted('chat-input')?.[0]).toEqual(['Hello']);
  });

  it('renders send button when not loading', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    const buttons = wrapper.findAll('.ai-chat-form .nav-button');
    expect(buttons[1].text()).toBe('Send');
  });

  it('renders stop button when loading', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: true,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    const buttons = wrapper.findAll('.ai-chat-form .nav-button');
    expect(buttons[0].text()).toBe('Stop');
  });

  it('disables send button when input is empty', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    const buttons = wrapper.findAll('.ai-chat-form .nav-button');
    expect(buttons[1].attributes('disabled')).toBeDefined();
  });

  it('enables send button when input has text', () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: 'Hello',
        canAnnouncePage: false
      }
    });
    
    const buttons = wrapper.findAll('.ai-chat-form .nav-button');
    expect(buttons[1].attributes('disabled')).toBeUndefined();
  });

  it('emits send-chat on form submit', async () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: false,
        aiOnline: true,
        aiChatInput: 'Hello',
        canAnnouncePage: false
      }
    });
    
    await wrapper.find('.ai-chat-form').trigger('submit');
    
    expect(wrapper.emitted('send-chat')).toBeTruthy();
  });

  it('emits cancel-chat on stop button click', async () => {
    const wrapper = mount(SidebarAiPanel, {
      props: {
        aiChatHistory: [],
        chatStreamBuffer: '',
        aiStreamMode: 'none',
        isLoading: true,
        aiOnline: true,
        aiChatInput: '',
        canAnnouncePage: false
      }
    });
    
    await wrapper.findAll('.ai-chat-form .nav-button')[0].trigger('click');
    
    expect(wrapper.emitted('cancel-chat')).toBeTruthy();
  });
});
