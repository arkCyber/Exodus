/**
 * Exodus Browser — ChromeSettingsPage extensions section (real ExtensionsSettings panel).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { mount, flushPromises } from '@vue/test-utils';
import ChromeSettingsPage from './ChromeSettingsPage.vue';

vi.mock('@/lib/browserSettings', () => ({
  writeShowBookmarkBar: vi.fn(),
}));

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

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'get_ai_config') {
      return { extension_store_url: '', confirm_host_permissions_on_install: true };
    }
    return {};
  }),
}));

vi.mock('$lib/extensions/api', () => ({
  listExtensions: vi.fn(async () => []),
  listStoreExtensions: vi.fn(async () => []),
  fetchRemoteStoreExtensions: vi.fn(async () => []),
  rescanExtensions: vi.fn(async () => 0),
  setExtensionEnabled: vi.fn(),
  installExtensionFolder: vi.fn(),
  installExtensionCrx: vi.fn(),
  uninstallExtension: vi.fn(),
  setConfirmHostPermissionsOnInstall: vi.fn(),
  listExtensionSitePermissions: vi.fn(async () => []),
  revokeExtensionSitePermissions: vi.fn(),
  revokeAllExtensionSitePermissions: vi.fn(),
}));

vi.mock('$lib/extensions/backgroundHosts', () => ({
  ensureExtensionBackgrounds: vi.fn(),
}));

vi.mock('@/lib/newTabWallpaper', () => ({
  WALLPAPER_FEATURE_ENABLED: false,
}));

describe('ChromeSettingsPage extensions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('renders full extensions panel for chrome://extensions section', async () => {
    const wrapper = mount(ChromeSettingsPage, {
      props: { initialSection: 'extensions', uiLocale: 'en' },
    });
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-section-extensions"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="extensions-settings-panel"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="extensions-confirm-host"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="extensions-install-folder"]').exists()).toBe(true);
  });

  it('emits navigate to chrome://apps from extensions panel', async () => {
    const wrapper = mount(ChromeSettingsPage, {
      props: { initialSection: 'extensions' },
    });
    await flushPromises();
    await wrapper.find('[data-testid="extensions-open-apps"]').trigger('click');
    expect(wrapper.emitted('navigate')?.pop()?.[0]).toBe('chrome://apps');
  });
});
