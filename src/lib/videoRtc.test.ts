/**
 * Exodus Browser — Video RTC API tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { peerTopic } from './webrtc/rtcSignaling';
import {
  videoRtcCallStart,
  videoRtcMeetingCreate,
  videoRtcServiceStart,
} from './videoRtc';

describe('videoRtc', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('peerTopic is symmetric', () => {
    expect(peerTopic('exodus-b', 'exodus-a')).toBe(peerTopic('exodus-a', 'exodus-b'));
  });

  it('videoRtcServiceStart invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({ nodeId: 'exodus-1', displayName: 'User' });
    const info = await videoRtcServiceStart();
    expect(invokeMock).toHaveBeenCalledWith('video_rtc_service_start');
    expect(info.nodeId).toBe('exodus-1');
  });

  it('videoRtcCallStart invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({
      sessionId: 's1',
      callerNode: 'a',
      calleeNode: 'b',
      status: 'ringing',
    });
    await videoRtcCallStart('exodus-b', 'Peer B');
    expect(invokeMock).toHaveBeenCalledWith('video_rtc_call_start', {
      calleeNode: 'exodus-b',
      calleeName: 'Peer B',
      videoEnabled: true,
      audioEnabled: true,
    });
  });

  it('videoRtcMeetingCreate invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({ meetingId: 'mtg-abc', title: 'Standup' });
    const room = await videoRtcMeetingCreate('Standup', 6);
    expect(invokeMock).toHaveBeenCalledWith('video_rtc_meeting_create', {
      title: 'Standup',
      maxParticipants: 6,
    });
    expect(room.meetingId).toBe('mtg-abc');
  });
});
