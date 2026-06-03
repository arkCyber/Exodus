/**
 * Exodus Browser — NTP layout mode unit tests.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { clearAllNtpLayoutStorage, isNtpLayoutCustomized, markNtpLayoutCustomized, resetAllNtpLayout } from './ntpLayoutStore';
import { buildNtpQuickLinks } from './ntpQuickLinksStore';
import { buildNtpTopSitesGrid, removeNtpTopSite } from './ntpTopSitesStore';

describe('ntpLayoutStore', () => {
  beforeEach(() => {
    clearAllNtpLayoutStorage();
  });

  it('starts in first-run mode with bundled defaults', () => {
    expect(isNtpLayoutCustomized()).toBe(false);
    expect(buildNtpTopSitesGrid()).toHaveLength(8);
    expect(buildNtpQuickLinks()).toHaveLength(4);
  });

  it('enters customized mode after remove without refilling', () => {
    const google = buildNtpTopSitesGrid().find((s) => s.url.includes('google.com'));
    removeNtpTopSite(google!);
    expect(isNtpLayoutCustomized()).toBe(true);
    expect(buildNtpTopSitesGrid()).toHaveLength(7);
  });

  it('markNtpLayoutCustomized sets flag', () => {
    markNtpLayoutCustomized();
    expect(isNtpLayoutCustomized()).toBe(true);
  });

  it('resetAllNtpLayout restores first-run defaults', () => {
    const google = buildNtpTopSitesGrid().find((s) => s.url.includes('google.com'));
    removeNtpTopSite(google!);
    expect(isNtpLayoutCustomized()).toBe(true);
    expect(buildNtpTopSitesGrid()).toHaveLength(7);

    resetAllNtpLayout();
    expect(isNtpLayoutCustomized()).toBe(false);
    expect(buildNtpTopSitesGrid()).toHaveLength(8);
    expect(buildNtpQuickLinks()).toHaveLength(4);
  });
});
