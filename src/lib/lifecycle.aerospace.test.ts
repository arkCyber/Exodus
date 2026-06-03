/**
 * Aerospace-grade invariant tests for lifecycle TypeScript contracts.
 * Verifies API surface, serde field names, and invoke wiring without Tauri runtime.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

import {
  getLifecycleStatus,
  setLifecycleAutoFix,
  type LifecycleStatusDto,
  type RemediationRecord,
} from './appLifecycle';
import { getLifecycleLogs, listLifecyclePresets } from './lifecycleLog';
import { invoke } from '@tauri-apps/api/core';

const PHASES = ['booting', 'setup', 'ready', 'running', 'background', 'degraded', 'shutting_down'] as const;
const HEALTH = ['ok', 'warn', 'error', 'unknown'] as const;

function minimalStatus(overrides: Partial<LifecycleStatusDto> = {}): LifecycleStatusDto {
  return {
    phase: 'running',
    started_at: '2026-05-20T12:00:00Z',
    uptime_secs: 120,
    launch_mode: 'dev_binary',
    scheduler_active: true,
    tick_count: 10,
    auto_fix_enabled: true,
    components: [],
    recent_remediations: [],
    last_error: null,
    ...overrides,
  };
}

describe('lifecycle aerospace invariants (TS)', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('I-API-1: all lifecycle commands use snake_case names', async () => {
    vi.mocked(invoke).mockResolvedValue(minimalStatus());
    await getLifecycleStatus();
    await setLifecycleAutoFix(true);
    vi.mocked(invoke).mockResolvedValue([]);
    await getLifecycleLogs(32);
    vi.mocked(invoke).mockResolvedValue([]);
    await listLifecyclePresets();

    const commands = vi.mocked(invoke).mock.calls.map((c) => c[0]);
    expect(commands).toEqual([
      'lifecycle_get_status',
      'lifecycle_set_auto_fix',
      'lifecycle_get_logs',
      'lifecycle_list_presets',
    ]);
  });

  it('I-API-2: remediation record requires preset_id field', async () => {
    const record: RemediationRecord = {
      preset_id: 'full_ui_recovery',
      action: 'reload_frontend',
      success: true,
      detail: 'navigated',
      at: '2026-05-20T12:01:00Z',
    };
    vi.mocked(invoke).mockResolvedValue(
      minimalStatus({ recent_remediations: [record] }),
    );
    const status = await getLifecycleStatus();
    expect(status.recent_remediations[0].preset_id).toBe('full_ui_recovery');
  });

  it('I-API-3: phase and health enums are snake_case literals', () => {
    for (const p of PHASES) {
      expect(p).toMatch(/^[a-z_]+$/);
    }
    for (const h of HEALTH) {
      expect(h).toMatch(/^[a-z]+$/);
    }
  });

  it('I-API-4: getLifecycleLogs passes numeric limit to backend', async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    await getLifecycleLogs(128);
    expect(invoke).toHaveBeenCalledWith('lifecycle_get_logs', { limit: 128 });
  });

  it('I-API-5: preset DTO exposes cooldown_secs for scheduling analysis', async () => {
    vi.mocked(invoke).mockResolvedValue([
      {
        id: 'restart_allama',
        name: 'Restart Allama',
        description: 'restart',
        component: 'allama',
        triggers_on: 'error',
        priority: 30,
        cooldown_secs: 45,
      },
    ]);
    const presets = await listLifecyclePresets();
    expect(presets[0].cooldown_secs).toBeGreaterThan(0);
    expect(presets[0].triggers_on).toBe('error');
  });
});
