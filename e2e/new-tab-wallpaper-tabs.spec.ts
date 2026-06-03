/**
 * Playwright — per-tab new tab wallpaper (Cmd+T / new tab button).
 */
import { test, expect } from '@playwright/test';
import { OMNIBOX, TAB_STRIP, gotoBrowserShell } from './helpers/shell';

test.describe('new tab wallpaper per tab', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
  });

  test('second new tab gets a different wallpaper id', async ({ page }) => {
    const firstId = await page
      .locator('.ntp.exodus-new-tab')
      .getAttribute('data-ntp-wallpaper-id');
    expect(firstId).toBeTruthy();

    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 10_000 });

    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(2, { timeout: 8000 });
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });

    const secondId = await page
      .locator('.ntp.exodus-new-tab')
      .getAttribute('data-ntp-wallpaper-id');
    expect(secondId).toBeTruthy();
    expect(secondId).not.toBe(firstId);
  });

  test('restored closed new tab receives a wallpaper id', async ({ page }) => {
    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(1);

    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 10_000 });

    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(2, { timeout: 8000 });

    await page.keyboard.press('Meta+w');
    await expect(page.locator(TAB_STRIP)).toHaveCount(1, { timeout: 8000 });

    await page.keyboard.press('Meta+Shift+t');
    await expect(page.locator(TAB_STRIP)).toHaveCount(2, { timeout: 8000 });
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });

    const restoredId = await page
      .locator('.ntp.exodus-new-tab')
      .getAttribute('data-ntp-wallpaper-id');
    expect(restoredId).toBeTruthy();
  });
});
