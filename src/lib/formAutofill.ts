/**
 * Exodus Browser — address/form autofill helpers.
 */

import { invoke } from '@tauri-apps/api/core';
import { evalInTab, evalTabReturning } from '$lib/exodusBrowser';

/** Saved form field entry. */
export type FormAutofillEntry = {
  id: string;
  field_type: string | { Custom?: string } | Record<string, unknown>;
  value: string;
  label: string;
  domain: string;
  last_used: number;
  use_count: number;
};

/** Form autofill settings from backend. */
export type FormAutofillSettings = {
  enabled: boolean;
  save_passwords: boolean;
  save_addresses: boolean;
  save_credit_cards: boolean;
  autofill_on_load: boolean;
  require_confirmation: boolean;
  max_entries_per_type: number;
};

/** Pending capture from in-page script. */
export type FormCapturePayload = {
  field_type: string;
  value: string;
  label: string;
  domain: string;
};

const FILL_FIELD_TYPES = ['email', 'phone', 'name', 'address', 'city', 'zip', 'country'] as const;

/** Normalize field_type from backend enum JSON. */
export function fieldTypeKey(entry: FormAutofillEntry): string {
  const ft = entry.field_type;
  if (typeof ft === 'string') return ft.toLowerCase();
  if (ft && typeof ft === 'object') {
    if ('Custom' in ft && typeof ft.Custom === 'string') return ft.Custom.toLowerCase();
    const key = Object.keys(ft)[0];
    if (key) return key.toLowerCase();
  }
  return 'custom';
}

/** Load form autofill settings. */
export async function loadFormAutofillSettings(): Promise<FormAutofillSettings> {
  return invoke<FormAutofillSettings>('get_autofill_settings');
}

/** Save form autofill settings. */
export async function saveFormAutofillSettings(settings: FormAutofillSettings): Promise<void> {
  await invoke('update_autofill_settings', { settings });
}

/** List all saved autofill entries. */
export async function listFormAutofillEntries(): Promise<FormAutofillEntry[]> {
  try {
    return await invoke<FormAutofillEntry[]>('get_all_autofill_entries');
  } catch (error) {
    console.error('get_all_autofill_entries failed:', error);
    return [];
  }
}

/** Remove a saved entry. */
export async function removeFormAutofillEntry(id: string): Promise<void> {
  await invoke('remove_autofill_entry', { id });
}

/** Persist a captured field value. */
export async function saveFormCapture(capture: FormCapturePayload): Promise<void> {
  await invoke('add_autofill_entry', {
    field_type: capture.field_type,
    value: capture.value,
    label: capture.label,
    domain: capture.domain,
  });
}

/** Read and clear pending captures from the active tab. */
export async function pullFormCaptures(tabLabel: string): Promise<FormCapturePayload[]> {
  const raw = await evalTabReturning(
    tabLabel,
    `(function(){
      var list = window.__exodusFormCaptures || [];
      window.__exodusFormCaptures = [];
      return list.length ? JSON.stringify(list) : '';
    })()`,
  );
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as FormCapturePayload[];
    return Array.isArray(parsed) ? parsed.filter((c) => c.value && c.field_type) : [];
  } catch {
    return [];
  }
}

/** Apply saved autofill values on page load when enabled. */
export async function applyFormAutofillOnLoad(tabLabel: string, url: string): Promise<void> {
  if (!url.startsWith('http://') && !url.startsWith('https://')) return;
  let settings: FormAutofillSettings;
  try {
    settings = await loadFormAutofillSettings();
  } catch {
    return;
  }
  if (!settings.enabled || !settings.autofill_on_load) return;

  let host = '';
  try {
    host = new URL(url).hostname;
  } catch {
    return;
  }

  const pairs: [string, string][] = [];
  for (const ft of FILL_FIELD_TYPES) {
    try {
      const rows = await invoke<FormAutofillEntry[]>('get_autofill_entries', {
        field_type: ft,
        domain: host,
      });
      const best = rows[0];
      if (best?.value) pairs.push([ft, best.value]);
    } catch (error) {
      console.error('get_autofill_entries failed:', error);
    }
  }
  if (pairs.length === 0) return;

  try {
    const script = await invoke<string>('form_build_fill_script', { pairs });
    await evalInTab(tabLabel, script);
  } catch (error) {
    console.error('form autofill on load failed:', error);
  }
}

/** Flush in-page captures to the autofill store. */
export async function flushFormCaptures(tabLabel: string): Promise<number> {
  let settings: FormAutofillSettings;
  try {
    settings = await loadFormAutofillSettings();
  } catch {
    return 0;
  }
  if (!settings.enabled || !settings.save_addresses) return 0;

  const captures = await pullFormCaptures(tabLabel);
  let saved = 0;
  for (const c of captures) {
    try {
      await saveFormCapture(c);
      saved += 1;
    } catch (error) {
      console.error('saveFormCapture failed:', error);
    }
  }
  return saved;
}
