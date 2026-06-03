/**
 * Exodus Browser — StatusBar component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import StatusBar from './StatusBar.vue';

describe('StatusBar', () => {
  it('renders when message is set', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: 'Ready',
        privateMode: false,
        httpsOnly: false,
        blockPopups: false,
        privacyStats: null,
      },
    });

    expect(wrapper.find('.status-bar').exists()).toBe(true);
  });

  it('does not render when no badges or message', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: '',
        privateMode: false,
        httpsOnly: false,
        blockPopups: false,
        privacyStats: null,
      },
    });

    expect(wrapper.find('.status-bar').exists()).toBe(false);
  });

  it('displays status message', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: 'Loading...',
        privateMode: false,
        httpsOnly: false,
        blockPopups: false,
        privacyStats: null,
      },
    });

    expect(wrapper.text()).toContain('Loading...');
  });

  it('displays private mode badge when enabled', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: '',
        privateMode: true,
        httpsOnly: false,
        blockPopups: false,
        privacyStats: null,
      },
    });

    expect(wrapper.text()).toContain('Private');
  });

  it('displays HTTPS-only badge when enabled', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: '',
        privateMode: false,
        httpsOnly: true,
        blockPopups: false,
        privacyStats: null,
      },
    });

    expect(wrapper.text()).toContain('HTTPS only');
  });

  it('displays popup blocker badge when enabled', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: '',
        privateMode: false,
        httpsOnly: false,
        blockPopups: true,
        privacyStats: null,
      },
    });

    expect(wrapper.text()).toContain('Popups blocked');
  });

  it('displays tracker badge when stats show blocks', () => {
    const wrapper = mount(StatusBar, {
      props: {
        message: '',
        privateMode: false,
        httpsOnly: false,
        blockPopups: false,
        privacyStats: {
          trackers_blocked: 3,
          trackers_allowed: 0,
          fingerprinting_blocked: 0,
          fingerprinting_allowed: 0,
        },
      },
    });

    expect(wrapper.text()).toContain('3 trackers blocked');
  });
});
