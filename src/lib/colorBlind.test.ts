/**
 * Exodus Browser — color blind mode API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  enableColorBlind,
  disableColorBlind,
  isColorBlindEnabled,
  setColorBlindType,
  getColorBlindType,
  setColorBlindIntensity,
  getColorBlindIntensity,
  getColorBlindSettings,
} from './colorBlind';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('colorBlind', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('enables color blind mode', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await enableColorBlind();

    expect(invoke).toHaveBeenCalledWith('enable_color_blind');
  });

  it('disables color blind mode', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await disableColorBlind();

    expect(invoke).toHaveBeenCalledWith('disable_color_blind');
  });

  it('checks if color blind mode is enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const enabled = await isColorBlindEnabled();

    expect(enabled).toBe(true);
    expect(invoke).toHaveBeenCalledWith('is_color_blind_enabled');
  });

  it('sets color blind type', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setColorBlindType('protanopia');

    expect(invoke).toHaveBeenCalledWith('set_color_blind_type', { colorBlindType: 'protanopia' });
  });

  it('gets color blind type', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('deuteranopia');

    const type = await getColorBlindType();

    expect(type).toBe('deuteranopia');
    expect(invoke).toHaveBeenCalledWith('get_color_blind_type');
  });

  it('sets color blind intensity', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setColorBlindIntensity(0.8);

    expect(invoke).toHaveBeenCalledWith('set_color_blind_intensity', { intensity: 0.8 });
  });

  it('gets color blind intensity', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(0.5);

    const intensity = await getColorBlindIntensity();

    expect(intensity).toBe(0.5);
    expect(invoke).toHaveBeenCalledWith('get_color_blind_intensity');
  });

  it('gets color blind settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockSettings = {
      enabled: true,
      color_blind_type: 'protanopia',
      intensity: 0.8,
    };
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const settings = await getColorBlindSettings();

    expect(settings).toEqual(mockSettings);
    expect(invoke).toHaveBeenCalledWith('get_color_blind_settings');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(enableColorBlind()).rejects.toThrow('API error');
  });
});
