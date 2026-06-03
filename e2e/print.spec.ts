/**
 * Playwright — print pipeline (⌘P / chrome menu).
 */
import { test, expect } from '@playwright/test';
import { OMNIBOX, STATUS_MESSAGE, gotoBrowserShell } from './helpers/shell';

test.describe('print pipeline', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
  });

  test('⌘P on new tab reports print unavailable', async ({ page }) => {
    await page.locator('body').click();
    await page.keyboard.press('Meta+p');
    await expect(page.locator(STATUS_MESSAGE)).toContainText('Print not available', {
      timeout: 5000,
    });
  });

  test('⌘P on chrome://settings reports print unavailable', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('chrome://settings');
    await addressBar.press('Enter');
    await expect(page.locator('.chrome-internal-page')).toBeVisible({ timeout: 10_000 });

    await page.locator('body').click();
    await page.keyboard.press('Meta+p');
    await expect(page.locator(STATUS_MESSAGE)).toContainText('Print not available', {
      timeout: 5000,
    });
  });

  test('chrome menu Print on https page reports dialog opened', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 10_000 });
    await expect(page.locator('iframe.browser-webview')).toBeVisible({ timeout: 10_000 });

    await page.locator('.perm-backdrop').evaluateAll((nodes) => {
      for (const el of nodes) {
        (el as HTMLElement).style.pointerEvents = 'none';
      }
    });
    await page.locator('.chrome-menu-btn').evaluate((el) => (el as HTMLButtonElement).click());
    await expect(page.locator('.chrome-menu-dropdown')).toBeVisible();
    await page
      .locator('.chrome-menu-dropdown button.menu-item')
      .filter({ hasText: 'Print' })
      .evaluate((el) => (el as HTMLButtonElement).click());

    await expect(page.locator(STATUS_MESSAGE)).toContainText('Print dialog opened', {
      timeout: 8000,
    });
  });
});
