/**
 * Exodus Browser — TabGroupEditPrompt component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import TabGroupEditPrompt from './TabGroupEditPrompt.vue';

vi.mock('$lib/tabGroups', () => ({
  TAB_GROUP_COLORS: ['blue', 'red', 'green', 'yellow', 'purple', 'orange'],
  tabGroupColorCss: vi.fn((color) => `#${color}`),
  type: {},
}));

describe('TabGroupEditPrompt', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when offer is null', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: { offer: null }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(false);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(false);
  });

  it('renders dialog when offer is provided', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(true);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(true);
  });

  it('displays edit tab group title', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Edit tab group');
  });

  it('populates title input from offer', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    const titleInput = wrapper.find('input[type="text"]');
    expect(titleInput.element.value).toBe('Work Tabs');
  });

  it('populates color from offer', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Red'
        }
      }
    });
    
    const selectedSwatch = wrapper.find('.color-swatch.selected');
    expect(selectedSwatch.exists()).toBe(true);
  });

  it('renders all color swatches', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    const swatches = wrapper.findAll('.color-swatch');
    expect(swatches.length).toBe(6);
  });

  it('selects color when swatch is clicked', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    const swatches = wrapper.findAll('.color-swatch');
    await swatches[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.color-swatch.selected').length).toBe(1);
    expect(wrapper.findAll('.color-swatch.selected')[0].attributes('title')).toBe('red');
  });

  it('emits cancel when backdrop is clicked', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    await wrapper.find('.prompt-backdrop').trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits cancel when cancel button is clicked', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    const cancelButton = wrapper.find('.btn.secondary');
    await cancelButton.trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits save with title and color when save button is clicked', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    await wrapper.find('input[type="text"]').setValue('Updated Title');
    const swatches = wrapper.findAll('.color-swatch');
    await swatches[1].trigger('click');
    
    await wrapper.find('.btn.primary').trigger('click');
    
    expect(wrapper.emitted('save')).toBeTruthy();
    expect(wrapper.emitted('save')?.[0]).toEqual(['Updated Title', 'red']);
  });

  it('disables save button when title is empty', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    await wrapper.find('input[type="text"]').setValue('');
    await wrapper.vm.$nextTick();
    
    const saveButton = wrapper.find('.btn.primary');
    expect(saveButton.attributes('disabled')).toBeDefined();
  });

  it('disables save button when title is only whitespace', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    await wrapper.find('input[type="text"]').setValue('   ');
    await wrapper.vm.$nextTick();
    
    const saveButton = wrapper.find('.btn.primary');
    expect(saveButton.attributes('disabled')).toBeDefined();
  });

  it('disables buttons when busy is true', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
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
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        },
        busy: true
      }
    });
    
    const saveButton = wrapper.find('.btn.primary');
    expect(saveButton.text()).toBe('Saving…');
  });

  it('shows save text when busy is false', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        },
        busy: false
      }
    });
    
    const saveButton = wrapper.find('.btn.primary');
    expect(saveButton.text()).toBe('Save');
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    expect(wrapper.find('.prompt-dialog').attributes('role')).toBe('dialog');
    expect(wrapper.find('.prompt-dialog').attributes('aria-labelledby')).toBe('tg-edit-title');
    expect(wrapper.find('.prompt-backdrop').attributes('aria-label')).toBe('Cancel');
  });

  it('updates form when offer prop changes', async () => {
    const wrapper = mount(TabGroupEditPrompt, {
      props: {
        offer: {
          groupId: '1',
          title: 'Work Tabs',
          color: 'Blue'
        }
      }
    });
    
    await wrapper.setProps({
      offer: {
        groupId: '2',
        title: 'Personal Tabs',
        color: 'Red'
      }
    });
    await wrapper.vm.$nextTick();
    
    const titleInput = wrapper.find('input[type="text"]');
    expect(titleInput.element.value).toBe('Personal Tabs');
  });
});
