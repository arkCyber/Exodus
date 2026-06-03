/**
 * Playwright — new tab page visual parity (web shell matches expected NTP layout).
 */
import { test, expect } from '@playwright/test';

const OMNIBOX = '.url-input, #exodus-omnibox-input';

test.describe('New Tab Page parity', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
  });

  test('uses glassmorphism search bar', async ({ page }) => {
    const search = page.locator('.ntp-search-input');
    await expect(search).toBeVisible();
    const backdrop = await search.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return style.backdropFilter || style.webkitBackdropFilter;
    });
    expect(backdrop).toContain('blur');
  });

  test('shows wallpaper image with absolute URL', async ({ page }) => {
    const bg = page.locator('.ntp-bg-image');
    await expect(bg).toBeVisible({ timeout: 10_000 });
    const src = await bg.getAttribute('src');
    expect(src).toBeTruthy();
    expect(src).toMatch(/^https?:|^blob:|^data:|^asset:/);
  });

  test('content host uses NTP background class', async ({ page }) => {
    await expect(page.locator('.browser-content--ntp')).toBeVisible();
  });

  test('shows 8 top-site tiles and 4 quick-link chips', async ({ page }) => {
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(8);
    await expect(page.locator('.ntp-tile-label').first()).toContainText('google.com');
    await expect(page.locator('.ntp-chip[data-ntp-chip-url]')).toHaveCount(4);
    await expect(page.locator('.ntp-chip[data-ntp-chip-url]').first()).toContainText('DuckDuckGo');
    await expect(page.locator('.ntp-chip-add')).toBeVisible();
  });

  test('top-site tiles use glass blur', async ({ page }) => {
    const tile = page.locator('.ntp-tile').first();
    const backdrop = await tile.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return style.backdropFilter || style.webkitBackdropFilter;
    });
    expect(backdrop).toContain('blur');
  });
});
