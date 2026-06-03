/**
 * Exodus Browser — StatusBar component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import StatusBar from './StatusBar.vue';

describe('StatusBar', () => {
  it('does not render when no props are set', () => {
    const wrapper = mount(StatusBar);
    
    expect(wrapper.find('.status-bar').exists()).toBe(false);
  });

  it('renders when message is provided', () => {
    const wrapper = mount(StatusBar, {
      props: { message: 'Status message' }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.message').text()).toBe('Status message');
  });

  it('renders when private mode is enabled', () => {
    const wrapper = mount(StatusBar, {
      props: { privateMode: true }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-private').exists()).toBe(true);
    expect(wrapper.find('.badge-private').text()).toBe('Private');
  });

  it('renders when HTTPS only is enabled', () => {
    const wrapper = mount(StatusBar, {
      props: { httpsOnly: true }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-https').exists()).toBe(true);
    expect(wrapper.find('.badge-https').text()).toBe('HTTPS only');
  });

  it('renders when popups are blocked', () => {
    const wrapper = mount(StatusBar, {
      props: { blockPopups: true }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-popup').exists()).toBe(true);
    expect(wrapper.find('.badge-popup').text()).toBe('Popups blocked');
  });

  it('renders tracker count when trackers are blocked', () => {
    const wrapper = mount(StatusBar, {
      props: {
        privacyStats: {
          trackers_blocked: 42,
          trackers_allowed: 10,
          fingerprinting_blocked: 5,
          fingerprinting_allowed: 2
        }
      }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-trackers').exists()).toBe(true);
    expect(wrapper.find('.badge-trackers').text()).toBe('42 trackers blocked');
  });

  it('does not render tracker badge when no trackers blocked', () => {
    const wrapper = mount(StatusBar, {
      props: {
        privacyStats: {
          trackers_blocked: 0,
          trackers_allowed: 10,
          fingerprinting_blocked: 0,
          fingerprinting_allowed: 2
        }
      }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(false);
  });

  it('renders all badges when multiple modes are enabled', () => {
    const wrapper = mount(StatusBar, {
      props: {
        privateMode: true,
        httpsOnly: true,
        blockPopups: true,
        privacyStats: {
          trackers_blocked: 10,
          trackers_allowed: 0,
          fingerprinting_blocked: 0,
          fingerprinting_allowed: 0
        }
      }
    });
    
    expect(wrapper.find('.badge-private').exists()).toBe(true);
    expect(wrapper.find('.badge-https').exists()).toBe(true);
    expect(wrapper.find('.badge-popup').exists()).toBe(true);
    expect(wrapper.find('.badge-trackers').exists()).toBe(true);
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(StatusBar, {
      props: { message: 'Test' }
    });
    
    expect(wrapper.find('.status-bar').attributes('role')).toBe('status');
    expect(wrapper.find('.status-bar').attributes('aria-live')).toBe('polite');
  });

  it('displays badges container', () => {
    const wrapper = mount(StatusBar, {
      props: { privateMode: true }
    });
    
    expect(wrapper.find('.badges').exists()).toBe(true);
  });

  it('has correct title attributes for badges', () => {
    const wrapper = mount(StatusBar, {
      props: {
        privateMode: true,
        httpsOnly: true,
        blockPopups: true
      }
    });
    
    expect(wrapper.find('.badge-private').attributes('title')).toBe('Private browsing — visits are not recorded');
    expect(wrapper.find('.badge-https').attributes('title')).toBe('HTTPS-only mode');
    expect(wrapper.find('.badge-popup').attributes('title')).toBe('Popup windows are blocked');
  });

  it('renders online status badge', () => {
    const wrapper = mount(StatusBar, {
      props: { isOnline: true }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-network').exists()).toBe(true);
    expect(wrapper.find('.badge-network--online').exists()).toBe(true);
    expect(wrapper.find('.badge-network').text()).toBe('Online');
  });

  it('renders offline status badge', () => {
    const wrapper = mount(StatusBar, {
      props: { isOnline: false }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-network').exists()).toBe(true);
    expect(wrapper.find('.badge-network--offline').exists()).toBe(true);
    expect(wrapper.find('.badge-network').text()).toBe('Offline');
  });

  it('renders AI model badge with correct format', () => {
    const wrapper = mount(StatusBar, {
      props: { aiModel: 'qwen3.6-35b-a3b' }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-model').exists()).toBe(true);
    expect(wrapper.find('.badge-model').text()).toBe('AI-Model: qwen3.6-35b-a3b');
    expect(wrapper.find('.badge-model').attributes('title')).toBe('Using AI model: qwen3.6-35b-a3b');
  });

  it('renders agent executing badge', () => {
    const wrapper = mount(StatusBar, {
      props: { isAgentExecuting: true }
    });
    
    expect(wrapper.find('.status-bar').exists()).toBe(true);
    expect(wrapper.find('.badge-agent').exists()).toBe(true);
    expect(wrapper.find('.badge-agent').text()).toBe('Agent running');
  });

  it('renders agent command in badge', () => {
    const wrapper = mount(StatusBar, {
      props: {
        isAgentExecuting: true,
        agentCommand: 'scroll down'
      }
    });
    
    expect(wrapper.find('.badge-agent').exists()).toBe(true);
    expect(wrapper.find('.badge-agent').text()).toBe('Agent: scroll down');
    expect(wrapper.find('.badge-agent').attributes('title')).toBe('Agent executing: scroll down');
  });

  it('truncates long agent command', () => {
    const wrapper = mount(StatusBar, {
      props: {
        isAgentExecuting: true,
        agentCommand: 'this is a very long command that should be truncated'
      }
    });
    
    expect(wrapper.find('.badge-agent').text()).toBe('Agent: this is a very long command...');
  });

  it('renders DOM summary badge when agent is executing', () => {
    const wrapper = mount(StatusBar, {
      props: {
        isAgentExecuting: true,
        agentDomSummary: 'Page has 3 buttons and 2 links'
      }
    });
    
    expect(wrapper.find('.badge-dom').exists()).toBe(true);
    expect(wrapper.find('.badge-dom').text()).toBe('Page has 3 buttons and 2...');
  });

  it('renders log badge when agent is executing', () => {
    const wrapper = mount(StatusBar, {
      props: {
        isAgentExecuting: true,
        agentLog: ['Step 1: Scrolling', 'Step 2: Clicking button']
      }
    });
    
    expect(wrapper.find('.badge-log').exists()).toBe(true);
    expect(wrapper.find('.badge-log').text()).toBe('Step 2: Clicking button');
  });

  it('does not render agent badges when not executing', () => {
    const wrapper = mount(StatusBar, {
      props: {
        isAgentExecuting: false,
        agentCommand: 'scroll down',
        agentDomSummary: 'Page has 3 buttons',
        agentLog: ['Log entry']
      }
    });
    
    expect(wrapper.find('.badge-agent').exists()).toBe(false);
    expect(wrapper.find('.badge-dom').exists()).toBe(false);
    expect(wrapper.find('.badge-log').exists()).toBe(false);
  });

  it('badges are ordered correctly: network, model, private, https, popup, trackers, agent', () => {
    const wrapper = mount(StatusBar, {
      props: {
        isOnline: true,
        aiModel: 'qwen3.6-35b-a3b',
        privateMode: true,
        httpsOnly: true,
        blockPopups: true,
        privacyStats: { trackers_blocked: 5, trackers_allowed: 0, fingerprinting_blocked: 0, fingerprinting_allowed: 0 },
        isAgentExecuting: true
      }
    });
    
    const badges = wrapper.findAll('.badge');
    expect(badges.length).toBe(6);
    expect(badges[0].classes()).toContain('badge-network');
    expect(badges[1].classes()).toContain('badge-model');
    expect(badges[2].classes()).toContain('badge-private');
    expect(badges[3].classes()).toContain('badge-https');
    expect(badges[4].classes()).toContain('badge-popup');
    expect(badges[5].classes()).toContain('badge-trackers');
  });
});
