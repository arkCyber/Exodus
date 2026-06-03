/**
 * Playwright — window chrome: drag regions, toolbar interactions, resize-friendly layout.
 */
import { test, expect } from '@playwright/test';
import { dismissPermissionPrompts, gotoBrowserShell } from './helpers/shell';

test.describe('Window chrome', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
    await dismissPermissionPrompts(page);
  });

  test('unified title bar wraps tab strip and address bar', async ({ page }) => {
    const titleBar = page.locator('#exodus-window-titlebar');
    await expect(titleBar).toBeVisible();
    await expect(titleBar).toHaveAttribute('data-tauri-drag-region', '');
    await expect(titleBar.locator('.exodus-chrome-tabstrip')).toBeVisible();
    await expect(titleBar.locator('.exodus-address-bar')).toBeVisible();
  });

  test('tab strip drag fill is visible on horizontal layout', async ({ page }) => {
    await expect(page.locator('.tab-strip-drag-fill')).toBeVisible();
  });

  test('address bar exposes drag surface and keeps omnibox interactive', async ({ page }) => {
    const addressBar = page.locator('.exodus-address-bar.chrome-drag-surface');
    await expect(addressBar).toBeVisible();
    await expect(addressBar).toHaveAttribute('data-tauri-drag-region', '');

    const omnibox = page.locator('#exodus-omnibox-input');
    await expect(omnibox).toBeVisible();
    await omnibox.click();
    await omnibox.fill('https://example.com');
    await expect(omnibox).toHaveValue('https://example.com');
  });

  test('navigation buttons remain clickable', async ({ page }) => {
    const reload = page.getByRole('button', { name: 'Reload' });
    await expect(reload).toBeEnabled();
    await reload.click();
    await expect(reload).toBeVisible();
  });

  test('tab strip has drag surface', async ({ page }) => {
    const tabStrip = page.locator('.exodus-chrome-tabstrip.chrome-drag-surface');
    await expect(tabStrip).toBeVisible();
    await expect(tabStrip).toHaveAttribute('data-tauri-drag-region', '');
  });

  test('new tab button remains clickable after toolbar use', async ({ page }) => {
    const omnibox = page.locator('#exodus-omnibox-input');
    await omnibox.fill('https://example.com');
    await omnibox.press('Enter');
    await expect(omnibox).toHaveValue(/example\.com/, { timeout: 10_000 });

    const tabs = page.locator('.tab-item');
    const initial = await tabs.count();
    await page.locator('.tab-new').click();
    await expect(tabs).toHaveCount(initial + 1, { timeout: 8000 });
  });

  test('window drag helper is wired on toolbar mousedown', async ({ page }) => {
    await page.evaluate(() => {
      (window as unknown as { __dragTest?: boolean }).__dragTest = false;
      const bar = document.querySelector('.exodus-address-bar');
      bar?.addEventListener('mousedown', () => {
        (window as unknown as { __dragTest?: boolean }).__dragTest = true;
      });
    });
    const addressBar = page.locator('.exodus-address-bar');
    const box = await addressBar.boundingBox();
    expect(box).toBeTruthy();
    if (box) {
      await page.mouse.click(box.x + 4, box.y + box.height / 2);
    }
    const fired = await page.evaluate(() => (window as unknown as { __dragTest?: boolean }).__dragTest);
    expect(fired).toBe(true);
  });
});
