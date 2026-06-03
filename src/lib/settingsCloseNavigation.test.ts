/**
 * Exodus Browser — settings close navigation tests.
 */

import { describe, it, expect } from 'vitest';
import { NEWTAB_INTERNAL_URL } from '@/lib/newTabPage';
import { resolveSettingsCloseTarget, shouldRememberSettingsReturnUrl } from './settingsCloseNavigation';

describe('settingsCloseNavigation', () => {
  it('restores saved return URL when closing settings', () => {
    expect(
      resolveSettingsCloseTarget({
        url: 'chrome://settings',
        settingsReturnUrl: 'https://example.com',
      }),
    ).toBe('https://example.com');
  });

  it('falls back to new tab when no return URL', () => {
    expect(resolveSettingsCloseTarget({ url: 'chrome://settings' })).toBe(NEWTAB_INTERNAL_URL);
  });

  it('does not remember return URL on settings subsection hop', () => {
    expect(shouldRememberSettingsReturnUrl('chrome://settings', true)).toBe(false);
  });

  it('remembers return URL from a normal page', () => {
    expect(shouldRememberSettingsReturnUrl('https://example.com', false)).toBe(true);
  });
});
