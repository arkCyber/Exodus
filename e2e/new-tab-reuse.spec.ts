/**
 * Playwright — reuse existing empty new-tab page (Chrome parity).
 */
import { test, expect } from '@playwright/test';
import { OMNIBOX, TAB_STRIP, gotoBrowserShell } from './helpers/shell';

test.describe('new tab reuse', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
  });

  test('single new-tab page shows icon title and close (Chrome)', async ({ page }) => {
    await expect(page.locator(TAB_STRIP)).toHaveCount(1);
    await expect(page.locator(`${TAB_STRIP} .tab-title`)).toHaveText('New Tab');
    await expect(page.locator(`${TAB_STRIP} .tab-close`)).toHaveCount(1);
    const favicon = page.locator(`${TAB_STRIP} .tab-favicon`).first();
    await expect(favicon).toBeVisible();
    await expect(favicon).toHaveAttribute('src', /data:image\/svg/);
  });

  test('navigated tab shows site favicon not document icon', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://www.google.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/google/i, { timeout: 10_000 });
    const favicon = page.locator(`${TAB_STRIP}.active .tab-favicon, ${TAB_STRIP} .tab-item.active .tab-favicon`).first();
    await expect(favicon).toHaveAttribute('src', /google\.com\/s2\/favicons|gstatic|google/i, {
      timeout: 10_000,
    });
  });

  test('does not create a second tab while an empty new-tab page exists', async ({ page }) => {
    await expect(page.locator(TAB_STRIP)).toHaveCount(1);
    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(1);
    await expect(page.locator('.browser-page')).toHaveAttribute(
      'data-active-tab-url',
      /exodus-new-tab/,
    );
  });

  test('creates a new tab after navigating away from the new-tab page', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 10_000 });

    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(2, { timeout: 8000 });
    await expect(page.locator(TAB_STRIP).last()).toHaveAttribute('aria-label', /New Tab/i);
    await expect(page.locator(`${TAB_STRIP} .tab-title`).last()).toHaveText('New Tab');
    await expect(page.locator(`${TAB_STRIP} .tab-close`)).toHaveCount(2);

    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(2);
  });
});
