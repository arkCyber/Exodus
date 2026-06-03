/**
 * Exodus Browser — audio visualization API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  enableAudioVisualization,
  disableAudioVisualization,
  isAudioVisualizationEnabled,
  setVisualizationType,
  getVisualizationType,
  setVisualizationSensitivity,
  getVisualizationSensitivity,
  setVisualizationSmoothing,
  getVisualizationSmoothing,
  setVisualizationColorScheme,
  getVisualizationColorScheme,
  getAudioVisualizationSettings,
} from './audioVisualization';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('audioVisualization', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('enables audio visualization', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await enableAudioVisualization();

    expect(invoke).toHaveBeenCalledWith('enable_audio_visualization');
  });

  it('disables audio visualization', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await disableAudioVisualization();

    expect(invoke).toHaveBeenCalledWith('disable_audio_visualization');
  });

  it('checks if audio visualization is enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const enabled = await isAudioVisualizationEnabled();

    expect(enabled).toBe(true);
    expect(invoke).toHaveBeenCalledWith('is_audio_visualization_enabled');
  });

  it('sets visualization type', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setVisualizationType('bars');

    expect(invoke).toHaveBeenCalledWith('set_visualization_type', { vizType: 'bars' });
  });

  it('gets visualization type', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('waveform');

    const vizType = await getVisualizationType();

    expect(vizType).toBe('waveform');
    expect(invoke).toHaveBeenCalledWith('get_visualization_type');
  });

  it('sets visualization sensitivity', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setVisualizationSensitivity(0.8);

    expect(invoke).toHaveBeenCalledWith('set_visualization_sensitivity', { sensitivity: 0.8 });
  });

  it('gets visualization sensitivity', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(0.5);

    const sensitivity = await getVisualizationSensitivity();

    expect(sensitivity).toBe(0.5);
    expect(invoke).toHaveBeenCalledWith('get_visualization_sensitivity');
  });

  it('sets visualization smoothing', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setVisualizationSmoothing(0.9);

    expect(invoke).toHaveBeenCalledWith('set_visualization_smoothing', { smoothing: 0.9 });
  });

  it('gets visualization smoothing', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(0.7);

    const smoothing = await getVisualizationSmoothing();

    expect(smoothing).toBe(0.7);
    expect(invoke).toHaveBeenCalledWith('get_visualization_smoothing');
  });

  it('sets visualization color scheme', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setVisualizationColorScheme('rainbow');

    expect(invoke).toHaveBeenCalledWith('set_visualization_color_scheme', { scheme: 'rainbow' });
  });

  it('gets visualization color scheme', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('gradient');

    const scheme = await getVisualizationColorScheme();

    expect(scheme).toBe('gradient');
    expect(invoke).toHaveBeenCalledWith('get_visualization_color_scheme');
  });

  it('gets audio visualization settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockSettings = {
      enabled: true,
      visualization_type: 'bars',
      sensitivity: 0.8,
      smoothing: 0.9,
      color_scheme: 'rainbow',
    };
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const settings = await getAudioVisualizationSettings();

    expect(settings).toEqual(mockSettings);
    expect(invoke).toHaveBeenCalledWith('get_audio_visualization_settings');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(enableAudioVisualization()).rejects.toThrow('API error');
  });
});
