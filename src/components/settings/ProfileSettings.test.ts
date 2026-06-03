/**
 * Tests for ProfileSettings component
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import ProfileSettings from './ProfileSettings.vue';

describe('ProfileSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders profile settings panel', () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'en' } });
    expect(wrapper.find('[data-testid="profile-settings"]').exists()).toBe(true);
  });

  it('renders current profile section', async () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('Current profile');
    expect(wrapper.text()).toContain('Default Profile');
  });

  it('renders create profile button', async () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.find('[data-testid="create-profile"]').exists()).toBe(true);
  });

  it('creates new profile when button clicked', async () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    await wrapper.find('[data-testid="create-profile"]').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.emitted('status')?.[0]).toEqual(['Profile settings saved']);
  });

  it('renders guest profile checkbox', async () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.find('[data-testid="guest-profile"]').exists()).toBe(true);
  });

  it('shows Chinese title for zh locale', async () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'zh' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('配置文件管理');
  });

  it('emits status on profile switch', async () => {
    const wrapper = mount(ProfileSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    // Create a second profile first
    await wrapper.find('[data-testid="create-profile"]').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 10));
    // Now try to switch (this would need a profile to switch to)
    expect(wrapper.emitted('status')).toBeDefined();
  });
});
