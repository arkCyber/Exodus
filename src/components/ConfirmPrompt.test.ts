/**
 * Exodus Browser — ConfirmPrompt component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ConfirmPrompt from './ConfirmPrompt.vue';

describe('ConfirmPrompt', () => {
  it('renders nothing when offer is null', () => {
    const wrapper = mount(ConfirmPrompt, { props: { offer: null } });
    expect(wrapper.find('.prompt-dialog').exists()).toBe(false);
  });

  it('emits confirm and cancel', async () => {
    const wrapper = mount(ConfirmPrompt, {
      props: {
        offer: {
          title: 'Delete?',
          message: 'Cannot undo.',
          confirmLabel: 'Delete',
          danger: true,
        },
      },
    });
    expect(wrapper.find('#confirm-title').text()).toBe('Delete?');
    await wrapper.find('.btn.danger').trigger('click');
    expect(wrapper.emitted('confirm')).toHaveLength(1);
    await wrapper.find('.btn.secondary').trigger('click');
    expect(wrapper.emitted('cancel')).toHaveLength(1);
  });
});
