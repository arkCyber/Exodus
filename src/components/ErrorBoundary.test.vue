/**
 * Exodus Browser — ErrorBoundary component tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import ErrorBoundary from './ErrorBoundary.vue';

describe('ErrorBoundary', () => {
  it('renders slot content when no error', () => {
    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: '<div class="test-content">Test Content</div>'
      }
    });
    
    expect(wrapper.find('.error-boundary').exists()).toBe(false);
    expect(wrapper.find('.test-content').exists()).toBe(true);
  });

  it('renders error UI when error occurs', async () => {
    const wrapper = mount(ErrorBoundary, {
      slots: {
        default: '<div class="test-content">Test Content</div>'
      }
    });
    
    // Simulate error by directly setting the internal error ref
    // This is a workaround since onErrorCaptured is hard to test directly
    const component = wrapper.vm as any;
    component.error = new Error('Test error');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.error-boundary').exists()).toBe(true);
    expect(wrapper.find('.error-content h2').text()).toBe('Something went wrong');
    expect(wrapper.find('.error-content p').text()).toBe('Test error');
  });

  it('displays error stack when available', async () => {
    const wrapper = mount(ErrorBoundary);
    
    const component = wrapper.vm as any;
    const testError = new Error('Test error');
    testError.stack = 'Error: Test error\n    at test.js:1:1';
    component.error = testError;
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.error-stack').exists()).toBe(true);
    expect(wrapper.find('.error-stack pre').text()).toContain('Error: Test error');
  });

  it('resets error when retry button is clicked', async () => {
    const wrapper = mount(ErrorBoundary);
    
    const component = wrapper.vm as any;
    component.error = new Error('Test error');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.error-boundary').exists()).toBe(true);
    
    await wrapper.find('.retry-button').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.error-boundary').exists()).toBe(false);
  });

  it('calls resetError function on retry', async () => {
    const wrapper = mount(ErrorBoundary);
    
    const component = wrapper.vm as any;
    component.error = new Error('Test error');
    await wrapper.vm.$nextTick();
    
    const resetSpy = vi.spyOn(component, 'resetError');
    await wrapper.find('.retry-button').trigger('click');
    
    expect(resetSpy).toHaveBeenCalled();
  });
});
