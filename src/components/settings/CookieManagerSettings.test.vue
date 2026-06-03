/**
 * Exodus Browser — CookieManagerSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import CookieManagerSettings from './CookieManagerSettings.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('CookieManagerSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(CookieManagerSettings);
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(CookieManagerSettings);
    
    expect(wrapper.find('h3').text()).toBe('Cookies');
  });

  it('renders toolbar', () => {
    const wrapper = mount(CookieManagerSettings);
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders search input', () => {
    const wrapper = mount(CookieManagerSettings);
    
    expect(wrapper.find('input[type="search"]').exists()).toBe(true);
  });

  it('has correct placeholder on search input', () => {
    const wrapper = mount(CookieManagerSettings);
    
    expect(wrapper.find('input[type="search"]').attributes('placeholder')).toBe('Search domain or name…');
  });

  it('renders delete all button', () => {
    const wrapper = mount(CookieManagerSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Delete all');
  });

  it('shows empty state when no cookies', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.hint').text()).toBe('No cookies stored.');
  });

  it('renders cookie list when cookies exist', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.list').exists()).toBe(true);
  });

  it('displays cookie name', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row strong').text()).toBe('session');
  });

  it('displays cookie domain and path', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row .muted').text()).toBe('example.com/');
  });

  it('renders delete button on cookie row', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.row .nav-button');
    expect(buttons[0].text()).toBe('Delete');
  });

  it('deletes cookie on delete button click', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('delete_cookie', { id: '1' });
  });

  it('emits status on successful delete', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Cookie deleted']);
  });

  it('deletes all cookies on delete all button click', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    const confirmMock = vi.fn(() => true);
    global.confirm = confirmMock;
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(confirmMock).toHaveBeenCalledWith('Delete all cookies?');
    expect(invoke).toHaveBeenCalledWith('delete_all_cookies');
  });

  it('does not delete all when cancelled', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    const confirmMock = vi.fn(() => false);
    global.confirm = confirmMock;
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    
    expect(invoke).not.toHaveBeenCalledWith('delete_all_cookies');
  });

  it('emits status on successful clear all', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' }
    ]);
    global.confirm = vi.fn(() => true);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['All cookies deleted']);
  });

  it('filters cookies by search query', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' },
      { id: '2', domain: 'test.com', name: 'token', value: 'xyz789', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="search"]').setValue('session');
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(1);
    expect(rows[0].text()).toContain('session');
  });

  it('filters cookies by domain', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' },
      { id: '2', domain: 'test.com', name: 'token', value: 'xyz789', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="search"]').setValue('example');
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(1);
    expect(rows[0].text()).toContain('example.com');
  });

  it('shows all cookies when search is empty', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'example.com', name: 'session', value: 'abc123', path: '/' },
      { id: '2', domain: 'test.com', name: 'token', value: 'xyz789', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(2);
  });

  it('case insensitive search', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([
      { id: '1', domain: 'EXAMPLE.COM', name: 'SESSION', value: 'abc123', path: '/' }
    ]);
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="search"]').setValue('session');
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(1);
  });

  it('loads cookies on mount', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue([]);
    
    mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invoke).toHaveBeenCalledWith('list_cookies');
  });

  it('emits status on load error', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockRejectedValue(new Error('Failed to load'));
    
    const wrapper = mount(CookieManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to load cookies']);
  });
});
