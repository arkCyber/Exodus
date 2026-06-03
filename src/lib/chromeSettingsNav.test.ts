/**
 * Exodus Browser — chrome settings navigation tests.
 */

import { describe, expect, it } from 'vitest';
import {
  chromeSettingsUrlForSection,
  chromeSettingsStrings,
  filterChromeSettingsNav,
  isChromeSettingsSectionHop,
  isChromeSettingsUrl,
  normalizeChromeSettingsSection,
  parseChromeSettingsSection,
  chromeSettingsNavItems,
} from './chromeSettingsNav';

describe('chromeSettingsNav', () => {
  it('normalizeChromeSettingsSection maps aliases', () => {
    expect(normalizeChromeSettingsSection('privacy')).toBe('privacy');
    expect(normalizeChromeSettingsSection('passwords')).toBe('autofill');
    expect(normalizeChromeSettingsSection('tabs')).toBe('sidebar');
    expect(normalizeChromeSettingsSection('unknown-x')).toBe('browser');
  });

  it('parseChromeSettingsSection reads path, hash, and query', () => {
    expect(parseChromeSettingsSection('chrome://settings')).toBe('browser');
    expect(parseChromeSettingsSection('chrome://settings/privacy')).toBe('privacy');
    expect(parseChromeSettingsSection('chrome://settings#extensions')).toBe('extensions');
    expect(parseChromeSettingsSection('chrome://settings?section=ai')).toBe('ai');
  });

  it('chromeSettingsUrlForSection builds deep links', () => {
    expect(chromeSettingsUrlForSection('browser')).toBe('chrome://settings');
    expect(chromeSettingsUrlForSection('privacy')).toBe('chrome://settings/privacy');
  });

  it('filterChromeSettingsNav filters by label', () => {
    const items = chromeSettingsNavItems('en');
    const filtered = filterChromeSettingsNav(items, 'privacy');
    expect(filtered.length).toBe(1);
    expect(filtered[0]?.id).toBe('privacy');
  });

  it('isChromeSettingsUrl matches settings paths', () => {
    expect(isChromeSettingsUrl('chrome://settings')).toBe(true);
    expect(isChromeSettingsUrl('chrome://settings/privacy')).toBe(true);
    expect(isChromeSettingsUrl('chrome://extensions')).toBe(false);
  });

  it('chromeSettingsStrings returns Japanese nav labels', () => {
    expect(chromeSettingsStrings('ja').nav('appearance')).toBe('外観');
  });

  it('isChromeSettingsSectionHop detects subsection navigation', () => {
    expect(
      isChromeSettingsSectionHop('chrome://settings', 'chrome://settings/privacy'),
    ).toBe(true);
    expect(
      isChromeSettingsSectionHop('chrome://settings/privacy', 'chrome://settings/ai'),
    ).toBe(true);
    expect(
      isChromeSettingsSectionHop('chrome://settings', 'chrome://extensions'),
    ).toBe(false);
  });
});
