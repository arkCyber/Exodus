/**
 * Exodus Browser — NTP quick-link chip store unit tests (add / remove).
 */
import { describe, it, expect, beforeEach } from 'vitest';
import type { QuickLink } from '@/lib/browserTypes';
import {
  addNtpQuickLink,
  buildNtpQuickLinks,
  canAddNtpQuickLink,
  isNtpQuickLink,
  isNtpQuickLinksFull,
  loadCustomQuickLinks,
  removeNtpQuickLink,
  resetNtpQuickLinks,
} from './ntpQuickLinksStore';
import { clearAllNtpLayoutStorage } from './ntpLayoutStore';

const customLink: QuickLink = {
  title: 'Example',
  url: 'https://example.com',
};

describe('ntpQuickLinksStore', () => {
  beforeEach(() => {
    clearAllNtpLayoutStorage();
    resetNtpQuickLinks();
  });

  it('returns four default quick links initially', () => {
    const chips = buildNtpQuickLinks();
    expect(chips).toHaveLength(4);
    expect(chips[0].title).toBe('DuckDuckGo');
  });

  it('addNtpQuickLink inserts a custom chip at the front', () => {
    expect(addNtpQuickLink(customLink)).toBe(true);
    const chips = buildNtpQuickLinks();
    expect(chips[0].url).toContain('example.com');
    expect(loadCustomQuickLinks()).toHaveLength(1);
  });

  it('removeNtpQuickLink removes default chip without refilling', () => {
    const duck = buildNtpQuickLinks().find((chip) => chip.title === 'DuckDuckGo');
    expect(duck).toBeTruthy();

    removeNtpQuickLink(duck!);
    const chips = buildNtpQuickLinks();
    expect(chips.some((chip) => chip.title === 'DuckDuckGo')).toBe(false);
    expect(chips).toHaveLength(3);
  });

  it('isNtpQuickLink reflects current chip row', () => {
    expect(isNtpQuickLink('https://duckduckgo.com')).toBe(true);
    removeNtpQuickLink({ title: 'DuckDuckGo', url: 'https://duckduckgo.com' });
    expect(isNtpQuickLink('https://duckduckgo.com')).toBe(false);
  });

  it('re-add clears removed flag for default chip', () => {
    removeNtpQuickLink({ title: 'DuckDuckGo', url: 'https://duckduckgo.com' });
    addNtpQuickLink({ title: 'DuckDuckGo', url: 'https://duckduckgo.com' });
    expect(buildNtpQuickLinks().some((chip) => chip.title === 'DuckDuckGo')).toBe(true);
  });

  it('isNtpQuickLinksFull when row has sixteen chips', () => {
    while (buildNtpQuickLinks().length > 0) {
      removeNtpQuickLink(buildNtpQuickLinks()[0]!);
    }
    for (let i = 0; i < 16; i += 1) {
      expect(addNtpQuickLink({ title: `Site ${i}`, url: `https://site-${i}.example.com` })).toBe(true);
    }
    expect(isNtpQuickLinksFull()).toBe(true);
    expect(canAddNtpQuickLink('https://extra.example.com')).toBe(false);
    expect(addNtpQuickLink({ title: 'Extra', url: 'https://extra.example.com' })).toBe(false);
  });
});
