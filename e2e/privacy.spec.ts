/**
 * Playwright — privacy settings and status bar badges.
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

async function openSettings(page: import('@playwright/test').Page): Promise<void> {
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
  await expect(page.getByTestId('chrome-settings-page')).toBeVisible({ timeout: 15_000 });
}

test.describe('Privacy Features E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
    await page
      .getByRole('button', { name: 'Deny', exact: true })
      .click({ force: true, timeout: 12_000 })
      .catch(() => {});
    await dismissPermissionPrompts(page);
  });

  test('enables private mode and shows status badge', async ({ page }) => {
    await openSettings(page);
    await page.getByTestId('chrome-settings-nav-privacy').click();
    const privateToggle = page
      .locator('[data-testid="settings-section-privacy"] label')
      .filter({ hasText: 'Private mode' })
      .locator('input');
    await privateToggle.check();
    await expect(page.getByTestId('chrome-settings-autosave-status')).toContainText(/Saved|已保存/, {
      timeout: 8000,
    });
    await expect(page.locator('.badge-private')).toBeVisible({ timeout: 8000 });
  });

  test('enables HTTPS-only mode in settings', async ({ page }) => {
    await openSettings(page);
    await page.getByTestId('chrome-settings-nav-privacy').click();
    const httpsToggle = page
      .locator('[data-testid="settings-section-privacy"] label')
      .filter({ hasText: 'HTTPS-only mode' })
      .locator('input');
    await expect(httpsToggle).toBeVisible();
    await httpsToggle.check();
    await expect(httpsToggle).toBeChecked();
  });

  test('toggles popup blocker in settings', async ({ page }) => {
    await openSettings(page);
    await page.getByTestId('chrome-settings-nav-privacy').click();
    const popupToggle = page
      .locator('[data-testid="settings-section-privacy"] label')
      .filter({ hasText: 'Block popups' })
      .locator('input');
    await expect(popupToggle).toBeVisible();
    const wasChecked = await popupToggle.isChecked();
    if (wasChecked) {
      await popupToggle.uncheck();
      await expect(popupToggle).not.toBeChecked();
      await popupToggle.check();
    } else {
      await popupToggle.check();
    }
    await expect(popupToggle).toBeChecked();
  });

  test('shows shields button after https navigation', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await expect(addressBar).toHaveValue(/example\.com/, { timeout: 8000 });
    await expect(page.locator('.shields-btn')).toBeVisible({ timeout: 8000 });
  });
});
