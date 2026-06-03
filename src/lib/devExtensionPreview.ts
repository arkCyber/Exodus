/**
 * Exodus Browser — dev-only extension toolbar preview when not running in Tauri.
 */

import type { ExtensionInfo } from '@/lib/extensions/types';

/** Sample extensions shown in Vite / Playwright so the toolbar matches Chrome layout. */
export const DEV_TOOLBAR_EXTENSIONS: ExtensionInfo[] = [
  {
    id: 'sample-hello',
    name: 'Exodus Sample Hello',
    version: '1.1.0',
    description: 'Reference Web Extension',
    enabled: true,
    pinned: true,
    permissions: ['storage', 'tabs'],
    path: 'extensions/sample-hello',
    actionPopup: 'popup.html',
  },
  {
    id: 'sample-all-frames',
    name: 'Exodus Sample All Frames',
    version: '1.1.0',
    enabled: true,
    pinned: false,
    permissions: [],
    path: 'extensions/sample-all-frames',
    actionPopup: null,
  },
];
