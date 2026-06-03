/**
 * Exodus Browser — load persisted AI / Allama settings from Tauri.
 */

import { invoke } from '@tauri-apps/api/core';
import type { ExodusConfigDto } from '$lib/browserSettings';
import { ALLAMA_DEFAULT_PORT, allamaBaseUrl } from '$lib/allamaClient';

/** Default chat model when Allama is available (directory name under ~/.allama/models). */
export const DEFAULT_CHAT_MODEL = 'gemma4-e2b';

/** Prefer these names when the saved model is missing or legacy. */
export const CHAT_MODEL_PREFERENCES = [
  DEFAULT_CHAT_MODEL,
  'gemma-4-e4b',
  'gemma4-4b',
  'gemma4-26b-moe',
  'gemma4-31b',
  'qwen3.6-35b-a3b',
];

const LEGACY_CHAT_MODELS = new Set(['llama2', 'exodus-default', 'llama3', '']);

/**
 * List model names from Allama `GET /api/tags`.
 */
export async function listAllamaModelNames(port: number = ALLAMA_DEFAULT_PORT): Promise<string[]> {
  try {
    const res = await fetch(`${allamaBaseUrl(port)}/api/tags`, {
      signal: AbortSignal.timeout(4000),
    });
    if (!res.ok) return [];
    const data = (await res.json()) as { models?: Array<{ name?: string }> };
    return (data.models ?? [])
      .map((m) => m.name)
      .filter((n): n is string => typeof n === 'string' && n.length > 0);
  } catch (error) {
    console.error('listAllamaModelNames failed:', error);
    return [];
  }
}

/**
 * Pick a chat model from available names (keeps `current` when still valid).
 */
export function pickPreferredChatModel(names: string[], current?: string): string {
  if (current && names.includes(current)) return current;
  for (const preferred of CHAT_MODEL_PREFERENCES) {
    if (names.includes(preferred)) return preferred;
  }
  return names[0] ?? DEFAULT_CHAT_MODEL;
}

/**
 * Align saved chat model with models Allama actually serves.
 */
export async function normalizeChatModelFromAllama(
  port: number,
  current: string,
): Promise<{ model: string; changed: boolean }> {
  const names = await listAllamaModelNames(port);
  const stale = LEGACY_CHAT_MODELS.has(current);
  if (!names.length) {
    const model = stale ? DEFAULT_CHAT_MODEL : current;
    return { model, changed: model !== current };
  }
  const picked = pickPreferredChatModel(names, stale ? undefined : current);
  return { model: picked, changed: picked !== current };
}

/** Load `get_ai_config` with sane defaults. */
export async function loadAiConfig(): Promise<ExodusConfigDto> {
  try {
    const cfg = await invoke<ExodusConfigDto>('get_ai_config');
    return {
      ...cfg,
      ai_port: cfg.ai_port ?? ALLAMA_DEFAULT_PORT,
      spawn_allama: cfg.spawn_allama ?? true,
      spawn_sidecar: cfg.spawn_sidecar ?? false,
    };
  } catch (error) {
    console.error('loadAiConfig failed:', error);
    return {
      ai_port: ALLAMA_DEFAULT_PORT,
      ai_model: DEFAULT_CHAT_MODEL,
      embedding_model: 'nomic-embed-text',
      homepage_url: 'https://duckduckgo.com',
      search_engine_url: 'https://duckduckgo.com/?q={query}',
      status_clear_ms: 5000,
      spawn_sidecar: false,
      spawn_allama: true,
    };
  }
}
