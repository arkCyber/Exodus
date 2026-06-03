/**
 * Exodus Browser — GPU settings component tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import GpuSettings from '@/components/settings/GpuSettings.vue';
import { invoke } from '@tauri-apps/api/tauri';

// Mock Tauri API
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
}));

describe('GpuSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('renders GPU information when available', async () => {
    (invoke as any).mockResolvedValue({
      vendor: 'NVIDIA',
      renderer: 'GeForce RTX 3080',
      driver_version: '515.65.01',
      api_type: 'OpenGL/Vulkan',
      max_texture_size: 16384,
      max_viewport_dims: [16384, 16384],
    });

    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();
    expect(wrapper.text()).toContain('GPU Information');
  });

  it('renders WebGL support information', async () => {
    (invoke as any).mockImplementation((cmd: string) => {
      if (cmd === 'get_webgl_support') {
        return Promise.resolve({
          webgl1_available: true,
          webgl2_available: true,
          webgl_version: '2.0',
          max_texture_size: 16384,
          max_renderbuffer_size: 16384,
        });
      }
      return Promise.resolve(null);
    });

    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();
    expect(wrapper.text()).toContain('WebGL');
  });

  it('renders WebGPU support information', async () => {
    (invoke as any).mockImplementation((cmd: string) => {
      if (cmd === 'get_webgpu_support') {
        return Promise.resolve({
          available: true,
          adapter_info: 'Metal backend',
          features: ['timestamp-query', 'pipeline-statistics-query'],
        });
      }
      return Promise.resolve(null);
    });

    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();
    expect(wrapper.text()).toContain('WebGPU');
  });

  it('toggles GPU acceleration', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const checkbox = wrapper.find('[data-testid="gpu-acceleration-enabled"]');
    await checkbox.setValue(false);
    await flushPromises();

    expect(invoke).toHaveBeenCalledWith('disable_gpu_acceleration');
  });

  it('toggles WebGL enabled', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const checkbox = wrapper.find('[data-testid="webgl-enabled"]');
    await checkbox.setValue(false);
    await flushPromises();

    expect(invoke).toHaveBeenCalledWith('set_webgl_enabled', { enabled: false });
  });

  it('toggles WebGPU enabled', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const checkbox = wrapper.find('[data-testid="webgpu-enabled"]');
    await checkbox.setValue(false);
    await flushPromises();

    expect(invoke).toHaveBeenCalledWith('set_webgpu_enabled', { enabled: false });
  });

  it('changes ANGLE backend', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const select = wrapper.find('[data-testid="angle-backend"]');
    await select.setValue('metal');
    await flushPromises();

    expect(invoke).toHaveBeenCalledWith('set_angle_backend', { backend: 'metal' });
  });

  it('refreshes performance metrics', async () => {
    (invoke as any).mockImplementation((cmd: string) => {
      if (cmd === 'get_gpu_performance_metrics') {
        return Promise.resolve({
          timestamp: Date.now(),
          memory_used: 1024 * 1024 * 512,
          memory_total: 1024 * 1024 * 8192,
          gpu_utilization: 0.45,
          temperature: 65.5,
          power_usage: 120.0,
        });
      }
      return Promise.resolve(null);
    });

    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const button = wrapper.find('[data-testid="refresh-metrics"]');
    await button.trigger('click');
    await flushPromises();

    expect(invoke).toHaveBeenCalledWith('get_gpu_performance_metrics');
  });

  it('resets to default settings', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const button = wrapper.find('[data-testid="gpu-reset"]');
    await button.trigger('click');
    await flushPromises();

    expect(localStorage.getItem('exodus-gpu-settings')).toBeTruthy();
  });

  it('emits status message on save', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const checkbox = wrapper.find('[data-testid="gpu-acceleration-enabled"]');
    await checkbox.setValue(false);
    await flushPromises();

    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['GPU settings saved']);
  });

  it('emits status message on error', async () => {
    (invoke as any).mockRejectedValue(new Error('Failed to save'));

    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    const checkbox = wrapper.find('[data-testid="gpu-acceleration-enabled"]');
    await checkbox.setValue(false);
    await flushPromises();

    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Failed to save GPU settings']);
  });

  it('formats bytes correctly', async () => {
    const wrapper = mount(GpuSettings, {
      props: {
        uiLocale: 'en',
      },
    });

    await flushPromises();

    // Test the formatBytes function through the component
    const vm = wrapper.vm as any;
    expect(vm.formatBytes(0)).toBe('0 B');
    expect(vm.formatBytes(1024)).toBe('1 KB');
    expect(vm.formatBytes(1024 * 1024)).toBe('1 MB');
    expect(vm.formatBytes(1024 * 1024 * 1024)).toBe('1 GB');
  });
});
