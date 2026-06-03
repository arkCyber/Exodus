/**
 * Exodus Browser — InferenceEngineSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import InferenceEngineSettings from './InferenceEngineSettings.vue';

vi.mock('$lib/inferenceClient', () => ({
  inferenceGetLoadedModel: vi.fn(),
  inferenceGetStats: vi.fn(),
  inferenceGetStatus: vi.fn(),
  inferenceListModels: vi.fn(),
  inferenceLoadModel: vi.fn(),
  inferenceUnloadModel: vi.fn()
}));

describe('InferenceEngineSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    expect(wrapper.find('h3').text()).toBe('Inference engine');
  });

  it('renders hint', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    expect(wrapper.find('.hint').text()).toBe('Local model runtime');
  });

  it('renders model label', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    expect(wrapper.find('label').text()).toContain('Model');
  });

  it('renders model select', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    expect(wrapper.find('select').exists()).toBe(true);
  });

  it('renders model options', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([
      { name: 'llama2', loaded: false },
      { name: 'mistral', loaded: true }
    ]);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const options = wrapper.findAll('option');
    expect(options.length).toBe(2);
  });

  it('displays loaded indicator for loaded models', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([
      { name: 'llama2', loaded: true },
      { name: 'mistral', loaded: false }
    ]);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const options = wrapper.findAll('option');
    expect(options[0].text()).toContain('(loaded)');
  });

  it('renders toolbar', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders refresh button', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Refresh');
  });

  it('renders load button', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].text()).toBe('Load');
  });

  it('renders unload button', () => {
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[2].text()).toBe('Unload');
  });

  it('disables buttons when busy', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
      return [];
    });
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.nav-button');
    buttons.forEach(btn => {
      expect(btn.attributes('disabled')).toBeDefined();
    });
  });

  it('disables load button when no model selected', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([]);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].attributes('disabled')).toBeDefined();
  });

  it('refreshes on refresh button click', async () => {
    const { inferenceListModels, inferenceGetLoadedModel, inferenceGetStatus, inferenceGetStats } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([]);
    inferenceGetLoadedModel.mockResolvedValue(null);
    inferenceGetStatus.mockResolvedValue('ready');
    inferenceGetStats.mockResolvedValue({});
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(inferenceListModels).toHaveBeenCalled();
  });

  it('loads model on load button click', async () => {
    const { inferenceListModels, inferenceLoadModel } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([{ name: 'llama2', loaded: false }]);
    inferenceLoadModel.mockResolvedValue(undefined);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(inferenceLoadModel).toHaveBeenCalledWith('llama2');
  });

  it('emits status on successful load', async () => {
    const { inferenceListModels, inferenceLoadModel } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([{ name: 'llama2', loaded: false }]);
    inferenceLoadModel.mockResolvedValue(undefined);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Loaded llama2']);
  });

  it('emits status on failed load', async () => {
    const { inferenceListModels, inferenceLoadModel } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([{ name: 'llama2', loaded: false }]);
    inferenceLoadModel.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toContain('Load failed');
  });

  it('unloads model on unload button click', async () => {
    const { inferenceUnloadModel } = require('$lib/inferenceClient');
    inferenceUnloadModel.mockResolvedValue(undefined);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await wrapper.findAll('.nav-button')[2].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(inferenceUnloadModel).toHaveBeenCalled();
  });

  it('emits status on successful unload', async () => {
    const { inferenceUnloadModel } = require('$lib/inferenceClient');
    inferenceUnloadModel.mockResolvedValue(undefined);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await wrapper.findAll('.nav-button')[2].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Model unloaded']);
  });

  it('emits status on failed unload', async () => {
    const { inferenceUnloadModel } = require('$lib/inferenceClient');
    inferenceUnloadModel.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await wrapper.findAll('.nav-button')[2].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toContain('Unload failed');
  });

  it('displays stats preview', async () => {
    const { inferenceListModels, inferenceGetStats } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([]);
    inferenceGetStats.mockResolvedValue({ memory: '1GB', tokens: 1000 });
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.mono').exists()).toBe(true);
  });

  it('truncates stats preview', async () => {
    const { inferenceListModels, inferenceGetStats } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([]);
    inferenceGetStats.mockResolvedValue({ memory: '1GB', tokens: 1000, longField: 'x'.repeat(300) });
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.mono').text().length).toBeLessThanOrEqual(200);
  });

  it('selects aiModel prop when available', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([
      { name: 'llama2', loaded: false },
      { name: 'mistral', loaded: false }
    ]);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.selectedModel).toBe('llama2');
  });

  it('selects loaded model when available', async () => {
    const { inferenceListModels, inferenceGetLoadedModel } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([
      { name: 'llama2', loaded: false },
      { name: 'mistral', loaded: false }
    ]);
    inferenceGetLoadedModel.mockResolvedValue('mistral');
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.selectedModel).toBe('mistral');
  });

  it('selects first model when no aiModel or loaded model', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([
      { name: 'llama2', loaded: false },
      { name: 'mistral', loaded: false }
    ]);
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: '' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.selectedModel).toBe('llama2');
  });

  it('loads data on mount', async () => {
    const { inferenceListModels, inferenceGetLoadedModel, inferenceGetStatus, inferenceGetStats } = require('$lib/inferenceClient');
    inferenceListModels.mockResolvedValue([]);
    inferenceGetLoadedModel.mockResolvedValue(null);
    inferenceGetStatus.mockResolvedValue('ready');
    inferenceGetStats.mockResolvedValue({});
    
    mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(inferenceListModels).toHaveBeenCalled();
    expect(inferenceGetLoadedModel).toHaveBeenCalled();
    expect(inferenceGetStatus).toHaveBeenCalled();
    expect(inferenceGetStats).toHaveBeenCalled();
  });

  it('emits status on refresh error', async () => {
    const { inferenceListModels } = require('$lib/inferenceClient');
    inferenceListModels.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(InferenceEngineSettings, {
      props: { aiModel: 'llama2' }
    });
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toContain('Inference');
  });
});
