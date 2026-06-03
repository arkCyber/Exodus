/**
 * Exodus Browser — PasswordManagerSettings smoke tests (localized panel).
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import PasswordManagerSettings from './PasswordManagerSettings.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => (cmd === 'list_passwords' ? [] : '')),
}));

describe('PasswordManagerSettings smoke', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders localized panel and search field', () => {
    const wrapper = mount(PasswordManagerSettings, { props: { uiLocale: 'en' } });
    expect(wrapper.find('[data-testid="password-manager-panel"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="password-manager-search"]').exists()).toBe(true);
    expect(wrapper.text()).toContain('Password manager');
  });

  it('renders Chinese title for zh locale', () => {
    const wrapper = mount(PasswordManagerSettings, { props: { uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('密码管理器');
  });
});
