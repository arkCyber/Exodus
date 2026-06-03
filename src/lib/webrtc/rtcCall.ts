/**
 * Exodus Browser — 1:1 WebRTC call (WeChat-style voice/video).
 */
import { getRtcConfiguration } from './rtcConfig';
import {
  peerTopic,
  pollSignals,
  publishSignal,
  type RtcSignalMessage,
} from './rtcSignaling';

export type CallCallbacks = {
  onLocalStream?: (stream: MediaStream) => void;
  onRemoteStream?: (stream: MediaStream) => void;
  onStateChange?: (state: string) => void;
  onError?: (err: string) => void;
};

/**
 * Manages a single 1:1 RTCPeerConnection with gossip signaling.
 */
export class RtcOneToOneCall {
  private pc: RTCPeerConnection | null = null;
  private localStream: MediaStream | null = null;
  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private sinceTs = 0;
  private makingOffer = false;
  private ignoreOffer = false;
  private polite = false;

  constructor(
    readonly sessionId: string,
    readonly localNode: string,
    readonly remoteNode: string,
    readonly isCaller: boolean,
    private readonly callbacks: CallCallbacks = {}
  ) {
    this.polite = !isCaller;
  }

  private topic(): string {
    return peerTopic(this.localNode, this.remoteNode);
  }

  /** Acquire camera/mic and start signaling loop. */
  async start(video: boolean, audio: boolean): Promise<void> {
    this.callbacks.onStateChange?.('connecting');
    this.localStream = await navigator.mediaDevices.getUserMedia({ video, audio });
    this.callbacks.onLocalStream?.(this.localStream);

    this.pc = new RTCPeerConnection(getRtcConfiguration());
    for (const track of this.localStream.getTracks()) {
      this.pc.addTrack(track, this.localStream);
    }
    this.pc.ontrack = (ev) => {
      const [remote] = ev.streams;
      if (remote) this.callbacks.onRemoteStream?.(remote);
    };
    this.pc.onicecandidate = (ev) => {
      if (!ev.candidate) return;
      void publishSignal(this.topic(), {
        signalType: 'ice',
        sessionId: this.sessionId,
        fromNode: this.localNode,
        toNode: this.remoteNode,
        candidate: ev.candidate.toJSON(),
      });
    };
    this.pc.onconnectionstatechange = () => {
      const st = this.pc?.connectionState ?? 'unknown';
      this.callbacks.onStateChange?.(st);
    };

    this.startPolling();

    if (this.isCaller) {
      await this.createAndSendOffer();
    }
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
        if (msg.toNode && msg.toNode !== this.localNode) continue;
        await this.handleSignal(msg);
      }
    } catch (e) {
      this.callbacks.onError?.(String(e));
    }
  }

  private async handleSignal(msg: RtcSignalMessage): Promise<void> {
    if (!this.pc) return;
    const offerCollision =
      msg.signalType === 'offer' && this.makingOffer && this.pc.signalingState !== 'stable';
    this.ignoreOffer = !this.polite && offerCollision;
    if (this.ignoreOffer) return;

    if (msg.signalType === 'offer' && msg.sdp) {
      await this.pc.setRemoteDescription(msg.sdp);
      const answer = await this.pc.createAnswer();
      await this.pc.setLocalDescription(answer);
      await publishSignal(this.topic(), {
        signalType: 'answer',
        sessionId: this.sessionId,
        fromNode: this.localNode,
        toNode: this.remoteNode,
        sdp: answer,
      });
    } else if (msg.signalType === 'answer' && msg.sdp) {
      await this.pc.setRemoteDescription(msg.sdp);
    } else if (msg.signalType === 'ice' && msg.candidate) {
      try {
        await this.pc.addIceCandidate(msg.candidate);
      } catch {
        /* ignore stale candidates */
      }
    } else if (msg.signalType === 'hangup' || msg.signalType === 'reject') {
      this.callbacks.onStateChange?.('ended');
      await this.stop(false);
    }
  }

  private async createAndSendOffer(): Promise<void> {
    if (!this.pc) return;
    this.makingOffer = true;
    const offer = await this.pc.createOffer();
    await this.pc.setLocalDescription(offer);
    await publishSignal(this.topic(), {
      signalType: 'offer',
      sessionId: this.sessionId,
      fromNode: this.localNode,
      toNode: this.remoteNode,
      sdp: offer,
    });
    this.makingOffer = false;
  }

  /** Callee accepts — send accept + wait for offer or create answer path. */
  async accept(): Promise<void> {
    await publishSignal(this.topic(), {
      signalType: 'accept',
      sessionId: this.sessionId,
      fromNode: this.localNode,
      toNode: this.remoteNode,
    });
    if (!this.localStream) {
      await this.start(true, true);
    }
  }

  async hangup(): Promise<void> {
    await publishSignal(this.topic(), {
      signalType: 'hangup',
      sessionId: this.sessionId,
      fromNode: this.localNode,
      toNode: this.remoteNode,
    });
    await this.stop(false);
  }

  async stop(sendHangup: boolean): Promise<void> {
    if (sendHangup) {
      try {
        await this.hangup();
      } catch {
        /* already ending */
      }
    }
    this.stopPolling();
    this.pc?.close();
    this.pc = null;
    this.localStream?.getTracks().forEach((t) => t.stop());
    this.localStream = null;
  }
}
