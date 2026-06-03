/**
 * Exodus Browser — NTP top-site store unit tests (add / pin / remove).
 */
import { describe, it, expect, beforeEach } from 'vitest';
import type { QuickLink } from '@/lib/browserTypes';
import {
  addNtpTopSite,
  buildNtpTopSitesGrid,
  canAddNtpTopSite,
  isNtpTopSitePinned,
  isNtpTopSitesGridFull,
  loadCustomTopSites,
  loadPinnedTopSites,
  normalizeNtpSiteUrl,
  pinNtpTopSite,
  removeNtpTopSite,
  resetNtpTopSites,
  unpinNtpTopSite,
} from './ntpTopSitesStore';
import { clearAllNtpLayoutStorage } from './ntpLayoutStore';

const exampleSite: QuickLink = {
  title: 'Example Site',
  url: 'https://example.com',
};

describe('ntpTopSitesStore', () => {
  beforeEach(() => {
    clearAllNtpLayoutStorage();
    resetNtpTopSites();
  });

  it('normalizeNtpSiteUrl strips hash and normalizes protocol', () => {
    expect(normalizeNtpSiteUrl('https://www.example.com/path#frag')).toBe(
      'https://www.example.com/path',
    );
  });

  it('addNtpTopSite inserts a custom site at the front', () => {
    removeNtpTopSite(buildNtpTopSitesGrid()[0]!);
    expect(addNtpTopSite(exampleSite)).toBe(true);
    const custom = loadCustomTopSites();
    expect(custom).toHaveLength(1);
    expect(custom[0].url).toBe('https://example.com/');
  });

  it('pinNtpTopSite marks site as pinned and visible on grid', () => {
    removeNtpTopSite(buildNtpTopSitesGrid()[0]!);
    expect(pinNtpTopSite(exampleSite)).toBe(true);
    expect(isNtpTopSitePinned(exampleSite.url)).toBe(true);
    expect(loadPinnedTopSites()).toHaveLength(1);
    const grid = buildNtpTopSitesGrid();
    expect(grid[0].url).toBe('https://example.com/');
  });

  it('unpinNtpTopSite removes pin but keeps site if custom', () => {
    removeNtpTopSite(buildNtpTopSitesGrid()[0]!);
    pinNtpTopSite(exampleSite);
    unpinNtpTopSite(exampleSite);
    expect(isNtpTopSitePinned(exampleSite.url)).toBe(false);
    expect(loadPinnedTopSites()).toHaveLength(0);
    expect(loadCustomTopSites()).toHaveLength(1);
  });

  it('removeNtpTopSite removes site from grid and blocks reappearance', () => {
    const gridBefore = buildNtpTopSitesGrid();
    const google = gridBefore.find((site) => site.url.includes('google.com'));
    expect(google).toBeTruthy();

    removeNtpTopSite(google!);
    const gridAfter = buildNtpTopSitesGrid();
    expect(gridAfter.some((site) => site.url.includes('google.com'))).toBe(false);
    expect(gridAfter).toHaveLength(7);
  });

  it('remove also clears pinned and custom entries', () => {
    pinNtpTopSite(exampleSite);
    removeNtpTopSite(exampleSite);
    expect(loadPinnedTopSites()).toHaveLength(0);
    expect(loadCustomTopSites()).toHaveLength(0);
    expect(buildNtpTopSitesGrid().some((site) => site.url.includes('example.com'))).toBe(false);
  });

  it('re-add clears removed flag', () => {
    removeNtpTopSite({ title: 'Google', url: 'https://www.google.com' });
    addNtpTopSite({ title: 'Google', url: 'https://www.google.com' });
    expect(buildNtpTopSitesGrid().some((site) => site.url.includes('google.com'))).toBe(true);
  });

  it('isNtpTopSitesGridFull when eight sites visible', () => {
    expect(isNtpTopSitesGridFull()).toBe(true);
    expect(canAddNtpTopSite('https://example.com')).toBe(false);
  });

  it('addNtpTopSite rejects new site when grid is full', () => {
    expect(addNtpTopSite(exampleSite)).toBe(false);
    expect(buildNtpTopSitesGrid().some((site) => site.url.includes('example.com'))).toBe(false);
  });

  it('addNtpTopSite allows reorder when site already visible', () => {
    const github = buildNtpTopSitesGrid().find((site) => site.url.includes('github.com'));
    expect(github).toBeTruthy();
    expect(addNtpTopSite(github!)).toBe(true);
  });
});
