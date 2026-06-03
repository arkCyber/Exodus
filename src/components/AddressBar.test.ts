import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import AddressBar from './AddressBar.vue';
import { CHROME_LAYOUT } from '$lib/chromeLayout';

vi.mock('@/components/ExtensionActionBar.vue', () => ({
  default: {
    name: 'ExtensionActionBar',
    props: ['inline', 'refreshKey'],
    template: '<div class="extension-toolbar-stub" />',
  },
}));

describe('AddressBar', () => {
  it('renders navigation buttons', () => {
    const wrapper = mount(AddressBar, {
      props: {
        canGoBack: true,
        canGoForward: true,
        currentUrl: 'https://example.com',
        isBookmarked: false,
      },
    });
    
    expect(wrapper.find('.nav-icon-btn').exists()).toBe(true);
    expect(wrapper.find('.url-input').exists()).toBe(true);
  });

  it('renders 16px nav SVG icons via chrome layout class', () => {
    const wrapper = mount(AddressBar, {
      props: {
        canGoBack: true,
        canGoForward: false,
        currentUrl: '',
        isBookmarked: false,
      },
    });
    expect(wrapper.find('.nav-svg').exists()).toBe(true);
    expect(wrapper.find('.toolbar-svg').exists()).toBe(true);
  });

  it('uses Chrome-aligned toolbar class', () => {
    const wrapper = mount(AddressBar, {
      props: {
        canGoBack: false,
        canGoForward: false,
        currentUrl: '',
        isBookmarked: false,
      },
    });
    expect(wrapper.find('.exodus-chrome-toolbar').exists()).toBe(true);
    expect(wrapper.find('.chrome-drag-surface').exists()).toBe(true);
    expect(wrapper.find('.exodus-address-bar').attributes('data-tauri-drag-region')).toBeDefined();
  });

  it('matches Chrome-aligned toolbar and icon tokens', () => {
    expect(CHROME_LAYOUT.toolbarHeight).toBe(48);
    expect(CHROME_LAYOUT.omniboxHeight).toBe(34);
    expect(CHROME_LAYOUT.toolbarButtonSize).toBe(32);
    expect(CHROME_LAYOUT.toolbarIconSize).toBe(16);
  });

  it('emits navigate event on Enter key', async () => {
    const wrapper = mount(AddressBar, {
      props: {
        canGoBack: false,
        canGoForward: false,
        currentUrl: 'https://example.com',
        isBookmarked: false,
      },
    });
    
    const input = wrapper.find('.url-input');
    await input.trigger('keydown', { key: 'Enter' });
    
    expect(wrapper.emitted('navigate')).toBeTruthy();
  });

  it('shows site indicator for HTTPS URLs', () => {
    const wrapper = mount(AddressBar, {
      props: {
        canGoBack: false,
        canGoForward: false,
        currentUrl: 'https://example.com',
        isBookmarked: false,
      },
    });

    expect(wrapper.find('.site-indicator.secure').exists()).toBe(true);
    expect(wrapper.find('.site-indicator-icon').exists()).toBe(true);
  });

  it('shows site indicator for HTTP URLs', () => {
    const wrapper = mount(AddressBar, {
      props: {
        canGoBack: false,
        canGoForward: false,
        currentUrl: 'http://example.com',
        isBookmarked: false,
      },
    });

    expect(wrapper.find('.site-indicator').exists()).toBe(true);
  });

  it('shows P2P CDN badge when label provided', () => {
    const wrapper = mount(AddressBar, {
      props: {
        currentUrl: 'https://example.com',
        cdnStatusLabel: 'P2P · 2',
      },
    });
    expect(wrapper.find('.cdn-omnibox-badge').text()).toContain('P2P · 2');
  });

  it('shows chrome menu with closed tab count', async () => {
    const wrapper = mount(AddressBar, {
      props: {
        currentUrl: 'https://example.com',
        showMenu: true,
        closedTabsCount: 2,
      },
    });
    expect(wrapper.text()).toContain('Reopen closed tab');
    expect(wrapper.text()).toContain('(2)');
  });

  it('emits saveToReadingList from chrome menu', async () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com', showMenu: true },
    });
    const btn = wrapper
      .findAll('.menu-item')
      .find((b) => b.text().includes('Save to reading list'));
    expect(btn).toBeTruthy();
    await btn!.trigger('click');
    expect(wrapper.emitted('saveToReadingList')).toHaveLength(1);
    expect(wrapper.emitted('closeMenu')).toBeTruthy();
  });

  it('emits openSidebarCustomize from chrome menu', async () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com', showMenu: true },
    });
    const btn = wrapper
      .findAll('.menu-item')
      .find((b) => b.text().includes('Customize sidebar'));
    expect(btn).toBeTruthy();
    await btn!.trigger('click');
    expect(wrapper.emitted('openSidebarCustomize')).toHaveLength(1);
  });

  it('emits toggleWebChat when WebChat toolbar button is clicked', async () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' },
    });
    await wrapper.find('.webchat-toggle-btn').trigger('click');
    expect(wrapper.emitted('toggleWebChat')).toHaveLength(1);
  });

  it('embeds Chrome-style extension toolbar strip before menu', () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' },
    });
    expect(wrapper.find('.toolbar-end .extension-toolbar-stub').exists()).toBe(true);
    expect(wrapper.find('.toolbar-end .chrome-menu-btn').exists()).toBe(true);
  });
});
