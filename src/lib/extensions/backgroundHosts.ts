/**
 * Exodus Browser — hidden WebViews for Web Extension background service workers.
 */

import { invoke } from '@tauri-apps/api/core';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import { Webview } from '@tauri-apps/api/webview';

/** Background host metadata from the Rust extension manager. */
export type ExtensionBackgroundSpec = {
  extensionId: string;
  webviewLabel: string;
  bootScript: string;
};

/**
 * Create 1×1 off-screen webviews for extension backgrounds and boot service workers.
 * @param container Host element used only for initial layout coordinates.
 */
export async function ensureExtensionBackgrounds(
  container: HTMLElement,
): Promise<void> {
  const specs = await invoke<ExtensionBackgroundSpec[]>('extension_background_specs');
  const rect = container.getBoundingClientRect();

  for (const spec of specs) {
    const existing = await Webview.getByLabel(spec.webviewLabel);
    if (!existing) {
      await invoke('browser_create_tab', {
        label: spec.webviewLabel,
        url: 'about:blank',
        x: -10000,
        y: -10000,
        width: 1,
        height: 1,
      });
      const wv = await Webview.getByLabel(spec.webviewLabel);
      if (wv) {
        await wv.setPosition(new LogicalPosition(-10000, -10000));
        await wv.setSize(new LogicalSize(1, 1));
      }
    }

    try {
      await invoke('extension_background_boot', { extensionId: spec.extensionId });
    } catch (error) {
      console.error(`extension_background_boot(${spec.extensionId}) failed:`, error);
    }
  }

  // Re-apply position in case create used container rect.
  for (const spec of specs) {
    const wv = await Webview.getByLabel(spec.webviewLabel);
    if (wv) {
      await wv.setPosition(new LogicalPosition(rect.left - 10000, rect.top - 10000));
    }
  }
}
