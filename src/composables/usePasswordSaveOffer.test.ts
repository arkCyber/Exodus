/**
 * Exodus Browser — usePasswordSaveOffer composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ref } from 'vue';
import { usePasswordSaveOffer } from './usePasswordSaveOffer';

vi.mock('@/lib/passwordAutofill', () => ({
  applyPasswordAutofill: vi.fn(),
  getPasswordForPage: vi.fn(),
  loadPasswordManagerSettings: vi.fn(),
  pullPasswordCapture: vi.fn(),
  savePasswordCapture: vi.fn(),
}));

vi.mock('@/lib/passwordNeverSave', () => ({
  addNeverSavePasswordHost: vi.fn(),
  isNeverSavePasswordUrl: vi.fn(),
}));

vi.mock('@/lib/newTabPage', () => ({
  isNewTabUrl: vi.fn(),
}));

describe('usePasswordSaveOffer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with default state', () => {
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { passwordSaveOffer, passwordSaveBusy } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    expect(passwordSaveOffer.value).toBe(null);
    expect(passwordSaveBusy.value).toBe(false);
  });

  it('does not run autofill hooks in private mode', async () => {
    const { loadPasswordManagerSettings } = await import('@/lib/passwordAutofill');
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(true);
    const onStatus = vi.fn();

    const { runPasswordAutofillHooks } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await runPasswordAutofillHooks('https://example.com');

    expect(loadPasswordManagerSettings).not.toHaveBeenCalled();
  });

  it('does not run autofill hooks for non-http/https URLs', async () => {
    const { loadPasswordManagerSettings } = await import('@/lib/passwordAutofill');
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { runPasswordAutofillHooks } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await runPasswordAutofillHooks('file:///path');

    expect(loadPasswordManagerSettings).not.toHaveBeenCalled();
  });

  it('applies password autofill when enabled', async () => {
    const { loadPasswordManagerSettings, getPasswordForPage, applyPasswordAutofill } = await import('@/lib/passwordAutofill');
    vi.mocked(loadPasswordManagerSettings).mockResolvedValue({ 
      auto_fill: true, 
      auto_save: true,
      require_master_password: false,
      min_password_length: 8,
      require_strength_check: false,
      enable_breach_detection: false,
    } as any);
    vi.mocked(getPasswordForPage).mockResolvedValue({ 
      id: '1',
      username: 'test', 
      password: 'pass',
      url: 'https://example.com',
      site_name: 'Example',
      created_at: Date.now(),
      updated_at: Date.now(),
    } as any);

    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { runPasswordAutofillHooks } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await runPasswordAutofillHooks('https://example.com');

    expect(applyPasswordAutofill).toHaveBeenCalledWith('tab-1', expect.any(Object));
  });

  it('does not offer password save in private mode', async () => {
    const { loadPasswordManagerSettings } = await import('@/lib/passwordAutofill');
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(true);
    const onStatus = vi.fn();

    const { offerPasswordSave } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await offerPasswordSave();

    expect(loadPasswordManagerSettings).not.toHaveBeenCalled();
  });

  it('offers password save when capture is available', async () => {
    const { loadPasswordManagerSettings, pullPasswordCapture, getPasswordForPage } = await import('@/lib/passwordAutofill');
    const { isNeverSavePasswordUrl } = await import('@/lib/passwordNeverSave');
    
    vi.mocked(loadPasswordManagerSettings).mockResolvedValue({ auto_fill: true, auto_save: true } as any);
    vi.mocked(pullPasswordCapture).mockResolvedValue({ url: 'https://example.com', username: 'test', password: 'pass' });
    vi.mocked(getPasswordForPage).mockResolvedValue(null);
    vi.mocked(isNeverSavePasswordUrl).mockReturnValue(false);

    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { offerPasswordSave, passwordSaveOffer } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await offerPasswordSave();

    expect(passwordSaveOffer.value).toEqual({ url: 'https://example.com', username: 'test', password: 'pass' });
  });

  it('does not offer save for never-save URLs', async () => {
    const { loadPasswordManagerSettings, pullPasswordCapture } = await import('@/lib/passwordAutofill');
    const { isNeverSavePasswordUrl } = await import('@/lib/passwordNeverSave');
    
    vi.mocked(loadPasswordManagerSettings).mockResolvedValue({ auto_fill: true, auto_save: true } as any);
    vi.mocked(pullPasswordCapture).mockResolvedValue({ url: 'https://example.com', username: 'test', password: 'pass' });
    vi.mocked(isNeverSavePasswordUrl).mockReturnValue(true);

    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { offerPasswordSave, passwordSaveOffer } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await offerPasswordSave();

    expect(passwordSaveOffer.value).toBe(null);
  });

  it('confirms password save', async () => {
    const { savePasswordCapture } = await import('@/lib/passwordAutofill');
    vi.mocked(savePasswordCapture).mockResolvedValue(undefined);

    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { confirmPasswordSave, passwordSaveOffer } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    passwordSaveOffer.value = { url: 'https://example.com', username: 'test', password: 'pass' };

    await confirmPasswordSave();

    expect(savePasswordCapture).toHaveBeenCalledWith('https://example.com', 'test', 'pass');
    expect(onStatus).toHaveBeenCalledWith('Password saved');
    expect(passwordSaveOffer.value).toBe(null);
  });

  it('dismisses password save', () => {
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { dismissPasswordSave, passwordSaveOffer } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    passwordSaveOffer.value = { url: 'https://example.com', username: 'test', password: 'pass' };

    dismissPasswordSave();

    expect(passwordSaveOffer.value).toBe(null);
  });

  it('never saves password for site', async () => {
    const { addNeverSavePasswordHost } = await import('@/lib/passwordNeverSave');
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { neverSavePasswordForSite, passwordSaveOffer } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    passwordSaveOffer.value = { url: 'https://example.com', username: 'test', password: 'pass' };

    neverSavePasswordForSite();

    expect(addNeverSavePasswordHost).toHaveBeenCalledWith('https://example.com');
    expect(onStatus).toHaveBeenCalledWith('Passwords will not be saved for this site');
    expect(passwordSaveOffer.value).toBe(null);
  });

  it('handles save errors gracefully', async () => {
    const { savePasswordCapture } = await import('@/lib/passwordAutofill');
    vi.mocked(savePasswordCapture).mockRejectedValue(new Error('Save failed'));

    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { confirmPasswordSave, passwordSaveOffer, passwordSaveBusy } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    passwordSaveOffer.value = { url: 'https://example.com', username: 'test', password: 'pass' };

    await confirmPasswordSave();

    expect(onStatus).toHaveBeenCalledWith('Failed to save password');
    expect(passwordSaveBusy.value).toBe(false);
  });

  it('does not confirm save when no offer exists', async () => {
    const { savePasswordCapture } = await import('@/lib/passwordAutofill');
    const getActiveTabLabel = () => 'tab-1';
    const useNativeWebview = ref(true);
    const privateMode = ref(false);
    const onStatus = vi.fn();

    const { confirmPasswordSave } = usePasswordSaveOffer({
      getActiveTabLabel,
      useNativeWebview,
      privateMode,
      onStatus,
    });

    await confirmPasswordSave();

    expect(savePasswordCapture).not.toHaveBeenCalled();
  });
});
