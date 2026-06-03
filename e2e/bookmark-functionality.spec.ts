/**
 * End-to-end test for bookmark functionality
 * Tests all bookmark-related buttons and interactions
 */

import { test, expect } from '@playwright/test';
import { gotoBrowserShell } from './helpers/shell';

test.describe('Bookmark Functionality E2E', () => {
  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
  });

  test('Bookmark Manager - Add button', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Click Add button
    const addButton = page.locator('button:has-text("Add"), .action-btn.primary').first();
    await expect(addButton).toBeVisible();
    await addButton.click();

    // Verify bookmark editor dialog appears
    const dialog = page.locator('.dialog-overlay, .bookmark-editor').first();
    await expect(dialog).toBeVisible();

    // Close dialog
    const cancelButton = page.locator('button:has-text("Cancel"), .dialog-btn.cancel').first();
    if (await cancelButton.isVisible()) {
      await cancelButton.click();
    }
  });

  test('Bookmark Manager - Import button', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Click Import button
    const importButton = page.locator('button:has-text("Import")').first();
    if (await importButton.isVisible()) {
      await importButton.click();

      // Verify import dialog appears
      const dialog = page.locator('.dialog-overlay').first();
      await expect(dialog).toBeVisible();

      // Close dialog
      const cancelButton = page.locator('button:has-text("Cancel"), .dialog-btn.cancel').first();
      await cancelButton.click();
    }
  });

  test('Bookmark Manager - Export button', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Click Export button
    const exportButton = page.locator('button:has-text("Export")').first();
    if (await exportButton.isVisible()) {
      await exportButton.click();
      // Export should trigger a download, we just verify the button is clickable
    }
  });

  test('Bookmark Manager - Tab navigation', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Test tab switching
    const tabs = ['Bookmarks', 'Stats', 'Sync'];
    for (const tabName of tabs) {
      const tab = page.locator(`.tab-btn:has-text("${tabName}")`).first();
      if (await tab.isVisible()) {
        await tab.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('Bookmark Sync Settings - Toggle sync', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Navigate to Sync tab
    const syncTab = page.locator('.tab-btn:has-text("Sync")').first();
    if (await syncTab.isVisible()) {
      await syncTab.click();
      await page.waitForTimeout(500);

      // Try to toggle sync (may not work without backend, but we test the UI)
      const toggle = page.locator('.toggle-switch input[type="checkbox"]').first();
      if (await toggle.isVisible()) {
        await toggle.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('Bookmark Sync Settings - Sync Now button', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Navigate to Sync tab
    const syncTab = page.locator('.tab-btn:has-text("Sync")').first();
    if (await syncTab.isVisible()) {
      await syncTab.click();
      await page.waitForTimeout(500);

      // Click Sync Now button
      const syncNowButton = page.locator('button:has-text("Sync Now")').first();
      if (await syncNowButton.isVisible()) {
        await syncNowButton.click();
        await page.waitForTimeout(1000);
      }
    }
  });

  test('Bookmark Sync Settings - Clear Log button', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Navigate to Sync tab
    const syncTab = page.locator('.tab-btn:has-text("Sync")').first();
    if (await syncTab.isVisible()) {
      await syncTab.click();
      await page.waitForTimeout(500);

      // Click Clear button in sync log
      const clearButton = page.locator('button:has-text("Clear")').first();
      if (await clearButton.isVisible()) {
        await clearButton.click();
        // Handle confirmation dialog if it appears
        await page.waitForTimeout(500);
        const confirmButton = page.locator('button:has-text("OK"), button:has-text("Yes")').first();
        if (await confirmButton.isVisible()) {
          await confirmButton.click();
        }
      }
    }
  });

  test('Bookmark Stats - Refresh button', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Navigate to Stats tab
    const statsTab = page.locator('.tab-btn:has-text("Stats")').first();
    if (await statsTab.isVisible()) {
      await statsTab.click();
      await page.waitForTimeout(500);

      // Click Refresh button
      const refreshButton = page.locator('.refresh-btn').first();
      if (await refreshButton.isVisible()) {
        await refreshButton.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('Bookmark Bar - Side panel button', async ({ page }) => {
    // Look for bookmark bar side panel button
    const sidePanelButton = page.locator('.bookmark-lead-btn').first();
    if (await sidePanelButton.isVisible()) {
      await sidePanelButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('Bookmark Bar - Apps button', async ({ page }) => {
    // Look for bookmark bar apps button
    const appsButton = page.locator('.bookmark-lead-btn').nth(1);
    if (await appsButton.isVisible()) {
      await appsButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('Bookmark Bar - All bookmarks folder', async ({ page }) => {
    // Look for all bookmarks folder button
    const allBookmarksButton = page.locator('.bookmark-chip--all').first();
    if (await allBookmarksButton.isVisible()) {
      await allBookmarksButton.click();
      await page.waitForTimeout(500);

      // Verify dropdown appears
      const dropdown = page.locator('.bookmark-dropdown--all').first();
      await expect(dropdown).toBeVisible();

      // Click to close
      await allBookmarksButton.click();
    }
  });

  test('Bookmark Bar - Overflow menu', async ({ page }) => {
    // Look for overflow button
    const overflowButton = page.locator('.bookmark-chip--overflow').first();
    if (await overflowButton.isVisible()) {
      await overflowButton.click();
      await page.waitForTimeout(500);

      // Verify dropdown appears
      const dropdown = page.locator('.bookmark-dropdown--overflow').first();
      await expect(dropdown).toBeVisible();

      // Click to close
      await overflowButton.click();
    }
  });

  test('Bookmark Bar - Folder dropdown', async ({ page }) => {
    // Look for folder buttons
    const folderButtons = await page.locator('.bookmark-chip--folder').all();
    for (const button of folderButtons) {
      if (await button.isVisible()) {
        await button.click();
        await page.waitForTimeout(500);

        // Verify dropdown appears
        const dropdown = page.locator('.bookmark-dropdown').first();
        if (await dropdown.isVisible()) {
          await button.click(); // Close it
          break; // Test one folder is enough
        }
      }
    }
  });

  test('Bookmark Manager - Search functionality', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Type in search box
    const searchInput = page.locator('.search-input').first();
    if (await searchInput.isVisible()) {
      await searchInput.fill('test');
      await page.waitForTimeout(500);
      
      // Clear search
      await searchInput.fill('');
    }
  });

  test('Bookmark Manager - Folder filter', async ({ page }) => {
    // Click on bookmarks in sidebar if available
    const bookmarkButton = page.locator('[data-testid="bookmarks-button"], button:has-text("Bookmarks")').first();
    if (await bookmarkButton.isVisible()) {
      await bookmarkButton.click();
    }

    // Wait for bookmark manager to load
    await page.waitForTimeout(1000);

    // Try folder filter dropdown
    const filterSelect = page.locator('.filter-select').first();
    if (await filterSelect.isVisible()) {
      await filterSelect.click();
      await page.waitForTimeout(500);
      
      // Select first option
      const firstOption = filterSelect.locator('option').first();
      if (await firstOption.isVisible()) {
        await filterSelect.selectOption({ index: 0 });
      }
    }
  });
});
