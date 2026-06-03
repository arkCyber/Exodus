/**
 * Exodus Browser — NewTabLayoutSettings component tests.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import NewTabLayoutSettings from './NewTabLayoutSettings.vue';
import { clearAllNtpLayoutStorage, isNtpLayoutCustomized } from '@/lib/ntpLayoutStore';
import { buildNtpTopSitesGrid, removeNtpTopSite } from '@/lib/ntpTopSitesStore';

describe('NewTabLayoutSettings', () => {
  beforeEach(() => {
    clearAllNtpLayoutStorage();
  });

  it('renders restore default layout button', () => {
    const wrapper = mount(NewTabLayoutSettings);
    expect(wrapper.get('button').text()).toContain('Restore default layout');
  });

  it('emits ntpLayoutReset and status after factory reset', async () => {
    const google = buildNtpTopSitesGrid().find((site) => site.url.includes('google.com'));
    removeNtpTopSite(google!);
    expect(isNtpLayoutCustomized()).toBe(true);

    const wrapper = mount(NewTabLayoutSettings);
    await wrapper.get('button').trigger('click');

    expect(isNtpLayoutCustomized()).toBe(false);
    expect(buildNtpTopSitesGrid()).toHaveLength(8);
    expect(wrapper.emitted('ntpLayoutReset')).toHaveLength(1);
    expect(wrapper.emitted('status')?.[0]).toEqual(['New tab page layout restored to defaults']);
  });
});
