/**
 * Unit tests for new-tab page URL helpers.
 */

import { describe, expect, it } from 'vitest';
import {
  buildNewTabHtml,
  DEFAULT_QUICK_LINKS,
  initNewTabPage,
  isNewTabUrl,
  NEWTAB_PAGE_MARKER,
  NEWTAB_PAGE_URL,
} from './newTabPage';

describe('isNewTabUrl', () => {
  it('recognizes the built-in new tab data URL', () => {
    expect(isNewTabUrl(NEWTAB_PAGE_URL)).toBe(true);
  });

  it('rejects normal https URLs', () => {
    expect(isNewTabUrl('https://example.com')).toBe(false);
  });
});

describe('buildNewTabHtml', () => {
  it('includes marker and wallpaper background only (overlay UI in Svelte)', () => {
    const html = buildNewTabHtml({ wallpaperId: 'sunset' });
    expect(html).toContain(NEWTAB_PAGE_MARKER);
    expect(html).toContain('sunset.svg');
    expect(html).not.toContain('Quick tips');
  });
});

describe('initNewTabPage', () => {
  it('returns a data URL usable by isNewTabUrl', () => {
    const url = initNewTabPage({ wallpaperId: 'aurora' });
    expect(url.startsWith('data:text/html')).toBe(true);
    expect(isNewTabUrl(url)).toBe(true);
  });
});

describe('DEFAULT_QUICK_LINKS', () => {
  it('includes four starter links', () => {
    expect(DEFAULT_QUICK_LINKS).toHaveLength(4);
    expect(DEFAULT_QUICK_LINKS[0].url).toContain('duckduckgo');
  });
});
