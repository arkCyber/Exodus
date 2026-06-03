/**
 * Exodus Browser — Tauri app lifecycle monitor API (health checks + auto-remediation).
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { logStartup, logStartupError } from '$lib/startupLog';

/** Component health from the lifecycle manager. */
export type ComponentHealth = 'ok' | 'warn' | 'error' | 'unknown';

export type ComponentSnapshot = {
  name: string;
  health: ComponentHealth;
  message: string;
  checked_at: string;
};

export type RemediationRecord = {
  preset_id: string;
  action: string;
  success: boolean;
  detail: string;
  at: string;
};

export type { LifecycleLogEntry, RemediationPresetDto } from '$lib/lifecycleLog';
export { getLifecycleLogs, listLifecyclePresets } from '$lib/lifecycleLog';

export type LifecycleStatusDto = {
  phase: string;
  started_at: string;
  uptime_secs: number;
  launch_mode: string;
  scheduler_active: boolean;
  tick_count: number;
  auto_fix_enabled: boolean;
  components: ComponentSnapshot[];
  recent_remediations: RemediationRecord[];
  last_error: string | null;
};

/** Fetch current lifecycle status from Rust. */
export async function getLifecycleStatus(): Promise<LifecycleStatusDto> {
  if (!isTauri()) {
    throw new Error('Not running in Tauri environment');
  }
  return invoke<LifecycleStatusDto>('lifecycle_get_status');
}

/** Ask backend to show main window and re-apply dock policy. */
export async function showMainWindowViaLifecycle(): Promise<LifecycleStatusDto> {
  if (!isTauri()) {
    throw new Error('Not running in Tauri environment');
  }
  return invoke<LifecycleStatusDto>('lifecycle_show_main_window');
}

/** Run health checks + auto-remediation immediately. */
export async function runLifecycleHealthTick(): Promise<LifecycleStatusDto> {
  if (!isTauri()) {
    throw new Error('Not running in Tauri environment');
  }
  return invoke<LifecycleStatusDto>('lifecycle_run_health_tick');
}

/** Toggle automatic remediation (checks still run). */
export async function setLifecycleAutoFix(enabled: boolean): Promise<boolean> {
  if (!isTauri()) {
    throw new Error('Not running in Tauri environment');
  }
  return invoke<boolean>('lifecycle_set_auto_fix', { enabled });
}

/**
 * Subscribe to lifecycle events; auto-recovery runs on the Rust side.
 * Returns an unlisten function.
 */
export async function bindLifecycleRecovery(): Promise<() => void> {
  if (!isTauri()) {
    logStartupError('bindLifecycleRecovery: Not running in Tauri environment', new Error('Not in Tauri environment'));
    return () => {};
  }
  
  const unsubs: Array<() => void> = [];
  try {
    unsubs.push(
      await listen<LifecycleStatusDto>('exodus-lifecycle-ready', async (ev) => {
        logStartup('exodus-lifecycle-ready', ev.payload);
        try {
          await showMainWindowViaLifecycle();
        } catch (error) {
          logStartupError('lifecycle show window on ready failed', error);
        }
      }),
    );
    unsubs.push(
      await listen<RemediationRecord>('exodus-lifecycle-remediation', (ev) => {
        logStartup(`auto-fix [${ev.payload.preset_id}]: ${ev.payload.action}`, ev.payload);
      }),
    );
    unsubs.push(
      await listen<string>('exodus-lifecycle-frontend-down', (ev) => {
        logStartupError('frontend dev server down', ev.payload);
      }),
    );
    unsubs.push(
      await listen<LifecycleStatusDto>('exodus-lifecycle-tick', (ev) => {
        if (ev.payload.phase === 'degraded') {
          logStartup('lifecycle degraded', ev.payload.components);
        }
      }),
    );
    const status = await getLifecycleStatus();
    logStartup('lifecycle initial status', {
      phase: status.phase,
      auto_fix: status.auto_fix_enabled,
      components: status.components.length,
    });
  } catch (error) {
    logStartupError('bindLifecycleRecovery failed', error);
  }
  return () => {
    for (const u of unsubs) u();
  };
}

logStartup('appLifecycle module loaded');
