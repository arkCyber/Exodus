/**
 * Exodus Browser — extension host permission API invoke tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  listExtensionSitePermissions,
  revokeAllExtensionSitePermissions,
  revokeExtensionSitePermissions,
} from './api';

describe('extension host permissions api', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('listExtensionSitePermissions invokes list command', async () => {
    invokeMock.mockResolvedValue(['https://*.example.com/*']);
    const patterns = await listExtensionSitePermissions('ext-1');
    expect(invokeMock).toHaveBeenCalledWith('extension_site_permissions_list', {
      extensionId: 'ext-1',
    });
    expect(patterns).toEqual(['https://*.example.com/*']);
  });

  it('revokeExtensionSitePermissions passes patterns', async () => {
    await revokeExtensionSitePermissions('ext-1', ['https://a.com/*']);
    expect(invokeMock).toHaveBeenCalledWith('extension_site_permissions_revoke', {
      extensionId: 'ext-1',
      patterns: ['https://a.com/*'],
    });
  });

  it('revokeAllExtensionSitePermissions invokes revoke_all', async () => {
    await revokeAllExtensionSitePermissions('ext-1');
    expect(invokeMock).toHaveBeenCalledWith('extension_site_permissions_revoke_all', {
      extensionId: 'ext-1',
    });
  });
});
