/**
 * Playwright — WebChat full-view (main content area) smoke tests.
 */
import { test, expect } from '@playwright/test';
import { dismissPermissionPrompts, gotoBrowserShell } from './helpers/shell';

test.describe('WebChat full view', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
    await dismissPermissionPrompts(page);
  });

  test('opens WebChat in main content area from address bar', async ({ page }) => {
    await expect(page.locator('.webchat-toggle-btn')).toBeVisible();
    await page.locator('.webchat-toggle-btn').click();

    const mainWebChat = page.locator('.browser-content .webchat-main-view.im-messenger.full-width');
    await expect(mainWebChat).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.webchat-toggle-btn.active')).toBeVisible();
    await expect(page.locator('.browser-content .exodus-new-tab')).not.toBeVisible();
  });

  test('toggles back to browser from WebChat', async ({ page }) => {
    await page.locator('.webchat-toggle-btn').click();
    await expect(page.locator('.browser-content .webchat-main-view')).toBeVisible({ timeout: 10_000 });

    await page.locator('.webchat-toggle-btn').click();
    await expect(page.locator('.browser-content .webchat-main-view')).not.toBeVisible();
    await expect(page.locator('.browser-content .exodus-new-tab')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.webchat-toggle-btn.active')).not.toBeVisible();
  });

  test('WebChat uses WeChat desktop three-column layout', async ({ page }) => {
    await page.locator('.webchat-toggle-btn').click();
    const root = page.locator('.browser-content .im-messenger.wechat-desktop.full-width');
    await expect(root).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.wechat-list-toolbar')).toBeVisible();
    await expect(page.locator('.nav-sidebar')).toBeVisible();
    await expect(page.locator('.content-sidebar')).toBeVisible();
  });

  test('WebChat has collections and starred navigation', async ({ page }) => {
    await page.locator('.webchat-toggle-btn').click();
    await expect(page.locator('.browser-content .im-messenger')).toBeVisible({ timeout: 10_000 });

    await expect(page.locator('.nav-item[title="收藏"]')).toBeVisible();
    await expect(page.locator('.nav-item[title="星标"]')).toBeVisible();
  });

  test('P2P sidebar still exposes WebChat tab when opened separately', async ({ page }) => {
    const p2pBtn = page.locator('.toolbar-icon-btn').filter({ hasText: 'P2P' });
    await p2pBtn.click();
    await expect(page.locator('.p2p-sidebar .sub-tabs')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('.p2p-sidebar').getByRole('tab', { name: /webchat/i })).toBeVisible();
  });
});
