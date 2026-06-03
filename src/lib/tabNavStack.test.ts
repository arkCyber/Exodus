/**
 * Unit tests for iframe / shared tab navigation stack helpers.
 */

import { describe, expect, it } from 'vitest';
import { navFlagsFromTrack, recordTabNavigation } from './tabNavStack';

describe('recordTabNavigation', () => {
  it('tracks back and forward', () => {
    const map = new Map<string, { stack: string[]; index: number }>();
    recordTabNavigation(map, 't1', 'https://a.com');
    recordTabNavigation(map, 't1', 'https://b.com');
    recordTabNavigation(map, 't1', 'https://c.com');
    expect(navFlagsFromTrack(map.get('t1'))).toEqual({ canGoBack: true, canGoForward: false });
    recordTabNavigation(map, 't1', 'https://b.com');
    expect(navFlagsFromTrack(map.get('t1'))).toEqual({ canGoBack: true, canGoForward: true });
    recordTabNavigation(map, 't1', 'https://c.com');
    expect(navFlagsFromTrack(map.get('t1'))).toEqual({ canGoBack: true, canGoForward: false });
  });
});
