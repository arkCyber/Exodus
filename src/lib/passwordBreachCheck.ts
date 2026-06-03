/**
 * Exodus Browser — Password Breach Check Client
 * Provides interface to check if passwords have been compromised via HaveIBeenPwned API
 */

import { invoke } from '@tauri-apps/api/core';

/** Breach status */
export type BreachStatus = 'safe' | 'compromised' | 'unknown';

/** Check if a password has been compromised */
export async function checkPasswordCompromised(password: string): Promise<BreachStatus> {
  const result = await invoke<string>('check_password_compromised', { password });
  return result as BreachStatus;
}

/** Check password strength */
export async function checkPasswordStrength(password: string): Promise<string> {
  return invoke<string>('check_password_strength', { password });
}

/** Generate a strong password */
export async function generatePassword(length: number = 16): Promise<string> {
  return invoke<string>('generate_password', { length });
}
