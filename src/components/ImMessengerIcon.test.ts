/**
 * Exodus Browser — ImMessengerIcon component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ImMessengerIcon from './ImMessengerIcon.vue';

describe('ImMessengerIcon', () => {
  it('renders chat icon with active fill class', () => {
    const wrapper = mount(ImMessengerIcon, {
      props: { name: 'chat', active: true, size: 24 },
    });

    expect(wrapper.find('svg.im-icon--chat').exists()).toBe(true);
    expect(wrapper.find('svg.im-icon--active').exists()).toBe(true);
    expect(wrapper.find('path[fill="currentColor"]').exists()).toBe(true);
  });

  it('renders toolbar icons by name', () => {
    const names = ['search', 'plus', 'chevron-right', 'chevron-down', 'contacts-manage', 'webchat-logo'] as const;
    for (const name of names) {
      const wrapper = mount(ImMessengerIcon, { props: { name } });
      expect(wrapper.find(`svg.im-icon--${name}`).exists()).toBe(true);
    }
  });
});
