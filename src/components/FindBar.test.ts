import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import FindBar from './FindBar.vue';

describe('FindBar', () => {
  it('renders when open is true', () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findQuery: '',
        findResults: 0,
        currentFindIndex: 0,
      },
    });
    
    expect(wrapper.find('.find-bar').exists()).toBe(true);
  });

  it('does not render when open is false', () => {
    const wrapper = mount(FindBar, {
      props: {
        open: false,
        findQuery: '',
        findResults: 0,
        currentFindIndex: 0,
      },
    });
    
    expect(wrapper.find('.find-bar').exists()).toBe(false);
  });

  it('emits find-input event when input changes', async () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findQuery: '',
        findResults: 0,
        currentFindIndex: 0,
      },
    });
    
    const input = wrapper.find('.find-input');
    await input.setValue('test');
    await input.trigger('input');
    
    expect(wrapper.emitted('findInput')).toBeTruthy();
  });

  it('emits close event when close button is clicked', async () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findQuery: '',
        findResults: 0,
        currentFindIndex: 0,
      },
    });
    
    await wrapper.find('.find-btn.close').trigger('click');
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('displays find results', () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findQuery: 'test',
        findResults: 5,
        currentFindIndex: 2,
      },
    });
    
    expect(wrapper.text()).toContain('2/5');
  });
});
