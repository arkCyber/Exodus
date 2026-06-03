/**
 * Exodus Browser — HomePage component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import HomePage from './HomePage.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => false)
}));

describe('HomePage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders home page', () => {
    const wrapper = mount(HomePage);
    
    expect(wrapper.find('.home-page').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(HomePage);
    
    expect(wrapper.find('h1').text()).toBe('Exodus Browser');
  });

  it('renders migration message', () => {
    const wrapper = mount(HomePage);
    
    expect(wrapper.find('p').text()).toContain('Vue 3 + Tauri migration in progress');
  });

  it('renders status', () => {
    const wrapper = mount(HomePage);
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[1].text()).toContain('Status: Ready');
  });

  it('renders test Tauri button', () => {
    const wrapper = mount(HomePage);
    
    expect(wrapper.find('button').text()).toBe('Test Tauri');
  });

  it('does not render Tauri result initially', () => {
    const wrapper = mount(HomePage);
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs.length).toBe(2);
  });

  it('handles non-Tauri environment on button click', async () => {
    const { isTauri } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(false);
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.status).toBe('Not in Tauri environment');
    expect(wrapper.vm.tauriResult).toBe('Dev mode - Tauri API unavailable');
  });

  it('displays dev mode message when not in Tauri', async () => {
    const { isTauri } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(false);
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await wrapper.vm.$nextTick();
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[paragraphs.length - 1].text()).toContain('Dev mode - Tauri API unavailable');
  });

  it('invokes Tauri command when in Tauri environment', async () => {
    const { isTauri, invoke } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(true);
    invoke.mockResolvedValue('Exodus Browser');
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('get_app_name');
  });

  it('displays success status on successful Tauri call', async () => {
    const { isTauri, invoke } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(true);
    invoke.mockResolvedValue('Exodus Browser');
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[1].text()).toBe('Status: Success');
  });

  it('displays Tauri result on success', async () => {
    const { isTauri, invoke } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(true);
    invoke.mockResolvedValue('Exodus Browser');
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[paragraphs.length - 1].text()).toBe('Tauri Result: Exodus Browser');
  });

  it('displays error status on Tauri failure', async () => {
    const { isTauri, invoke } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(true);
    invoke.mockRejectedValue(new Error('Command failed'));
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[1].text()).toBe('Status: Error');
  });

  it('displays error message on Tauri failure', async () => {
    const { isTauri, invoke } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(true);
    invoke.mockRejectedValue(new Error('Command failed'));
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[paragraphs.length - 1].text()).toContain('Error: Error: Command failed');
  });

  it('displays testing status while calling Tauri', async () => {
    const { isTauri, invoke } = require('@tauri-apps/api/core');
    isTauri.mockReturnValue(true);
    invoke.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
      return 'Exodus Browser';
    });
    
    const wrapper = mount(HomePage);
    
    await wrapper.find('button').trigger('click');
    await wrapper.vm.$nextTick();
    
    const paragraphs = wrapper.findAll('p');
    expect(paragraphs[1].text()).toBe('Status: Testing Tauri...');
  });
});
