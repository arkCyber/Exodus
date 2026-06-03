import { describe, it, expect } from 'vitest';
import {
  isChromeInternalUrl,
  normalizeChromeInternalUrl,
  parseChromeInternalUrl,
  chromeInternalRoutePath,
} from './chromeInternal';

describe('chromeInternal', () => {
  it('detects chrome URLs', () => {
    expect(isChromeInternalUrl('chrome://settings')).toBe(true);
    expect(isChromeInternalUrl('https://example.com')).toBe(false);
  });

  it('parses settings host', () => {
    expect(parseChromeInternalUrl('chrome://settings')).toBe('settings');
    expect(parseChromeInternalUrl('chrome://extensions')).toBe('extensions');
    expect(parseChromeInternalUrl('chrome://apps')).toBe('apps');
  });

  it('normalizes chrome URLs', () => {
    expect(normalizeChromeInternalUrl('chrome:settings')).toBe('chrome://settings');
  });

  it('maps route paths', () => {
    expect(chromeInternalRoutePath('settings')).toBe('/chrome/settings');
    expect(chromeInternalRoutePath('apps')).toBe('/chrome/apps');
    expect(chromeInternalRoutePath('unknown')).toBeNull();
  });
});
