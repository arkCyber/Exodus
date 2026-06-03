/**
 * Exodus Browser — LoadingSpinner component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import LoadingSpinner from './LoadingSpinner.vue';

describe('LoadingSpinner', () => {
  it('renders with default props', () => {
    const wrapper = mount(LoadingSpinner);
    
    expect(wrapper.find('.loading-spinner').exists()).toBe(true);
    expect(wrapper.find('.spinner').exists()).toBe(true);
    expect(wrapper.classes()).toContain('medium');
  });

  it('renders with small size', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { size: 'small' }
    });
    
    expect(wrapper.classes()).toContain('small');
  });

  it('renders with large size', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { size: 'large' }
    });
    
    expect(wrapper.classes()).toContain('large');
  });

  it('renders with message', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { message: 'Loading...' }
    });
    
    expect(wrapper.find('.loading-message').exists()).toBe(true);
    expect(wrapper.find('.loading-message').text()).toBe('Loading...');
  });

  it('does not render message when not provided', () => {
    const wrapper = mount(LoadingSpinner);
    
    expect(wrapper.find('.loading-message').exists()).toBe(false);
  });

  it('renders spinner element', () => {
    const wrapper = mount(LoadingSpinner);
    
    const spinner = wrapper.find('.spinner');
    expect(spinner.exists()).toBe(true);
  });
});
