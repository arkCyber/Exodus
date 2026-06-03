/**
 * Unit tests for sidebar layout helpers.
 */

import { describe, expect, it, beforeEach } from 'vitest';
import {
  SIDEBAR_CONTENT_DEFAULT_PX,
  SIDEBAR_CONTENT_MIN_PX,
  loadSidebarContentWidth,
  saveSidebarContentWidth,
  sidebarTotalWidthPx,
} from './sidebarLayout';
import { CHROME_LAYOUT } from './chromeLayout';

describe('sidebarLayout', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('defaults content width to 320px', () => {
    expect(loadSidebarContentWidth()).toBe(SIDEBAR_CONTENT_DEFAULT_PX);
  });

  it('persists content width', () => {
    saveSidebarContentWidth(400);
    expect(loadSidebarContentWidth()).toBe(400);
  });

  it('clamps saved width to minimum', () => {
    saveSidebarContentWidth(100);
    expect(loadSidebarContentWidth()).toBe(SIDEBAR_CONTENT_MIN_PX);
  });

  it('computes total width with icon rail', () => {
    expect(sidebarTotalWidthPx(320, false)).toBe(CHROME_LAYOUT.sidebarIconRail + 320);
    expect(sidebarTotalWidthPx(320, true)).toBe(CHROME_LAYOUT.sidebarIconRail);
  });
});
