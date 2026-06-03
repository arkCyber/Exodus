/**
 * Exodus Browser — WebRTC signaling over gossip + in-process RTC hub.
 */
import { invoke } from '@tauri-apps/api/core';

export type RtcSignalType =
  | 'ring'
  | 'accept'
  | 'reject'
  | 'hangup'
  | 'offer'
  | 'answer'
  | 'ice'
  | 'join'
  | 'leave';

export type RtcSignalMessage = {
  id?: string;
  signalType: RtcSignalType;
  sessionId: string;
  fromNode: string;
  toNode?: string;
  displayName?: string;
  sdp?: RTCSessionDescriptionInit;
  candidate?: RTCIceCandidateInit;
  timestamp: number;
};

/** Symmetric topic for two node ids (matches Rust `VideoRtcState::peer_topic`). */
export function peerTopic(localNode: string, remoteNode: string): string {
  const ids = [localNode, remoteNode].sort();
  return `exodus-rtc-peer-${ids[0]}-${ids[1]}`;
}

export function meetingTopic(meetingId: string): string {
  return `exodus-rtc-meeting-${meetingId}`;
}

export async function publishSignal(
  topic: string,
  signal: Omit<RtcSignalMessage, 'timestamp'> & { timestamp?: number }
): Promise<string> {
  return invoke<string>('video_rtc_publish_signal', {
    topic,
    signal: {
      ...signal,
      timestamp: signal.timestamp ?? Math.floor(Date.now() / 1000),
    },
  });
}

export async function pollSignals(
  topic: string,
  since: number
): Promise<RtcSignalMessage[]> {
  return invoke<RtcSignalMessage[]>('video_rtc_poll_signals', { topic, since });
}
