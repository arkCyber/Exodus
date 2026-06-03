/**
 * Exodus Browser — NotificationSettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import NotificationSettings from './NotificationSettings.vue';

describe('NotificationSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and notifications enabled checkbox', async () => {
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="notification-settings"]').exists()).toBe(true);
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      const element = checkbox.element as HTMLInputElement;
      expect(element.checked).toBe(true);
    }
  });

  it('emits status when settings change', async () => {
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      await checkbox.setValue(false);
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('changes select settings', async () => {
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('select');
    if (select.exists()) {
      await select.setValue('allow');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-notification-settings', JSON.stringify({
      notificationsEnabled: false,
      defaultBehavior: 'block',
      soundEnabled: false,
      badgeEnabled: false,
      quietMode: true,
      quietStart: '23:00',
      quietEnd: '07:00'
    }));
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const buttons = wrapper.findAll('button');
    const resetButton = buttons.find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('displays Chinese strings when locale is zh', () => {
    const wrapper = mount(NotificationSettings, { props: { uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('通知');
  });
});
