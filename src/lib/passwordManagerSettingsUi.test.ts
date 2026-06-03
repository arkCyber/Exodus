/**
 * Exodus Browser — passwordManagerSettingsUi tests.
 */

import { describe, it, expect } from 'vitest';
import { passwordManagerSettingsStrings } from './passwordManagerSettingsUi';

describe('passwordManagerSettingsUi', () => {
  it('returns localized autofill title', () => {
    expect(passwordManagerSettingsStrings('en').title).toBe('Password manager');
    expect(passwordManagerSettingsStrings('zh').title).toBe('密码管理器');
  });
});
