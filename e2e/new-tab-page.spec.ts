/**
 * Playwright — new tab page (Vue overlay) tests.
 */
import { test, expect } from '@playwright/test';

const OMNIBOX = '.url-input, #exodus-omnibox-input';

test.describe('New Tab Page Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
  });

  test('new tab page shows on startup', async ({ page }) => {
    const newTabPage = page.locator('.ntp.exodus-new-tab');
    await expect(newTabPage).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.ntp-logo')).toContainText('Exodus');
    await expect(page.getByText('TEST: App.vue is rendering')).toHaveCount(0);
  });

  test('new tab page displays 8 default top sites', async ({ page }) => {
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('[aria-label="Top sites"]')).toBeVisible();
    const tiles = page.locator('.ntp-tile[data-ntp-site-url]');
    await expect(tiles).toHaveCount(8);
    await expect(page.locator('.ntp-tile-label').first()).toContainText('google.com');
  });

  test('shows 8 default tiles when bookmarks exist in localStorage', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem(
        'browser-bookmarks',
        JSON.stringify([
          {
            id: 'bad-1',
            url: '"https',
            title: '"https',
            created_at: new Date().toISOString(),
            createdAt: Date.now(),
          },
          {
            id: 'ntp-1',
            url: 'about:blank#exodus-new-tab',
            title: 'New Tab',
            created_at: new Date().toISOString(),
            createdAt: Date.now(),
          },
        ]),
      );
    });
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.ntp-tile[data-ntp-site-url]')).toHaveCount(8);
    await expect(page.locator('.ntp-tile-label').first()).toContainText('google.com');
  });

  test('new tab page has search box and quick links', async ({ page }) => {
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.ntp-search-input')).toBeVisible();
    await expect(page.locator('.ntp-search-input')).toHaveAttribute('placeholder', 'Search or enter address');
    const chips = page.locator('.ntp-chip');
    expect(await chips.count()).toBeGreaterThanOrEqual(4);
  });

  test('new tab page has settings button', async ({ page }) => {
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.ntp-settings-btn')).toBeVisible();
  });

  test('wallpaper background image or solid underlay is present', async ({ page }) => {
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    const bg = page.locator('.ntp-bg-image');
    const hasBgImg = await bg.isVisible().catch(() => false);
    if (hasBgImg) {
      const src = await bg.getAttribute('src');
      expect(src).toBeTruthy();
    } else {
      await expect(page.locator('.ntp.exodus-new-tab')).toHaveCSS('background-color', 'rgb(10, 10, 15)');
    }
  });

  test('search box navigates on Enter', async ({ page }) => {
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    const searchInput = page.locator('.ntp-search-input');
    await searchInput.fill('example.com');
    await searchInput.press('Enter');
    await expect(page.locator('.ntp.exodus-new-tab')).toBeHidden({ timeout: 10_000 });
  });

  test('context menu appears on tile right-click', async ({ page }) => {
    await expect(page.locator('.ntp.exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    const firstTile = page.locator('.ntp-tile').first();
    await firstTile.click({ button: 'right' });
    await expect(page.locator('.ntp-context-menu')).toBeVisible({ timeout: 2000 });
  });
});
