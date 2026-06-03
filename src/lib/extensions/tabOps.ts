/**
 * Exodus Browser — extension tab ops (update / remove / reload) from flush or commands.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** Single tab operation from extension shims. */
export type ExtensionTabOp = {
  op: 'update' | 'remove' | 'reload' | string;
  extensionId?: string;
  chromeTabId?: number;
  tabIds?: number[];
  updateProperties?: { url?: string; active?: boolean };
};

export type ExtensionTabOpsEvent = {
  ops: ExtensionTabOp[];
};

/**
 * Listen for extension-requested tab control operations.
 */
export function listenExtensionTabOps(
  onOps: (ops: ExtensionTabOp[]) => void | Promise<void>,
): Promise<UnlistenFn> {
  return listen<ExtensionTabOpsEvent>('exodus-extension-tabs-ops', async (event) => {
    const ops = event.payload.ops ?? [];
    if (ops.length === 0) return;
    try {
      await onOps(ops);
    } catch (error) {
      console.error('exodus-extension-tabs-ops handler failed:', error);
    }
  });
}
