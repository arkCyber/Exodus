/**
 * Exodus Browser — clearBrowsingDataSettingsUi tests.
 */

import { describe, it, expect } from 'vitest';
import { clearBrowsingDataSettingsStrings } from './clearBrowsingDataSettingsUi';

describe('clearBrowsingDataSettingsUi', () => {
  it('returns English strings by default', () => {
    expect(clearBrowsingDataSettingsStrings().title).toBe('Clear browsing data');
  });

  it('returns Chinese strings for zh', () => {
    expect(clearBrowsingDataSettingsStrings('zh').title).toBe('清除浏览数据');
  });
});
