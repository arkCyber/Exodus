/**
 * Playwright — bookmark bar groups (Chrome tab-group style).
 */
import { test, expect } from '@playwright/test';
import { gotoBrowserShell } from './helpers/shell';

test.describe('Bookmark bar groups E2E', () => {
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('exodus-bookmark-bar-groups', '[]');
      localStorage.setItem('exodus-bookmark-folder-colors', '{}');
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
        ]),
      );
      localStorage.setItem('exodus-show-bookmark-bar', '1');
    });
    await gotoBrowserShell(page);
  });

  test('shows bookmark groups lead button with side panel and apps', async ({ page }) => {
    const bar = page.locator('.exodus-chrome-bookmarks');
    await expect(bar).toBeVisible({ timeout: 10_000 });
    await expect(bar.locator('.bookmark-bar__lead .bookmark-lead-btn')).toHaveCount(3);
    await expect(page.getByTestId('bookmark-groups-btn')).toBeVisible();
  });

  test('opens groups menu and creates a new group', async ({ page }) => {
    await page.getByTestId('bookmark-groups-btn').click();
    await expect(page.getByTestId('bookmark-groups-menu')).toBeVisible();
    await page.getByTestId('bookmark-group-create').click();
    await expect(page.getByTestId('bookmark-group-prompt')).toBeVisible();
    await page.getByTestId('bookmark-group-name-input').fill('Work Projects');
    await page.getByRole('button', { name: /^Create$|^创建$/ }).click();
    await expect(page.getByTestId('bookmark-group-prompt')).toBeHidden({ timeout: 5000 });
    await expect(page.locator('.bookmark-chip--folder .bookmark-label', { hasText: 'Work Projects' })).toBeVisible();
  });

  test('rejects reserved group name All bookmarks', async ({ page }) => {
    await page.getByTestId('bookmark-groups-btn').click();
    await page.getByTestId('bookmark-group-create').click();
    await page.getByTestId('bookmark-group-name-input').fill('All bookmarks');
    await expect(page.locator('.bookmark-group-prompt__error')).toBeVisible();
    await expect(page.getByRole('button', { name: /^Create$|^创建$/ })).toBeDisabled();
  });

  test('lists existing group in menu after creation', async ({ page }) => {
    await page.getByTestId('bookmark-groups-btn').click();
    await page.getByTestId('bookmark-group-create').click();
    await page.getByTestId('bookmark-group-name-input').fill('Dev');
    await page.getByRole('button', { name: /^Create$|^创建$/ }).click();
    await page.getByTestId('bookmark-groups-btn').click();
    await expect(page.locator('.bookmark-groups-menu__group-name', { hasText: 'Dev' })).toBeVisible();
  });
});
