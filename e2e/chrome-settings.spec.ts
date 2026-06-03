/**
 * Playwright — Chrome-style full-page settings.
 */
import { test, expect } from '@playwright/test';
import { gotoBrowserShell, openChromeSettings, OMNIBOX, TAB_STRIP } from './helpers/shell';

test.describe('Chrome settings full page', () => {
  test.setTimeout(60_000);

  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
  });

  test('opens full-page settings from chrome menu', async ({ page }) => {
    await page.locator('.perm-backdrop').evaluateAll((nodes) => {
      for (const el of nodes) {
        (el as HTMLElement).style.pointerEvents = 'none';
      }
    });
    await page.locator('.chrome-menu-btn').click();
    await page.locator('.chrome-menu-dropdown button.menu-item').filter({ hasText: 'Settings' }).click();
    await expect(page.getByTestId('chrome-settings-page')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId('chrome-settings-nav-browser')).toBeVisible();
  });

  test('navigates privacy section via sidebar', async ({ page }) => {
    await openChromeSettings(page);
    await page.getByTestId('chrome-settings-nav-privacy').click();
    await expect(page.locator('[data-testid="settings-section-privacy"]')).toBeVisible();
    await expect(page.locator('[data-testid="settings-section-privacy"]')).toContainText('HTTPS-only');
  });

  test('deep link chrome://settings/extensions opens extensions panel', async ({ page }) => {
    await openChromeSettings(page, 'extensions');
    await expect(page.getByTestId('chrome-settings-page')).toHaveAttribute('data-section', 'extensions');
    await expect(page.getByTestId('extensions-settings-panel')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('extensions-confirm-host')).toBeVisible();
    await expect(page.getByTestId('extensions-install-folder')).toBeVisible();
  });

  test('deep link chrome://settings/downloads opens download preferences', async ({ page }) => {
    await openChromeSettings(page, 'downloads');
    await expect(page.getByTestId('chrome-settings-page')).toHaveAttribute('data-section', 'downloads');
    await expect(page.getByTestId('downloads-settings-panel')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('settings-about-version')).toHaveCount(0);
  });

  test('about section shows version', async ({ page }) => {
    await openChromeSettings(page, 'about');
    await expect(page.getByTestId('chrome-settings-page')).toHaveAttribute('data-section', 'about');
    await expect(page.getByTestId('settings-about-version')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('settings-about-version')).toContainText(/\d+\.\d+/);
  });

  test('chrome://extensions opens extensions manager', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('chrome://extensions');
    await addressBar.press('Enter');
    await expect(page.getByTestId('chrome-settings-page')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId('chrome-settings-page')).toHaveAttribute('data-section', 'extensions');
    await expect(page.getByTestId('extensions-settings-panel')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('extensions-installed-list')).toBeVisible();
  });

  test('close settings returns to browsable shell', async ({ page }) => {
    await openChromeSettings(page);
    await page.getByTestId('chrome-settings-close').click();
    await expect(page.getByTestId('chrome-settings-page')).toHaveCount(0, { timeout: 15_000 });
    const omnibox = page.locator(OMNIBOX).first();
    await expect(omnibox).not.toHaveValue(/chrome:\/\/settings/i, { timeout: 10_000 });
    await expect(page.locator(TAB_STRIP).first()).toBeVisible();
  });

  test('search filters sidebar items', async ({ page }) => {
    await openChromeSettings(page);
    await page.getByTestId('chrome-settings-search').fill('AI');
    await expect(page.getByTestId('chrome-settings-nav-ai')).toBeVisible();
    await expect(page.getByTestId('chrome-settings-nav-browser')).toHaveCount(0);
  });

  test('browser section exposes homepage and bookmark bar fields', async ({ page }) => {
    await openChromeSettings(page, 'browser');
    await expect(page.getByTestId('settings-homepage-url')).toBeVisible();
    await expect(page.getByTestId('settings-search-url')).toBeVisible();
    await expect(page.getByTestId('settings-bookmark-bar')).toBeVisible();
  });

  test('privacy section shows clear browsing data panel', async ({ page }) => {
    await openChromeSettings(page, 'privacy');
    await expect(page.getByTestId('clear-browsing-data-panel')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('clear-data-cookies')).toBeVisible();
    await expect(page.getByTestId('settings-https-only')).toBeVisible();
  });

  test('startup section shows session restore toggle', async ({ page }) => {
    await openChromeSettings(page, 'startup');
    await expect(page.getByTestId('settings-session-restore')).toBeVisible();
  });

  test('autofill section shows password manager panel', async ({ page }) => {
    await openChromeSettings(page, 'autofill');
    await expect(page.getByTestId('password-manager-panel')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('password-manager-search')).toBeVisible();
  });

  test('history section shows history manager panel', async ({ page }) => {
    await openChromeSettings(page, 'history');
    await expect(page.getByTestId('history-manager-panel')).toBeVisible({ timeout: 10_000 });
  });

  test('sidebar navigates all primary settings sections', async ({ page }) => {
    await openChromeSettings(page);
    const sections = ['browser', 'privacy', 'appearance', 'startup', 'extensions', 'about'] as const;
    for (const id of sections) {
      await page.getByTestId(`chrome-settings-nav-${id}`).click();
      await expect(page.getByTestId('chrome-settings-page')).toHaveAttribute('data-section', id);
    }
  });
});
