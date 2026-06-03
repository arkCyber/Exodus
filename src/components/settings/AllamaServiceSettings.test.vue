/**
 * Exodus Browser — AllamaServiceSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import AllamaServiceSettings from './AllamaServiceSettings.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('AllamaServiceSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    expect(wrapper.find('h3').text()).toBe('Allama service');
  });

  it('renders hint with port', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    expect(wrapper.find('.hint').text()).toContain('port 11434');
  });

  it('renders spawn checkbox', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: true, aiPort: 11434 }
    });
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    expect(checkbox.exists()).toBe(true);
    expect(checkbox.element.checked).toBe(true);
  });

  it('emits spawnAllamaChange on checkbox change', async () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.find('input[type="checkbox"]').trigger('change');
    
    expect(wrapper.emitted('spawnAllamaChange')).toBeTruthy();
    expect(wrapper.emitted('spawnAllamaChange')?.[0]).toEqual([true]);
  });

  it('renders port input', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    const input = wrapper.find('input[type="number"]');
    expect(input.exists()).toBe(true);
    expect(input.element.value).toBe('11434');
  });

  it('has correct min and max attributes on port input', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    const input = wrapper.find('input[type="number"]');
    expect(input.attributes('min')).toBe('1');
    expect(input.attributes('max')).toBe('65535');
  });

  it('emits aiPortChange on port input', async () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.find('input[type="number"]').setValue('11435');
    
    expect(wrapper.emitted('aiPortChange')).toBeTruthy();
    expect(wrapper.emitted('aiPortChange')?.[0]).toEqual([11435]);
  });

  it('renders status box when status is available', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.status-box').exists()).toBe(true);
  });

  it('displays status state', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.status-box').text()).toContain('running');
  });

  it('displays status port', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.status-box').text()).toContain('port 11434');
  });

  it('displays status detail', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.status-box .hint').text()).toBe('Ready');
  });

  it('applies online class when endpoint is online', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.status-box .online').exists()).toBe(true);
  });

  it('does not apply online class when endpoint is offline', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'stopped',
      port: 11434,
      detail: 'Not running',
      endpointOnline: false
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.status-box .online').exists()).toBe(false);
  });

  it('renders toolbar', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders refresh button', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Refresh');
  });

  it('renders start button', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].text()).toBe('Start');
  });

  it('renders stop button', () => {
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[2].text()).toBe('Stop');
  });

  it('refreshes status on refresh button click', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('allama_service_status');
  });

  it('starts service on start button click', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Started',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('allama_service_start');
  });

  it('emits status on successful start', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Started',
      endpointOnline: true
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Allama started']);
  });

  it('emits status on failed start', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to start Allama']);
  });

  it('stops service on stop button click', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'stopped',
      port: 11434,
      detail: 'Stopped',
      endpointOnline: false
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[2].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('allama_service_stop');
  });

  it('emits status on successful stop', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'stopped',
      port: 11434,
      detail: 'Stopped',
      endpointOnline: false
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[2].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Allama stopped']);
  });

  it('emits status on failed stop', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[2].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to stop Allama']);
  });

  it('disables buttons when busy', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
      return { state: 'running', port: 11434, detail: 'Ready', endpointOnline: true };
    });
    
    const wrapper = mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.nav-button');
    buttons.forEach(btn => {
      expect(btn.attributes('disabled')).toBeDefined();
    });
  });

  it('refreshes status on mount', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      state: 'running',
      port: 11434,
      detail: 'Ready',
      endpointOnline: true
    });
    
    mount(AllamaServiceSettings, {
      props: { spawnAllama: false, aiPort: 11434 }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('allama_service_status');
  });
});
