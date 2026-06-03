/**
 * Exodus Browser — lifecycle structured log API (Rust `lifecycle_log` module).
 */

import { invoke } from '@tauri-apps/api/core';
import { logStartup } from '$lib/startupLog';

export type LifecycleLogLevel = 'debug' | 'info' | 'warn' | 'error';

export type LifecycleLogCategory =
  | 'phase'
  | 'check'
  | 'tick'
  | 'preset'
  | 'remediation'
  | 'scheduler'
  | 'system';

export type LifecycleLogEntry = {
  at: string;
  level: LifecycleLogLevel;
  category: LifecycleLogCategory;
  message: string;
  detail: string | null;
};

export type RemediationPresetDto = {
  id: string;
  name: string;
  description: string;
  component: string;
  triggers_on: string;
  priority: number;
  cooldown_secs: number;
};

/** Fetch recent lifecycle log entries from the backend ring buffer. */
export async function getLifecycleLogs(limit = 64): Promise<LifecycleLogEntry[]> {
  return invoke<LifecycleLogEntry[]>('lifecycle_get_logs', { limit });
}

/** List built-in auto-remediation preset playbooks. */
export async function listLifecyclePresets(): Promise<RemediationPresetDto[]> {
  return invoke<RemediationPresetDto[]>('lifecycle_list_presets');
}

logStartup('lifecycleLog module loaded');
