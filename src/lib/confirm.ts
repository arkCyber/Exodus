/**
 * Exodus Browser — reusable confirmation dialog types.
 */

import type { InjectionKey, Ref } from 'vue';

export type ConfirmOffer = {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};

/** Shell confirmation dialog API (provided by BrowserPage). */
export type ConfirmDialogApi = {
  confirmOffer: Ref<ConfirmOffer | null>;
  confirmBusy: Ref<boolean>;
  openConfirmDialog: (offer: ConfirmOffer, action: () => Promise<void>) => void;
  cancelConfirmDialog: () => void;
  runConfirmDialog: () => Promise<void>;
};

/** Vue provide/inject key for the shell confirmation dialog. */
export const CONFIRM_DIALOG_KEY: InjectionKey<ConfirmDialogApi> = Symbol('exodusConfirmDialog');
