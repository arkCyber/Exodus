/**
 * Playwright — NTP layout factory reset, empty grid add, and grid capacity.
 */
import { test, expect } from '@playwright/test';
import {
  NTP_LAYOUT_STORAGE_KEYS,
  STATUS_MESSAGE,
  gotoBrowserShell,
  openSettingsModal,
} from './helpers/shell';

test.describe('NTP layout settings and capacity', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript((keys: string[]) => {
      for (const key of keys) localStorage.removeItem(key);
    }, [...NTP_LAYOUT_STORAGE_KEYS]);
    await gotoBrowserShell(page);
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
  });

  test('restores default layout from settings', async ({ page }) => {
    const googleTile = page.locator('.ntp-tile[data-ntp-site-url*="google.com"]').first();
    await googleTile.click({ button: 'right' });
    await page.locator('.ntp-context-item', { hasText: 'Remove from top sites' }).click();
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(7);

    await openSettingsModal(page);
    await page.locator('#settings-section-ntp-layout button', { hasText: 'Restore default layout' }).click();
    await expect(page.locator('.ntp-tile[data-ntp-site-url*="google.com"]')).toBeVisible();
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(8);
    await expect(page.locator(STATUS_MESSAGE)).toContainText('restored to defaults');
  });

  test('adds top site via + when grid is empty', async ({ page }) => {
    const tiles = page.locator('.ntp-tile[data-ntp-site-url]');
    while ((await tiles.count()) > 0) {
      await tiles.first().click({ button: 'right' });
      await page.locator('.ntp-context-item', { hasText: 'Remove from top sites' }).click();
    }
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(0);
    await expect(page.locator('.ntp-empty-hint')).toBeVisible();

    await page.locator('.ntp-tile-add').click();
    await page.locator('.ntp-url-dialog-input').fill('https://example.com');
    await page.locator('.ntp-url-dialog-btn.primary').click();
    await expect(page.locator('.ntp-tile[data-ntp-site-url*="example.com"]')).toBeVisible();
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(1);
  });

  test('rejects add when top sites grid is full', async ({ page }) => {
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(8);

    await page.locator('.ntp-tile-add').click();
    await page.locator('.ntp-url-dialog-input').fill('https://example.com');
    await page.locator('.ntp-url-dialog-btn.primary').click();
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(8);
    await expect(page.locator('.ntp-tile[data-ntp-site-url*="example.com"]')).toHaveCount(0);
    await expect(page.locator(STATUS_MESSAGE)).toContainText('full');
  });
});
