/**
 * Exodus Browser — match pattern unit tests.
 */

import { describe, expect, it } from 'vitest';
import { urlMatchesPattern } from '$lib/extensions/matchPatterns';

describe('urlMatchesPattern', () => {
  it('matches all_urls for http(s)', () => {
    expect(urlMatchesPattern('https://example.com/', '<all_urls>')).toBe(true);
    expect(urlMatchesPattern('file:///x', '<all_urls>')).toBe(false);
  });

  it('matches host wildcard', () => {
    expect(urlMatchesPattern('https://www.google.com/x', '*://*.google.com/*')).toBe(true);
    expect(urlMatchesPattern('https://example.com/', '*://*.google.com/*')).toBe(false);
  });
});
