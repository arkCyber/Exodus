/**
 * Exodus Browser — confirm dialog types tests.
 */
import { describe, it, expect } from 'vitest';
import { CONFIRM_DIALOG_KEY } from './confirm';

describe('confirm', () => {
  it('exports confirm dialog key symbol', () => {
    expect(CONFIRM_DIALOG_KEY).toBeDefined();
    expect(typeof CONFIRM_DIALOG_KEY).toBe('symbol');
    expect(CONFIRM_DIALOG_KEY.description).toBe('exodusConfirmDialog');
  });
});
