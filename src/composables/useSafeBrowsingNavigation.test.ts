/**
 * Exodus Browser — useSafeBrowsingNavigation unit tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useSafeBrowsingNavigation } from './useSafeBrowsingNavigation';

vi.mock('$lib/browserIntegrations', () => ({
  checkNavigationGuard: vi.fn(),
  recordMaliciousSiteBlocked: vi.fn(),
}));

describe('useSafeBrowsingNavigation', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('allows when guard passes', async () => {
    const { checkNavigationGuard } = await import('$lib/browserIntegrations');
    vi.mocked(checkNavigationGuard).mockResolvedValue({
      allowed: true,
      reason: '',
      canProceed: false,
    });
    const nav = useSafeBrowsingNavigation({ onStatus: vi.fn() });
    const ok = await nav.ensureNavigationAllowed('https://safe.test');
    expect(ok).toBe(true);
    expect(nav.safeBrowsingOffer.value).toBeNull();
  });

  it('shows offer when guard allows proceed', async () => {
    const { checkNavigationGuard } = await import('$lib/browserIntegrations');
    vi.mocked(checkNavigationGuard).mockResolvedValue({
      allowed: false,
      reason: 'blocked',
      canProceed: true,
    });
    const nav = useSafeBrowsingNavigation({ onStatus: vi.fn() });
    const ok = await nav.ensureNavigationAllowed('https://evil.test');
    expect(ok).toBe(false);
    expect(nav.safeBrowsingOffer.value?.url).toBe('https://evil.test');
  });

  it('reports status when hard blocked', async () => {
    const { checkNavigationGuard } = await import('$lib/browserIntegrations');
    vi.mocked(checkNavigationGuard).mockResolvedValue({
      allowed: false,
      reason: 'Hard block',
      canProceed: false,
    });
    const onStatus = vi.fn();
    const nav = useSafeBrowsingNavigation({ onStatus });
    const ok = await nav.ensureNavigationAllowed('https://evil.test');
    expect(ok).toBe(false);
    expect(onStatus).toHaveBeenCalledWith('Hard block');
    expect(nav.safeBrowsingOffer.value).toBeNull();
  });
});
