/**
 * Exodus Browser — PasswordSavePrompt component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import PasswordSavePrompt from './PasswordSavePrompt.vue';

describe('PasswordSavePrompt', () => {
  it('does not render when capture is null', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: { capture: null }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(false);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(false);
  });

  it('renders dialog when capture is provided', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(true);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(true);
  });

  it('displays title', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Save password?');
  });

  it('extracts and displays hostname from URL', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com/login',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    expect(wrapper.find('.prompt-host').text()).toBe('example.com');
  });

  it('displays URL as host when URL parsing fails', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'invalid-url',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    expect(wrapper.find('.prompt-host').text()).toBe('invalid-url');
  });

  it('displays username field', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    const usernameInput = wrapper.findAll('input')[0];
    expect(usernameInput.attributes('value')).toBe('testuser');
    expect(usernameInput.attributes('readonly')).toBeDefined();
  });

  it('displays (none) when username is empty', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: '',
          password: 'testpass'
        }
      }
    });
    
    const usernameInput = wrapper.findAll('input')[0];
    expect(usernameInput.attributes('value')).toBe('(none)');
  });

  it('displays password field', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    const passwordInput = wrapper.findAll('input')[1];
    expect(passwordInput.attributes('value')).toBe('testpass');
    expect(passwordInput.attributes('type')).toBe('password');
    expect(passwordInput.attributes('readonly')).toBeDefined();
  });

  it('shows never button by default', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    expect(wrapper.find('.btn.never').exists()).toBe(true);
  });

  it('hides never button when showNever is false', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        },
        showNever: false
      }
    });
    
    expect(wrapper.find('.btn.never').exists()).toBe(false);
  });

  it('emits dismiss when backdrop is clicked', async () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    await wrapper.find('.prompt-backdrop').trigger('click');
    
    expect(wrapper.emitted('dismiss')).toBeTruthy();
  });

  it('emits dismiss when not now button is clicked', async () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    const secondaryButton = wrapper.find('.btn.secondary');
    await secondaryButton.trigger('click');
    
    expect(wrapper.emitted('dismiss')).toBeTruthy();
  });

  it('emits save when save button is clicked', async () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    const primaryButton = wrapper.find('.btn.primary');
    await primaryButton.trigger('click');
    
    expect(wrapper.emitted('save')).toBeTruthy();
  });

  it('emits never when never button is clicked', async () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    const neverButton = wrapper.find('.btn.never');
    await neverButton.trigger('click');
    
    expect(wrapper.emitted('never')).toBeTruthy();
  });

  it('disables buttons when busy is true', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        },
        busy: true
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeDefined();
    });
  });

  it('shows saving text when busy is true', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        },
        busy: true
      }
    });
    
    const primaryButton = wrapper.find('.btn.primary');
    expect(primaryButton.text()).toBe('Saving…');
  });

  it('shows save text when busy is false', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        },
        busy: false
      }
    });
    
    const primaryButton = wrapper.find('.btn.primary');
    expect(primaryButton.text()).toBe('Save');
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'testuser',
          password: 'testpass'
        }
      }
    });
    
    expect(wrapper.find('.prompt-dialog').attributes('role')).toBe('dialog');
    expect(wrapper.find('.prompt-dialog').attributes('aria-labelledby')).toBe('pw-save-title');
    expect(wrapper.find('.prompt-backdrop').attributes('aria-label')).toBe('Dismiss');
  });
});
