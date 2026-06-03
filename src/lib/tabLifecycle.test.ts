/**
 * Unit tests for tab lifecycle helpers (invoke mocked).
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';
import { registerTabLifecycle, markTabActiveLifecycle } from './tabLifecycle';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/exodusBrowser', () => ({
  tabWebviewLabel: (id: string) => `tab-${id}`,
}));

describe('tabLifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('registerTabLifecycle calls freezer and sleep register', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    await registerTabLifecycle({ id: 'abc', url: 'https://example.com', title: 'Ex' });
    expect(invoke).toHaveBeenCalledWith('register_tab', expect.any(Object));
    expect(invoke).toHaveBeenCalledWith('tab_sleep_register', expect.any(Object));
  });

  it('markTabActiveLifecycle updates activity', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    await markTabActiveLifecycle('abc');
    expect(invoke).toHaveBeenCalledWith('update_tab_activity', { label: 'tab-abc' });
  });
});
