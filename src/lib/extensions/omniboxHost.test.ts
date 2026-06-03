/**
 * Exodus Browser — extension omnibox host helpers tests.
 */

import { describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

import { matchExtensionOmniboxKeyword } from './omniboxHost';

describe('matchExtensionOmniboxKeyword', () => {
  const keywords = [
    { extensionId: 'ext', extensionName: 'Demo', keyword: 'ex' },
  ];

  it('matches keyword prefix with query', () => {
    const m = matchExtensionOmniboxKeyword('ex hello world', keywords);
    expect(m?.entry.keyword).toBe('ex');
    expect(m?.query).toBe('hello world');
  });

  it('returns null for unrelated input', () => {
    expect(matchExtensionOmniboxKeyword('example.com', keywords)).toBeNull();
  });
});
