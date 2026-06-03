/**
 * Exodus Browser — omnibox + HTTPS-only integration tests.
 */

import { describe, expect, it } from 'vitest';
import { applyHttpsOnly } from '$lib/privacySettings';
import { resolveOmniboxInput } from '$lib/omnibox';

describe('omnibox with HTTPS-only', () => {
  it('upgrades typed http URL before navigation', () => {
    const resolved = resolveOmniboxInput('http://example.com', 'https://duckduckgo.com/?q={query}');
    expect(resolved?.kind).toBe('navigate');
    if (resolved?.kind === 'navigate') {
      expect(applyHttpsOnly(resolved.url, true)).toBe('https://example.com/');
    }
  });

  it('leaves search URLs unchanged under HTTPS-only', () => {
    const resolved = resolveOmniboxInput('rust language', 'https://duckduckgo.com/?q={query}');
    expect(resolved?.kind).toBe('navigate');
    if (resolved?.kind === 'navigate') {
      expect(applyHttpsOnly(resolved.url, true)).toContain('duckduckgo.com');
    }
  });
});
