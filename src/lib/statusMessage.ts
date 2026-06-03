/**
 * Exodus Browser — transient status bar messages with auto-clear.
 */

/** Default time before the status bar clears (ms). */
export const STATUS_CLEAR_MS = 4000;

export type ShowStatusOptions = {
  /** If true, message stays until replaced. */
  persist?: boolean;
  clearMs?: number;
};

export type StatusNotifier = {
  show: (message: string, options?: ShowStatusOptions) => void;
  dispose: () => void;
};

/**
 * Build a status notifier bound to page state.
 * @param setMessage - updates the status string (e.g. Svelte $state field)
 */
export function createStatusNotifier(
  setMessage: (message: string) => void,
  getDefaultClearMs: () => number = () => STATUS_CLEAR_MS,
): StatusNotifier {
  let clearTimer: ReturnType<typeof setTimeout> | undefined;

  return {
    show(message: string, options?: ShowStatusOptions) {
      if (clearTimer) {
        clearTimeout(clearTimer);
        clearTimer = undefined;
      }
      setMessage(message);
      if (!message || options?.persist) return;
      const ms = options?.clearMs ?? getDefaultClearMs();
      clearTimer = setTimeout(() => {
        setMessage('');
        clearTimer = undefined;
      }, ms);
    },
    dispose() {
      if (clearTimer) {
        clearTimeout(clearTimer);
        clearTimer = undefined;
      }
    },
  };
}

/**
 * Format an unknown error for the status bar.
 */
export function formatStatusError(error: unknown, prefix: string): string {
  const detail = error instanceof Error ? error.message : 'Unknown error';
  return `${prefix}: ${detail}`;
}
