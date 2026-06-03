/**
 * Playwright — NTP top-site add/remove via context menu.
 */
import { test, expect } from '@playwright/test';
import { OMNIBOX, gotoBrowserShell } from './helpers/shell';

test.describe('NTP top-site management', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const keys = [
        'exodus-top-sites',
        'exodus-pinned-sites',
        'exodus-removed-ntp-sites',
        'exodus-ntp-quick-links',
        'exodus-removed-ntp-quick-links',
        'exodus-ntp-layout-customized-v1',
      ];
      for (const key of keys) localStorage.removeItem(key);
    });
    await gotoBrowserShell(page);
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
  });

  test('removes a default top site from the grid', async ({ page }) => {
    const googleTile = page.locator('.ntp-tile[data-ntp-site-url*="google.com"]').first();
    await expect(googleTile).toBeVisible();

    await googleTile.click({ button: 'right' });
    await expect(page.locator('.ntp-context-menu')).toBeVisible();
    await page.locator('.ntp-context-item', { hasText: 'Remove from top sites' }).click();

    await expect(page.locator('.ntp-tile[data-ntp-site-url*="google.com"]')).toHaveCount(0);
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(7);
  });

  test('pins a site via tile context menu', async ({ page }) => {
    const githubTile = page.locator('.ntp-tile[data-ntp-site-url*="github.com"]').first();
    await githubTile.click({ button: 'right' });
    await page.locator('.ntp-context-item', { hasText: 'Pin to front' }).click();

    await githubTile.click({ button: 'right' });
    await expect(page.locator('.ntp-context-item', { hasText: 'Pin to front' })).toBeDisabled();
    await expect(page.locator('.ntp-context-item', { hasText: 'Unpin' })).toBeEnabled();
  });
});
