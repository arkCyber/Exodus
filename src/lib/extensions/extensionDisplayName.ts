/**
 * Exodus Browser — resolve extension label for permission prompts.
 */

import type { ExtensionInfo } from '$lib/extensions/types';
import type { ExtensionPermissionRequestEvent } from '$lib/extensions/extensionEvents';

/**
 * Prefer manifest name from the event payload, then installed catalog, then id.
 */
export function extensionDisplayName(
  request: Pick<ExtensionPermissionRequestEvent, 'extensionId' | 'extensionName'> | null,
  installed: ExtensionInfo[] = [],
): string {
  if (!request) return 'Extension';
  if (request.extensionName?.trim()) return request.extensionName;
  const hit = installed.find((e) => e.id === request.extensionId);
  return hit?.name ?? request.extensionId;
}
