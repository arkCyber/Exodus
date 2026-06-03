import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import AgentPanel from './AgentPanel.vue';

describe('AgentPanel', () => {
  it('renders run button and log area', () => {
    const wrapper = mount(AgentPanel, {
      props: {
        command: 'scroll down',
        log: ['ok'],
        executing: false,
        domSummary: '',
      },
    });
    expect(wrapper.text()).toContain('Run');
    expect(wrapper.text()).toContain('ok');
  });

  it('emits execute on run click', async () => {
    const wrapper = mount(AgentPanel, {
      props: {
        command: '',
        log: [],
        executing: false,
        domSummary: '',
      },
    });
    await wrapper.find('.agent-btn-primary').trigger('click');
    expect(wrapper.emitted('execute')).toBeTruthy();
  });
});
