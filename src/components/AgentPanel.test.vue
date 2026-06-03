/**
 * Exodus Browser — AgentPanel component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import AgentPanel from './AgentPanel.vue';

vi.mock('$lib/agentActions', () => ({
  AGENT_PRESETS: [
    { id: 'scroll', label: 'Scroll down', action: { type: 'scroll' } },
    { id: 'click', label: 'Click element', action: { type: 'click' } }
  ]
}));

vi.mock('$lib/hermesStrategies', () => ({
  buildCustomTemplateFromCommand: vi.fn((name, desc, command) => ({
    id: 'custom-1',
    name,
    description: desc,
    command
  })),
  deleteCustomHermesTemplate: vi.fn(() => true),
  isBuiltinHermesTemplate: vi.fn(() => false),
  listHermesStrategyTemplates: vi.fn(() => [
    { id: 'builtin-1', name: 'Builtin Strategy 1', description: 'Test', command: 'test' },
    { id: 'custom-1', name: 'Custom Strategy 1', description: 'Test', command: 'test' }
  ]),
  upsertCustomHermesTemplate: vi.fn()
}));

describe('AgentPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockProps = {
    command: '{"type":"scroll"}',
    log: ['Step 1: Scrolling', 'Step 2: Done'],
    executing: false,
    domSummary: 'Page has 5 elements'
  };

  it('renders agent panel', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-panel').exists()).toBe(true);
  });

  it('renders DOM summary when provided', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-dom-summary').exists()).toBe(true);
    expect(wrapper.find('.agent-dom-summary').text()).toBe('Page has 5 elements');
  });

  it('does not render DOM summary when not provided', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, domSummary: '' }
    });
    
    expect(wrapper.find('.agent-dom-summary').exists()).toBe(false);
  });

  it('renders strategy label', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-strategy-label').text()).toBe('Strategy');
  });

  it('renders strategy select', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-strategy-select').exists()).toBe(true);
  });

  it('renders strategy options', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const options = wrapper.findAll('.agent-strategy-select option');
    expect(options.length).toBe(2);
    expect(options[0].text()).toBe('Builtin Strategy 1');
  });

  it('disables strategy select when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    expect(wrapper.find('.agent-strategy-select').attributes('disabled')).toBeDefined();
  });

  it('renders run strategy button', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.agent-strategy-run');
    expect(buttons.length).toBe(1);
    expect(buttons[0].text()).toBe('Run strategy');
  });

  it('disables run strategy button when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    const runButton = wrapper.find('.agent-strategy-run');
    expect(runButton.attributes('disabled')).toBeDefined();
  });

  it('disables run strategy button when no strategy selected', () => {
    const { listHermesStrategyTemplates } = require('$lib/hermesStrategies');
    listHermesStrategyTemplates.mockReturnValue([]);
    
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const runButton = wrapper.find('.agent-strategy-run');
    expect(runButton.attributes('disabled')).toBeDefined();
  });

  it('emits run-strategy when run button is clicked', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    await wrapper.find('.agent-strategy-run').trigger('click');
    
    expect(wrapper.emitted('run-strategy')).toBeTruthy();
  });

  it('renders save button', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.agent-preset-btn');
    expect(buttons.some(b => b.text() === 'Save…')).toBe(true);
  });

  it('toggles save form on save button click', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.agent-save-strategy').exists()).toBe(true);
    expect(saveButton.text()).toBe('Cancel');
  });

  it('hides save form on cancel', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.agent-save-strategy').exists()).toBe(false);
  });

  it('renders delete button for custom strategies', () => {
    const { isBuiltinHermesTemplate } = require('$lib/hermesStrategies');
    isBuiltinHermesTemplate.mockReturnValue(false);
    
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-strategy-delete').exists()).toBe(true);
  });

  it('does not render delete button for builtin strategies', () => {
    const { isBuiltinHermesTemplate } = require('$lib/hermesStrategies');
    isBuiltinHermesTemplate.mockReturnValue(true);
    
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-strategy-delete').exists()).toBe(false);
  });

  it('renders save form inputs', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('#strategy-save-name').exists()).toBe(true);
    expect(wrapper.find('#strategy-save-desc').exists()).toBe(true);
  });

  it('has correct placeholders on save form inputs', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('#strategy-save-name').attributes('placeholder')).toBe('My workflow');
    expect(wrapper.find('#strategy-save-desc').attributes('placeholder')).toBe('Optional note');
  });

  it('disables save form inputs when executing', async () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('#strategy-save-name').attributes('disabled')).toBeDefined();
  });

  it('renders save button in save form', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.agent-save-strategy .agent-btn-small').text()).toBe('Save to browser');
  });

  it('disables save button when command is empty', async () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, command: '' }
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.agent-save-strategy .agent-btn-small').attributes('disabled')).toBeDefined();
  });

  it('renders preset buttons', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const presetButtons = wrapper.findAll('.agent-quick-row .agent-preset-btn');
    expect(presetButtons.length).toBe(2);
    expect(presetButtons[0].text()).toBe('Scroll down');
    expect(presetButtons[1].text()).toBe('Click element');
  });

  it('emits preset when preset button is clicked', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    await wrapper.findAll('.agent-quick-row .agent-preset-btn')[0].trigger('click');
    
    expect(wrapper.emitted('preset')).toBeTruthy();
    expect(wrapper.emitted('preset')?.[0]).toEqual(['{"type":"scroll"}']);
  });

  it('disables preset buttons when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    const presetButtons = wrapper.findAll('.agent-quick-row .agent-preset-btn');
    presetButtons.forEach(btn => {
      expect(btn.attributes('disabled')).toBeDefined();
    });
  });

  it('renders command input', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-input').exists()).toBe(true);
  });

  it('displays command value', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-input').element.value).toBe('{"type":"scroll"}');
  });

  it('has correct placeholder on command input', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-input').attributes('placeholder')).toBe('JSON, plan: goal, scroll down, or ask: question');
  });

  it('disables command input when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    expect(wrapper.find('.agent-input').attributes('disabled')).toBeDefined();
  });

  it('emits command-change on input', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    await wrapper.find('.agent-input').setValue('{"type":"click"}');
    
    expect(wrapper.emitted('command-change')).toBeTruthy();
    expect(wrapper.emitted('command-change')?.[0]).toEqual(['{"type":"click"}']);
  });

  it('emits execute on Enter key', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    await wrapper.find('.agent-input').trigger('keydown.enter');
    
    expect(wrapper.emitted('execute')).toBeTruthy();
  });

  it('renders action buttons', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.agent-buttons button');
    expect(buttons.length).toBe(4);
  });

  it('renders run button', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-btn-primary').text()).toBe('Run');
  });

  it('displays running text when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    expect(wrapper.find('.agent-btn-primary').text()).toBe('Running…');
  });

  it('disables run button when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    expect(wrapper.find('.agent-btn-primary').attributes('disabled')).toBeDefined();
  });

  it('emits execute when run button is clicked', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    await wrapper.find('.agent-btn-primary').trigger('click');
    
    expect(wrapper.emitted('execute')).toBeTruthy();
  });

  it('emits compress when compress button is clicked', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.agent-buttons .agent-btn-small');
    await buttons[0].trigger('click');
    
    expect(wrapper.emitted('compress')).toBeTruthy();
  });

  it('emits ask-ai when ask AI button is clicked', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.agent-buttons .agent-btn-small');
    await buttons[1].trigger('click');
    
    expect(wrapper.emitted('ask-ai')).toBeTruthy();
  });

  it('emits back when back button is clicked', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.agent-buttons .agent-btn-small');
    await buttons[2].trigger('click');
    
    expect(wrapper.emitted('back')).toBeTruthy();
  });

  it('disables compress button when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    const buttons = wrapper.findAll('.agent-buttons .agent-btn-small');
    expect(buttons[0].attributes('disabled')).toBeDefined();
  });

  it('disables ask AI button when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    const buttons = wrapper.findAll('.agent-buttons .agent-btn-small');
    expect(buttons[1].attributes('disabled')).toBeDefined();
  });

  it('does not disable back button when executing', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, executing: true }
    });
    
    const buttons = wrapper.findAll('.agent-buttons .agent-btn-small');
    expect(buttons[2].attributes('disabled')).toBeUndefined();
  });

  it('renders log area', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-log').exists()).toBe(true);
  });

  it('has correct ARIA attributes on log', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    expect(wrapper.find('.agent-log').attributes('role')).toBe('log');
    expect(wrapper.find('.agent-log').attributes('aria-live')).toBe('polite');
  });

  it('renders log entries', () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const logEntries = wrapper.findAll('.log-entry');
    expect(logEntries.length).toBe(2);
    expect(logEntries[0].text()).toBe('Step 1: Scrolling');
  });

  it('shows empty log message when no entries', () => {
    const wrapper = mount(AgentPanel, {
      props: { ...mockProps, log: [] }
    });
    
    expect(wrapper.find('.agent-log-empty').exists()).toBe(true);
    expect(wrapper.find('.agent-log-empty').text()).toContain('Run a preset or enter JSON');
  });

  it('emits strategy-saved on successful save', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('#strategy-save-name').setValue('Test Strategy');
    await wrapper.find('.agent-save-strategy .agent-btn-small').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('strategy-saved')).toBeTruthy();
  });

  it('shows save error on save failure', async () => {
    const { buildCustomTemplateFromCommand } = require('$lib/hermesStrategies');
    buildCustomTemplateFromCommand.mockImplementation(() => {
      throw new Error('Invalid command');
    });
    
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    const saveButton = wrapper.findAll('.agent-preset-btn').find(b => b.text() === 'Save…');
    await saveButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.agent-save-strategy .agent-btn-small').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.agent-save-error').exists()).toBe(true);
    expect(wrapper.find('.agent-save-error').text()).toBe('Invalid command');
  });

  it('emits strategy-saved on delete', async () => {
    const wrapper = mount(AgentPanel, {
      props: mockProps
    });
    
    await wrapper.find('.agent-strategy-delete').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('strategy-saved')).toBeTruthy();
    expect(wrapper.emitted('strategy-saved')?.[0]).toEqual(['Custom strategy deleted']);
  });
});
