import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import BrowserTabBar from './BrowserTabBar.vue';

vi.mock('$lib/platformChrome', () => ({
  isMacTauriOverlayTitlebar: vi.fn(() => false),
}));

describe('BrowserTabBar', () => {
  it('renders Chrome-aligned tab strip class', () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [{ id: '1', title: 'Tab 1', url: 'https://example.com' }],
        activeTabId: '1',
        sortedTabs: [{ id: '1', title: 'Tab 1', url: 'https://example.com' }],
      },
    });
    expect(wrapper.find('.exodus-chrome-tabstrip').exists()).toBe(true);
  });

  it('renders tabs correctly', () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [
          { id: '1', title: 'Tab 1', url: 'https://example.com' },
          { id: '2', title: 'Tab 2', url: 'https://example.org' },
        ],
        activeTabId: '1',
        sortedTabs: [
          { id: '1', title: 'Tab 1', url: 'https://example.com' },
          { id: '2', title: 'Tab 2', url: 'https://example.org' },
        ],
      },
    });

    expect(wrapper.findAll('.tab-item').length).toBe(2);
    expect(wrapper.findAll('.tab-favicon').length).toBe(2);
    expect(wrapper.findAll('.tab-title').length).toBe(2);
    expect(wrapper.find('.tab-title').text()).toBe('Tab 1');
  });

  it('shows icon title and close on new-tab page (Chrome)', () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [{ id: '1', title: 'New Tab', url: 'about:blank#exodus-new-tab' }],
        activeTabId: '1',
        sortedTabs: [{ id: '1', title: 'New Tab', url: 'about:blank#exodus-new-tab' }],
      },
    });
    expect(wrapper.find('.tab-close').exists()).toBe(true);
    expect(wrapper.find('.tab-title').text()).toBe('New Tab');
    expect(wrapper.find('.tab-favicon').attributes('src')).toContain('data:image');
  });

  it('shows close button when multiple tabs', () => {
    const tabList = [
      { id: '1', title: 'A', url: 'https://a.com' },
      { id: '2', title: 'B', url: 'https://b.com' },
    ];
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: tabList,
        activeTabId: '1',
        sortedTabs: tabList,
      },
    });
    expect(wrapper.findAll('.tab-close').length).toBeGreaterThan(0);
  });

  it('shows tab titles in vertical mode', () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [{ id: '1', title: 'Tab 1', url: 'https://example.com' }],
        activeTabId: '1',
        sortedTabs: [{ id: '1', title: 'Tab 1', url: 'https://example.com' }],
        vertical: true,
      },
    });
    expect(wrapper.find('.tab-title').text()).toBe('Tab 1');
  });

  it('emits switchTab event when clicking a tab', async () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [
          { id: '1', title: 'Tab 1', url: 'https://example.com' },
        ],
        activeTabId: '1',
        sortedTabs: [
          { id: '1', title: 'Tab 1', url: 'https://example.com' },
        ],
      },
    });
    
    await wrapper.find('.tab-item').trigger('click');
    expect(wrapper.emitted('switchTab')).toBeTruthy();
  });

  it('emits reorderTabs when dropping a tab onto another', async () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [
          { id: '1', title: 'Tab 1', url: 'https://a.com' },
          { id: '2', title: 'Tab 2', url: 'https://b.com' },
        ],
        activeTabId: '1',
        sortedTabs: [
          { id: '1', title: 'Tab 1', url: 'https://a.com' },
          { id: '2', title: 'Tab 2', url: 'https://b.com' },
        ],
      },
    });
    const items = wrapper.findAll('.tab-item');
    await items[0].trigger('dragstart', { dataTransfer: { effectAllowed: 'move', setData: () => {} } });
    await items[1].trigger('drop', { preventDefault: () => {}, dataTransfer: { getData: () => '1' } });
    expect(wrapper.emitted('reorderTabs')).toBeTruthy();
  });

  it('emits newTab event when clicking new tab button', async () => {
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: [],
        activeTabId: '',
        sortedTabs: [],
      },
    });
    
    await wrapper.find('.tab-new').trigger('click');
    expect(wrapper.emitted('newTab')).toBeTruthy();
  });

  it('emits closeTab event when clicking close button', async () => {
    const tabList = [
      { id: '1', title: 'Tab 1', url: 'https://example.com' },
      { id: '2', title: 'Tab 2', url: 'https://example.org' },
    ];
    const wrapper = mount(BrowserTabBar, {
      props: {
        tabs: tabList,
        activeTabId: '1',
        sortedTabs: tabList,
      },
    });

    await wrapper.find('.tab-close').trigger('click');
    expect(wrapper.emitted('closeTab')).toBeTruthy();
  });
});
