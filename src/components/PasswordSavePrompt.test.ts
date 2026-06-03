import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import PasswordSavePrompt from './PasswordSavePrompt.vue';

describe('PasswordSavePrompt', () => {
  it('renders when capture is set', () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com/login',
          username: 'user',
          password: 'secret',
        },
      },
      attachTo: document.body,
    });
    expect(wrapper.text()).toContain('Save password?');
    expect(wrapper.text()).toContain('example.com');
    wrapper.unmount();
  });

  it('emits save on primary button', async () => {
    const wrapper = mount(PasswordSavePrompt, {
      props: {
        capture: {
          url: 'https://example.com',
          username: 'u',
          password: 'p',
        },
      },
      attachTo: document.body,
    });
    await wrapper.find('.btn.primary').trigger('click');
    expect(wrapper.emitted('save')).toBeTruthy();
    wrapper.unmount();
  });
});
