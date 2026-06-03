/**
 * Exodus Browser — DownloadsSettings component tests.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import DownloadsSettings from './DownloadsSettings.vue';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'get_download_settings') {
      return {
        default_directory: '/Users/test/Downloads',
        ask_for_location: true,
        max_concurrent_downloads: 2,
        auto_resume: true,
        max_retry_attempts: 3,
        speed_limit: 0,
        clear_completed_on_exit: false,
        show_notifications: true,
      };
    }
    return {};
  }),
}));

import { invoke } from '@tauri-apps/api/core';

describe('DownloadsSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('loads and renders download settings fields', async () => {
    const wrapper = mount(DownloadsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="downloads-settings-panel"]').exists()).toBe(true);
    const dirInput = wrapper.find('[data-testid="downloads-default-dir"]').element as HTMLInputElement;
    expect(dirInput.value).toBe('/Users/test/Downloads');
    const askInput = wrapper.find('[data-testid="downloads-ask-location"]').element as HTMLInputElement;
    expect(askInput.checked).toBe(true);
  });

  it('persists settings on save', async () => {
    const wrapper = mount(DownloadsSettings);
    await flushPromises();
    await wrapper.find('[data-testid="downloads-save"]').trigger('click');
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith(
      'update_download_settings',
      expect.objectContaining({
        settings: expect.objectContaining({
          default_directory: '/Users/test/Downloads',
          ask_for_location: true,
        }),
      }),
    );
  });
});
