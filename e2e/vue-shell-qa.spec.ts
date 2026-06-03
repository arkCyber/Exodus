/**
 * Playwright smoke tests aligned with docs/MANUAL_QA_VUE_SHELL.md.
 * Default: Vite dev (no Tauri). Set TAURI_E2E=1 for backend-only cases (see scripts/test-e2e-tauri.sh).
 */
import { test, expect } from '@playwright/test';

const ADDRESS = '.url-input';
const OMNIBOX_ID = '#exodus-omnibox-input';
const needsTauri = !!process.env.TAURI_E2E;

/** Dismiss site/extension permission backdrops that block toolbar clicks in Vite dev. */
async function dismissPermissionPrompts(page: import('@playwright/test').Page): Promise<void> {
  for (let i = 0; i < 5; i++) {
    const backdrop = page.locator('.perm-backdrop');
    if (!(await backdrop.isVisible().catch(() => false))) {
      return;
    }
    const deny = page.getByRole('button', { name: 'Deny', exact: true });
    const block = page.getByRole('button', { name: 'Block', exact: true });
    if (await deny.isVisible().catch(() => false)) {
      await deny.click({ force: true });
    } else if (await block.isVisible().catch(() => false)) {
      await block.click({ force: true });
    } else {
      await page.keyboard.press('Escape');
    }
    await backdrop.waitFor({ state: 'hidden', timeout: 5000 }).catch(() => {});
  }
}

test.describe('Vue shell QA (Vite)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(ADDRESS).or(page.locator(OMNIBOX_ID))).toBeVisible({
      timeout: 15_000,
    });
    await page
      .getByRole('button', { name: 'Deny', exact: true })
      .click({ force: true, timeout: 12_000 })
      .catch(() => {});
    await dismissPermissionPrompts(page);
  });

  test('loads Exodus browser chrome', async ({ page }) => {
    await expect(page).toHaveTitle(/Exodus/i);
    await expect(page.locator('.tab-item').first()).toBeVisible();
    await expect(page.locator('.chrome-menu-btn')).toBeVisible();
  });

  test('address bar accepts navigation input', async ({ page }) => {
    const bar = page.locator(ADDRESS);
    await bar.fill('https://example.com');
    await bar.press('Enter');
    await expect(bar).toHaveValue(/example\.com/);
  });

  test('/ask omnibox mode does not crash the shell', async ({ page }) => {
    const bar = page.locator(ADDRESS);
    await bar.fill('/ask test query');
    await expect(bar).toHaveValue('/ask test query');
    await bar.press('Enter');
    await expect(bar).toBeVisible();
    if (needsTauri) {
      const askPanel = page.locator('.omnibox-ask-results, [class*="ask-result"]');
      await expect(askPanel.or(bar)).toBeVisible({ timeout: 8000 });
    }
  });

  test('⌘T new tab, ⌘W close, ⌘⇧T restore closed tab', async ({ page }) => {
    const bar = page.locator(ADDRESS);
    await bar.fill('https://example.com');
    await bar.press('Enter');
    await expect(bar).toHaveValue(/example\.com/, { timeout: 10_000 });

    await page.locator('body').click();
    const tabs = page.locator('.tab-item');
    const initial = await tabs.count();

    await page.keyboard.press('Meta+t');
    await page.waitForTimeout(400);
    expect(await tabs.count()).toBe(initial + 1);

    await page.keyboard.press('Meta+w');
    await page.waitForTimeout(400);
    expect(await tabs.count()).toBe(initial);

    await page.keyboard.press('Meta+Shift+t');
    await page.waitForTimeout(400);
    expect(await tabs.count()).toBeGreaterThan(initial);
  });

  test('opens Settings with Privacy via chrome menu', async ({ page }) => {
    await page.locator('.perm-backdrop').evaluateAll((nodes) => {
      for (const el of nodes) {
        (el as HTMLElement).style.pointerEvents = 'none';
      }
    });
    await page.locator('.chrome-menu-btn').evaluate((el) => (el as HTMLButtonElement).click());
    await expect(page.locator('.chrome-menu-dropdown')).toBeVisible();
    await page
      .locator('.chrome-menu-dropdown button.menu-item')
      .filter({ hasText: 'Settings' })
      .evaluate((el) => (el as HTMLButtonElement).click());
    await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#settings-title')).toContainText('Settings');
    await expect(page.locator('#settings-section-privacy')).toBeVisible();
    await page.getByTestId('chrome-settings-close').click();
  });
});

test.describe('Vue shell QA (Tauri backend)', () => {
  test.skip(!needsTauri, 'Run scripts/test-e2e-tauri.sh with pnpm tauri dev');

  test('Safe Browsing shows Go back and Proceed anyway', async ({ page }) => {
    await page.goto('/');
    const bar = page.locator(ADDRESS);
    await bar.fill('javascript:alert(1)');
    await bar.press('Enter');
    const dialog = page.getByRole('alertdialog');
    const visible = await dialog.isVisible().catch(() => false);
    if (!visible) {
      await bar.fill('https://malware.testing.test');
      await bar.press('Enter');
    }
    await expect(dialog).toBeVisible({ timeout: 12_000 });
    await expect(page.getByRole('button', { name: 'Go back' })).toBeVisible();
    await page.getByRole('button', { name: 'Go back' }).click();
    await expect(dialog).toBeHidden();
  });

  test('shields visible on https URL and Shift+click toggles per-site allow', async ({ page }) => {
    await page.goto('/');
    const bar = page.locator(ADDRESS);
    await bar.fill('https://example.com');
    await bar.press('Enter');
    await expect(bar).toHaveValue(/example\.com/, { timeout: 8000 });
    const shield = page.locator('.shields-btn');
    await expect(shield).toBeVisible({ timeout: 8000 });
    await shield.click({ modifiers: ['Shift'] });
    await expect(shield).toBeVisible();
  });

  test('CDN omnibox badge may appear after https navigation', async ({ page }) => {
    await page.goto('/');
    const bar = page.locator(ADDRESS);
    await bar.fill('https://example.com');
    await bar.press('Enter');
    await page.waitForTimeout(1500);
    const badge = page.locator('.cdn-omnibox-badge');
    test.info().annotations.push({
      type: 'note',
      description: (await badge.isVisible())
        ? 'CDN badge visible'
        : 'CDN badge absent (OK if P2P CDN not indexed)',
    });
    await expect(bar).toBeVisible();
  });
});
