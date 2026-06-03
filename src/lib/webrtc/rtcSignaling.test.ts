/**
 * Exodus Browser — WebRTC signaling tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  peerTopic,
  meetingTopic,
  publishSignal,
  pollSignals,
} from './rtcSignaling';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('rtcSignaling', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('generates peer topic symmetrically', () => {
    const topic1 = peerTopic('node-a', 'node-b');
    const topic2 = peerTopic('node-b', 'node-a');

    expect(topic1).toBe(topic2);
    expect(topic1).toBe('exodus-rtc-peer-node-a-node-b');
  });

  it('generates meeting topic', () => {
    const topic = meetingTopic('meeting-123');

    expect(topic).toBe('exodus-rtc-meeting-meeting-123');
  });

  it('publishes signal with timestamp', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('signal-id-123');

    const signal = {
      signalType: 'offer' as const,
      sessionId: 'session-1',
      fromNode: 'node-a',
      toNode: 'node-b',
      sdp: { type: 'offer' as const, sdp: 'test-sdp' },
    } as any;

    const id = await publishSignal('topic-1', signal);

    expect(id).toBe('signal-id-123');
    expect(invoke).toHaveBeenCalledWith('video_rtc_publish_signal', {
      topic: 'topic-1',
      signal: expect.objectContaining({
        ...signal,
        timestamp: expect.any(Number),
      }),
    });
  });

  it('publishes signal with custom timestamp', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('signal-id-123');

    const signal = {
      signalType: 'offer' as const,
      sessionId: 'session-1',
      fromNode: 'node-a',
      timestamp: 1234567890,
    };

    await publishSignal('topic-1', signal);

    expect(invoke).toHaveBeenCalledWith('video_rtc_publish_signal', {
      topic: 'topic-1',
      signal: expect.objectContaining({
        timestamp: 1234567890,
      }),
    });
  });

  it('polls signals', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockSignals = [
      { signalType: 'offer', sessionId: 'session-1', fromNode: 'node-a', timestamp: 1234567890 },
    ];
    vi.mocked(invoke).mockResolvedValue(mockSignals);

    const signals = await pollSignals('topic-1', 1234567800);

    expect(signals).toEqual(mockSignals);
    expect(invoke).toHaveBeenCalledWith('video_rtc_poll_signals', { topic: 'topic-1', since: 1234567800 });
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Signaling error'));

    await expect(publishSignal('topic-1', { signalType: 'offer' as const, sessionId: 'session-1', fromNode: 'node-a' })).rejects.toThrow('Signaling error');
  });
});
