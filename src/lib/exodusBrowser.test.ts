/**
 * Exodus Browser — exodusBrowser helper unit tests.
 */

import { describe, expect, it } from 'vitest';
import { tabWebviewLabel, canUseNativeWebview } from './exodusBrowser';

describe('tabWebviewLabel', () => {
  it('generates valid webview label from tab id', () => {
    expect(tabWebviewLabel('tab-123')).toBe('exodus-tab-tab-123');
  });

  it('handles simple tab ids', () => {
    expect(tabWebviewLabel('1')).toBe('exodus-tab-1');
  });
});

describe('canUseNativeWebview', () => {
  it('returns boolean indicating native webview availability', () => {
    const result = canUseNativeWebview();
    expect(typeof result).toBe('boolean');
  });
});
