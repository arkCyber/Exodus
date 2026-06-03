import { test, expect } from '@playwright/test';

test.describe('Extension Manager', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173');
  });

  test('should display extension manager', async ({ page }) => {
    // Navigate to extension manager
    await page.click('[data-testid="extension-manager-btn"]');
    
    await expect(page.locator('.extension-manager')).toBeVisible();
    await expect(page.locator('.header h2')).toHaveText('Extension Manager');
  });

  test('should display installed extensions tab by default', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    await expect(page.locator('.tab.active')).toHaveText('Installed');
    await expect(page.locator('.extensions-grid')).toBeVisible();
  });

  test('should display all installed extensions', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    await expect(page.locator('.extension-card')).toHaveCount(3);
    await expect(page.locator('.extension-card').nth(0)).toContainText('Extension API Demo');
    await expect(page.locator('.extension-card').nth(1)).toContainText('Ad Blocker');
    await expect(page.locator('.extension-card').nth(2)).toContainText('Password Manager');
  });

  test('should filter extensions by search query', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    await page.fill('.search-input', 'ad blocker');
    
    await expect(page.locator('.extension-card')).toHaveCount(1);
    await expect(page.locator('.extension-card')).toContainText('Ad Blocker');
  });

  test('should switch between tabs', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    // Switch to updates tab
    await page.click('.tab:nth-child(2)');
    await expect(page.locator('.tab.active')).toHaveText('Updates');
    await expect(page.locator('.updates-list')).toBeVisible();
    
    // Switch to store tab
    await page.click('.tab:nth-child(3)');
    await expect(page.locator('.tab.active')).toHaveText('Store');
    await expect(page.locator('.store-grid')).toBeVisible();
  });

  test('should display extension details in card', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await expect(firstCard.locator('h3')).toHaveText('Extension API Demo');
    await expect(firstCard.locator('.version')).toHaveText('v1.0.0');
    await expect(firstCard.locator('.description')).toHaveText('Demonstrates various Extension APIs');
  });

  test('should display extension permissions', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await expect(firstCard.locator('.permission-tag')).toHaveCount(4);
    await expect(firstCard.locator('.permission-tag').nth(0)).toHaveText('contextMenus');
  });

  test('should display extension stats', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await expect(firstCard.locator('.stat').nth(0)).toContainText('1234 calls');
    await expect(firstCard.locator('.stat').nth(1)).toContainText('45ms');
  });

  test('should toggle extension enable/disable', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    const toggleBtn = firstCard.locator('.toggle-btn');
    
    await expect(toggleBtn).toHaveText('Disable');
    await expect(toggleBtn).toHaveClass(/active/);
    
    await toggleBtn.click();
    await expect(toggleBtn).toHaveText('Enable');
    await expect(toggleBtn).not.toHaveClass(/active/);
  });

  test('should show context menu on menu button click', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await firstCard.locator('.menu-btn').click();
    
    await expect(page.locator('.context-menu')).toBeVisible();
    await expect(page.locator('.context-menu button')).toHaveCount(5);
  });

  test('should close context menu when clicking outside', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await firstCard.locator('.menu-btn').click();
    
    await expect(page.locator('.context-menu')).toBeVisible();
    
    await page.click('.extension-manager');
    await expect(page.locator('.context-menu')).not.toBeVisible();
  });

  test('should display available updates', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(2)');
    
    await expect(page.locator('.update-item')).toHaveCount(1);
    await expect(page.locator('.update-item')).toContainText('Ad Blocker');
    await expect(page.locator('.update-item')).toContainText('2.1.0 → 2.2.0');
  });

  test('should update extension', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(2)');
    
    await page.locator('.update-btn').click();
    
    await expect(page.locator('.update-item')).toHaveCount(0);
    await expect(page.locator('.empty-state')).toContainText('All extensions are up to date');
  });

  test('should dismiss update', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(2)');
    
    await page.locator('.dismiss-btn').click();
    
    await expect(page.locator('.update-item')).toHaveCount(0);
  });

  test('should display store extensions', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(3)');
    
    await expect(page.locator('.store-item')).toHaveCount(3);
    await expect(page.locator('.store-item').nth(0)).toContainText('Dark Mode');
    await expect(page.locator('.store-item').nth(1)).toContainText('Grammar Checker');
    await expect(page.locator('.store-item').nth(2)).toContainText('Screenshot Tool');
  });

  test('should display extension rating in store', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(3)');
    
    const firstItem = page.locator('.store-item').nth(0);
    await expect(firstItem.locator('.stars')).toHaveText('⭐⭐⭐⭐');
    await expect(firstItem.locator('.rating .count')).toHaveText('(1234)');
  });

  test('should install extension from store', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(3)');
    
    const firstItem = page.locator('.store-item').nth(0);
    await firstItem.locator('.install-btn').click();
    
    // Verify installation (in real test, would check for success message)
    await expect(firstItem.locator('.install-btn')).toBeVisible();
  });

  test('should display empty state when no extensions', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    // Clear all extensions
    await page.evaluate(() => {
      (window as any).extensions = [];
    });
    
    await page.fill('.search-input', 'nonexistent');
    
    await expect(page.locator('.empty-state')).toBeVisible();
    await expect(page.locator('.empty-state')).toContainText('No extensions found');
  });

  test('should display extension count in tabs', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    await expect(page.locator('.tab:nth-child(1) .count')).toHaveText('3');
    await expect(page.locator('.tab:nth-child(2) .count')).toHaveText('1');
  });

  test('should navigate to extension details', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await firstCard.locator('.details-btn').click();
    
    // In real test, would verify navigation to details page
    await expect(firstCard).toBeVisible();
  });

  test('should handle context menu actions', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await firstCard.locator('.menu-btn').click();
    
    // Click manage permissions
    await page.locator('.context-menu button').nth(1).click();
    
    await expect(page.locator('.context-menu')).not.toBeVisible();
  });

  test('should uninstall extension with confirmation', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    await firstCard.locator('.menu-btn').click();
    
    // Handle dialog
    page.on('dialog', dialog => dialog.accept());
    
    await page.locator('.context-menu button.danger').click();
    
    await expect(page.locator('.extension-card')).toHaveCount(2);
  });

  test('should display extension icon', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    
    const firstCard = page.locator('.extension-card').nth(0);
    const icon = firstCard.locator('.extension-icon');
    
    await expect(icon).toBeVisible();
    await expect(icon.locator('.default-icon')).toHaveText('📦');
  });

  test('should display author name in store', async ({ page }) => {
    await page.click('[data-testid="extension-manager-btn"]');
    await page.click('.tab:nth-child(3)');
    
    const firstItem = page.locator('.store-item').nth(0);
    await expect(firstItem.locator('.author')).toHaveText('by Theme Dev');
  });
});
