/**
 * Exodus Browser — GPU settings E2E tests.
 */
import { test, expect } from '@playwright/test';

test.describe('GPU Settings E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to settings
    await page.goto('chrome://settings/gpu');
    await page.waitForLoadState('networkidle');
  });

  test('GPU settings page loads and displays GPU information', async ({ page }) => {
    // Check that the GPU settings section is visible
    const gpuSection = page.locator('[data-testid="settings-section-gpu"]');
    await expect(gpuSection).toBeVisible();

    // Check for GPU information card
    const gpuInfo = page.locator('.gpu-info-card');
    await expect(gpuInfo).toBeVisible();

    // Check for GPU acceleration toggle
    const gpuAccelerationToggle = page.locator('[data-testid="gpu-acceleration-enabled"]');
    await expect(gpuAccelerationToggle).toBeVisible();
  });

  test('can toggle GPU acceleration', async ({ page }) => {
    const gpuAccelerationToggle = page.locator('[data-testid="gpu-acceleration-enabled"]');
    
    // Get initial state
    const initialState = await gpuAccelerationToggle.isChecked();
    
    // Toggle
    await gpuAccelerationToggle.click();
    
    // Wait for the change to be applied
    await page.waitForTimeout(500);
    
    // Verify the toggle changed
    const newState = await gpuAccelerationToggle.isChecked();
    expect(newState).toBe(!initialState);
  });

  test('can toggle WebGL', async ({ page }) => {
    const webglToggle = page.locator('[data-testid="webgl-enabled"]');
    
    // Get initial state
    const initialState = await webglToggle.isChecked();
    
    // Toggle
    await webglToggle.click();
    
    // Wait for the change to be applied
    await page.waitForTimeout(500);
    
    // Verify the toggle changed
    const newState = await webglToggle.isChecked();
    expect(newState).toBe(!initialState);
  });

  test('can toggle WebGPU', async ({ page }) => {
    const webgpuToggle = page.locator('[data-testid="webgpu-enabled"]');
    
    // Get initial state
    const initialState = await webgpuToggle.isChecked();
    
    // Toggle
    await webgpuToggle.click();
    
    // Wait for the change to be applied
    await page.waitForTimeout(500);
    
    // Verify the toggle changed
    const newState = await webgpuToggle.isChecked();
    expect(newState).toBe(!initialState);
  });

  test('can change ANGLE backend', async ({ page }) => {
    const angleBackendSelect = page.locator('[data-testid="angle-backend"]');
    
    // Open the select
    await angleBackendSelect.click();
    
    // Select a different option
    await page.locator('option[value="metal"]').click();
    
    // Wait for the change to be applied
    await page.waitForTimeout(500);
    
    // Verify the selection changed
    const selectedValue = await angleBackendSelect.inputValue();
    expect(selectedValue).toBe('metal');
  });

  test('can toggle GPU rasterization', async ({ page }) => {
    const gpuRasterizationToggle = page.locator('[data-testid="gpu-rasterization"]');
    
    // Get initial state
    const initialState = await gpuRasterizationToggle.isChecked();
    
    // Toggle
    await gpuRasterizationToggle.click();
    
    // Wait for the change to be applied
    await page.waitForTimeout(500);
    
    // Verify the toggle changed
    const newState = await gpuRasterizationToggle.isChecked();
    expect(newState).toBe(!initialState);
  });

  test('can refresh performance metrics', async ({ page }) => {
    const refreshButton = page.locator('[data-testid="refresh-metrics"]');
    
    // Click refresh
    await refreshButton.click();
    
    // Wait for metrics to update
    await page.waitForTimeout(1000);
    
    // Verify performance metrics section is still visible
    const performanceMetrics = page.locator('.performance-metrics');
    await expect(performanceMetrics).toBeVisible();
  });

  test('can reset to default settings', async ({ page }) => {
    const resetButton = page.locator('[data-testid="gpu-reset"]');
    
    // Click reset
    await resetButton.click();
    
    // Wait for reset to complete
    await page.waitForTimeout(500);
    
    // Verify GPU acceleration is enabled (default)
    const gpuAccelerationToggle = page.locator('[data-testid="gpu-acceleration-enabled"]');
    const isEnabled = await gpuAccelerationToggle.isChecked();
    expect(isEnabled).toBe(true);
  });

  test('displays WebGL support information', async ({ page }) => {
    const webglInfo = page.locator('.webgl-info');
    await expect(webglInfo).toBeVisible();
    
    // Check for WebGL version
    const webglVersion = page.locator('.webgl-info .value');
    await expect(webglVersion.first()).toBeVisible();
  });

  test('displays WebGPU support information', async ({ page }) => {
    const webgpuInfo = page.locator('.webgpu-info');
    await expect(webgpuInfo).toBeVisible();
    
    // Check for WebGPU availability
    const webgpuAvailable = page.locator('.webgpu-info .value');
    await expect(webgpuAvailable.first()).toBeVisible();
  });

  test('displays performance metrics', async ({ page }) => {
    const performanceMetrics = page.locator('.performance-metrics');
    await expect(performanceMetrics).toBeVisible();
    
    // Check for memory usage
    const memoryUsed = page.locator('.metric-item .label');
    await expect(memoryUsed.filter({ hasText: 'Memory Used' })).toBeVisible();
  });

  test('GPU settings are accessible from settings navigation', async ({ page }) => {
    // Navigate to main settings
    await page.goto('chrome://settings');
    await page.waitForLoadState('networkidle');
    
    // Click on GPU section in navigation
    const gpuNav = page.locator('[data-testid="chrome-settings-nav-gpu"]');
    await gpuNav.click();
    
    // Verify URL changed to GPU settings
    expect(page.url()).toContain('chrome://settings/gpu');
    
    // Verify GPU section is visible
    const gpuSection = page.locator('[data-testid="settings-section-gpu"]');
    await expect(gpuSection).toBeVisible();
  });

  test('GPU settings persist after page reload', async ({ page }) => {
    const gpuAccelerationToggle = page.locator('[data-testid="gpu-acceleration-enabled"]');
    
    // Get initial state
    const initialState = await gpuAccelerationToggle.isChecked();
    
    // Toggle
    await gpuAccelerationToggle.click();
    await page.waitForTimeout(500);
    
    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    // Verify the setting persisted
    const newState = await gpuAccelerationToggle.isChecked();
    expect(newState).toBe(!initialState);
  });

  test('displays warning for ignore GPU blocklist', async ({ page }) => {
    const ignoreBlocklistToggle = page.locator('[data-testid="ignore-gpu-blocklist"]');
    const warning = page.locator('.warning');
    
    // The warning should be visible
    await expect(warning).toBeVisible();
    
    // Toggle the option
    await ignoreBlocklistToggle.click();
    await page.waitForTimeout(500);
    
    // Warning should still be visible
    await expect(warning).toBeVisible();
  });
});
