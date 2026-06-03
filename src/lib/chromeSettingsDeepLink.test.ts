/**
 * Exodus Browser — settings deep-link URL parsing (chrome://settings/{section}).
 */

import { describe, it, expect } from 'vitest';
import { parseChromeSettingsSection, chromeSettingsUrlForSection } from './chromeSettingsNav';

describe('chrome settings deep links', () => {
  it('parses subsection paths', () => {
    expect(parseChromeSettingsSection('chrome://settings/extensions')).toBe('extensions');
    expect(parseChromeSettingsSection('chrome://settings/downloads')).toBe('downloads');
    expect(parseChromeSettingsSection('chrome://settings/about')).toBe('about');
  });

  it('builds subsection URLs', () => {
    expect(chromeSettingsUrlForSection('extensions')).toBe('chrome://settings/extensions');
    expect(chromeSettingsUrlForSection('downloads')).toBe('chrome://settings/downloads');
  });
});
