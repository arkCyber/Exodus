/**
 * Exodus Browser — PasswordManagerSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import PasswordManagerSettings from './PasswordManagerSettings.vue';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  isTauri: () => true,
}));

describe('PasswordManagerSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    expect(wrapper.find('h3').text()).toBe('Password manager');
  });

  it('renders toolbar', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders search input', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    expect(wrapper.find('input[type="search"]').exists()).toBe(true);
  });

  it('has correct placeholder on search input', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    expect(wrapper.find('input[type="search"]').attributes('placeholder')).toBe('Search passwords…');
  });

  it('renders add button', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Add');
  });

  it('renders generate button', () => {
    const wrapper = mount(PasswordManagerSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].text()).toBe('Generate');
  });

  it('shows empty state when no passwords', async () => {
    invokeMock.mockResolvedValue([]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.hint').text()).toBe('No saved passwords.');
  });

  it('renders password list when passwords exist', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.list').exists()).toBe(true);
  });

  it('displays site name', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row strong').text()).toBe('Example');
  });

  it('displays username and URL', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row .muted').text()).toContain('user');
    expect(wrapper.find('.row .muted').text()).toContain('https://example.com');
  });

  it('renders copy button', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.actions .nav-button');
    expect(buttons[0].text()).toBe('Copy');
  });

  it('copies password on copy button click', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass123', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    const writeTextMock = vi.fn();
    global.navigator.clipboard = { writeText: writeTextMock };
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.actions .nav-button')[0].trigger('click');
    
    expect(writeTextMock).toHaveBeenCalledWith('pass123');
  });

  it('emits status on copy', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    global.navigator.clipboard = { writeText: vi.fn() };
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.actions .nav-button')[0].trigger('click');
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Copied to clipboard']);
  });

  it('renders delete button', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.actions .nav-button');
    expect(buttons[1].text()).toBe('Delete');
  });

  it('deletes password on delete button click', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.actions .nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invokeMock).toHaveBeenCalledWith('delete_password', { id: '1' });
  });

  it('emits status on successful delete', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.actions .nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Password removed']);
  });

  it('shows add dialog on add button click', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-backdrop').exists()).toBe(true);
  });

  it('hides add dialog on backdrop click', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog-backdrop').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-backdrop').exists()).toBe(false);
  });

  it('renders add dialog title', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h4').text()).toBe('Add password');
  });

  it('renders site name input in add dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[0].text()).toContain('Site');
  });

  it('renders URL input in add dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[1].text()).toContain('URL');
  });

  it('renders username input in add dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[2].text()).toContain('Username');
  });

  it('renders password input in add dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const labels = wrapper.findAll('.dialog label');
    expect(labels[3].text()).toContain('Password');
  });

  it('password input has type password', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const passwordInput = wrapper.findAll('.dialog input[type="password"]');
    expect(passwordInput.length).toBe(1);
  });

  it('renders cancel button in add dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.dialog-actions .nav-button');
    expect(buttons[0].text()).toBe('Cancel');
  });

  it('hides dialog on cancel button click', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.dialog-actions .nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-backdrop').exists()).toBe(false);
  });

  it('renders save button in add dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.dialog-actions .nav-button');
    expect(buttons[1].text()).toBe('Save');
  });

  it('saves password on save button click', async () => {
    invokeMock.mockResolvedValue([]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Example');
    await inputs[1].setValue('https://example.com');
    await inputs[2].setValue('user');
    await inputs[3].setValue('pass');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.dialog-actions .nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invokeMock).toHaveBeenCalledWith('save_password', {
      entry: expect.objectContaining({
        site_name: 'Example',
        url: 'https://example.com',
        username: 'user',
        password: 'pass'
      })
    });
  });

  it('emits status on successful save', async () => {
    invokeMock.mockResolvedValue([]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Example');
    await inputs[1].setValue('https://example.com');
    await inputs[2].setValue('user');
    await inputs[3].setValue('pass');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.dialog-actions .nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Password saved']);
  });

  it('clears form after save', async () => {
    invokeMock.mockResolvedValue([]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.dialog .field');
    await inputs[0].setValue('Example');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.dialog-actions .nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.form.site_name).toBe('');
  });

  it('shows generate dialog on generate button click', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog-backdrop');
    expect(dialogs[1].exists()).toBe(true);
  });

  it('renders generate dialog title', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    expect(dialogs[1].find('h4').text()).toBe('Generate password');
  });

  it('renders length slider', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    expect(dialogs[1].find('input[type="range"]').exists()).toBe(true);
  });

  it('has correct min and max on length slider', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    const slider = dialogs[1].find('input[type="range"]');
    expect(slider.attributes('min')).toBe('8');
    expect(slider.attributes('max')).toBe('32');
  });

  it('renders symbols checkbox', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    expect(dialogs[1].find('input[type="checkbox"]').exists()).toBe(true);
  });

  it('renders generate button in dialog', async () => {
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    const buttons = dialogs[1].findAll('.nav-button');
    expect(buttons[0].text()).toBe('Generate');
  });

  it('generates password on generate button click', async () => {
    invokeMock.mockResolvedValue('GeneratedPass123!');
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    await dialogs[1].findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invokeMock).toHaveBeenCalledWith('generate_password', {
      length: 16,
      includeSymbols: true
    });
  });

  it('displays generated password', async () => {
    invokeMock.mockResolvedValue('GeneratedPass123!');
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    await dialogs[1].findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(dialogs[1].find('.mono').text()).toBe('GeneratedPass123!');
  });

  it('renders copy button for generated password', async () => {
    invokeMock.mockResolvedValue('GeneratedPass123!');
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    await dialogs[1].findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = dialogs[1].findAll('.nav-button');
    expect(buttons[1].text()).toBe('Copy');
  });

  it('copies generated password', async () => {
    invokeMock.mockResolvedValue('GeneratedPass123!');
    const writeTextMock = vi.fn();
    global.navigator.clipboard = { writeText: writeTextMock };
    
    const wrapper = mount(PasswordManagerSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const dialogs = wrapper.findAll('.dialog');
    await dialogs[1].findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await dialogs[1].findAll('.nav-button')[1].trigger('click');
    
    expect(writeTextMock).toHaveBeenCalledWith('GeneratedPass123!');
  });

  it('filters passwords by search query', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 },
      { id: '2', site_name: 'Test', url: 'https://test.com', username: 'admin', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="search"]').setValue('example');
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(1);
  });

  it('filters by username', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'Example', url: 'https://example.com', username: 'user', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 },
      { id: '2', site_name: 'Test', url: 'https://test.com', username: 'admin', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="search"]').setValue('admin');
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(1);
  });

  it('case insensitive search', async () => {
    invokeMock.mockResolvedValue([
      { id: '1', site_name: 'EXAMPLE', url: 'https://EXAMPLE.com', username: 'USER', password: 'pass', created_at: Date.now(), updated_at: Date.now(), use_count: 0 }
    ]);
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="search"]').setValue('example');
    await wrapper.vm.$nextTick();
    
    const rows = wrapper.findAll('.row');
    expect(rows.length).toBe(1);
  });

  it('loads passwords on mount', async () => {
    invokeMock.mockResolvedValue([]);
    
    mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(invokeMock).toHaveBeenCalledWith('list_passwords');
  });

  it('emits status on load error', async () => {
    invokeMock.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(PasswordManagerSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to load passwords']);
  });
});
