/**
 * Exodus Browser — HistoryManagerSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import HistoryManagerSettings from './HistoryManagerSettings.vue';

vi.mock('$lib/historyManager', () => ({
  clearAllManagedHistory: vi.fn(),
  getManagedHistoryStats: vi.fn(),
  getRecentManagedHistory: vi.fn(),
  loadHistoryManagerSettings: vi.fn(),
  removeManagedHistoryEntry: vi.fn(),
  saveHistoryManagerSettings: vi.fn(),
  searchManagedHistory: vi.fn()
}));

describe('HistoryManagerSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    expect(wrapper.find('h3').text()).toBe('Browsing history');
  });

  it('renders hint', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    expect(wrapper.find('.hint').text()).toContain('Managed history store');
  });

  it('renders settings when loaded', async () => {
    const { loadHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.checkbox-row').exists()).toBe(true);
  });

  it('renders enable checkbox', async () => {
    const { loadHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[0].exists()).toBe(true);
  });

  it('renders remember browsing checkbox', async () => {
    const { loadHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[1].exists()).toBe(true);
  });

  it('renders retention input', async () => {
    const { loadHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('input[type="number"]').exists()).toBe(true);
  });

  it('has correct min and max on retention input', async () => {
    const { loadHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const input = wrapper.find('input[type="number"]');
    expect(input.attributes('min')).toBe('0');
    expect(input.attributes('max')).toBe('3650');
  });

  it('displays stats when available', async () => {
    const { loadHistoryManagerSettings, getManagedHistoryStats } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getManagedHistoryStats.mockResolvedValue({ total_entries: 100, unique_domains: 25 });
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const hints = wrapper.findAll('.hint');
    expect(hints[1].text()).toContain('100 entries');
    expect(hints[1].text()).toContain('25 domains');
  });

  it('renders toolbar', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders search input', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    expect(wrapper.find('input[type="search"]').exists()).toBe(true);
  });

  it('has correct placeholder on search input', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    expect(wrapper.find('input[type="search"]').attributes('placeholder')).toBe('Search URL or title…');
  });

  it('renders search button', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Search');
  });

  it('renders refresh button', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].text()).toBe('Refresh');
  });

  it('renders history list when entries exist', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.list').exists()).toBe(true);
  });

  it('displays entry title', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row strong').text()).toBe('Example');
  });

  it('displays URL when title is missing', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: '', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row strong').text()).toBe('https://example.com');
  });

  it('displays entry URL', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row .muted').text()).toBe('https://example.com');
  });

  it('renders remove button on entry', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row .nav-button').text()).toBe('Remove');
  });

  it('removes entry on remove button click', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory, removeManagedHistoryEntry } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(removeManagedHistoryEntry).toHaveBeenCalledWith('1');
  });

  it('emits status on successful remove', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Entry removed']);
  });

  it('shows empty state when no entries', async () => {
    const { loadHistoryManagerSettings, getRecentManagedHistory } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getRecentManagedHistory.mockResolvedValue([]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const hints = wrapper.findAll('.hint');
    expect(hints[hints.length - 1].text()).toBe('No history entries.');
  });

  it('renders clear history button', () => {
    const wrapper = mount(HistoryManagerSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[buttons.length - 1].text()).toBe('Clear full history');
  });

  it('shows confirm text on first clear click', async () => {
    const wrapper = mount(HistoryManagerSettings);
    
    await wrapper.findAll('.nav-button')[wrapper.findAll('.nav-button').length - 1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[buttons.length - 1].text()).toBe('Click again to confirm');
  });

  it('clears history on second click', async () => {
    const { clearAllManagedHistory, getManagedHistoryStats } = require('$lib/historyManager');
    clearAllManagedHistory.mockResolvedValue(undefined);
    getManagedHistoryStats.mockResolvedValue({ total_entries: 0, unique_domains: 0 });
    
    const wrapper = mount(HistoryManagerSettings);
    
    const clearButton = wrapper.findAll('.nav-button')[wrapper.findAll('.nav-button').length - 1];
    await clearButton.trigger('click');
    await wrapper.vm.$nextTick();
    await clearButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(clearAllManagedHistory).toHaveBeenCalled();
  });

  it('emits status on successful clear', async () => {
    const { clearAllManagedHistory, getManagedHistoryStats } = require('$lib/historyManager');
    clearAllManagedHistory.mockResolvedValue(undefined);
    getManagedHistoryStats.mockResolvedValue({ total_entries: 0, unique_domains: 0 });
    
    const wrapper = mount(HistoryManagerSettings);
    
    const clearButton = wrapper.findAll('.nav-button')[wrapper.findAll('.nav-button').length - 1];
    await clearButton.trigger('click');
    await wrapper.vm.$nextTick();
    await clearButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['History cleared']);
  });

  it('searches history on search button click', async () => {
    const { searchManagedHistory } = require('$lib/historyManager');
    searchManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await wrapper.find('input[type="search"]').setValue('example');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(searchManagedHistory).toHaveBeenCalledWith('example');
  });

  it('searches history on enter key', async () => {
    const { searchManagedHistory } = require('$lib/historyManager');
    searchManagedHistory.mockResolvedValue([
      { id: '1', url: 'https://example.com', title: 'Example', timestamp: Date.now() }
    ]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await wrapper.find('input[type="search"]').setValue('example');
    await wrapper.find('input[type="search"]').trigger('keydown.enter');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(searchManagedHistory).toHaveBeenCalledWith('example');
  });

  it('loads recent history when search is empty', async () => {
    const { getRecentManagedHistory } = require('$lib/historyManager');
    getRecentManagedHistory.mockResolvedValue([]);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await wrapper.find('input[type="search"]').setValue('');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(getRecentManagedHistory).toHaveBeenCalledWith(80);
  });

  it('persists settings on checkbox change', async () => {
    const { loadHistoryManagerSettings, saveHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    saveHistoryManagerSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('input[type="checkbox"]')[0].trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(saveHistoryManagerSettings).toHaveBeenCalled();
  });

  it('emits status on successful settings save', async () => {
    const { loadHistoryManagerSettings, saveHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    saveHistoryManagerSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('input[type="checkbox"]')[0].trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['History settings saved']);
  });

  it('persists settings on retention change', async () => {
    const { loadHistoryManagerSettings, saveHistoryManagerSettings } = require('$lib/historyManager');
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    saveHistoryManagerSettings.mockResolvedValue(undefined);
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="number"]').trigger('change');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(saveHistoryManagerSettings).toHaveBeenCalled();
  });

  it('loads data on mount', async () => {
    const { getRecentManagedHistory, loadHistoryManagerSettings, getManagedHistoryStats } = require('$lib/historyManager');
    getRecentManagedHistory.mockResolvedValue([]);
    loadHistoryManagerSettings.mockResolvedValue({
      enabled: true,
      remember_browsing: true,
      retention_days: 90
    });
    getManagedHistoryStats.mockResolvedValue({ total_entries: 0, unique_domains: 0 });
    
    mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(getRecentManagedHistory).toHaveBeenCalledWith(80);
    expect(loadHistoryManagerSettings).toHaveBeenCalled();
    expect(getManagedHistoryStats).toHaveBeenCalled();
  });

  it('emits status on load error', async () => {
    const { getRecentManagedHistory } = require('$lib/historyManager');
    getRecentManagedHistory.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(HistoryManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to load history']);
  });
});
