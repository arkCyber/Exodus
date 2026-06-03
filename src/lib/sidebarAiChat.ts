/**
 * Exodus Browser — sidebar AI chat via Allama HTTP (allamaClient), not Tauri invoke.
 */

import type { AiChatMessage } from '$lib/browserTypes';
import {
  ALLAMA_DEFAULT_PORT,
  allamaHealth,
  streamAllamaChatCompletions,
  type AllamaChatMessage,
  type AllamaStreamCallbacks,
} from '$lib/allamaClient';

const SYSTEM_CHAT =
  'You are Exodus, a helpful privacy-focused browser assistant. Be concise and practical.';

const SYSTEM_SUMMARY =
  'You are a helpful AI assistant. Provide concise, clear summaries.';

export type SidebarAiOptions = {
  port?: number;
  model: string;
  signal?: AbortSignal;
};

/**
 * Probe Allama on the configured port (`GET /api/tags`).
 */
export async function checkSidebarAiOnline(
  port: number = ALLAMA_DEFAULT_PORT,
): Promise<boolean> {
  return allamaHealth(port);
}

/**
 * Build Allama messages from sidebar chat history (includes multi-turn context).
 */
export function chatMessagesFromHistory(history: AiChatMessage[]): AllamaChatMessage[] {
  return [
    { role: 'system', content: SYSTEM_CHAT },
    ...history.map((m) => ({
      role: m.role,
      content: m.content,
    })),
  ];
}

/**
 * Stream a sidebar chat reply using full `history` (last entry is usually the new user turn).
 */
export async function streamSidebarChat(
  history: AiChatMessage[],
  options: SidebarAiOptions,
  callbacks: AllamaStreamCallbacks,
): Promise<void> {
  const messages = chatMessagesFromHistory(history);
  await streamAllamaChatCompletions(messages, options, callbacks);
}

/**
 * Stream a summary of selected page text in the sidebar AI panel.
 */
export async function streamSidebarSummarize(
  text: string,
  options: SidebarAiOptions,
  callbacks: AllamaStreamCallbacks,
): Promise<void> {
  const messages: AllamaChatMessage[] = [
    { role: 'system', content: SYSTEM_SUMMARY },
    {
      role: 'user',
      content: `Please summarize this text:\n\n${text}`,
    },
  ];
  await streamAllamaChatCompletions(messages, options, callbacks);
}
