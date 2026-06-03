/**
 * Exodus Browser — SafeBrowsingPrompt component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SafeBrowsingPrompt from './SafeBrowsingPrompt.vue';

describe('SafeBrowsingPrompt', () => {
  it('does not render when offer is null', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: { offer: null }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(false);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(false);
  });

  it('renders dialog when offer is provided', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    expect(wrapper.find('.prompt-backdrop').exists()).toBe(true);
    expect(wrapper.find('.prompt-dialog').exists()).toBe(true);
  });

  it('displays security warning title', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Security warning');
  });

  it('displays the reason from offer', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    expect(wrapper.find('.reason').text()).toBe('This site is known to distribute malware');
  });

  it('displays the URL from offer', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com/path',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    expect(wrapper.find('.url').text()).toBe('https://malicious-site.com/path');
  });

  it('emits cancel when backdrop is clicked', async () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    await wrapper.find('.prompt-backdrop').trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits cancel when go back button is clicked', async () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    const cancelButton = wrapper.find('.btn.secondary');
    await cancelButton.trigger('click');
    
    expect(wrapper.emitted('cancel')).toBeTruthy();
  });

  it('emits proceed when proceed anyway button is clicked', async () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    const proceedButton = wrapper.find('.btn.danger');
    await proceedButton.trigger('click');
    
    expect(wrapper.emitted('proceed')).toBeTruthy();
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    expect(wrapper.find('.prompt-dialog').attributes('role')).toBe('alertdialog');
    expect(wrapper.find('.prompt-dialog').attributes('aria-labelledby')).toBe('sb-title');
    expect(wrapper.find('.prompt-backdrop').attributes('aria-label')).toBe('Close warning');
  });

  it('has danger styling for proceed button', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    const proceedButton = wrapper.find('.btn.danger');
    expect(proceedButton.classes()).toContain('danger');
  });

  it('has secondary styling for cancel button', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://malicious-site.com',
          reason: 'This site is known to distribute malware'
        }
      }
    });
    
    const cancelButton = wrapper.find('.btn.secondary');
    expect(cancelButton.classes()).toContain('secondary');
  });
});
