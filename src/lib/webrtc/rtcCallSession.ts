/**
 * Exodus Browser — active 1:1 call session (shared by IM + contacts).
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { RtcOneToOneCall } from './rtcCall';
import {
  videoRtcCallStart,
  videoRtcCallUpdate,
  videoRtcNodeInfo,
  videoRtcServiceStart,
  type VideoRtcNodeInfo,
} from '$lib/videoRtc';

export type CallUiPhase = 'idle' | 'ringing' | 'connecting' | 'connected' | 'ended' | 'error';

export type CallSessionCallbacks = {
  onPhase?: (phase: CallUiPhase) => void;
  onLocalStream?: (s: MediaStream | null) => void;
  onRemoteStream?: (s: MediaStream | null) => void;
  onIncoming?: (from: string, sessionId: string) => void;
  onError?: (msg: string) => void;
};

const sharedManager = new RtcCallSessionManager();

/** App-wide 1:1 call manager (IM, contacts, group chat). */
export function getRtcCallManager(): RtcCallSessionManager {
  return sharedManager;
}

export type CallPhaseListener = (phase: CallUiPhase) => void;

/**
 * Manages one active WebRTC call for the whole app.
 */
export class RtcCallSessionManager {
  private phaseListeners = new Set<CallPhaseListener>();
  phase: CallUiPhase = 'idle';
  sessionId = '';
  remoteNode = '';
  remoteName = '';
  nodeInfo: VideoRtcNodeInfo | null = null;
  private call: RtcOneToOneCall | null = null;
  private unlistenIncoming: UnlistenFn | null = null;

  subscribePhase(listener: CallPhaseListener): () => void {
    this.phaseListeners.add(listener);
    listener(this.phase);
    return () => this.phaseListeners.delete(listener);
  }

  private setPhase(phase: CallUiPhase, cb?: CallSessionCallbacks): void {
    this.phase = phase;
    cb?.onPhase?.(phase);
    for (const fn of this.phaseListeners) fn(phase);
  }

  async init(): Promise<VideoRtcNodeInfo> {
    try {
      this.nodeInfo = await videoRtcServiceStart();
    } catch {
      this.nodeInfo = await videoRtcNodeInfo();
    }
    return this.nodeInfo;
  }

  async listenIncoming(cb: CallSessionCallbacks): Promise<void> {
    this.unlistenIncoming?.();
    this.unlistenIncoming = await listen<{
      sessionId: string;
      fromNode: string;
      displayName?: string;
    }>('exodus-rtc-incoming-call', (ev) => {
      if (this.phase !== 'idle') return;
      this.sessionId = ev.payload.sessionId;
      this.remoteNode = ev.payload.fromNode;
      this.remoteName = ev.payload.displayName ?? ev.payload.fromNode;
      cb.onIncoming?.(this.remoteName, this.sessionId);
      this.setPhase('ringing', cb);
    });
  }

  dispose(): void {
    this.unlistenIncoming?.();
    this.unlistenIncoming = null;
  }

  /** Outgoing voice or video call to a contact node id. */
  async startOutgoing(
    remoteNode: string,
    remoteName: string,
    video: boolean,
    audio: boolean,
    cb: CallSessionCallbacks
  ): Promise<void> {
    if (!this.nodeInfo) await this.init();
    const info = this.nodeInfo!;
    this.remoteNode = remoteNode;
    this.remoteName = remoteName;
    this.setPhase('connecting', cb);
    try {
      const session = await videoRtcCallStart(remoteNode, remoteName, video, audio);
      this.sessionId = session.sessionId;
      this.call = new RtcOneToOneCall(this.sessionId, info.nodeId, remoteNode, true, {
        onLocalStream: (s) => cb.onLocalStream?.(s),
        onRemoteStream: (s) => cb.onRemoteStream?.(s),
        onStateChange: (st) => {
          if (st === 'connected') {
            this.setPhase('connected', cb);
            void videoRtcCallUpdate(this.sessionId, 'connected');
          }
        },
        onError: (e) => {
          this.setPhase('error', cb);
          cb.onError?.(e);
        },
      });
      await this.call.start(video, audio);
      this.setPhase('ringing', cb);
    } catch (e) {
      this.setPhase('error', cb);
      cb.onError?.(String(e));
    }
  }

  async acceptIncoming(
    video: boolean,
    audio: boolean,
    cb: CallSessionCallbacks
  ): Promise<void> {
    if (!this.nodeInfo) await this.init();
    const info = this.nodeInfo!;
    this.phase = 'connecting';
    cb.onPhase?.('connecting');
    this.call = new RtcOneToOneCall(this.sessionId, info.nodeId, this.remoteNode, false, {
      onLocalStream: (s) => cb.onLocalStream?.(s),
      onRemoteStream: (s) => cb.onRemoteStream?.(s),
      onStateChange: (st) => {
        if (st === 'connected') {
          this.phase = 'connected';
          cb.onPhase?.('connected');
          void videoRtcCallUpdate(this.sessionId, 'connected');
        }
      },
      onError: (e) => {
        this.phase = 'error';
        cb.onPhase?.('error');
        cb.onError?.(e);
      },
    });
    await this.call.accept();
    await this.call.start(video, audio);
  }

  async hangup(cb: CallSessionCallbacks): Promise<void> {
    await this.call?.stop(false);
    this.call = null;
    if (this.sessionId) {
      await videoRtcCallUpdate(this.sessionId, 'ended').catch(() => {});
    }
    cb.onLocalStream?.(null);
    cb.onRemoteStream?.(null);
    this.setPhase('idle', cb);
    this.sessionId = '';
  }
}
