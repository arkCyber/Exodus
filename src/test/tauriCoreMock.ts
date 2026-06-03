/**
 * Shared Tauri core mock factory for Vitest files.
 */

import { vi } from 'vitest';

export type InvokeMock = ReturnType<typeof vi.fn>;

/**
 * Standard `@tauri-apps/api/core` mock with `isTauri` enabled.
 */
export function createTauriCoreMock(invokeImpl?: InvokeMock) {
  const invoke = invokeImpl ?? vi.fn(async () => undefined);
  return {
    invoke,
    isTauri: () => true,
  };
}
