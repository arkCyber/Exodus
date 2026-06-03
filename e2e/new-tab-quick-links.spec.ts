/**
 * Playwright — NTP quick-link chip add/remove.
 */
import { test, expect } from '@playwright/test';
import { gotoBrowserShell } from './helpers/shell';

test.describe('NTP quick-link chips', () => {
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

  test('removes a default quick link chip', async ({ page }) => {
    const duckChip = page.locator('.ntp-chip[data-ntp-chip-url*="duckduckgo"]').first();
    await expect(duckChip).toBeVisible();

    await duckChip.click({ button: 'right' });
    await page.locator('.ntp-context-item', { hasText: 'Remove quick link' }).click();

    await expect(page.locator('.ntp-chip[data-ntp-chip-url*="duckduckgo"]')).toHaveCount(0);
    await expect(page.locator('.ntp-chip[data-ntp-chip-url]')).toHaveCount(3);
  });

  test('adds a top site to quick links via tile context menu', async ({ page }) => {
    const googleTile = page.locator('.ntp-tile[data-ntp-site-url*="google.com"]').first();
    await googleTile.click({ button: 'right' });
    await page.locator('.ntp-context-item', { hasText: 'Add to quick links' }).click();

    await expect(page.locator('.ntp-chip[data-ntp-chip-url*="google.com"]')).toBeVisible();
  });

  test('adds a quick link via + chip dialog', async ({ page }) => {
    await page.locator('.ntp-chip-add').click();
    await expect(page.locator('.ntp-url-dialog')).toBeVisible();
    await page.locator('.ntp-url-dialog-input').fill('https://example.com');
    await page.locator('.ntp-url-dialog-btn.primary').click();
    await expect(page.locator('.ntp-chip[data-ntp-chip-url*="example.com"]')).toBeVisible();
  });
});
