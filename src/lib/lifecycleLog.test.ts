/**
 * Unit tests for lifecycle log TypeScript API.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { getLifecycleLogs, listLifecyclePresets } from './lifecycleLog';
import { invoke } from '@tauri-apps/api/core';

describe('lifecycleLog', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('getLifecycleLogs invokes lifecycle_get_logs with limit', async () => {
    vi.mocked(invoke).mockResolvedValue([
      {
        at: '2026-01-01T00:00:00Z',
        level: 'info',
        category: 'preset',
        message: 'playbook plan: [reload_frontend]',
        detail: 'health tick',
      },
    ]);
    const logs = await getLifecycleLogs(32);
    expect(invoke).toHaveBeenCalledWith('lifecycle_get_logs', { limit: 32 });
    expect(logs).toHaveLength(1);
    expect(logs[0].category).toBe('preset');
  });

  it('listLifecyclePresets invokes lifecycle_list_presets', async () => {
    vi.mocked(invoke).mockResolvedValue([
      {
        id: 'show_main_window',
        name: 'Show Main Window',
        description: 'Show window',
        component: 'main_window',
        triggers_on: 'warn',
        priority: 10,
        cooldown_secs: 45,
      },
    ]);
    const presets = await listLifecyclePresets();
    expect(invoke).toHaveBeenCalledWith('lifecycle_list_presets');
    expect(presets[0].id).toBe('show_main_window');
    expect(presets[0].cooldown_secs).toBe(45);
  });
});
