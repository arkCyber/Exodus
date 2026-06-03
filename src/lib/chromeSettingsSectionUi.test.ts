/**
 * Exodus Browser — chromeSettingsSectionUi tests.
 */

import { describe, it, expect } from 'vitest';
import { chromeSettingsSectionUi } from './chromeSettingsSectionUi';

describe('chromeSettingsSectionUi', () => {
  it('returns English browser strings by default', () => {
    const ui = chromeSettingsSectionUi();
    expect(ui.browser.generalTitle).toBe('General');
    expect(ui.privacy.httpsOnly).toBe('HTTPS-only mode');
  });

  it('returns Chinese section strings for zh locale', () => {
    const ui = chromeSettingsSectionUi('zh');
    expect(ui.browser.generalTitle).toBe('常规');
    expect(ui.about.versionLabel).toBe('版本');
  });
});
