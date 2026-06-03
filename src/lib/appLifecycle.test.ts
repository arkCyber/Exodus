/**
 * Unit tests for app lifecycle TypeScript API.
 */

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

import {
  getLifecycleStatus,
  runLifecycleHealthTick,
  setLifecycleAutoFix,
} from './appLifecycle';
import { invoke } from '@tauri-apps/api/core';

describe('appLifecycle', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('getLifecycleStatus invokes backend command', async () => {
    vi.mocked(invoke).mockResolvedValue({
      phase: 'running',
      started_at: '2026-01-01T00:00:00Z',
      uptime_secs: 10,
      launch_mode: 'dev_binary',
      scheduler_active: true,
      tick_count: 3,
      components: [],
      last_error: null,
    });
    vi.mocked(invoke).mockResolvedValue({
      phase: 'running',
      started_at: '2026-01-01T00:00:00Z',
      uptime_secs: 10,
      launch_mode: 'dev_binary',
      scheduler_active: true,
      tick_count: 3,
      auto_fix_enabled: true,
      components: [],
      recent_remediations: [],
      last_error: null,
    });
    const status = await getLifecycleStatus();
    expect(invoke).toHaveBeenCalledWith('lifecycle_get_status');
    expect(status.phase).toBe('running');
    expect(status.auto_fix_enabled).toBe(true);
  });

  it('runLifecycleHealthTick invokes lifecycle_run_health_tick', async () => {
    vi.mocked(invoke).mockResolvedValue({
      phase: 'running',
      started_at: '2026-01-01T00:00:00Z',
      uptime_secs: 1,
      launch_mode: 'dev_binary',
      scheduler_active: true,
      tick_count: 1,
      auto_fix_enabled: true,
      components: [],
      recent_remediations: [],
      last_error: null,
    });
    await runLifecycleHealthTick();
    expect(invoke).toHaveBeenCalledWith('lifecycle_run_health_tick');
  });

  it('setLifecycleAutoFix passes enabled flag', async () => {
    vi.mocked(invoke).mockResolvedValue(false);
    const result = await setLifecycleAutoFix(false);
    expect(invoke).toHaveBeenCalledWith('lifecycle_set_auto_fix', { enabled: false });
    expect(result).toBe(false);
  });

  it('remediation record includes preset_id', async () => {
    vi.mocked(invoke).mockResolvedValue({
      phase: 'degraded',
      started_at: '2026-01-01T00:00:00Z',
      uptime_secs: 5,
      launch_mode: 'dev_binary',
      scheduler_active: true,
      tick_count: 2,
      auto_fix_enabled: true,
      components: [],
      recent_remediations: [
        {
          preset_id: 'reload_frontend',
          action: 'reload_frontend',
          success: true,
          detail: 'navigated',
          at: '2026-01-01T00:00:01Z',
        },
      ],
      last_error: null,
    });
    const status = await getLifecycleStatus();
    expect(status.recent_remediations[0].preset_id).toBe('reload_frontend');
  });
});
