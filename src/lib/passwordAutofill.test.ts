/**
 * Exodus Browser — passwordAutofill unit tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('$lib/exodusBrowser', () => ({
  evalInTab: vi.fn(),
  evalTabReturning: vi.fn(),
}));

import { getPasswordForPage, loadPasswordManagerSettings } from './passwordAutofill';

describe('passwordAutofill', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('loadPasswordManagerSettings invokes backend', async () => {
    invokeMock.mockResolvedValue({ auto_save: true, auto_fill: true });
    const s = await loadPasswordManagerSettings();
    expect(s.auto_fill).toBe(true);
    expect(invokeMock).toHaveBeenCalledWith('get_password_manager_settings');
  });

  it('getPasswordForPage returns entry', async () => {
    invokeMock.mockResolvedValue({ id: '1', username: 'u', password: 'p' });
    const entry = await getPasswordForPage('https://example.com');
    expect(entry?.username).toBe('u');
    expect(invokeMock).toHaveBeenCalledWith('get_password_for_page', {
      url: 'https://example.com',
    });
  });
});
