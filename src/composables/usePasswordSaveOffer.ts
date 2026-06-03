/**
 * Exodus Browser — password save offer after form submit (native webview capture).
 */
import { ref, type Ref } from 'vue';
import {
  applyPasswordAutofill,
  getPasswordForPage,
  loadPasswordManagerSettings,
  pullPasswordCapture,
  savePasswordCapture,
  type PasswordCapturePayload,
} from '@/lib/passwordAutofill';
import { addNeverSavePasswordHost, isNeverSavePasswordUrl } from '@/lib/passwordNeverSave';
import { isNewTabUrl } from '@/lib/newTabPage';

export type UsePasswordSaveOfferOptions = {
  getActiveTabLabel: () => string;
  useNativeWebview: Ref<boolean>;
  privateMode: Ref<boolean>;
  onStatus: (message: string) => void;
};

/**
 * Password capture dialog state and autofill/save hooks.
 */
export function usePasswordSaveOffer(options: UsePasswordSaveOfferOptions) {
  const passwordSaveOffer = ref<PasswordCapturePayload | null>(null);
  const passwordSaveBusy = ref(false);

  async function runPasswordAutofillHooks(url: string): Promise<void> {
    if (options.privateMode.value || !options.useNativeWebview.value || isNewTabUrl(url)) {
      return;
    }
    if (!url.startsWith('http://') && !url.startsWith('https://')) return;
    try {
      const settings = await loadPasswordManagerSettings();
      if (settings.auto_fill) {
        const entry = await getPasswordForPage(url);
        if (entry) {
          await applyPasswordAutofill(options.getActiveTabLabel(), entry);
        }
      }
      setTimeout(() => {
        void offerPasswordSave();
      }, 2500);
    } catch (error) {
      console.error('runPasswordAutofillHooks failed:', error);
    }
  }

  async function offerPasswordSave(): Promise<void> {
    if (options.privateMode.value || !options.useNativeWebview.value) return;
    try {
      const settings = await loadPasswordManagerSettings();
      if (!settings.auto_save) return;
      const capture = await pullPasswordCapture(options.getActiveTabLabel());
      if (!capture) return;
      if (isNeverSavePasswordUrl(capture.url)) return;
      const existing = await getPasswordForPage(capture.url);
      if (
        existing &&
        existing.username === capture.username &&
        existing.password === capture.password
      ) {
        return;
      }
      passwordSaveOffer.value = capture;
    } catch (error) {
      console.error('offerPasswordSave failed:', error);
    }
  }

  async function confirmPasswordSave(): Promise<void> {
    if (!passwordSaveOffer.value) return;
    passwordSaveBusy.value = true;
    const capture = passwordSaveOffer.value;
    try {
      await savePasswordCapture(capture.url, capture.username, capture.password);
      options.onStatus('Password saved');
      passwordSaveOffer.value = null;
    } catch (error) {
      console.error('confirmPasswordSave failed:', error);
      options.onStatus('Failed to save password');
    } finally {
      passwordSaveBusy.value = false;
    }
  }

  function dismissPasswordSave(): void {
    passwordSaveOffer.value = null;
  }

  function neverSavePasswordForSite(): void {
    if (passwordSaveOffer.value) {
      addNeverSavePasswordHost(passwordSaveOffer.value.url);
      options.onStatus('Passwords will not be saved for this site');
    }
    passwordSaveOffer.value = null;
  }

  return {
    passwordSaveOffer,
    passwordSaveBusy,
    runPasswordAutofillHooks,
    offerPasswordSave,
    confirmPasswordSave,
    dismissPasswordSave,
    neverSavePasswordForSite,
  };
}
