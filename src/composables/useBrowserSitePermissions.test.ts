/**
 * Exodus Browser — useBrowserSitePermissions composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useBrowserSitePermissions } from './useBrowserSitePermissions';

vi.mock('@/lib/extensions/extensionEvents', () => ({
  listenBrowserSitePermissionRequests: vi.fn(),
}));

vi.mock('@/lib/promptQueue', () => ({
  enqueuePrompt: vi.fn(),
  advancePromptQueue: vi.fn(),
}));

describe('useBrowserSitePermissions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    const { sitePermRequest } = useBrowserSitePermissions();

    expect(sitePermRequest.value).toBe(null);
  });

  it('sets up site permission listener', async () => {
    const { listenBrowserSitePermissionRequests } = await import('@/lib/extensions/extensionEvents');
    const { enqueuePrompt } = await import('@/lib/promptQueue');
    
    const mockRequest = { requestId: '1', kind: 'camera', origin: 'https://example.com', webviewLabel: 'tab-1' };
    vi.mocked(listenBrowserSitePermissionRequests).mockResolvedValue(vi.fn());
    vi.mocked(enqueuePrompt).mockReturnValue({ active: mockRequest, queue: [] });

    const { setupSitePermissionListener } = useBrowserSitePermissions();

    await setupSitePermissionListener();

    expect(listenBrowserSitePermissionRequests).toHaveBeenCalled();
  });

  it('advances site permission queue', async () => {
    const { advancePromptQueue } = await import('@/lib/promptQueue');
    const mockRequest = { requestId: '1', kind: 'microphone', origin: 'https://example.com', webviewLabel: 'tab-1' };
    vi.mocked(advancePromptQueue).mockReturnValue(mockRequest);

    const { advanceSitePermQueue } = useBrowserSitePermissions();

    advanceSitePermQueue();

    expect(advancePromptQueue).toHaveBeenCalled();
  });

  it('tears down site permission listener', async () => {
    const unlistenFn = vi.fn();
    const { listenBrowserSitePermissionRequests } = await import('@/lib/extensions/extensionEvents');
    vi.mocked(listenBrowserSitePermissionRequests).mockResolvedValue(unlistenFn);

    const { setupSitePermissionListener, teardownSitePermissionListener, sitePermRequest } = useBrowserSitePermissions();

    await setupSitePermissionListener();
    sitePermRequest.value = { requestId: '1', kind: 'camera', origin: 'https://example.com', webviewLabel: 'tab-1' };

    teardownSitePermissionListener();

    expect(unlistenFn).toHaveBeenCalled();
    expect(sitePermRequest.value).toBe(null);
  });

  it('does not setup listener twice', async () => {
    const { listenBrowserSitePermissionRequests } = await import('@/lib/extensions/extensionEvents');
    vi.mocked(listenBrowserSitePermissionRequests).mockResolvedValue(vi.fn());

    const { setupSitePermissionListener } = useBrowserSitePermissions();

    await setupSitePermissionListener();
    await setupSitePermissionListener();

    expect(listenBrowserSitePermissionRequests).toHaveBeenCalledTimes(1);
  });

  it('handles listener setup errors gracefully', async () => {
    const { listenBrowserSitePermissionRequests } = await import('@/lib/extensions/extensionEvents');
    vi.mocked(listenBrowserSitePermissionRequests).mockRejectedValue(new Error('Setup failed'));

    const { setupSitePermissionListener } = useBrowserSitePermissions();

    await setupSitePermissionListener();

    // Should not throw
    expect(true).toBe(true);
  });

  it('processes permission requests through queue', async () => {
    const { listenBrowserSitePermissionRequests } = await import('@/lib/extensions/extensionEvents');
    const { enqueuePrompt } = await import('@/lib/promptQueue');
    
    const request = { requestId: '1', kind: 'camera', origin: 'https://example.com', webviewLabel: 'tab-1' };
    vi.mocked(listenBrowserSitePermissionRequests).mockImplementation((callback) => {
      callback(request);
      return Promise.resolve(vi.fn());
    });
    vi.mocked(enqueuePrompt).mockReturnValue({ active: request, queue: [] });

    const { setupSitePermissionListener, sitePermRequest } = useBrowserSitePermissions();

    await setupSitePermissionListener();

    expect(enqueuePrompt).toHaveBeenCalledWith(null, [], request);
    expect(sitePermRequest.value).toEqual(request);
  });
});
