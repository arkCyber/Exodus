/**
 * Exodus Browser — IM + contact integration tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { dmRoomId, IM_OPEN_CONTACT_EVENT, P2P_TAB_EVENT } from './imChat';

describe('imChat', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('dmRoomId is symmetric', () => {
    expect(dmRoomId('exodus-b', 'exodus-a')).toBe(dmRoomId('exodus-a', 'exodus-b'));
    expect(dmRoomId('exodus-a', 'exodus-b')).toMatch(/^dm-/);
  });

  it('exports event names for IM integration', () => {
    expect(IM_OPEN_CONTACT_EVENT).toBe('exodus-open-im');
    expect(P2P_TAB_EVENT).toBe('exodus-p2p-tab');
  });
});
