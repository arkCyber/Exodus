/**
 * Exodus Browser — browserIntegrations unit tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const evalTabReturningMock = vi.fn();

vi.mock('$lib/exodusBrowser', () => ({
  evalTabReturning: (...args: unknown[]) => evalTabReturningMock(...args),
}));

import {
  checkNavigationGuard,
  clearBrowsingData,
  fetchOmniboxSuggestions,
  flushTrackerBlockReports,
  omniboxSuggestionTypeLabel,
  syncSuggestionHistory,
} from './browserIntegrations';

describe('browserIntegrations', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    evalTabReturningMock.mockReset();
  });

  it('fetchOmniboxSuggestions returns empty for blank query', async () => {
    const rows = await fetchOmniboxSuggestions('  ');
    expect(rows).toEqual([]);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('fetchOmniboxSuggestions invokes get_suggestions', async () => {
    invokeMock.mockResolvedValue([{ id: '1', text: 'GitHub', url: 'https://github.com' }]);
    const rows = await fetchOmniboxSuggestions('git', 5);
    expect(invokeMock).toHaveBeenCalledWith('get_suggestions', { query: 'git', limit: 5 });
    expect(rows).toHaveLength(1);
  });

  it('syncSuggestionHistory invokes backend', async () => {
    invokeMock.mockResolvedValue(undefined);
    await syncSuggestionHistory('https://a.com', 'A');
    expect(invokeMock).toHaveBeenCalledWith('add_suggestion_history_entry', {
      url: 'https://a.com',
      title: 'A',
    });
  });

  it('clearBrowsingData passes flags', async () => {
    invokeMock.mockResolvedValue('Cleared: cookies');
    const msg = await clearBrowsingData({ clearCookies: true });
    expect(msg).toContain('cookies');
    expect(invokeMock).toHaveBeenCalledWith('clear_browsing_data', {
      clearCache: false,
      clearCookies: true,
      clearLocalStorage: false,
      clearHistory: false,
    });
  });

  it('checkNavigationGuard allows clean URLs', async () => {
    invokeMock.mockResolvedValueOnce({
      enabled: true,
      block_malware: true,
      block_phishing: true,
      block_unwanted_software: true,
      show_warnings: true,
      allow_proceed: true,
    });
    invokeMock.mockResolvedValueOnce(null);
    const guard = await checkNavigationGuard('https://example.com');
    expect(guard.allowed).toBe(true);
  });

  it('checkNavigationGuard blocks phishing patterns', async () => {
    invokeMock.mockResolvedValueOnce({
      enabled: true,
      block_malware: true,
      block_phishing: true,
      block_unwanted_software: true,
      show_warnings: true,
      allow_proceed: false,
    });
    invokeMock.mockResolvedValueOnce({
      id: 't1',
      url_pattern: 'login-update-account.com',
      threat_type: 'phishing',
      severity: 8,
      added_at: 1,
      block_count: 0,
    });
    const guard = await checkNavigationGuard('https://login-update-account.com/foo');
    expect(guard.allowed).toBe(false);
    expect(guard.canProceed).toBe(false);
  });

  it('flushTrackerBlockReports records unique domains', async () => {
    evalTabReturningMock.mockResolvedValue(JSON.stringify(['ads.example.com', 'ads.example.com']));
    invokeMock.mockResolvedValue(undefined);
    const n = await flushTrackerBlockReports('tab-1');
    expect(n).toBe(1);
    expect(invokeMock).toHaveBeenCalledWith('record_tracker_blocked', {
      domain: 'ads.example.com',
    });
  });

  it('omniboxSuggestionTypeLabel maps known types', () => {
    expect(omniboxSuggestionTypeLabel('History')).toBe('History');
    expect(omniboxSuggestionTypeLabel('Popular')).toBe('Popular');
  });
});
