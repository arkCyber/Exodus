/**
 * Playwright helpers — WebAssembly smoke tests in Exodus native tab webviews.
 */
import { expect, type Page } from '@playwright/test';
import { dismissPermissionPrompts, gotoBrowserShell, OMNIBOX } from './shell';

/** Relative path served from Vite `public/test/wasm-smoke.html`. */
export const WASM_SMOKE_PATH = '/test/wasm-smoke.html';

/** Build webview label (must match `tabWebviewLabel` in exodusBrowser.ts). */
export function tabWebviewLabelFromId(tabId: string): string {
  const safe = tabId.replace(/[^a-zA-Z0-9_-]/g, '');
  return `exodus-tab-${safe}`;
}

/** Read active tab id from shell DOM (requires `data-active-tab-id` on `.browser-page`). */
export async function getActiveTabId(page: Page): Promise<string> {
  const tabId = await page.locator('.browser-page').getAttribute('data-active-tab-id');
  if (!tabId) {
    throw new Error('data-active-tab-id missing on .browser-page');
  }
  return tabId;
}

/** Navigate omnibox to a URL and wait for address bar to reflect it. */
export async function navigateOmnibox(page: Page, url: string): Promise<void> {
  const bar = page.locator(OMNIBOX).first();
  await bar.fill(url);
  await bar.press('Enter');
  const hostPart = url.replace(/^https?:\/\//, '').split('/')[0];
  await expect(bar).toHaveValue(new RegExp(hostPart.replace(/\./g, '\\.')), { timeout: 20_000 });
}

/** True when Playwright page has Tauri IPC (real app window), not plain Chromium. */
export async function pageHasTauriIpc(page: Page): Promise<boolean> {
  return page.evaluate(
    () => typeof (window as Window & { __TAURI_INTERNALS__?: { invoke?: unknown } }).__TAURI_INTERNALS__?.invoke === 'function',
  );
}

/**
 * Evaluate JS in the active tab native webview via `browser_eval_return`.
 * Requires the page to run inside the Tauri shell (`__TAURI_INTERNALS__.invoke`).
 */
export async function evalActiveTabWebview(page: Page, expr: string): Promise<string> {
  return page.evaluate(
    async ({ expr }) => {
      const w = window as Window & {
        __TAURI_INTERNALS__?: { invoke?: (cmd: string, args: Record<string, string>) => Promise<string> };
      };
      const invoke = w.__TAURI_INTERNALS__?.invoke;
      if (typeof invoke !== 'function') {
        throw new Error('NOT_TAURI_IPC');
      }
      const tabId = document.querySelector('.browser-page')?.getAttribute('data-active-tab-id');
      if (!tabId) {
        throw new Error('data-active-tab-id not set');
      }
      const label = `exodus-tab-${tabId.replace(/[^a-zA-Z0-9_-]/g, '')}`;
      return invoke('browser_eval_return', { label, script: expr });
    },
    { expr },
  );
}

/** Open shell, dismiss prompts, navigate to WASM smoke page. */
export async function openWasmSmokeInTab(page: Page, baseURL: string): Promise<void> {
  await gotoBrowserShell(page);
  const url = `${baseURL.replace(/\/$/, '')}${WASM_SMOKE_PATH}`;
  await navigateOmnibox(page, url);
  await page.waitForTimeout(1500);
}

/** Assert native webview WASM smoke page passed all required checks. */
export async function expectWasmSmokePassInWebview(page: Page): Promise<void> {
  const raw = await evalActiveTabWebview(
    page,
    `document.body.getAttribute('data-wasm-all-pass') || document.getElementById('wasm-status')?.getAttribute('data-result') || 'pending'`,
  );
  expect(raw).toBe('1');
}

export { dismissPermissionPrompts };
