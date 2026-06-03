/**
 * Exodus Browser — ConfirmPrompt component tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import ConfirmPrompt from './ConfirmPrompt.vue';

describe('ConfirmPrompt', () => {
  it('does not render when offer is null', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: { offer: null }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(false);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(false);
  });

  it('renders dialog when offer is provided', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm Action',
          message: 'Are you sure you want to proceed?'
        }
      }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(true);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(true);
  });

  it('displays offer title and message', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Delete Item',
          message: 'This action cannot be undone.'
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Delete Item');
    expect(wrapper.find('p').text()).toBe('This action cannot be undone.');
  });

  it('uses default labels when not provided', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        }
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    expect(buttons[0].text()).toBe('Cancel');
    expect(buttons[1].text()).toBe('Confirm');
  });

  it('uses custom labels when provided', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message',
          cancelLabel: 'No',
          confirmLabel: 'Yes'
        }
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    expect(buttons[0].text()).toBe('No');
    expect(buttons[1].text()).toBe('Yes');
  });

  it('applies danger class when danger is true', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Delete',
          message: 'Are you sure?',
          danger: true
        }
      }
    });
    
    const confirmButton = wrapper.findAll('.btn')[1];
    expect(confirmButton.classes()).toContain('danger');
  });

  it('applies primary class when danger is false', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Save',
          message: 'Save changes?',
          danger: false
        }
      }
    });
    
    const confirmButton = wrapper.findAll('.btn')[1];
    expect(confirmButton.classes()).toContain('primary');
  });

  it('emits cancel when backdrop is clicked', async () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        }
      }
    });
    
    await wrapper.find('.prompt-backdrop').trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits cancel when cancel button is clicked', async () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        }
      }
    });
    
    const cancelButton = wrapper.findAll('.btn')[0];
    await cancelButton.trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits confirm when confirm button is clicked', async () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        }
      }
    });
    
    const confirmButton = wrapper.findAll('.btn')[1];
    await confirmButton.trigger('click');
    
    expect(wrapper.emitted('confirm')).toBeTruthy();
  });

  it('disables buttons when busy is true', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        },
        busy: true
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    expect(buttons[0].attributes('disabled')).toBeDefined();
    expect(buttons[1].attributes('disabled')).toBeDefined();
  });

  it('shows working text when busy is true', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        },
        busy: true
      }
    });
    
    const confirmButton = wrapper.findAll('.btn')[1];
    expect(confirmButton.text()).toBe('Working…');
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Confirm',
          message: 'Message'
        }
      }
    });
    
    expect(wrapper.find('.prompt-dialog').attributes('role')).toBe('alertdialog');
    expect(wrapper.find('.prompt-dialog').attributes('aria-labelledby')).toBe('confirm-title');
    expect(wrapper.find('.prompt-backdrop').attributes('aria-label')).toBe('Cancel');
  });
});
