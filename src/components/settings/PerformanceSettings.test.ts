/**
 * Tests for PerformanceSettings component
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import PerformanceSettings from './PerformanceSettings.vue';

describe('PerformanceSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders performance settings panel', () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    expect(wrapper.find('[data-testid="performance-settings"]').exists()).toBe(true);
  });

  it('renders memory section', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('Memory');
    expect(wrapper.find('[data-testid="memory-saver"]').exists()).toBe(true);
  });

  it('renders tab section', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('Tabs');
    expect(wrapper.find('[data-testid="suspend-tabs"]').exists()).toBe(true);
  });

  it('renders cache section', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('Cache');
    expect(wrapper.find('[data-testid="disk-cache"]').exists()).toBe(true);
  });

  it('renders rendering section', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('Rendering');
    expect(wrapper.find('[data-testid="gpu-acceleration"]').exists()).toBe(true);
  });

  it('emits status on settings change', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    const checkbox = wrapper.find('[data-testid="memory-saver"]') as any;
    await checkbox.setValue(true);
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.emitted('status')).toBeDefined();
  });

  it('shows Chinese title for zh locale', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'zh' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.text()).toContain('性能');
  });

  it('clears cache when button clicked', async () => {
    const wrapper = mount(PerformanceSettings, { props: { uiLocale: 'en' } });
    await new Promise(resolve => setTimeout(resolve, 10));
    await wrapper.find('[data-testid="clear-cache"]').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 10));
    expect(wrapper.emitted('status')?.[0]).toEqual(['Cache cleared']);
  });
});
