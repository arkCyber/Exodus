/**
 * Exodus Browser — appLocale tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  APP_LOCALES,
  resolveAppLocale,
  writeAppLocale,
  isAppLocale,
  appLocaleOptions,
} from './appLocale';

describe('appLocale', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('exposes nine supported locales', () => {
    expect(APP_LOCALES).toHaveLength(9);
    expect(isAppLocale('ja')).toBe(true);
    expect(isAppLocale('xx')).toBe(false);
  });

  it('resolveAppLocale uses explicit override', () => {
    expect(resolveAppLocale('fr')).toBe('fr');
  });

  it('resolveAppLocale reads persisted preference', () => {
    writeAppLocale('ko');
    expect(resolveAppLocale()).toBe('ko');
  });

  it('appLocaleOptions returns display labels', () => {
    const opts = appLocaleOptions();
    expect(opts.find((o) => o.value === 'zh')?.label).toBe('中文');
    expect(opts.find((o) => o.value === 'ja')?.label).toBe('日本語');
  });
});
