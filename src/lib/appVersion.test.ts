/**
 * Exodus Browser — appVersion tests.
 */

import { describe, it, expect } from 'vitest';
import { APP_PACKAGE_VERSION, APP_PRODUCT_NAME } from './appVersion';

describe('appVersion', () => {
  it('exposes package version string', () => {
    expect(APP_PACKAGE_VERSION).toMatch(/^\d+\.\d+\.\d+/);
  });

  it('exposes product name', () => {
    expect(APP_PRODUCT_NAME).toBe('Exodus Browser');
  });
});
