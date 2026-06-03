/**
 * Exodus Browser — tab mute API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  registerMuteTab,
  unregisterMuteTab,
  setTabMute,
  getTabMuteState,
  setTabAudioPlaying,
  getAudioPlayingTabs,
  getAllTabMuteStates,
} from './tabMute';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('tabMute', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('registers mute tab', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await registerMuteTab('tab-1');

    expect(invoke).toHaveBeenCalledWith('register_mute_tab', { label: 'tab-1' });
  });

  it('unregisters mute tab', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await unregisterMuteTab('tab-1');

    expect(invoke).toHaveBeenCalledWith('unregister_mute_tab', { label: 'tab-1' });
  });

  it('sets tab mute state', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setTabMute('tab-1', true);

    expect(invoke).toHaveBeenCalledWith('set_tab_mute', { label: 'tab-1', isMuted: true });
  });

  it('gets tab mute state', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const state = await getTabMuteState('tab-1');

    expect(state).toBe(true);
    expect(invoke).toHaveBeenCalledWith('get_tab_mute_state', { label: 'tab-1' });
  });

  it('returns null for non-existent tab mute state', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(null);

    const state = await getTabMuteState('non-existent');

    expect(state).toBe(null);
  });

  it('sets tab audio playing state', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setTabAudioPlaying('tab-1', true);

    expect(invoke).toHaveBeenCalledWith('set_tab_audio_playing', { label: 'tab-1', audioPlaying: true });
  });

  it('gets audio playing tabs', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockTabs = [
      { label: 'tab-1', is_muted: false, audio_playing: true },
      { label: 'tab-2', is_muted: true, audio_playing: true },
    ];
    vi.mocked(invoke).mockResolvedValue(mockTabs);

    const tabs = await getAudioPlayingTabs();

    expect(tabs).toEqual(mockTabs);
    expect(invoke).toHaveBeenCalledWith('get_audio_playing_tabs');
  });

  it('gets all tab mute states', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockStates = [
      { label: 'tab-1', is_muted: false, audio_playing: true },
      { label: 'tab-2', is_muted: true, audio_playing: false },
    ];
    vi.mocked(invoke).mockResolvedValue(mockStates);

    const states = await getAllTabMuteStates();

    expect(states).toEqual(mockStates);
    expect(invoke).toHaveBeenCalledWith('get_all_tab_mute_states');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(registerMuteTab('tab-1')).rejects.toThrow('API error');
  });
});
