/**
 * Unit tests for favicon and HTTPS helpers.
 */

import { describe, expect, it } from 'vitest';
import { faviconUrlFor, isSecureUrl } from './favicon';

describe('faviconUrlFor', () => {
  it('returns favicon URL for https pages', () => {
    const url = faviconUrlFor('https://example.com/path');
    expect(url).toContain('example.com');
    expect(url).toContain('favicons');
  });

  it('returns null for data URLs', () => {
    expect(faviconUrlFor('data:text/html,<p>x</p>')).toBeNull();
  });
});

describe('isSecureUrl', () => {
  it('detects https', () => {
    expect(isSecureUrl('https://example.com')).toBe(true);
  });

  it('rejects http', () => {
    expect(isSecureUrl('http://example.com')).toBe(false);
  });
});
