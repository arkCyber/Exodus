/**
 * Exodus Browser — SettingsModal component tests.
 */

import { describe, expect, it, vi } from 'vitest';
import { ref } from 'vue';
import { mount, flushPromises } from '@vue/test-utils';
import SettingsModal from './SettingsModal.vue';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => false,
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'list_cookies' || cmd === 'list_passwords' || cmd === 'list_managed_history') {
      return [];
    }
    return undefined;
  }),
}));

vi.mock('@/lib/browserSettings', () => ({
  writeShowBookmarkBar: vi.fn(),
}));

const mockBrowserConfig = {
  loading: ref(false),
  aiPort: ref(11435),
  aiModel: ref('test'),
  embeddingModel: ref('embed'),
  homepageUrl: ref('https://example.com'),
  searchEngineUrl: ref('https://example.com?q={query}'),
  statusClearMs: ref(4000),
  spawnSidecar: ref(false),
  spawnAllama: ref(true),
  httpsOnly: ref(false),
  privateMode: ref(false),
  blockPopups: ref(true),
  sessionRestore: ref(true),
  showBookmarkBar: ref(true),
  load: vi.fn(async () => {}),
  saveAll: vi.fn(async () => {}),
  saveAiConfig: vi.fn(async () => {}),
  savePrivacySettings: vi.fn(async () => {}),
};

vi.mock('@/composables/useBrowserConfig', () => ({
  useBrowserConfig: () => mockBrowserConfig,
}));

vi.mock('@/components/ExtensionsSettings.vue', () => ({
  default: {
    name: 'ExtensionsSettings',
    template: '<div class="extensions-settings-stub">Extensions</div>',
  },
}));

vi.mock('@/components/settings/PasswordManagerSettings.vue', () => ({
  default: {
    name: 'PasswordManagerSettings',
    template: '<div class="password-settings-stub">Passwords</div>',
  },
}));

describe('SettingsModal', () => {
  it('does not render when closed', () => {
    const wrapper = mount(SettingsModal, { props: { open: false } });
    expect(wrapper.find('[data-testid="chrome-settings-page"]').exists()).toBe(false);
  });

  it('renders settings sections when open', async () => {
    const wrapper = mount(SettingsModal, {
      props: { open: true },
      attachTo: document.body,
    });
    await flushPromises();
    expect(document.querySelector('[data-testid="chrome-settings-page"]')).toBeTruthy();
    expect(document.body.textContent).toContain('Browser');
    expect(document.body.textContent).toContain('Privacy and security');
    wrapper.unmount();
  });

  it('emits openSidebarCustomize from sidebar section', async () => {
    const wrapper = mount(SettingsModal, {
      props: { open: true },
      attachTo: document.body,
    });
    await flushPromises();
    const sidebarNav = document.querySelector('[data-testid="chrome-settings-nav-sidebar"]') as HTMLElement;
    expect(sidebarNav).toBeTruthy();
    sidebarNav?.click();
    await flushPromises();
    const btn = Array.from(document.querySelectorAll('button')).find((b) =>
      b.textContent?.includes('Customize sidebar'),
    );
    expect(btn).toBeTruthy();
    btn?.click();
    expect(wrapper.emitted('openSidebarCustomize')).toHaveLength(1);
    wrapper.unmount();
  });
});
