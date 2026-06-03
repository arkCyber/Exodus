import { describe, it, expect } from 'vitest';
import { resolveOmniboxInput } from './omnibox';

describe('resolveOmniboxInput chrome URLs', () => {
  it('resolves chrome://settings as navigation', () => {
    const result = resolveOmniboxInput('chrome://settings', 'https://duckduckgo.com/?q={query}');
    expect(result).toEqual({ kind: 'navigate', url: 'chrome://settings' });
  });
});
