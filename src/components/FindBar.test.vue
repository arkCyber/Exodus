/**
 * Exodus Browser — FindBar component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import FindBar from './FindBar.vue';

describe('FindBar', () => {
  it('does not render when open is false', () => {
    const wrapper = mount(FindBar, {
      props: { open: false }
    });
    
    expect(wrapper.find('.find-bar').exists()).toBe(false);
  });

  it('renders when open is true', () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    expect(wrapper.find('.find-bar').exists()).toBe(true);
  });

  it('renders find input with correct placeholder', () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const input = wrapper.find('.find-input');
    expect(input.exists()).toBe(true);
    expect(input.attributes('placeholder')).toBe('Find in page...');
  });

  it('displays find count with results', () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findResults: 5,
        currentFindIndex: 2
      }
    });
    
    expect(wrapper.find('.find-count').text()).toBe('2/5');
  });

  it('displays 0/0 when no results', () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findResults: 0,
        currentFindIndex: 0
      }
    });
    
    expect(wrapper.find('.find-count').text()).toBe('0/0');
  });

  it('emits update:findQuery on input', async () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const input = wrapper.find('.find-input');
    await input.setValue('test query');
    
    expect(wrapper.emitted('update:findQuery')).toBeTruthy();
    expect(wrapper.emitted('update:findQuery')?.[0]).toEqual(['test query']);
  });

  it('emits findInput on input', async () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const input = wrapper.find('.find-input');
    await input.setValue('test');
    
    expect(wrapper.emitted('findInput')).toBeTruthy();
  });

  it('emits find with next on Enter key', async () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const input = wrapper.find('.find-input');
    await input.trigger('keydown.enter');
    
    expect(wrapper.emitted('find')).toBeTruthy();
    expect(wrapper.emitted('find')?.[0]).toEqual(['next']);
  });

  it('emits find with prev when previous button is clicked', async () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const buttons = wrapper.findAll('.find-btn');
    await buttons[0].trigger('click');
    
    expect(wrapper.emitted('find')).toBeTruthy();
    expect(wrapper.emitted('find')?.[0]).toEqual(['prev']);
  });

  it('emits find with next when next button is clicked', async () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const buttons = wrapper.findAll('.find-btn');
    await buttons[1].trigger('click');
    
    expect(wrapper.emitted('find')).toBeTruthy();
    expect(wrapper.emitted('find')?.[0]).toEqual(['next']);
  });

  it('emits close when close button is clicked', async () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    const buttons = wrapper.findAll('.find-btn');
    await buttons[2].trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(FindBar, {
      props: { open: true }
    });
    
    expect(wrapper.find('.find-bar').attributes('role')).toBe('search');
    expect(wrapper.find('.find-input').attributes('aria-label')).toBe('Find in page');
    expect(wrapper.find('.find-count').attributes('aria-live')).toBe('polite');
  });

  it('binds findQuery to input value', () => {
    const wrapper = mount(FindBar, {
      props: {
        open: true,
        findQuery: 'search term'
      }
    });
    
    expect(wrapper.find('.find-input').element.value).toBe('search term');
  });
});
