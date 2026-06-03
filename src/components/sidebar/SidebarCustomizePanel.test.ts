/**
 * SidebarCustomizePanel tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SidebarCustomizePanel from './SidebarCustomizePanel.vue';
import { loadSidebarPreferences } from '$lib/sidebarPreferences';

describe('SidebarCustomizePanel', () => {
  it('emits position-change for left', async () => {
    const wrapper = mount(SidebarCustomizePanel, {
      props: { prefs: loadSidebarPreferences() },
    });
    const radios = wrapper.findAll('input[type="radio"]');
    await radios[0].setValue();
    expect(wrapper.emitted('position-change')?.[0]).toEqual(['left']);
  });

  it('emits vertical-tabs-change', async () => {
    const wrapper = mount(SidebarCustomizePanel, {
      props: { prefs: loadSidebarPreferences() },
    });
    const cb = wrapper.find('.exodus-customize-section input[type="checkbox"]');
    await cb.setValue(true);
    expect(wrapper.emitted('vertical-tabs-change')?.slice(-1)[0]).toEqual([true]);
  });
});
