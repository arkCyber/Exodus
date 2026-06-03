/**
 * Exodus Browser — TabGroupDeletePrompt component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import TabGroupDeletePrompt from './TabGroupDeletePrompt.vue';

describe('TabGroupDeletePrompt', () => {
  it('does not render when groupTitle is null', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: null }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(false);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(false);
  });

  it('renders dialog when groupTitle is provided', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(true);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(true);
  });

  it('displays delete tab group title', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    expect(wrapper.find('h3').text()).toBe('Delete tab group?');
  });

  it('displays group title in message', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    expect(wrapper.find('p').text()).toContain('Work Tabs');
    expect(wrapper.find('strong').text()).toBe('Work Tabs');
  });

  it('displays message about open tabs staying open', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    expect(wrapper.find('p').text()).toContain('Open tabs will stay open');
  });

  it('emits cancel when backdrop is clicked', async () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    await wrapper.find('.prompt-backdrop').trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits cancel when cancel button is clicked', async () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    const cancelButton = wrapper.find('.btn.secondary');
    await cancelButton.trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits confirm when delete button is clicked', async () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    const deleteButton = wrapper.find('.btn.danger');
    await deleteButton.trigger('click');
    
    expect(wrapper.emitted('confirm')).toBeTruthy();
  });

  it('disables buttons when busy is true', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: {
        groupTitle: 'Work Tabs',
        busy: true
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeDefined();
    });
  });

  it('shows deleting text when busy is true', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: {
        groupTitle: 'Work Tabs',
        busy: true
      }
    });
    
    const deleteButton = wrapper.find('.btn.danger');
    expect(deleteButton.text()).toBe('Deleting…');
  });

  it('shows delete text when busy is false', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: {
        groupTitle: 'Work Tabs',
        busy: false
      }
    });
    
    const deleteButton = wrapper.find('.btn.danger');
    expect(deleteButton.text()).toBe('Delete');
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    expect(wrapper.find('.prompt-dialog').attributes('role')).toBe('alertdialog');
    expect(wrapper.find('.prompt-dialog').attributes('aria-labelledby')).toBe('tg-del-title');
    expect(wrapper.find('.prompt-backdrop').attributes('aria-label')).toBe('Cancel');
  });

  it('has danger styling for delete button', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    const deleteButton = wrapper.find('.btn.danger');
    expect(deleteButton.classes()).toContain('danger');
  });

  it('has secondary styling for cancel button', () => {
    const wrapper = mount(TabGroupDeletePrompt, {
      props: { groupTitle: 'Work Tabs' }
    });
    
    const cancelButton = wrapper.find('.btn.secondary');
    expect(cancelButton.classes()).toContain('secondary');
  });
});
