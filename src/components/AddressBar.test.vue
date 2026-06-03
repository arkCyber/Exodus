/**
 * Exodus Browser — AddressBar component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import AddressBar from './AddressBar.vue';

vi.mock('@/composables/useOmnibox', () => ({
  useOmnibox: () => ({
    suggestions: ref([]),
    showSuggestions: ref(false),
    activeExtensionKeyword: ref(null),
    searchResults: ref([]),
    isSearching: ref(false),
    showSearchResults: ref(false),
    scheduleSuggestions: vi.fn(),
    hideSuggestions: vi.fn(),
    clearSearchResults: vi.fn(),
    handleOmniboxSubmit: vi.fn(async () => 'navigate'),
  }),
}));

vi.mock('$lib/favicon', () => ({
  isSecureUrl: vi.fn((url) => url.startsWith('https://')),
}));

vi.mock('$lib/newTabPage', () => ({
  isNewTabUrl: vi.fn(() => false),
}));

vi.mock('$lib/browserIntegrations', () => ({
  omniboxSuggestionTypeLabel: vi.fn((type) => type),
}));

import { ref } from 'vue';

describe('AddressBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders address bar', () => {
    const wrapper = mount(AddressBar);
    
    expect(wrapper.find('.address-bar').exists()).toBe(true);
  });

  it('renders navigation controls', () => {
    const wrapper = mount(AddressBar);
    
    expect(wrapper.find('.nav-controls').exists()).toBe(true);
    expect(wrapper.findAll('.nav-icon-btn').length).toBe(4);
  });

  it('disables back button when canGoBack is false', () => {
    const wrapper = mount(AddressBar, {
      props: { canGoBack: false }
    });
    
    const backButton = wrapper.findAll('.nav-icon-btn')[0];
    expect(backButton.attributes('disabled')).toBeDefined();
  });

  it('enables back button when canGoBack is true', () => {
    const wrapper = mount(AddressBar, {
      props: { canGoBack: true }
    });
    
    const backButton = wrapper.findAll('.nav-icon-btn')[0];
    expect(backButton.attributes('disabled')).toBeUndefined();
  });

  it('disables forward button when canGoForward is false', () => {
    const wrapper = mount(AddressBar, {
      props: { canGoForward: false }
    });
    
    const forwardButton = wrapper.findAll('.nav-icon-btn')[1];
    expect(forwardButton.attributes('disabled')).toBeDefined();
  });

  it('enables forward button when canGoForward is true', () => {
    const wrapper = mount(AddressBar, {
      props: { canGoForward: true }
    });
    
    const forwardButton = wrapper.findAll('.nav-icon-btn')[1];
    expect(forwardButton.attributes('disabled')).toBeUndefined();
  });

  it('emits goBack when back button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.nav-icon-btn')[0].trigger('click');
    
    expect(wrapper.emitted('goBack')).toBeTruthy();
  });

  it('emits goForward when forward button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.nav-icon-btn')[1].trigger('click');
    
    expect(wrapper.emitted('goForward')).toBeTruthy();
  });

  it('emits reload when reload button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.nav-icon-btn')[2].trigger('click');
    
    expect(wrapper.emitted('reload')).toBeTruthy();
  });

  it('emits home when home button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.nav-icon-btn')[3].trigger('click');
    
    expect(wrapper.emitted('home')).toBeTruthy();
  });

  it('renders omnibox input', () => {
    const wrapper = mount(AddressBar);
    
    expect(wrapper.find('.url-input').exists()).toBe(true);
  });

  it('displays current URL in input', () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' }
    });
    
    expect(wrapper.find('.url-input').element.value).toBe('https://example.com');
  });

  it('shows secure site indicator for HTTPS URLs', () => {
    const { isSecureUrl } = require('$lib/favicon');
    isSecureUrl.mockReturnValue(true);
    
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' }
    });
    
    expect(wrapper.find('.site-indicator').exists()).toBe(true);
    expect(wrapper.find('.site-indicator').text()).toBe('🔒');
    expect(wrapper.find('.site-indicator').classes()).toContain('secure');
  });

  it('shows insecure site indicator for HTTP URLs', () => {
    const { isSecureUrl } = require('$lib/favicon');
    isSecureUrl.mockReturnValue(false);
    
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'http://example.com' }
    });
    
    expect(wrapper.find('.site-indicator').exists()).toBe(true);
    expect(wrapper.find('.site-indicator').text()).toBe('⚠');
    expect(wrapper.find('.site-indicator').classes()).not.toContain('secure');
  });

  it('does not show site indicator for new tab URL', () => {
    const { isNewTabUrl } = require('$lib/newTabPage');
    isNewTabUrl.mockReturnValue(true);
    
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'exodus://newtab' }
    });
    
    expect(wrapper.find('.site-indicator').exists()).toBe(false);
  });

  it('shows CDN badge when cdnStatusLabel is provided', () => {
    const wrapper = mount(AddressBar, {
      props: { cdnStatusLabel: 'P2P: 3 peers' }
    });
    
    expect(wrapper.find('.cdn-omnibox-badge').exists()).toBe(true);
    expect(wrapper.find('.cdn-omnibox-badge').text()).toBe('P2P: 3 peers');
  });

  it('emits cdnBadgeClick when CDN badge is clicked', async () => {
    const wrapper = mount(AddressBar, {
      props: { cdnStatusLabel: 'P2P: 3 peers' }
    });
    
    await wrapper.find('.cdn-omnibox-badge').trigger('click');
    
    expect(wrapper.emitted('cdnBadgeClick')).toBeTruthy();
  });

  it('emits urlInput on input change', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.find('.url-input').setValue('https://test.com');
    
    expect(wrapper.emitted('urlInput')).toBeTruthy();
    expect(wrapper.emitted('urlInput')?.[0]).toEqual(['https://test.com']);
  });

  it('emits navigate on Enter key', async () => {
    const { handleOmniboxSubmit } = require('@/composables/useOmnibox');
    handleOmniboxSubmit.mockResolvedValue('navigate');
    
    const wrapper = mount(AddressBar);
    await wrapper.find('.url-input').setValue('https://test.com');
    
    await wrapper.find('.url-input').trigger('keydown', { key: 'Enter' });
    
    expect(wrapper.emitted('navigate')).toBeTruthy();
  });

  it('hides suggestions on Escape key', async () => {
    const { hideSuggestions } = require('@/composables/useOmnibox');
    
    const wrapper = mount(AddressBar);
    
    await wrapper.find('.url-input').trigger('keydown', { key: 'Escape' });
    
    expect(hideSuggestions).toHaveBeenCalled();
    expect(wrapper.emitted('closeMenu')).toBeTruthy();
  });

  it('shows bookmarked state when isBookmarked is true', () => {
    const wrapper = mount(AddressBar, {
      props: { isBookmarked: true }
    });
    
    const bookmarkBtn = wrapper.findAll('.toolbar-icon-btn')[0];
    expect(bookmarkBtn.classes()).toContain('bookmarked');
    expect(bookmarkBtn.text()).toBe('★');
  });

  it('shows unbookmarked state when isBookmarked is false', () => {
    const wrapper = mount(AddressBar, {
      props: { isBookmarked: false }
    });
    
    const bookmarkBtn = wrapper.findAll('.toolbar-icon-btn')[0];
    expect(bookmarkBtn.classes()).not.toContain('bookmarked');
    expect(bookmarkBtn.text()).toBe('☆');
  });

  it('emits toggleBookmark when bookmark button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.toolbar-icon-btn')[0].trigger('click');
    
    expect(wrapper.emitted('toggleBookmark')).toBeTruthy();
  });

  it('shows shields button when currentUrl is not new tab', () => {
    const { isNewTabUrl } = require('$lib/newTabPage');
    isNewTabUrl.mockReturnValue(false);
    
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' }
    });
    
    expect(wrapper.find('.shields-btn').exists()).toBe(true);
  });

  it('does not show shields button for new tab URL', () => {
    const { isNewTabUrl } = require('$lib/newTabPage');
    isNewTabUrl.mockReturnValue(true);
    
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'exodus://newtab' }
    });
    
    expect(wrapper.find('.shields-btn').exists()).toBe(false);
  });

  it('shows shields count badge when shieldsCount > 0', () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com', shieldsCount: 5 }
    });
    
    expect(wrapper.find('.shields-badge').exists()).toBe(true);
    expect(wrapper.find('.shields-badge').text()).toBe('5');
  });

  it('shows 99+ when shieldsCount > 99', () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com', shieldsCount: 150 }
    });
    
    expect(wrapper.find('.shields-badge').text()).toBe('99+');
  });

  it('applies shields-off class when shieldsEnabled is false', () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com', shieldsEnabled: false }
    });
    
    expect(wrapper.find('.shields-btn').classes()).toContain('shields-off');
  });

  it('emits openShields on shields click', async () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' }
    });
    
    await wrapper.find('.shields-btn').trigger('click');
    
    expect(wrapper.emitted('openShields')).toBeTruthy();
  });

  it('emits toggleSiteShields on shift+click shields', async () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' }
    });
    
    await wrapper.find('.shields-btn').trigger('click', { shiftKey: true });
    
    expect(wrapper.emitted('toggleSiteShields')).toBeTruthy();
  });

  it('renders toolbar actions', () => {
    const wrapper = mount(AddressBar);
    
    expect(wrapper.find('.toolbar-actions').exists()).toBe(true);
    expect(wrapper.findAll('.toolbar-actions .toolbar-icon-btn').length).toBe(5);
  });

  it('shows active class for active sidebar panel', () => {
    const wrapper = mount(AddressBar, {
      props: { sidebarOpen: true, sidebarPanel: 'ai' }
    });
    
    const aiBtn = wrapper.findAll('.toolbar-actions .toolbar-icon-btn')[0];
    expect(aiBtn.classes()).toContain('active');
  });

  it('emits openPanel when toolbar button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.toolbar-actions .toolbar-icon-btn')[0].trigger('click');
    
    expect(wrapper.emitted('openPanel')).toBeTruthy();
    expect(wrapper.emitted('openPanel')?.[0]).toEqual(['ai']);
  });

  it('shows downloads badge when downloadsBadge > 0', () => {
    const wrapper = mount(AddressBar, {
      props: { downloadsBadge: 3 }
    });
    
    const downloadsBtn = wrapper.findAll('.toolbar-actions .toolbar-icon-btn')[4];
    expect(downloadsBtn.find('.toolbar-badge').exists()).toBe(true);
    expect(downloadsBtn.find('.toolbar-badge').text()).toBe('3');
  });

  it('emits openDownloads when downloads button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.findAll('.toolbar-actions .toolbar-icon-btn')[4].trigger('click');
    
    expect(wrapper.emitted('openDownloads')).toBeTruthy();
  });

  it('renders chrome menu button', () => {
    const wrapper = mount(AddressBar);
    
    expect(wrapper.find('.chrome-menu-btn').exists()).toBe(true);
  });

  it('emits toggleMenu when chrome menu button is clicked', async () => {
    const wrapper = mount(AddressBar);
    
    await wrapper.find('.chrome-menu-btn').trigger('click');
    
    expect(wrapper.emitted('toggleMenu')).toBeTruthy();
  });

  it('shows menu dropdown when showMenu is true', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    expect(wrapper.find('.chrome-menu-dropdown').exists()).toBe(true);
  });

  it('does not show menu dropdown when showMenu is false', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: false }
    });
    
    expect(wrapper.find('.chrome-menu-dropdown').exists()).toBe(false);
  });

  it('emits closeMenu when menu backdrop is clicked', async () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    await wrapper.find('.menu-backdrop').trigger('click');
    
    expect(wrapper.emitted('closeMenu')).toBeTruthy();
  });

  it('renders all Chrome-style menu items', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    const menuItems = wrapper.findAll('.menu-item');
    expect(menuItems.length).toBe(18);
  });

  it('emits correct event for new tab menu action', async () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    await wrapper.findAll('.menu-item')[0].trigger('click');
    
    expect(wrapper.emitted('closeMenu')).toBeTruthy();
    expect(wrapper.emitted('newTab')).toBeTruthy();
  });

  it('emits correct event for bookmark menu action', async () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    await wrapper.findAll('.menu-item')[3].trigger('click');
    
    expect(wrapper.emitted('closeMenu')).toBeTruthy();
    expect(wrapper.emitted('toggleBookmark')).toBeTruthy();
  });

  it('displays correct bookmark label when bookmarked', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true, isBookmarked: true }
    });
    
    const bookmarkBtn = wrapper.findAll('.menu-item')[3];
    expect(bookmarkBtn.text()).toContain('Edit bookmark');
  });

  it('displays correct bookmark label when not bookmarked', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true, isBookmarked: false }
    });
    
    const bookmarkBtn = wrapper.findAll('.menu-item')[3];
    expect(bookmarkBtn.text()).toContain('Bookmark this page');
  });

  it('renders menu shortcuts', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    const shortcuts = wrapper.findAll('.menu-shortcut');
    expect(shortcuts.length).toBeGreaterThan(0);
  });

  it('emits zoom actions', async () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    await wrapper.findAll('.menu-item')[9].trigger('click');
    expect(wrapper.emitted('zoomIn')).toBeTruthy();
    
    await wrapper.findAll('.menu-item')[10].trigger('click');
    expect(wrapper.emitted('zoomOut')).toBeTruthy();
    
    await wrapper.findAll('.menu-item')[11].trigger('click');
    expect(wrapper.emitted('zoomReset')).toBeTruthy();
  });

  it('has correct ARIA attributes on navigation buttons', () => {
    const wrapper = mount(AddressBar);
    
    const backButton = wrapper.findAll('.nav-icon-btn')[0];
    expect(backButton.attributes('aria-label')).toBe('Back');
    expect(backButton.attributes('title')).toBe('Back');
  });

  it('has correct ARIA attributes on chrome menu button', () => {
    const wrapper = mount(AddressBar);
    
    expect(wrapper.find('.chrome-menu-btn').attributes('aria-label')).toBe('Menu');
  });

  it('has correct ARIA attributes on menu dropdown', () => {
    const wrapper = mount(AddressBar, {
      props: { showMenu: true }
    });
    
    expect(wrapper.find('.chrome-menu-dropdown').attributes('role')).toBe('menu');
  });

  it('updates urlInput when currentUrl prop changes', async () => {
    const wrapper = mount(AddressBar, {
      props: { currentUrl: 'https://example.com' }
    });
    
    await wrapper.setProps({ currentUrl: 'https://new.com' });
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.url-input').element.value).toBe('https://new.com');
  });
});
