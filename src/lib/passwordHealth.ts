/**
 * Password Health Monitoring Dashboard API for Exodus Browser
 * Provides password strength analysis, duplicate detection, and breach monitoring
 */

import { invoke } from '@tauri-apps/api/core';

export type PasswordHealthScore = 'excellent' | 'good' | 'fair' | 'weak' | 'critical';

export interface PasswordHealthEntry {
  id: string;
  title: string;
  url: string;
  username: string;
  strength_score: PasswordHealthScore;
  strength_percentage: number;
  is_compromised: boolean;
  is_duplicate: boolean;
  age_days: number;
  last_changed: string;
}

export interface PasswordHealthSummary {
  total_passwords: number;
  weak_passwords: number;
  duplicate_passwords: number;
  compromised_passwords: number;
  old_passwords: number;
  overall_health_score: PasswordHealthScore;
  health_percentage: number;
}

/**
 * Add password health entry
 */
export async function addPasswordHealthEntry(
  title: string,
  url: string,
  username: string,
  strengthScore: PasswordHealthScore,
  strengthPercentage: number
): Promise<string> {
  return invoke('add_password_health_entry', {
    title,
    url,
    username,
    strengthScore,
    strengthPercentage,
  });
}

/**
 * Remove password health entry
 */
export async function removePasswordHealthEntry(id: string): Promise<void> {
  return invoke('remove_password_health_entry', { id });
}

/**
 * Update password strength
 */
export async function updatePasswordStrength(
  id: string,
  strengthScore: PasswordHealthScore,
  strengthPercentage: number
): Promise<void> {
  return invoke('update_password_strength', {
    id,
    strengthScore,
    strengthPercentage,
  });
}

/**
 * Mark password as compromised
 */
export async function markPasswordCompromised(id: string, compromised: boolean): Promise<void> {
  return invoke('mark_password_compromised', { id, compromised });
}

/**
 * Mark password as duplicate
 */
export async function markPasswordDuplicate(id: string, duplicate: boolean): Promise<void> {
  return invoke('mark_password_duplicate', { id, duplicate });
}

/**
 * Get all password health entries
 */
export async function getPasswordHealthEntries(): Promise<PasswordHealthEntry[]> {
  return invoke('get_password_health_entries');
}

/**
 * Get weak passwords
 */
export async function getWeakPasswords(): Promise<PasswordHealthEntry[]> {
  return invoke('get_password_health_weak_passwords');
}

/**
 * Get duplicate passwords
 */
export async function getDuplicatePasswords(): Promise<PasswordHealthEntry[]> {
  return invoke('get_password_health_duplicate_passwords');
}

/**
 * Get compromised passwords
 */
export async function getCompromisedPasswords(): Promise<PasswordHealthEntry[]> {
  return invoke('get_password_health_compromised_passwords');
}

/**
 * Get old passwords
 */
export async function getOldPasswords(): Promise<PasswordHealthEntry[]> {
  return invoke('get_password_health_old_passwords');
}

/**
 * Get password health summary
 */
export async function getPasswordHealthSummary(): Promise<PasswordHealthSummary> {
  return invoke('get_password_health_summary');
}

/**
 * Run password health check
 */
export async function runPasswordHealthCheck(): Promise<PasswordHealthSummary> {
  return invoke('run_password_health_check');
}
