/**
 * Exodus Browser — queued site permission prompts (camera, mic, geolocation).
 */
import { shellLog } from '@/lib/diagnosticLog';
import { ref, onUnmounted } from 'vue';
import {
  listenBrowserSitePermissionRequests,
  type BrowserSitePermissionRequestEvent,
} from '@/lib/extensions/extensionEvents';
import { advancePromptQueue, enqueuePrompt } from '@/lib/promptQueue';

/**
 * Subscribe to `browser_site_permission_request` events and show one modal at a time.
 */
export function useBrowserSitePermissions() {
  const sitePermRequest = ref<BrowserSitePermissionRequestEvent | null>(null);
  const sitePermQueue = ref<BrowserSitePermissionRequestEvent[]>([]);
  let unlisten: (() => void) | undefined;

  async function setupSitePermissionListener(): Promise<void> {
    if (unlisten) return;
    try {
      unlisten = await listenBrowserSitePermissionRequests((req) => {
        const next = enqueuePrompt(sitePermRequest.value, sitePermQueue.value, req);
        sitePermRequest.value = next.active;
        sitePermQueue.value = next.queue;
      });
    } catch (error) {
      shellLog.error('listenBrowserSitePermissionRequests failed', error);
    }
  }

  /** Advance to the next queued site permission after resolve/dismiss. */
  function advanceSitePermQueue(): void {
    sitePermRequest.value = advancePromptQueue(sitePermQueue.value);
  }

  function teardownSitePermissionListener(): void {
    unlisten?.();
    unlisten = undefined;
    sitePermRequest.value = null;
    sitePermQueue.value = [];
  }

  // Auto cleanup on unmount
  onUnmounted(() => {
    shellLog.info('Cleaning up');
    teardownSitePermissionListener();
  });

  return {
    sitePermRequest,
    setupSitePermissionListener,
    advanceSitePermQueue,
    teardownSitePermissionListener,
  };
}
