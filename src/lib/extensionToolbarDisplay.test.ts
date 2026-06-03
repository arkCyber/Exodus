/**
 * Exodus Browser — extension toolbar display module tests.
 */

import { describe, expect, it } from 'vitest';
import type { ExtensionInfo } from '@/lib/extensions/types';
import {
  isPinnedToolbarExtension,
  pinnedToolbarActionTitle,
  pinnedToolbarExtensions,
  pinnedToolbarModuleLabel,
} from './extensionToolbarDisplay';

const base = (overrides: Partial<ExtensionInfo>): ExtensionInfo => ({
  id: 'ext-a',
  name: 'Alpha',
  version: '1.0.0',
  enabled: true,
  pinned: true,
  permissions: [],
  path: '/tmp/ext',
  ...overrides,
});

describe('extensionToolbarDisplay', () => {
  it('includes only enabled pinned extensions', () => {
    const list = [
      base({ id: 'a', name: 'Alpha', enabled: true, pinned: true }),
      base({ id: 'b', name: 'Beta', enabled: true, pinned: false }),
      base({ id: 'c', name: 'Gamma', enabled: false, pinned: true }),
      base({ id: 'd', name: 'Delta', enabled: true }),
    ];

    expect(pinnedToolbarExtensions(list).map((e) => e.id)).toEqual(['a', 'd']);
  });

  it('treats missing pinned flag as pinned (backward compatible)', () => {
    expect(isPinnedToolbarExtension(base({ pinned: undefined }))).toBe(true);
  });

  it('builds action titles like Chrome', () => {
    expect(pinnedToolbarActionTitle(base({ actionPopup: 'popup.html' }))).toBe('Alpha');
    expect(
      pinnedToolbarActionTitle(base({ actionPopup: null }), { nativePopups: true }),
    ).toContain('Extensions');
  });

  it('labels the toolbar module for accessibility', () => {
    expect(pinnedToolbarModuleLabel(0)).toContain('none pinned');
    expect(pinnedToolbarModuleLabel(2)).toContain('2 pinned');
  });
});
