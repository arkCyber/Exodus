/**
 * Exodus Browser — BrowserPage NTP top-site management tests.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import type { QuickLink } from '@/lib/browserTypes';
import { clearAllNtpLayoutStorage } from '@/lib/ntpLayoutStore';
import {
  addNtpTopSite,
  buildNtpTopSitesGrid,
  isNtpTopSitePinned,
  pinNtpTopSite,
  removeNtpTopSite,
  unpinNtpTopSite,
} from '@/lib/ntpTopSitesStore';

describe('BrowserPage NTP top-site management', () => {
  const mockSite: QuickLink = {
    title: 'Example Site',
    url: 'https://example.com',
  };

  beforeEach(() => {
    clearAllNtpLayoutStorage();
  });

  /** Free one grid slot (defaults fill all eight on first run). */
  function freeTopSiteSlot(): void {
    removeNtpTopSite(buildNtpTopSitesGrid()[0]!);
  }

  it('pins a site via store API', () => {
    freeTopSiteSlot();
    pinNtpTopSite(mockSite);
    expect(isNtpTopSitePinned(mockSite.url)).toBe(true);
  });

  it('does not duplicate pinned sites', () => {
    freeTopSiteSlot();
    pinNtpTopSite(mockSite);
    pinNtpTopSite(mockSite);
    expect(buildNtpTopSitesGrid().filter((site) => site.url.includes('example.com'))).toHaveLength(1);
  });

  it('unpins a site', () => {
    freeTopSiteSlot();
    pinNtpTopSite(mockSite);
    unpinNtpTopSite(mockSite);
    expect(isNtpTopSitePinned(mockSite.url)).toBe(false);
  });

  it('removes a site from the grid', () => {
    freeTopSiteSlot();
    addNtpTopSite(mockSite);
    removeNtpTopSite(mockSite);
    expect(buildNtpTopSitesGrid().some((site) => site.url.includes('example.com'))).toBe(false);
  });

  it('remove clears both custom and pinned records', () => {
    freeTopSiteSlot();
    pinNtpTopSite(mockSite);
    removeNtpTopSite(mockSite);
    expect(isNtpTopSitePinned(mockSite.url)).toBe(false);
    expect(buildNtpTopSitesGrid().some((site) => site.url.includes('example.com'))).toBe(false);
  });

  it('handles corrupted localStorage gracefully on rebuild', () => {
    localStorage.setItem('exodus-pinned-sites', 'invalid-json');
    const grid = buildNtpTopSitesGrid();
    expect(grid).toHaveLength(8);
  });
});
