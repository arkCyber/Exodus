/**
 * WebAssembly compatibility — local smoke page + optional public demos.
 *
 * - Without TAURI_E2E: validates the smoke HTML in Playwright Chromium (page logic only).
 * - With TAURI_E2E=1: validates WASM inside Exodus native tab webviews (requires `pnpm tauri dev`).
 */
import { test, expect } from '@playwright/test';
import {
  openWasmSmokeInTab,
  expectWasmSmokePassInWebview,
  pageHasTauriIpc,
  WASM_SMOKE_PATH,
  dismissPermissionPrompts,
  navigateOmnibox,
} from './helpers/wasm';
import { gotoBrowserShell, OMNIBOX } from './helpers/shell';

const wantsTauri = !!process.env.TAURI_E2E;
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:1421';

test.describe('WASM smoke page (Playwright browser)', () => {
  test('local wasm-smoke.html passes all required checks', async ({ page, baseURL: pwBase }) => {
    const root = (pwBase ?? baseURL).replace(/\/$/, '');
    await page.goto(`${root}${WASM_SMOKE_PATH}`);
    await expect(page.locator('#wasm-status')).toHaveAttribute('data-result', 'pass', {
      timeout: 10_000,
    });
    await expect(page.locator('body')).toHaveAttribute('data-wasm-all-pass', '1');
  });
});

test.describe('WASM shell navigation (TAURI_E2E)', () => {
  test.skip(!wantsTauri, 'Set TAURI_E2E=1 and run pnpm tauri dev (see scripts/test-wasm.sh)');

  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
    await dismissPermissionPrompts(page);
  });

  test('omnibox navigates to local wasm-smoke.html', async ({ page }) => {
    const url = `${baseURL.replace(/\/$/, '')}${WASM_SMOKE_PATH}`;
    await navigateOmnibox(page, url);
    await expect(page.locator('.webview-container, iframe.browser-webview').first()).toBeVisible({
      timeout: 15_000,
    });
  });
});

test.describe('WASM in native tab webview (Tauri IPC)', () => {
  test.skip(!wantsTauri, 'Set TAURI_E2E=1 and run pnpm tauri dev');

  test.beforeEach(async ({ page }) => {
    await gotoBrowserShell(page);
    await dismissPermissionPrompts(page);
    const ipc = await pageHasTauriIpc(page);
    test.skip(
      !ipc,
      'Playwright is not attached to the Tauri app window (no __TAURI_INTERNALS__). ' +
        'Use manual check: open wasm-smoke URL in Exodus, then DevTools: await __EXODUS_E2E__.runWasmSmokeCheck()',
    );
  });

  test('native webview runs local wasm-smoke.html', async ({ page }) => {
    await openWasmSmokeInTab(page, baseURL);
    await expectWasmSmokePassInWebview(page);
  });

  test('native webview exposes WebAssembly global', async ({ page }) => {
    await openWasmSmokeInTab(page, baseURL);
    const raw = await page.evaluate(async () => {
      const invoke = (
        window as Window & { __TAURI_INTERNALS__?: { invoke?: (c: string, a: object) => Promise<string> } }
      ).__TAURI_INTERNALS__!.invoke!;
      const tabId = document.querySelector('.browser-page')?.getAttribute('data-active-tab-id');
      if (!tabId) return 'no-tab-id';
      const label = `exodus-tab-${tabId.replace(/[^a-zA-Z0-9_-]/g, '')}`;
      return invoke('browser_eval_return', { label, script: 'typeof WebAssembly' });
    });
    expect(raw).toBe('object');
  });

  test('webassembly.org demo loads (network)', async ({ page }) => {
    test.slow();
    await navigateOmnibox(page, 'https://webassembly.org/demo/');
    await page.waitForTimeout(4000);
    const title = await page.evaluate(async () => {
      const invoke = (
        window as Window & { __TAURI_INTERNALS__?: { invoke?: (c: string, a: object) => Promise<string> } }
      ).__TAURI_INTERNALS__!.invoke!;
      const tabId = document.querySelector('.browser-page')?.getAttribute('data-active-tab-id');
      if (!tabId) return '';
      const label = `exodus-tab-${tabId.replace(/[^a-zA-Z0-9_-]/g, '')}`;
      return invoke('browser_eval_return', { label, script: 'document.title || ""' });
    });
    expect(title.length).toBeGreaterThan(0);
  });
});
