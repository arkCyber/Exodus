/**
 * Unit tests — tab strip Chrome parity helpers.
 */
import { describe, it, expect } from 'vitest';
import { canShowTabClose } from './tabStripChrome';

describe('canShowTabClose', () => {
  it('shows close on the only unpinned tab (Chrome new-tab page)', () => {
    expect(canShowTabClose(1, {})).toBe(true);
  });

  it('shows close when multiple tabs', () => {
    expect(canShowTabClose(2, {})).toBe(true);
  });

  it('hides close for pinned tabs', () => {
    expect(canShowTabClose(3, { pinned: true })).toBe(false);
  });
});
