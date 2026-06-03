/**
 * Exodus Browser — dev-only E2E helpers on `window.__EXODUS_E2E__`.
 *
 * Use inside the real Tauri app (DevTools console), e.g. after opening
 * http://localhost:1421/test/wasm-smoke.html in a tab:
 *   await __EXODUS_E2E__.evalActiveTab("document.body.getAttribute('data-wasm-all-pass')")
 */
import { canInvokeTauri } from '$lib/tauri';
import { evalTabReturning, tabWebviewLabel } from '$lib/exodusBrowser';

export type ExodusE2eBridge = {
  evalActiveTab: (script: string) => Promise<string>;
  runWasmSmokeCheck: () => Promise<{ pass: boolean; raw: string }>;
};

declare global {
  interface Window {
    __EXODUS_E2E__?: ExodusE2eBridge;
  }
}

function activeTabIdFromDom(): string {
  const tabId = document.querySelector('.browser-page')?.getAttribute('data-active-tab-id');
  if (!tabId) {
    throw new Error('data-active-tab-id missing — open the browser shell first');
  }
  return tabId;
}

/** Install dev bridge when Tauri IPC is available. */
export function installE2eBridge(): void {
  if (!canInvokeTauri()) {
    return;
  }
  window.__EXODUS_E2E__ = {
    async evalActiveTab(script: string): Promise<string> {
      const label = tabWebviewLabel(activeTabIdFromDom());
      return evalTabReturning(label, script);
    },
    async runWasmSmokeCheck(): Promise<{ pass: boolean; raw: string }> {
      const raw = await window.__EXODUS_E2E__!.evalActiveTab(
        `document.body.getAttribute('data-wasm-all-pass') || document.getElementById('wasm-status')?.getAttribute('data-result') || 'pending'`,
      );
      return { pass: raw === '1', raw };
    },
  };
  console.info('[Exodus E2E] window.__EXODUS_E2E__ ready (dev + Tauri IPC)');
}

if (import.meta.env.DEV) {
  installE2eBridge();
}
