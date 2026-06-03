/**
 * Exodus Browser — omnibox resolution unit tests.
 */

import { describe, expect, it } from 'vitest';
import { buildSearchUrl, resolveOmniboxInput } from './omnibox';

const SEARCH = 'https://duckduckgo.com/?q={query}';

describe('buildSearchUrl', () => {
  it('encodes query into template', () => {
    expect(buildSearchUrl(SEARCH, 'hello world')).toBe(
      'https://duckduckgo.com/?q=hello%20world',
    );
  });
});

describe('resolveOmniboxInput', () => {
  it('returns null for empty input', () => {
    expect(resolveOmniboxInput('  ', SEARCH)).toBeNull();
  });

  it('parses /ask local search', () => {
    expect(resolveOmniboxInput('/ask rust memory', SEARCH)).toEqual({
      kind: 'ask',
      query: 'rust memory',
    });
  });

  it('keeps https URLs', () => {
    expect(resolveOmniboxInput('https://example.com/path', SEARCH)).toEqual({
      kind: 'navigate',
      url: 'https://example.com/path',
    });
  });

  it('adds https to bare domains', () => {
    const r = resolveOmniboxInput('github.com', SEARCH);
    expect(r?.kind).toBe('navigate');
    if (r?.kind === 'navigate') {
      expect(r.url).toContain('github.com');
    }
  });

  it('treats spaced text as web search', () => {
    const r = resolveOmniboxInput('rust tutorial', SEARCH);
    expect(r?.kind).toBe('navigate');
    if (r?.kind === 'navigate') {
      expect(r.url).toContain('rust%20tutorial');
    }
  });

  it('supports localhost', () => {
    const r = resolveOmniboxInput('localhost:1420', SEARCH);
    expect(r?.kind).toBe('navigate');
    if (r?.kind === 'navigate') {
      expect(r.url).toBe('http://localhost:1420');
    }
  });
});
