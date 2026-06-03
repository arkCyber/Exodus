/**
 * Exodus Browser — MentionMessageBody Vue component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import MentionMessageBody from './MentionMessageBody.vue';

describe('MentionMessageBody', () => {
  it('renders plain text without mention tokens', () => {
    const wrapper = mount(MentionMessageBody, {
      props: { content: 'Hello world' },
    });
    expect(wrapper.text()).toBe('Hello world');
    expect(wrapper.find('.mention-name').exists()).toBe(false);
  });

  it('renders clickable mention and emits chat action', async () => {
    const wrapper = mount(MentionMessageBody, {
      props: { content: 'Hi @[Alice](node:peer-node-alice-001) there' },
    });
    expect(wrapper.find('.mention-name').text()).toBe('@Alice');
    await wrapper.find('.mention-name').trigger('click');
    expect(wrapper.emitted('mentionAction')?.[0]).toEqual([
      { nodeId: 'peer-node-alice-001', displayName: 'Alice' },
      'chat',
    ]);
  });

  it('emits voice call action from mention toolbar', async () => {
    const wrapper = mount(MentionMessageBody, {
      props: { content: '@[Bob](node:peer-node-bob-002)' },
    });
    const voiceBtn = wrapper.findAll('.mention-act')[0];
    await voiceBtn.trigger('click');
    expect(wrapper.emitted('mentionAction')?.[0]?.[1]).toBe('voice');
  });
});
