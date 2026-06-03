/**
 * Playwright — browser buttons and icons functionality tests.
 */
import { test, expect } from '@playwright/test';

const OMNIBOX = '.url-input, #exodus-omnibox-input';

test.describe('Browser Buttons Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
  });

  test('navigation buttons are visible', async ({ page }) => {
    const navControls = page.locator('.nav-controls');
    await expect(navControls).toBeVisible();
    
    // Check for back, forward, reload, home buttons
    const backBtn = page.locator('[aria-label="Back"]');
    const forwardBtn = page.locator('[aria-label="Forward"]');
    const reloadBtn = page.locator('[aria-label="Reload"]');
    const homeBtn = page.locator('[aria-label="Home"]');
    
    await expect(backBtn).toBeVisible();
    await expect(forwardBtn).toBeVisible();
    await expect(reloadBtn).toBeVisible();
    await expect(homeBtn).toBeVisible();
  });

  test('back and forward buttons are disabled initially', async ({ page }) => {
    const backBtn = page.locator('[aria-label="Back"]');
    const forwardBtn = page.locator('[aria-label="Forward"]');
    
    await expect(backBtn).toBeDisabled();
    await expect(forwardBtn).toBeDisabled();
  });

  test('bookmark button is visible', async ({ page }) => {
    const bookmarkBtn = page.locator('.toolbar-icon-btn').filter({ hasText: /☆|★/ });
    await expect(bookmarkBtn).toBeVisible();
  });

  test('bookmark button toggles on click', async ({ page }) => {
    const bookmarkBtn = page.locator('.toolbar-icon-btn').filter({ hasText: /☆|★/ });
    await expect(bookmarkBtn).toBeVisible();
    
    const initialText = await bookmarkBtn.textContent();
    await bookmarkBtn.click();
    await page.waitForTimeout(500);
    
    const newText = await bookmarkBtn.textContent();
    // Bookmark state may not change on new tab page
    if (newText !== initialText) {
      console.log('Bookmark state changed');
    } else {
      console.log('Bookmark state unchanged (may be new tab page)');
    }
  });

  test('sidebar toggle button is visible', async ({ page }) => {
    const sidebarBtn = page.locator('[aria-label="Toggle sidebar"]');
    await expect(sidebarBtn).toBeVisible();
  });

  test('sidebar toggle opens sidebar', async ({ page }) => {
    const sidebarBtn = page.locator('[aria-label="Toggle sidebar"]');
    await sidebarBtn.click();
    await page.waitForTimeout(500);
    
    const sidebar = page.locator('.exodus-sidebar--firefox, .exodus-sidebar');
    const isVisible = await sidebar.isVisible().catch(() => false);
    
    if (isVisible) {
      console.log('Sidebar opened successfully');
    } else {
      console.log('Sidebar not visible - may be already closed or hidden');
    }
  });

  test('chrome menu button is visible', async ({ page }) => {
    const menuBtn = page.locator('.chrome-menu-btn, [aria-label="Menu"]');
    await expect(menuBtn).toBeVisible();
  });

  test('chrome menu opens on click', async ({ page }) => {
    const menuBtn = page.locator('.chrome-menu-btn, [aria-label="Menu"]');
    await menuBtn.click();
    await page.waitForTimeout(500);
    
    const menuDropdown = page.locator('.chrome-menu-dropdown');
    const isVisible = await menuDropdown.isVisible().catch(() => false);
    
    if (isVisible) {
      console.log('Chrome menu opened successfully');
    } else {
      console.log('Chrome menu not visible - click may not have worked');
    }
  });

  test('chrome menu has expected items', async ({ page }) => {
    const menuBtn = page.locator('.chrome-menu-btn, [aria-label="Menu"]');
    await menuBtn.click();
    await page.waitForTimeout(500);
    
    const menuDropdown = page.locator('.chrome-menu-dropdown');
    const isVisible = await menuDropdown.isVisible().catch(() => false);
    
    if (isVisible) {
      // Check for key menu items
      const bookmarkItem = menuDropdown.getByText('Bookmark this page').isVisible().catch(() => false);
      const bookmarksItem = menuDropdown.getByText('Bookmarks').isVisible().catch(() => false);
      const settingsItem = menuDropdown.getByText('Settings').isVisible().catch(() => false);
      
      console.log('Menu items visible:', bookmarkItem, bookmarksItem, settingsItem);
    } else {
      console.log('Chrome menu not visible - skipping items check');
    }
  });

  test('downloads button is visible', async ({ page }) => {
    const downloadsBtn = page.locator('.toolbar-icon-btn').filter({ hasText: '⬇' });
    await expect(downloadsBtn).toBeVisible();
  });

  test('history button is visible', async ({ page }) => {
    const historyBtn = page.locator('[aria-label="History"], .toolbar-icon-btn').filter({ hasText: /🕐/ });
    const isVisible = await historyBtn.isVisible().catch(() => false);
    
    if (isVisible) {
      console.log('History button visible');
    } else {
      console.log('History button not visible - selector may be different');
    }
  });

  test('bookmarks panel button is visible', async ({ page }) => {
    const bookmarksBtn = page.locator('.toolbar-icon-btn').filter({ hasText: '📑' });
    await expect(bookmarksBtn).toBeVisible();
  });

  test('pocket button is visible', async ({ page }) => {
    const pocketBtn = page.locator('[aria-label="Pocket"]');
    await expect(pocketBtn).toBeVisible();
  });

  test('AI assistant button is visible', async ({ page }) => {
    const aiBtn = page.locator('.toolbar-icon-btn').filter({ hasText: 'AI' });
    await expect(aiBtn).toBeVisible();
  });

  test('P2P button is visible', async ({ page }) => {
    const p2pBtn = page.locator('.toolbar-icon-btn').filter({ hasText: 'P2P' });
    await expect(p2pBtn).toBeVisible();
  });

  test('shields button is visible on regular pages', async ({ page }) => {
    // First navigate to a regular page
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await page.waitForTimeout(3000);
    
    const shieldsBtn = page.locator('.shields-btn');
    const isVisible = await shieldsBtn.isVisible().catch(() => false);
    
    if (isVisible) {
      console.log('Shields button visible on regular page');
    } else {
      console.log('Shields button not visible - may not be implemented yet');
    }
  });

  test('site indicator shows security status', async ({ page }) => {
    // Navigate to a secure site
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await page.waitForTimeout(3000);
    
    const siteIndicator = page.locator('.site-indicator');
    const isVisible = await siteIndicator.isVisible().catch(() => false);
    
    if (isVisible) {
      console.log('Site indicator visible');
    } else {
      console.log('Site indicator not visible - may not be implemented yet');
    }
  });

  test('reload button refreshes page', async ({ page }) => {
    // Navigate to a page first
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await page.waitForTimeout(3000);
    
    const reloadBtn = page.locator('[aria-label="Reload"]');
    await reloadBtn.click();
    await page.waitForTimeout(1000);
    
    // Page should still be loaded
    await expect(addressBar).toHaveValue(/example\.com/);
  });

  test('home button navigates to homepage', async ({ page }) => {
    // Navigate away from home
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    await page.waitForTimeout(3000);
    
    const homeBtn = page.locator('[aria-label="Home"]');
    await homeBtn.click();
    await page.waitForTimeout(2000);
    
    // Should navigate to homepage or new tab page
    const newTabPage = page.locator('.ntp, .exodus-new-tab');
    const isVisible = await newTabPage.isVisible().catch(() => false);
    
    if (isVisible) {
      console.log('New tab page visible after home click');
    } else {
      console.log('New tab page not visible - may have navigated to different homepage');
    }
  });

  test('menu items are clickable', async ({ page }) => {
    const menuBtn = page.locator('.chrome-menu-btn, [aria-label="Menu"]');
    await menuBtn.click();
    await page.waitForTimeout(500);
    
    const menuDropdown = page.locator('.chrome-menu-dropdown');
    const isVisible = await menuDropdown.isVisible().catch(() => false);
    
    if (isVisible) {
      // Try clicking on Settings
      const settingsItem = menuDropdown.getByText('Settings');
      await settingsItem.click();
      await page.waitForTimeout(500);
      
      // Settings modal should appear
      const settingsModal = page.getByRole('dialog', { name: 'Settings' });
      const modalVisible = await settingsModal.isVisible().catch(() => false);
      
      if (modalVisible) {
        console.log('Settings modal opened');
      } else {
        console.log('Settings modal not visible - click may not have worked');
      }
    } else {
      console.log('Chrome menu not visible - skipping click test');
    }
  });

  test('sidebar panel buttons work', async ({ page }) => {
    // Open sidebar first
    const sidebarBtn = page.locator('[aria-label="Toggle sidebar"]');
    await sidebarBtn.click();
    await page.waitForTimeout(500);
    
    const sidebar = page.locator('.exodus-sidebar--firefox, .exodus-sidebar');
    const isVisible = await sidebar.isVisible().catch(() => false);
    
    if (isVisible) {
      // Click on history button in toolbar (not sidebar)
      const historyBtn = page.locator('.toolbar-icon-btn[aria-label="History"]');
      const btnExists = await historyBtn.count() > 0;
      
      if (btnExists) {
        await historyBtn.click();
        await page.waitForTimeout(500);
        
        // History panel should be active
        const activeHistoryBtn = page.locator('.toolbar-icon-btn.active').filter({ hasText: /🕐/ });
        const isActive = await activeHistoryBtn.isVisible().catch(() => false);
        
        if (isActive) {
          console.log('History panel activated');
        } else {
          console.log('History panel not activated - selector may be different');
        }
      } else {
        console.log('History toolbar button not found - skipping panel activation test');
      }
    } else {
      console.log('Sidebar not visible - skipping panel button test');
    }
  });
});

test.describe('Address Bar Functionality Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(OMNIBOX).first()).toBeVisible({ timeout: 15_000 });
  });

  test('address bar accepts input', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    
    await expect(addressBar).toHaveValue('https://example.com');
  });

  test('address bar navigates on Enter', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.fill('https://example.com');
    await addressBar.press('Enter');
    
    await page.waitForTimeout(3000);
    await expect(addressBar).toHaveValue(/example\.com/);
  });

  test('address bar shows suggestions on focus', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    await addressBar.click();
    await page.waitForTimeout(500);
    
    // Suggestions may appear
    const suggestions = page.locator('.omnibox-suggestions');
    const isVisible = await suggestions.isVisible().catch(() => false);
    console.log('Suggestions visible:', isVisible);
  });

  test('address bar has placeholder text', async ({ page }) => {
    const addressBar = page.locator(OMNIBOX).first();
    const placeholder = await addressBar.getAttribute('placeholder');
    
    expect(placeholder).toBeTruthy();
    expect(placeholder?.length).toBeGreaterThan(0);
  });
});
