/**
 * Exodus Browser — data saver API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  enableDataSaver,
  disableDataSaver,
  isDataSaverEnabled,
  setDataSaverBlockImages,
  setDataSaverBlockVideos,
  setDataSaverBlockAutoplay,
  setDataSaverCompressImages,
  setDataSaverQualityLevel,
  recordDataSaverBytes,
  getDataSaverBytesSaved,
  resetDataSaverBytes,
  getDataSaverSettings,
} from './dataSaver';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('dataSaver', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('enables data saver', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await enableDataSaver();

    expect(invoke).toHaveBeenCalledWith('enable_data_saver');
  });

  it('disables data saver', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await disableDataSaver();

    expect(invoke).toHaveBeenCalledWith('disable_data_saver');
  });

  it('checks if data saver is enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const enabled = await isDataSaverEnabled();

    expect(enabled).toBe(true);
    expect(invoke).toHaveBeenCalledWith('is_data_saver_enabled');
  });

  it('sets block images', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setDataSaverBlockImages(true);

    expect(invoke).toHaveBeenCalledWith('set_data_saver_block_images', { block: true });
  });

  it('sets block videos', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setDataSaverBlockVideos(false);

    expect(invoke).toHaveBeenCalledWith('set_data_saver_block_videos', { block: false });
  });

  it('sets block autoplay media', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setDataSaverBlockAutoplay(true);

    expect(invoke).toHaveBeenCalledWith('set_data_saver_block_autoplay', { block: true });
  });

  it('sets compress images', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setDataSaverCompressImages(true);

    expect(invoke).toHaveBeenCalledWith('set_data_saver_compress_images', { compress: true });
  });

  it('sets quality level', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setDataSaverQualityLevel(80);

    expect(invoke).toHaveBeenCalledWith('set_data_saver_quality_level', { level: 80 });
  });

  it('records bytes saved', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await recordDataSaverBytes(1024);

    expect(invoke).toHaveBeenCalledWith('record_data_saver_bytes', { bytes: 1024 });
  });

  it('gets bytes saved', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(1048576);

    const bytes = await getDataSaverBytesSaved();

    expect(bytes).toBe(1048576);
    expect(invoke).toHaveBeenCalledWith('get_data_saver_bytes_saved');
  });

  it('resets bytes saved', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await resetDataSaverBytes();

    expect(invoke).toHaveBeenCalledWith('reset_data_saver_bytes');
  });

  it('gets data saver settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockSettings = {
      enabled: true,
      block_images: true,
      block_videos: false,
      block_autoplay_media: true,
      compress_images: true,
      quality_level: 80,
    };
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const settings = await getDataSaverSettings();

    expect(settings).toEqual(mockSettings);
    expect(invoke).toHaveBeenCalledWith('get_data_saver_settings');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(enableDataSaver()).rejects.toThrow('API error');
  });
});
