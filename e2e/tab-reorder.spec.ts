/**
 * Playwright — tab strip drag-reorder (Chrome parity).
 */
import { test, expect } from '@playwright/test';
import {
  OMNIBOX,
  TAB_STRIP,
  dragTabStripIndex,
  gotoBrowserShell,
  tabStripTitles,
} from './helpers/shell';

test.describe('tab drag reorder', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
  });

  test('horizontal tabs show favicon and title (Chrome)', async ({ page }) => {
    await expect(page.locator('.exodus-chrome-tabstrip .tab-favicon').first()).toBeVisible();
    await expect(page.locator('.exodus-chrome-tabstrip .tab-title').first()).toHaveText('New Tab');
  });

  test('reorders unpinned tabs via drag and drop', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 10_000 });
    await expect(page.locator('.exodus-chrome-tabstrip .tab-item.active')).toHaveAttribute(
      'aria-label',
      /Example|example\.com/i,
      { timeout: 10_000 },
    );

    const initialCount = await page.locator(TAB_STRIP).count();
    await page.locator('.tab-new').click();
    await expect(page.locator(TAB_STRIP)).toHaveCount(initialCount + 1, { timeout: 8000 });

    const before = await tabStripTitles(page);
    expect(before.length).toBeGreaterThanOrEqual(2);
    expect(before[0]).toMatch(/Example|example\.com/i);
    expect(before[1]).toMatch(/New Tab/i);

    await dragTabStripIndex(page, 1, 0);

    const after = await tabStripTitles(page);
    expect(after[0]).toMatch(/New Tab/i);
    expect(after[1]).toMatch(/Example|example\.com/i);
  });
});
