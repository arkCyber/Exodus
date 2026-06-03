/**
 * Exodus Browser — application version metadata for About settings.
 */

import { logStartup } from '@/lib/startupLog';

logStartup('appVersion module loaded');

import pkg from '../../package.json';

/** NPM package version (matches Tauri bundle when built together). */
export const APP_PACKAGE_VERSION = pkg.version ?? '0.0.0';

/** Product name shown in About. */
export const APP_PRODUCT_NAME = 'Exodus Browser';

/** Short build stack label for About. */
export const APP_BUILD_STACK = 'Vue 3 shell · Tauri 2';
