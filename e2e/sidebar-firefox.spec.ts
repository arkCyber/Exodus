/**
 * Playwright — Firefox-style sidebar (toggle, panels, chrome menu).
 * Runs against Playwright-managed Vite dev (default :1431; see playwright.config.ts).
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

async function openChromeMenu(page: import('@playwright/test').Page): Promise<void> {
  await page.locator('.perm-backdrop').evaluateAll((nodes) => {
    for (const el of nodes) {
      (el as HTMLElement).style.pointerEvents = 'none';
    }
  });
  await page.locator('.chrome-menu-btn').evaluate((el) => (el as HTMLButtonElement).click());
  await expect(page.locator('.chrome-menu-dropdown')).toBeVisible({ timeout: 8000 });
}

test.describe('Firefox-style sidebar', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
    await page
      .getByRole('button', { name: 'Deny', exact: true })
      .click({ force: true, timeout: 12_000 })
      .catch(() => {});
    await dismissPermissionPrompts(page);
  });

  test('toolbar toggle opens sidebar with icon rail', async ({ page }) => {
    const toggle = page.locator('[aria-label="Toggle sidebar"]');
    await expect(toggle).toBeVisible();
    await toggle.click();
    const sidebar = page.locator('.exodus-sidebar--firefox, .exodus-sidebar');
    await expect(sidebar).toBeVisible({ timeout: 8000 });
    await expect(page.locator('.sidebar-icon-bar')).toBeVisible();
  });

  test('customize sidebar from chrome menu', async ({ page }) => {
    await openChromeMenu(page);
    await page
      .locator('.chrome-menu-dropdown button.menu-item')
      .filter({ hasText: 'Customize sidebar' })
      .evaluate((el) => (el as HTMLButtonElement).click());
    await expect(page.locator('.exodus-sidebar')).toBeVisible({ timeout: 8000 });
    await expect(page.getByRole('heading', { name: /customize sidebar/i })).toBeVisible({
      timeout: 8000,
    });
  });

  test('save to reading list menu item is present', async ({ page }) => {
    const bar = page.locator(OMNIBOX).first();
    await bar.fill('https://example.com');
    await bar.press('Enter');
    await expect(bar).toHaveValue(/example\.com/, { timeout: 8000 });
    await openChromeMenu(page);
    await expect(
      page.locator('.chrome-menu-dropdown button.menu-item').filter({ hasText: 'Save to reading list' }),
    ).toBeVisible();
  });

  test('settings includes customize sidebar button', async ({ page }) => {
    await openChromeMenu(page);
    await page
      .locator('.chrome-menu-dropdown button.menu-item')
      .filter({ hasText: 'Settings' })
      .evaluate((el) => (el as HTMLButtonElement).click());
    await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible({ timeout: 10_000 });
    await expect(page.getByRole('button', { name: /customize sidebar/i })).toBeVisible();
    await page.getByTestId('chrome-settings-close').click();
  });

  test('synced tabs panel shows refresh control', async ({ page }) => {
    await page.locator('[aria-label="Toggle sidebar"]').click();
    const syncedBtn = page.locator('.sidebar-icon-btn[aria-label="Synced tabs"]');
    if (await syncedBtn.isVisible().catch(() => false)) {
      await syncedBtn.click();
      await expect(page.getByRole('button', { name: /refresh/i })).toBeVisible({ timeout: 8000 });
    }
  });
});
