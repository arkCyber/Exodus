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
    expect(wrapper.find('.loading-spinner').classes()).toContain('medium');
  });

  it('renders with small size', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { size: 'small' },
    });
    expect(wrapper.find('.loading-spinner').classes()).toContain('small');
  });

  it('renders with large size', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { size: 'large' },
    });
    expect(wrapper.find('.loading-spinner').classes()).toContain('large');
  });

  it('renders without message by default', () => {
    const wrapper = mount(LoadingSpinner);
    expect(wrapper.find('.loading-message').exists()).toBe(false);
  });

  it('renders with message', () => {
    const wrapper = mount(LoadingSpinner, {
      props: { message: 'Loading...' },
    });
    expect(wrapper.find('.loading-message').exists()).toBe(true);
    expect(wrapper.find('.loading-message').text()).toBe('Loading...');
  });

  it('renders spinner element', () => {
    const wrapper = mount(LoadingSpinner);
    expect(wrapper.find('.spinner').exists()).toBe(true);
  });
});
