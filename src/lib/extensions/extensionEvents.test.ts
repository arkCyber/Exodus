/**
 * Exodus Browser — extension runtime event helpers tests.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

import { flushExtensionTab, pumpExtensionRuntime } from './extensionEvents';

describe('extensionEvents', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('flushExtensionTab invokes browser_extension_flush_tab', async () => {
    await flushExtensionTab('exodus-tab-abc');
    expect(invokeMock).toHaveBeenCalledWith('browser_extension_flush_tab', {
      label: 'exodus-tab-abc',
    });
  });

  it('pumpExtensionRuntime passes active label', async () => {
    await pumpExtensionRuntime('exodus-tab-1');
    expect(invokeMock).toHaveBeenCalledWith('extension_pump_runtime', {
      activeLabel: 'exodus-tab-1',
    });
  });

  it('pumpExtensionRuntime allows null active label', async () => {
    await pumpExtensionRuntime();
    expect(invokeMock).toHaveBeenCalledWith('extension_pump_runtime', {
      activeLabel: null,
    });
  });
});
