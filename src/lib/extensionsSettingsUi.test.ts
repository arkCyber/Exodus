/**
 * Exodus Browser — extensionsSettingsUi tests.
 */

import { describe, it, expect } from 'vitest';
import { extensionsSettingsStrings } from './extensionsSettingsUi';

describe('extensionsSettingsUi', () => {
  it('returns English strings by default', () => {
    expect(extensionsSettingsStrings('en').pageTitle).toBe('Extensions');
  });

  it('returns Chinese strings for zh', () => {
    expect(extensionsSettingsStrings('zh').pageTitle).toBe('扩展程序');
  });
});
