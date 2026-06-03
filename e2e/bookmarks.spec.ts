/**
 * Playwright — bookmarks toolbar and sidebar panel.
 */
import { test, expect } from '@playwright/test';
import { gotoBrowserShell } from './helpers/shell';

test.describe('Bookmarks E2E Tests', () => {
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem(
        'browser-bookmarks',
        JSON.stringify([
          {
            id: 'bm-1',
            title: 'Example',
            url: 'https://example.com',
            created_at: new Date().toISOString(),
            bar_order: 0,
          },
          {
            id: 'bm-2',
            title: 'GitHub',
            url: 'https://github.com',
            created_at: new Date().toISOString(),
            bar_order: 1,
          },
        ]),
      );
      localStorage.setItem('exodus-show-bookmark-bar', '1');
    });
    await gotoBrowserShell(page);
  });

  test('shows Chrome extension puzzle button in toolbar', async ({ page }) => {
    const end = page.locator('.exodus-chrome-toolbar .toolbar-end');
    await expect(end.locator('.extension-puzzle-btn')).toBeVisible({ timeout: 10_000 });
    await expect(end.locator('.extension-action-letter').first()).toBeVisible();
  });

  test('opens extensions manager from puzzle button', async ({ page }) => {
    await page.locator('.exodus-chrome-toolbar .toolbar-end .extension-puzzle-btn').click();
    await expect(page.locator('.chrome-internal-page[data-page="extensions"]')).toBeVisible({
      timeout: 10_000,
    });
  });

  test('shows Chrome-aligned bookmark bar with chips', async ({ page }) => {
    const bar = page.locator('.exodus-chrome-bookmarks');
    await expect(bar).toBeVisible({ timeout: 10_000 });
    await expect(bar.locator('.bookmark-bar__lead .bookmark-lead-btn')).toHaveCount(3);
    await expect(page.getByTestId('bookmark-groups-btn')).toBeVisible();
    await expect(bar.locator('.bookmark-bar-separator')).toHaveCount(2);
    await expect(bar.locator('.bookmark-bar__scroll .bookmark-chip').first()).toBeVisible();
    await expect(bar.locator('.bookmark-label', { hasText: 'Example' })).toBeVisible();
    await expect(bar.locator('.bookmark-chip--all')).toBeVisible();
  });

  test('navigates when bookmark chip is clicked', async ({ page }) => {
    await page.locator('.exodus-chrome-bookmarks .bookmark-bar__scroll .bookmark-chip').first().click();
    await expect(page.locator('.url-input, #exodus-omnibox-input').first()).toHaveValue(/example\.com/, {
      timeout: 10_000,
    });
  });

  test('opens all bookmarks dropdown from bar folder', async ({ page }) => {
    await page.locator('.exodus-chrome-bookmarks .bookmark-chip--all').click();
    await expect(page.getByRole('menuitem', { name: 'Example' })).toBeVisible();
    await expect(page.getByRole('menuitem', { name: 'GitHub' })).toBeVisible();
  });

  test('opens bookmarks manager from all bookmarks menu', async ({ page }) => {
    await page.locator('.exodus-chrome-bookmarks .bookmark-chip--all').click();
    await page.getByRole('button', { name: /Open bookmarks manager|打开书签管理器/ }).click();
    await expect(page.locator('.exodus-sidebar')).toBeVisible({ timeout: 8000 });
    await expect(page.getByPlaceholder('Search bookmarks...')).toBeVisible();
  });

  test('toggles sidebar from bookmark bar side panel button', async ({ page }) => {
    const sidePanelBtn = page.locator('.exodus-chrome-bookmarks .bookmark-lead-btn').first();
    await expect(sidePanelBtn).toBeVisible();
    await sidePanelBtn.click();
    await expect(page.locator('.exodus-sidebar')).toBeVisible({ timeout: 8000 });
    await sidePanelBtn.click();
    await expect(page.locator('.exodus-sidebar')).toBeHidden({ timeout: 8000 });
  });

  test('opens apps page from bookmark bar apps button', async ({ page }) => {
    await page.locator('.exodus-chrome-bookmarks .bookmark-lead-btn').nth(1).click();
    await expect(page.locator('.chrome-internal-page[data-page="apps"]')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.url-input, #exodus-omnibox-input').first()).toHaveValue(/chrome:\/\/apps/, {
      timeout: 10_000,
    });
  });

  test('opens add bookmark dialog from blank bar context menu', async ({ page }) => {
    const addressBar = page.locator('.url-input, #exodus-omnibox-input').first();
    await addressBar.fill('https://example.org');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.org/, { timeout: 10_000 });

    const spacer = page.locator('.exodus-chrome-bookmarks .bookmark-bar__spacer');
    await expect(spacer).toBeVisible();
    await spacer.click({ button: 'right' });
    await page.getByRole('menuitem', { name: /Add page|添加网页/ }).click();
    await expect(page.locator('.dialog-overlay h3')).toHaveText(/Add Bookmark|添加书签/);
    await expect(page.locator('#bookmark-url')).toHaveValue(/example\.org/);
  });

  test('reorders bookmark chips via drag and drop', async ({ page }) => {
    const scroll = page.locator('.exodus-chrome-bookmarks .bookmark-bar__scroll');
    const example = scroll.locator('.bookmark-chip', { hasText: 'Example' });
    const github = scroll.locator('.bookmark-chip', { hasText: 'GitHub' });
    await expect(example).toBeVisible();
    await expect(github).toBeVisible();
    await example.dragTo(github);
    await expect(scroll.locator('.bookmark-chip').first()).toContainText('GitHub');
    await expect(scroll.locator('.bookmark-chip').nth(1)).toContainText('Example');
  });

  test('opens bookmark context menu and deletes bookmark', async ({ page }) => {
    const chip = page.locator('.exodus-chrome-bookmarks .bookmark-bar__scroll .bookmark-chip', {
      hasText: 'Example',
    });
    await chip.click({ button: 'right' });
    await page.getByRole('menuitem', { name: /Delete|删除/ }).click();
    await expect(
      page.locator('.exodus-chrome-bookmarks .bookmark-bar__scroll .bookmark-chip', { hasText: 'Example' }),
    ).toHaveCount(0);
    await expect(page.locator('.exodus-chrome-bookmarks .bookmark-chip', { hasText: 'GitHub' })).toBeVisible();
  });

  test('opens bookmarks panel from toolbar', async ({ page }) => {
    await page.locator('button[title="Bookmarks"]').click();
    await expect(page.locator('.exodus-sidebar')).toBeVisible({ timeout: 8000 });
    await expect(page.getByPlaceholder('Search bookmarks...')).toBeVisible();
  });

  test('toggles bookmark star on navigation', async ({ page }) => {
    const addressBar = page.locator('.url-input, #exodus-omnibox-input').first();
    await addressBar.fill('https://example.org');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.org/, { timeout: 8000 });

    const star = page.locator('button.toolbar-icon-btn').filter({ hasText: '☆' }).first();
    await expect(star).toBeVisible({ timeout: 8000 });
    await star.click();
    await expect(page.locator('button.toolbar-icon-btn.bookmarked')).toBeVisible();
  });

  test('searches bookmarks in sidebar panel', async ({ page }) => {
    await page.locator('button[title="Bookmarks"]').click();
    const searchInput = page.getByPlaceholder('Search bookmarks...');
    await expect(searchInput).toBeVisible({ timeout: 8000 });
    await searchInput.fill('example');
    await expect(searchInput).toHaveValue('example');
  });
});

test.describe('Bookmarks zh locale', () => {
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(navigator, 'language', { value: 'zh-CN', configurable: true });
      localStorage.setItem('exodus-show-bookmark-bar', '1');
    });
    await gotoBrowserShell(page);
  });

  test('shows localized all-bookmarks label', async ({ page }) => {
    await expect(page.locator('.exodus-chrome-bookmarks .bookmark-chip--all .bookmark-label')).toHaveText('所有书签', {
      timeout: 10_000,
    });
  });
});
