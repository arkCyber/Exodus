import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import TabGroupEditPrompt from './TabGroupEditPrompt.vue';

describe('TabGroupEditPrompt', () => {
  it('renders edit form when offer is set', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: { groupId: 'g1', title: 'Research', color: 'blue' },
      },
      attachTo: document.body,
    });
    expect(wrapper.text()).toContain('Edit tab group');
    expect((wrapper.find('input').element as HTMLInputElement).value).toBe('Research');
    wrapper.unmount();
  });

  it('emits save with title and color', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: { groupId: 'g1', title: 'Old', color: 'red' },
      },
      attachTo: document.body,
    });
    await wrapper.find('input').setValue('New name');
    await wrapper.find('.btn.primary').trigger('click');
    expect(wrapper.emitted('save')?.[0]).toEqual(['New name', 'red']);
    wrapper.unmount();
  });
});
