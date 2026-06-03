/**
 * Playwright — chrome:// internal URLs and hash routes.
 */
import { test, expect } from '@playwright/test';
import { gotoBrowserShell, gotoWithRetry, OMNIBOX } from './helpers/shell';

test.describe('chrome:// internal pages', () => {
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
  });

  test('navigates to chrome://settings in omnibox', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('chrome://settings');
    await addressBar.press('Enter');
    await expect(page.locator('.browser-page')).toHaveAttribute(
      'data-active-tab-url',
      /chrome:\/\/settings/,
      { timeout: 12_000 },
    );
    await expect(page.locator('.chrome-internal-page[data-page="settings"]')).toBeVisible();
    await expect(page.getByTestId('chrome-settings-page')).toBeVisible();
    await expect(page.getByTestId('chrome-settings-nav-privacy')).toBeVisible();
  });

  test('hash route /chrome/settings opens settings page', async ({ page }) => {
    await gotoWithRetry(page, '/#/chrome/settings');
    await expect(page.locator('.browser-page')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('.browser-page')).toHaveAttribute(
      'data-active-tab-url',
      /chrome:\/\/settings/,
      { timeout: 15_000 },
    );
    await expect(page.locator('.chrome-internal-page')).toBeVisible({ timeout: 10_000 });
  });

  test('navigates to chrome://apps and shows apps page', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('chrome://apps');
    await addressBar.press('Enter');
    await expect(page.locator('.browser-page')).toHaveAttribute(
      'data-active-tab-url',
      /chrome:\/\/apps/,
      { timeout: 12_000 },
    );
    await expect(page.locator('.chrome-internal-page[data-page="apps"]')).toBeVisible();
    await expect(page.locator('.chrome-internal-header h1')).toHaveText('Apps');
    await expect(page.locator('.chrome-apps-page')).toBeVisible();
    await expect(page.locator('.chrome-apps-grid')).toBeVisible();
  });
});
