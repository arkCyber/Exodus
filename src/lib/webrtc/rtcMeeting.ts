/**
 * Exodus Browser — multi-party meeting (mesh WebRTC, up to ~6 participants).
 */
import { getRtcConfiguration } from './rtcConfig';
import {
  meetingTopic,
  pollSignals,
  publishSignal,
  type RtcSignalMessage,
} from './rtcSignaling';

export type MeetingCallbacks = {
  onLocalStream?: (stream: MediaStream) => void;
  onRemoteStream?: (nodeId: string, stream: MediaStream) => void;
  onParticipantLeft?: (nodeId: string) => void;
  onError?: (err: string) => void;
};

/**
 * Full-mesh: one RTCPeerConnection per remote participant.
 */
export class RtcMeetingMesh {
  private localStream: MediaStream | null = null;
  private peers = new Map<string, RTCPeerConnection>();
  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private sinceTs = 0;

  constructor(
    readonly meetingId: string,
    readonly localNode: string,
    private readonly callbacks: MeetingCallbacks = {}
  ) {}

  private topic(): string {
    return meetingTopic(this.meetingId);
  }

  async start(video: boolean, audio: boolean): Promise<void> {
    this.localStream = await navigator.mediaDevices.getUserMedia({ video, audio });
    this.callbacks.onLocalStream?.(this.localStream);
    this.startPolling();
    await publishSignal(this.topic(), {
      signalType: 'join',
      sessionId: this.meetingId,
      fromNode: this.localNode,
      displayName: 'Participant',
    });
  }

  private startPolling(): void {
    this.stopPolling();
    this.pollTimer = setInterval(() => void this.pollOnce(), 400);
  }

  private stopPolling(): void {
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }

  private async pollOnce(): Promise<void> {
    try {
      const msgs = await pollSignals(this.topic(), this.sinceTs);
      for (const msg of msgs) {
        this.sinceTs = Math.max(this.sinceTs, msg.timestamp);
        if (msg.fromNode === this.localNode) continue;
        await this.handleSignal(msg);
      }
    } catch (e) {
      this.callbacks.onError?.(String(e));
    }
  }

  private async ensurePeer(remoteNode: string): Promise<RTCPeerConnection> {
    let pc = this.peers.get(remoteNode);
    if (pc) return pc;
    pc = new RTCPeerConnection(getRtcConfiguration());
    if (this.localStream) {
      for (const track of this.localStream.getTracks()) {
        pc.addTrack(track, this.localStream);
      }
    }
    pc.ontrack = (ev) => {
      const [stream] = ev.streams;
      if (stream) this.callbacks.onRemoteStream?.(remoteNode, stream);
    };
    pc.onicecandidate = (ev) => {
      if (!ev.candidate) return;
      void publishSignal(this.topic(), {
        signalType: 'ice',
        sessionId: this.meetingId,
        fromNode: this.localNode,
        toNode: remoteNode,
        candidate: ev.candidate.toJSON(),
      });
    };
    this.peers.set(remoteNode, pc);
    return pc;
  }

  private async handleSignal(msg: RtcSignalMessage): Promise<void> {
    if (msg.signalType === 'leave') {
      this.removePeer(msg.fromNode);
      return;
    }
    if (msg.signalType === 'join' && msg.fromNode !== this.localNode) {
      await this.connectToPeer(msg.fromNode, true);
      return;
    }
    const pc = await this.ensurePeer(msg.fromNode);
    if (msg.signalType === 'offer' && msg.sdp) {
      await pc.setRemoteDescription(msg.sdp);
      const answer = await pc.createAnswer();
      await pc.setLocalDescription(answer);
      await publishSignal(this.topic(), {
        signalType: 'answer',
        sessionId: this.meetingId,
        fromNode: this.localNode,
        toNode: msg.fromNode,
        sdp: answer,
      });
    } else if (msg.signalType === 'answer' && msg.sdp) {
      await pc.setRemoteDescription(msg.sdp);
    } else if (msg.signalType === 'ice' && msg.candidate) {
      try {
        await pc.addIceCandidate(msg.candidate);
      } catch {
        /* stale */
      }
    }
  }

  private async connectToPeer(remoteNode: string, asOfferer: boolean): Promise<void> {
    const pc = await this.ensurePeer(remoteNode);
    if (!asOfferer || pc.signalingState !== 'stable') return;
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    await publishSignal(this.topic(), {
      signalType: 'offer',
      sessionId: this.meetingId,
      fromNode: this.localNode,
      toNode: remoteNode,
      sdp: offer,
    });
  }

  private removePeer(nodeId: string): void {
    const pc = this.peers.get(nodeId);
    pc?.close();
    this.peers.delete(nodeId);
    this.callbacks.onParticipantLeft?.(nodeId);
  }

  async leave(): Promise<void> {
    await publishSignal(this.topic(), {
      signalType: 'leave',
      sessionId: this.meetingId,
      fromNode: this.localNode,
    });
    await this.stop();
  }

  async stop(): Promise<void> {
    this.stopPolling();
    for (const [, pc] of this.peers) pc.close();
    this.peers.clear();
    this.localStream?.getTracks().forEach((t) => t.stop());
    this.localStream = null;
  }
}
