/**
 * Exodus Browser — ChromeSettingsPage component tests.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { mount, flushPromises } from '@vue/test-utils';
import ChromeSettingsPage from './ChromeSettingsPage.vue';

vi.mock('@/lib/browserSettings', () => ({
  writeShowBookmarkBar: vi.fn(),
}));

import { writeShowBookmarkBar } from '@/lib/browserSettings';

const mockBrowserConfig = {
  homepageUrl: ref('https://duckduckgo.com'),
  searchEngineUrl: ref('https://duckduckgo.com/?q={query}'),
  showBookmarkBar: ref(true),
  httpsOnly: ref(false),
  privateMode: ref(false),
  blockPopups: ref(true),
  sessionRestore: ref(true),
  spawnAllama: ref(false),
  aiPort: ref(11434),
  aiModel: ref('exodus-default'),
  load: vi.fn(async () => {}),
  saveAll: vi.fn(async () => {}),
  saveAiConfig: vi.fn(async () => {}),
  savePrivacySettings: vi.fn(async () => {}),
};

vi.mock('@/composables/useBrowserConfig', () => ({
  useBrowserConfig: () => mockBrowserConfig,
}));

vi.mock('@/components/ExtensionsSettings.vue', () => ({
  default: { name: 'ExtensionsSettings', template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/AllamaServiceSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/InferenceEngineSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/PasswordManagerSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/CookieManagerSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/HistoryManagerSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/BrowserSitePermissionsSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/PrivacyShieldSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/VerticalTabsSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/NewTabWallpaperSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/NewTabLayoutSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/P2pCdnSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/GroupChatSettings.vue', () => ({
  default: { template: '<div class="panel-stub" />' },
}));
vi.mock('@/components/settings/DownloadsSettings.vue', () => ({
  default: { template: '<div class="panel-stub" data-testid="downloads-settings-panel" />' },
}));
vi.mock('@/components/settings/ClearBrowsingDataSettings.vue', () => ({
  default: { template: '<div class="panel-stub" data-testid="clear-browsing-data-panel" />' },
}));
vi.mock('@/lib/newTabWallpaper', () => ({ WALLPAPER_FEATURE_ENABLED: false }));

describe('ChromeSettingsPage', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
    mockBrowserConfig.showBookmarkBar.value = true;
  });

  it('renders full-page chrome settings shell', () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'browser' } });
    expect(wrapper.find('[data-testid="chrome-settings-page"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="chrome-settings-nav-privacy"]').exists()).toBe(true);
  });

  it('switches to privacy section from sidebar', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'browser' } });
    await wrapper.find('[data-testid="chrome-settings-nav-privacy"]').trigger('click');
    expect(wrapper.attributes('data-section')).toBe('privacy');
    expect(wrapper.find('[data-testid="settings-section-privacy"]').isVisible()).toBe(true);
  });

  it('opens extensions section from initialSection prop', () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'extensions' } });
    expect(wrapper.attributes('data-section')).toBe('extensions');
    expect(wrapper.find('[data-testid="settings-section-extensions"]').isVisible()).toBe(true);
  });

  it('emits close when close button clicked', async () => {
    const wrapper = mount(ChromeSettingsPage);
    await wrapper.find('[data-testid="chrome-settings-close"]').trigger('click');
    expect(wrapper.emitted('close')).toHaveLength(1);
  });

  it('auto-saves when bookmark bar setting changes', async () => {
    mount(ChromeSettingsPage, { props: { initialSection: 'browser' } });
    await flushPromises();
    mockBrowserConfig.showBookmarkBar.value = false;
    await flushPromises();
    expect(writeShowBookmarkBar).toHaveBeenCalledWith(false);
  });

  it('shows autosave status footer instead of save button', () => {
    const wrapper = mount(ChromeSettingsPage);
    expect(wrapper.find('[data-testid="chrome-settings-autosave-status"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="chrome-settings-save"]').exists()).toBe(false);
  });

  it('applies unified panel class on P2P section container', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'p2p' } });
    await flushPromises();
    const p2p = wrapper.find('[data-testid="settings-section-p2p"]');
    expect(p2p.exists()).toBe(true);
    expect(p2p.classes()).toContain('settings-section--stack');
  });

  it('renders appearance theme and language controls', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'appearance' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="appearance-theme-select"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="appearance-locale-select"]').exists()).toBe(true);
  });

  it('lazy-mounts heavy panels until section is visited', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'browser' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-section-extensions"]').exists()).toBe(false);
    await wrapper.find('[data-testid="chrome-settings-nav-extensions"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-section-extensions"]').exists()).toBe(true);
  });

  it('filters nav items when searching', async () => {
    const wrapper = mount(ChromeSettingsPage);
    await wrapper.find('[data-testid="chrome-settings-search"]').setValue('extension');
    const visibleNav = wrapper.findAll('.chrome-settings__nav-item');
    expect(visibleNav.length).toBe(1);
    expect(visibleNav[0].text()).toContain('Extensions');
  });

  it('emits navigate when sidebar section changes', async () => {
    const wrapper = mount(ChromeSettingsPage);
    await wrapper.find('[data-testid="chrome-settings-nav-privacy"]').trigger('click');
    expect(wrapper.emitted('navigate')?.some((e) => String(e[0]).includes('privacy'))).toBe(true);
  });

  it('emits close on Escape key', async () => {
    const wrapper = mount(ChromeSettingsPage);
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    expect(wrapper.emitted('close')).toHaveLength(1);
    wrapper.unmount();
  });

  it('switches section when search hides current category', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'browser' } });
    await wrapper.find('[data-testid="chrome-settings-search"]').setValue('privacy');
    expect(wrapper.attributes('data-section')).toBe('privacy');
  });

  it('renders localized browser section for zh locale', () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'browser', uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('常规');
    expect(wrapper.text()).toContain('显示书签栏');
  });

  it('mounts downloads section with download settings panel', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'downloads' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-section-downloads"]').isVisible()).toBe(true);
    expect(wrapper.find('[data-testid="downloads-settings-panel"]').exists()).toBe(true);
  });

  it('shows package version on about section', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'about' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-about-version"]').text()).toMatch(/0\.1\.0/);
  });

  it('mounts privacy clear-browsing panel when privacy section opens', async () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'privacy' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="clear-browsing-data-panel"]').exists()).toBe(true);
  });

  it('renders localized startup section for zh', () => {
    const wrapper = mount(ChromeSettingsPage, { props: { initialSection: 'startup', uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('恢复上次会话的标签页');
  });
});
