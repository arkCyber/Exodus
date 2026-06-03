/**
 * Exodus Browser — password autofill and save-offer helpers.
 */

import { invoke } from '@tauri-apps/api/core';
import type { PasswordEntry } from '$lib/browserTypes';
import { evalInTab, evalTabReturning } from '$lib/exodusBrowser';

/** Password manager settings from backend. */
export type PasswordManagerSettings = {
  auto_save: boolean;
  auto_fill: boolean;
  require_master_password: boolean;
  min_password_length: number;
  require_strength_check: boolean;
  enable_breach_detection: boolean;
  auto_lock_timeout: number;
  enable_sync: boolean;
};

/** Pending login capture from in-page script. */
export type PasswordCapturePayload = {
  url: string;
  username: string;
  password: string;
};

/** Load password manager settings. */
export async function loadPasswordManagerSettings(): Promise<PasswordManagerSettings> {
  return invoke<PasswordManagerSettings>('get_password_manager_settings');
}

/** Saved credentials for a page URL (host-aware). */
export async function getPasswordForPage(url: string): Promise<PasswordEntry | null> {
  try {
    return await invoke<PasswordEntry | null>('get_password_for_page', { url });
  } catch (error) {
    console.error('get_password_for_page failed:', error);
    return null;
  }
}

/** Save credentials after user confirms the offer dialog. */
export async function savePasswordCapture(
  url: string,
  username: string,
  password: string,
): Promise<void> {
  await invoke('save_password_capture', { url, username, password });
}

/** Apply autofill in a native tab webview. */
export async function applyPasswordAutofill(
  tabLabel: string,
  entry: PasswordEntry,
): Promise<void> {
  const script = await invoke<string>('password_build_fill_script', {
    username: entry.username,
    password: entry.password,
  });
  await evalInTab(tabLabel, script);
}

/** Read and clear a pending password capture from the page. */
export async function pullPasswordCapture(tabLabel: string): Promise<PasswordCapturePayload | null> {
  const raw = await evalTabReturning(
    tabLabel,
    `(function(){
      var p = window.__exodusPasswordCapture;
      window.__exodusPasswordCapture = null;
      return p ? JSON.stringify(p) : '';
    })()`,
  );
  if (!raw || raw === '""' || raw === 'null') return null;
  try {
    const parsed = JSON.parse(raw) as PasswordCapturePayload;
    if (!parsed.password) return null;
    return parsed;
  } catch {
    return null;
  }
}
