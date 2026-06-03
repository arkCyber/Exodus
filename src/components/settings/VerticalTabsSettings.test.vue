/**
 * Exodus Browser — VerticalTabsSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import VerticalTabsSettings from './VerticalTabsSettings.vue';

vi.mock('$lib/verticalTabs', () => ({
  loadVerticalTabSettings: vi.fn(),
  saveVerticalTabSettings: vi.fn()
}));

describe('VerticalTabsSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(VerticalTabsSettings);
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(VerticalTabsSettings);
    
    expect(wrapper.find('h3').text()).toBe('Tab layout');
  });

  it('renders hint', () => {
    const wrapper = mount(VerticalTabsSettings);
    
    expect(wrapper.find('.hint').text()).toContain('Chrome-style vertical tabs');
  });

  it('renders settings when loaded', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.checkbox-row').exists()).toBe(true);
  });

  it('renders enabled checkbox', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true);
  });

  it('checkbox reflects enabled state', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('input[type="checkbox"]').element.checked).toBe(true);
  });

  it('renders position select when enabled', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const selects = wrapper.findAll('select');
    expect(selects[0].exists()).toBe(true);
  });

  it('renders position options', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const selects = wrapper.findAll('select');
    const options = selects[0].findAll('option');
    expect(options[0].text()).toBe('Left');
    expect(options[1].text()).toBe('Right');
  });

  it('renders width mode select when enabled', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const selects = wrapper.findAll('select');
    expect(selects[1].exists()).toBe(true);
  });

  it('renders width mode options', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const selects = wrapper.findAll('select');
    const options = selects[1].findAll('option');
    expect(options[0].text()).toBe('Fixed');
    expect(options[1].text()).toBe('Auto');
    expect(options[2].text()).toBe('Compact');
  });

  it('does not render position select when disabled', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: false,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const selects = wrapper.findAll('select');
    expect(selects.length).toBe(0);
  });

  it('does not render width mode select when disabled', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: false,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const selects = wrapper.findAll('select');
    expect(selects.length).toBe(0);
  });

  it('persists settings on checkbox change', async () => {
    const { loadVerticalTabSettings, saveVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    saveVerticalTabSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="checkbox"]').trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(saveVerticalTabSettings).toHaveBeenCalled();
  });

  it('emits layoutChange on checkbox change', async () => {
    const { loadVerticalTabSettings, saveVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    saveVerticalTabSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="checkbox"]').trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('layoutChange')).toBeTruthy();
  });

  it('emits status on checkbox change', async () => {
    const { loadVerticalTabSettings, saveVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    saveVerticalTabSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="checkbox"]').trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Tab layout updated']);
  });

  it('persists settings on position change', async () => {
    const { loadVerticalTabSettings, saveVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    saveVerticalTabSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('select')[0].trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(saveVerticalTabSettings).toHaveBeenCalled();
  });

  it('persists settings on width mode change', async () => {
    const { loadVerticalTabSettings, saveVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    saveVerticalTabSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('select')[1].trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(saveVerticalTabSettings).toHaveBeenCalled();
  });

  it('loads settings on mount', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(loadVerticalTabSettings).toHaveBeenCalled();
  });

  it('emits layoutChange on mount', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockResolvedValue({
      enabled: true,
      position: 'Left',
      width_mode: 'Fixed'
    });
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('layoutChange')).toBeTruthy();
  });

  it('emits status on load error', async () => {
    const { loadVerticalTabSettings } = require('$lib/verticalTabs');
    loadVerticalTabSettings.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(VerticalTabsSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to load tab layout settings']);
  });
});
