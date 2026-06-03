/**
 * Exodus Browser — historyManagerSettingsUi tests.
 */

import { describe, it, expect } from 'vitest';
import { historyManagerSettingsStrings } from './historyManagerSettingsUi';

describe('historyManagerSettingsUi', () => {
  it('returns localized history title', () => {
    expect(historyManagerSettingsStrings('en').title).toBe('Browsing history');
    expect(historyManagerSettingsStrings('zh').title).toBe('浏览历史记录');
  });
});
