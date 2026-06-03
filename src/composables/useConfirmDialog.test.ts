/**
 * Exodus Browser — useConfirmDialog unit tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { useConfirmDialog } from './useConfirmDialog';

describe('useConfirmDialog', () => {
  it('opens offer and runs async action on confirm', async () => {
    const dialog = useConfirmDialog();
    const action = vi.fn(async () => {});

    dialog.openConfirmDialog(
      { title: 'Test', message: 'Proceed?', confirmLabel: 'OK', danger: true },
      action,
    );

    expect(dialog.confirmOffer.value?.title).toBe('Test');
    await dialog.runConfirmDialog();
    expect(action).toHaveBeenCalledOnce();
    expect(dialog.confirmOffer.value).toBeNull();
  });

  it('cancel clears pending action', () => {
    const dialog = useConfirmDialog();
    dialog.openConfirmDialog({ title: 'T', message: 'M' }, vi.fn());
    dialog.cancelConfirmDialog();
    expect(dialog.confirmOffer.value).toBeNull();
    expect(dialog.confirmBusy.value).toBe(false);
  });
});
