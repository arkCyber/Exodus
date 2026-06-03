import { test, expect } from '@playwright/test';

test.describe('Extension Permission Request', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173');
  });

  test('should display permission request dialog', async ({ page }) => {
    // Trigger permission request
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension',
            icon: 'https://example.com/icon.png'
          },
          requestedPermissions: [
            { id: 'storage', name: 'Storage', description: 'Store extension data', status: 'pending' },
            { id: 'tabs', name: 'Tabs', description: 'Access browser tabs', status: 'pending' }
          ],
          hostPermissions: ['https://*.example.com/*']
        }
      }));
    });

    // Wait for dialog to appear
    await expect(page.locator('.extension-permission-request')).toBeVisible();
    
    // Verify extension info
    await expect(page.locator('.extension-info h3')).toHaveText('Test Extension');
    await expect(page.locator('.version')).toHaveText('Version 1.0.0');
  });

  test('should display all requested permissions', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [
            { id: 'storage', name: 'Storage', description: 'Store extension data', status: 'pending' },
            { id: 'tabs', name: 'Tabs', description: 'Access browser tabs', status: 'pending' },
            { id: 'bookmarks', name: 'Bookmarks', description: 'Access bookmarks', status: 'pending' }
          ],
          hostPermissions: []
        }
      }));
    });

    await expect(page.locator('.permission-item')).toHaveCount(3);
    await expect(page.locator('.permission-item').nth(0)).toContainText('Storage');
    await expect(page.locator('.permission-item').nth(1)).toContainText('Tabs');
    await expect(page.locator('.permission-item').nth(2)).toContainText('Bookmarks');
  });

  test('should display host permissions', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [],
          hostPermissions: ['https://*.example.com/*', 'https://*.google.com/*']
        }
      }));
    });

    await expect(page.locator('.host-permissions-section')).toBeVisible();
    await expect(page.locator('.host-permission-item')).toHaveCount(2);
  });

  test('should show warning for dangerous permissions', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [
            { id: 'tabs', name: 'Tabs', description: 'Access browser tabs', status: 'pending' },
            { id: 'bookmarks', name: 'Bookmarks', description: 'Access bookmarks', status: 'pending' }
          ],
          hostPermissions: []
        }
      }));
    });

    await expect(page.locator('.warning-section')).toBeVisible();
    await expect(page.locator('.warning-box')).toContainText('dangerous');
  });

  test('should auto-approve safe permissions', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [
            { id: 'storage', name: 'Storage', description: 'Store extension data', status: 'pending' },
            { id: 'contextMenus', name: 'Context Menus', description: 'Add context menus', status: 'pending' }
          ],
          hostPermissions: []
        }
      }));
    });

    // Wait for auto-approval
    await page.waitForTimeout(100);

    const storageStatus = page.locator('.permission-item').nth(0).locator('.status');
    await expect(storageStatus).toHaveText('approved');
  });

  test('should close dialog when clicking close button', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [],
          hostPermissions: []
        }
      }));
    });

    await page.locator('.close-btn').click();
    await expect(page.locator('.extension-permission-request')).not.toBeVisible();
  });

  test('should emit approve event when clicking approve', async ({ page }) => {
    let approvedPermissions: string[] = [];
    
    await page.exposeFunction('handleApprove', (permissions: string[]) => {
      approvedPermissions = permissions;
    });

    await page.evaluate(() => {
      window.addEventListener('approve', (e: any) => {
        (window as any).handleApprove(e.detail);
      });
    });

    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [
            { id: 'storage', name: 'Storage', description: 'Store extension data', status: 'approved' },
            { id: 'tabs', name: 'Tabs', description: 'Access browser tabs', status: 'pending' }
          ],
          hostPermissions: []
        }
      }));
    });

    await page.locator('.btn-primary').click();
    
    // Wait for event to be emitted
    await page.waitForTimeout(100);
    
    expect(approvedPermissions).toContain('storage');
    expect(approvedPermissions).toContain('tabs');
  });

  test('should emit deny event when clicking deny all', async ({ page }) => {
    let deniedPermissions: string[] = [];
    
    await page.exposeFunction('handleDeny', (permissions: string[]) => {
      deniedPermissions = permissions;
    });

    await page.evaluate(() => {
      window.addEventListener('deny', (e: any) => {
        (window as any).handleDeny(e.detail);
      });
    });

    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [
            { id: 'storage', name: 'Storage', description: 'Store extension data', status: 'pending' },
            { id: 'tabs', name: 'Tabs', description: 'Access browser tabs', status: 'pending' }
          ],
          hostPermissions: []
        }
      }));
    });

    await page.locator('.btn-secondary').click();
    
    // Wait for event to be emitted
    await page.waitForTimeout(100);
    
    expect(deniedPermissions).toContain('storage');
    expect(deniedPermissions).toContain('tabs');
  });

  test('should display permission icons correctly', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [
            { id: 'storage', name: 'Storage', description: 'Store extension data', status: 'pending' },
            { id: 'tabs', name: 'Tabs', description: 'Access browser tabs', status: 'pending' }
          ],
          hostPermissions: []
        }
      }));
    });

    const storageIcon = page.locator('.permission-item').nth(0).locator('.icon');
    await expect(storageIcon).toHaveText('💾');
    
    const tabsIcon = page.locator('.permission-item').nth(1).locator('.icon');
    await expect(tabsIcon).toHaveText('📑');
  });

  test('should handle empty permissions gracefully', async ({ page }) => {
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('showPermissionRequest', {
        detail: {
          extension: {
            id: 'test-extension',
            name: 'Test Extension',
            version: '1.0.0',
            description: 'A test extension'
          },
          requestedPermissions: [],
          hostPermissions: []
        }
      }));
    });

    await expect(page.locator('.extension-permission-request')).toBeVisible();
    await expect(page.locator('.permissions-list')).toHaveCount(0);
  });
});
