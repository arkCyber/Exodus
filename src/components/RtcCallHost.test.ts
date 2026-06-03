/**
 * Exodus Browser — RtcCallHost global call wiring tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import RtcCallHost from './RtcCallHost.vue';

const startOutgoing = vi.fn();
const init = vi.fn(async () => ({ nodeId: 'node-local' }));
const listenIncoming = vi.fn(async () => undefined);
const subscribePhase = vi.fn(() => () => undefined);

vi.mock('$lib/webrtc/rtcCallSession', () => ({
  getRtcCallManager: () => ({
    init,
    listenIncoming,
    subscribePhase,
    startOutgoing,
    acceptIncoming: vi.fn(),
    hangup: vi.fn(),
    sessionId: 'session-1',
  }),
}));

vi.mock('./RtcCallOverlay.vue', () => ({
  default: {
    name: 'RtcCallOverlay',
    template: '<div class="rtc-call-overlay-stub" />',
  },
}));

vi.mock('$lib/imChat', () => ({
  IM_START_CALL_EVENT: 'exodus-start-call',
  openP2pTab: vi.fn(),
  openWebChat: vi.fn(),
}));

describe('RtcCallHost', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('starts outgoing call when IM_START_CALL_EVENT fires', async () => {
    const { openP2pTab, openWebChat } = await import('$lib/imChat');
    mount(RtcCallHost);
    await flushPromises();

    window.dispatchEvent(
      new CustomEvent('exodus-start-call', {
        detail: { nodeId: 'node-alice', name: 'Alice', video: false, audio: true },
      }),
    );
    await flushPromises();

    expect(openP2pTab).toHaveBeenCalledWith('im');
    expect(openWebChat).toHaveBeenCalled();
    expect(startOutgoing).toHaveBeenCalledWith(
      'node-alice',
      'Alice',
      false,
      true,
      expect.any(Object),
    );
  });
});
