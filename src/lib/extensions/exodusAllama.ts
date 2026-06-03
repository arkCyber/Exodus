/**
 * Exodus Browser — Allama helpers for extension authors (content scripts / background).
 *
 * Requires injected `window.exodus.allama` from the host (see chrome_bridge.rs).
 * Prefer fetch to `http://127.0.0.1:11435` only when the extension has host access to localhost.
 */

export type ExodusAllamaStreamCallbacks = {
  onChunk?: (content: string) => void;
  onDone?: () => void;
  onError?: (message: string) => void;
};

export type ExodusAllamaShim = {
  port: number;
  baseUrl: string;
  health: () => Promise<boolean>;
  chat: (
    messages: Array<{ role: string; content: string }>,
    model?: string,
  ) => Promise<string>;
  generate: (prompt: string, model?: string) => Promise<string>;
  embed: (text: string, model?: string) => Promise<number[]>;
  streamChat: (
    messages: Array<{ role: string; content: string }>,
    model: string | undefined,
    callbacks: ExodusAllamaStreamCallbacks,
  ) => Promise<void>;
};

declare global {
  interface Window {
    exodus?: {
      allama?: ExodusAllamaShim;
    };
  }
}

/** Returns injected shim when running inside an Exodus extension context. */
export function getExodusAllamaShim(): ExodusAllamaShim | null {
  if (typeof window === 'undefined') return null;
  return window.exodus?.allama ?? null;
}

/** Whether `window.exodus.allama` is available in this context. */
export function exodusAllamaAvailable(): boolean {
  return getExodusAllamaShim() != null;
}
