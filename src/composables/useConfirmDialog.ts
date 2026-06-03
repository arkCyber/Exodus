/**
 * Exodus Browser — global confirmation dialog state (replaces window.confirm).
 */
import { ref } from 'vue';
import type { ConfirmOffer } from '@/lib/confirm';

/**
 * Modal confirm flow with async action callback.
 */
export function useConfirmDialog() {
  const confirmOffer = ref<ConfirmOffer | null>(null);
  const confirmBusy = ref(false);
  let pendingAction: (() => Promise<void>) | null = null;

  function openConfirmDialog(offer: ConfirmOffer, action: () => Promise<void>): void {
    confirmOffer.value = offer;
    pendingAction = action;
  }

  function cancelConfirmDialog(): void {
    confirmOffer.value = null;
    pendingAction = null;
    confirmBusy.value = false;
  }

  async function runConfirmDialog(): Promise<void> {
    const action = pendingAction;
    if (!action) return;
    confirmBusy.value = true;
    try {
      await action();
    } finally {
      cancelConfirmDialog();
    }
  }

  return {
    confirmOffer,
    confirmBusy,
    openConfirmDialog,
    cancelConfirmDialog,
    runConfirmDialog,
  };
}
