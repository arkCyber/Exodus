/**
 * Playwright — core browser chrome navigation (tabs, omnibox, settings).
 * Aligned with vue-shell-qa.spec.ts selectors.
 */
import { test, expect } from '@playwright/test';

const OMNIBOX = '.url-input, #exodus-omnibox-input';

async function dismissPermissionPrompts(page: import('@playwright/test').Page): Promise<void> {
  for (let i = 0; i < 5; i++) {
    const backdrop = page.locator('.perm-backdrop');
    if (!(await backdrop.isVisible().catch(() => false))) return;
    const deny = page.getByRole('button', { name: 'Deny', exact: true });
    if (await deny.isVisible().catch(() => false)) {
      await deny.click({ force: true });
    } else {
      await page.keyboard.press('Escape');
    }
    await backdrop.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  }
}

test.describe('Exodus Browser E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
    await page
      .getByRole('button', { name: 'Deny', exact: true })
      .click({ force: true, timeout: 12_000 })
      .catch(() => {});
    await dismissPermissionPrompts(page);
  });

  test('loads homepage', async ({ page }) => {
    await expect(page).toHaveTitle(/Exodus/i);
    await expect(page.locator('.tab-item').first()).toBeVisible();
  });

  test('navigates to a URL', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 8000 });
    await expect(page.locator('iframe.browser-webview')).toBeVisible({ timeout: 8000 });
  });

  test('opens new tab with keyboard shortcut after leaving new-tab page', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 10_000 });

    const tabs = page.locator('.tab-item');
    const initial = await tabs.count();
    await page.locator('.tab-new').click();
    await expect(tabs).toHaveCount(initial + 1, { timeout: 8000 });
  });

  test('reuses empty new-tab page when clicking new-tab button', async ({ page }) => {
    const tabs = page.locator('.tab-item');
    await expect(tabs).toHaveCount(1);
    await page.locator('.tab-new').click();
    await expect(tabs).toHaveCount(1);
  });

  test('toggles sidebar', async ({ page }) => {
    const toggle = page.locator('[aria-label="Toggle sidebar"]');
    await expect(toggle).toBeVisible();
    await toggle.click();
    await expect(page.locator('.exodus-sidebar--firefox, .exodus-sidebar')).toBeVisible({
      timeout: 8000,
    });
  });

  test('opens settings modal via chrome menu', async ({ page }) => {
    await page.locator('.perm-backdrop').evaluateAll((nodes) => {
      for (const el of nodes) {
        (el as HTMLElement).style.pointerEvents = 'none';
      }
    });
    await page.locator('.chrome-menu-btn').evaluate((el) => (el as HTMLButtonElement).click());
    await page
      .locator('.chrome-menu-dropdown button.menu-item')
      .filter({ hasText: 'Settings' })
      .evaluate((el) => (el as HTMLButtonElement).click());
    await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible({ timeout: 10_000 });
  });
});
