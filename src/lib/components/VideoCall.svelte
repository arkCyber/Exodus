<script lang="ts">
  /**
   * Exodus Browser — 1:1 voice/video call (WeChat-style, WebRTC + P2P gossip signaling).
   */
  import { onDestroy, onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { RtcOneToOneCall } from '$lib/webrtc/rtcCall';
  import {
    videoRtcCallStart,
    videoRtcCallUpdate,
    videoRtcNodeInfo,
    videoRtcServiceStart,
    type VideoRtcNodeInfo,
  } from '$lib/videoRtc';

  type CallUiState = 'idle' | 'ringing' | 'connecting' | 'connected' | 'ended' | 'error';

  let nodeInfo: VideoRtcNodeInfo | null = null;
  let callState: CallUiState = 'idle';
  let sessionId = '';
  let remoteNode = '';
  let remoteName = '';
  let statusMessage = '';
  let duration = 0;
  let useVideo = true;
  let useAudio = true;
  let calleeInput = '';
  let showDial = false;
  let showIncoming = false;
  let incomingFrom = '';
  let incomingSession = '';

  let localVideoEl: HTMLVideoElement | undefined;
  let remoteVideoEl: HTMLVideoElement | undefined;
  let activeCall: RtcOneToOneCall | null = null;
  let timerId: ReturnType<typeof setInterval> | null = null;
  let unlistenIncoming: UnlistenFn | null = null;

  function attachStream(el: HTMLVideoElement | undefined, stream: MediaStream | null) {
    if (el) el.srcObject = stream;
  }

  async function initRtc() {
    try {
      nodeInfo = await videoRtcServiceStart();
    } catch {
      nodeInfo = await videoRtcNodeInfo();
    }
    statusMessage = `Node: ${nodeInfo?.nodeId?.slice(0, 20) ?? '?'}…`;
  }

  async function dialPeer() {
    if (!calleeInput.trim() || !nodeInfo) return;
    showDial = false;
    callState = 'connecting';
    try {
      const session = await videoRtcCallStart(
        calleeInput.trim(),
        calleeInput.trim(),
        useVideo,
        useAudio
      );
      sessionId = session.sessionId;
      remoteNode = session.calleeNode;
      remoteName = session.calleeName ?? remoteNode;
      activeCall = new RtcOneToOneCall(sessionId, nodeInfo.nodeId, remoteNode, true, {
        onLocalStream: (s) => attachStream(localVideoEl, s),
        onRemoteStream: (s) => attachStream(remoteVideoEl, s),
        onStateChange: (st) => {
          if (st === 'connected') {
            callState = 'connected';
            void videoRtcCallUpdate(sessionId, 'connected');
            startTimer();
          }
        },
        onError: (e) => {
          statusMessage = e;
          callState = 'error';
        },
      });
      await activeCall.start(useVideo, useAudio);
      callState = 'ringing';
      statusMessage = `Calling ${remoteName}…`;
    } catch (e) {
      callState = 'error';
      statusMessage = String(e);
    }
  }

  async function acceptIncoming() {
    if (!nodeInfo || !incomingSession) return;
    showIncoming = false;
    callState = 'connecting';
    sessionId = incomingSession;
    remoteNode = incomingFrom;
    remoteName = incomingFrom;
    try {
      activeCall = new RtcOneToOneCall(sessionId, nodeInfo.nodeId, remoteNode, false, {
        onLocalStream: (s) => attachStream(localVideoEl, s),
        onRemoteStream: (s) => attachStream(remoteVideoEl, s),
        onStateChange: (st) => {
          if (st === 'connected') {
            callState = 'connected';
            void videoRtcCallUpdate(sessionId, 'connected');
            startTimer();
          }
        },
        onError: (e) => {
          statusMessage = e;
          callState = 'error';
        },
      });
      await activeCall.accept();
      await activeCall.start(useVideo, useAudio);
      statusMessage = `In call with ${remoteName}`;
    } catch (e) {
      callState = 'error';
      statusMessage = String(e);
    }
  }

  function rejectIncoming() {
    showIncoming = false;
    callState = 'idle';
  }

  async function endCall() {
    await activeCall?.stop(false);
    activeCall = null;
    if (sessionId) await videoRtcCallUpdate(sessionId, 'ended').catch(() => {});
    stopTimer();
    attachStream(localVideoEl, null);
    attachStream(remoteVideoEl, null);
    callState = 'idle';
    sessionId = '';
    statusMessage = 'Call ended';
  }

  function startTimer() {
    stopTimer();
    duration = 0;
    timerId = setInterval(() => {
      duration += 1;
    }, 1000);
  }

  function stopTimer() {
    if (timerId) clearInterval(timerId);
    timerId = null;
  }

  function formatDuration(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  onMount(() => {
    void initRtc();
    void listen<{
      sessionId: string;
      fromNode: string;
      displayName?: string;
    }>('exodus-rtc-incoming-call', (ev) => {
      if (callState !== 'idle') return;
      incomingSession = ev.payload.sessionId;
      incomingFrom = ev.payload.displayName ?? ev.payload.fromNode;
      showIncoming = true;
    }).then((fn) => {
      unlistenIncoming = fn;
    });
  });

  onDestroy(() => {
    unlistenIncoming?.();
    void endCall();
  });
</script>

<div class="video-call">
  <header class="header">
    <h2>Call</h2>
    <p class="hint">{statusMessage}</p>
    {#if nodeInfo}
      <p class="node-id" title={nodeInfo.nodeId}>My ID: {nodeInfo.nodeId.slice(0, 24)}…</p>
    {/if}
    <div class="actions">
      {#if callState === 'idle'}
        <button type="button" class="btn btn-primary" onclick={() => (showDial = true)}>New call</button>
      {:else}
        <button type="button" class="btn btn-danger" onclick={() => void endCall()}>Hang up</button>
      {/if}
    </div>
  </header>

  <div class="stage">
    {#if callState === 'idle' && !showIncoming}
      <div class="idle">📞 Ready — share your node ID with peers on the same network</div>
    {:else if callState === 'connecting' || callState === 'ringing'}
      <div class="connecting">
        <div class="spinner"></div>
        <p>{callState === 'ringing' ? 'Ringing…' : 'Connecting…'}</p>
      </div>
    {:else if callState === 'connected'}
      <div class="video-grid">
        <video bind:this={localVideoEl} class="tile local" autoplay muted playsinline></video>
        <video bind:this={remoteVideoEl} class="tile remote" autoplay playsinline></video>
      </div>
      <div class="bar">
        <span>{remoteName}</span>
        <span>{formatDuration(duration)}</span>
        <button type="button" class="icon" onclick={() => (useAudio = !useAudio)} title="Mute"
          >{useAudio ? '🎤' : '🔇'}</button
        >
        <button type="button" class="icon" onclick={() => (useVideo = !useVideo)} title="Video"
          >{useVideo ? '📹' : '🚫'}</button
        >
      </div>
    {:else if callState === 'error'}
      <p class="error">{statusMessage}</p>
    {/if}
  </div>

  {#if showDial}
    <div
      class="overlay"
      role="button"
      tabindex="0"
      onclick={() => (showDial = false)}
      onkeydown={(e) => e.key === 'Escape' && (showDial = false)}
    >
      <div
        class="dialog"
        role="dialog"
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <h3>Call peer</h3>
        <label>
          Remote node ID
          <input type="text" bind:value={calleeInput} placeholder="exodus-…" />
        </label>
        <label><input type="checkbox" bind:checked={useVideo} /> Video</label>
        <label><input type="checkbox" bind:checked={useAudio} /> Audio</label>
        <div class="dialog-actions">
          <button type="button" class="btn btn-secondary" onclick={() => (showDial = false)}
            >Cancel</button
          >
          <button type="button" class="btn btn-primary" onclick={() => void dialPeer()}>Call</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showIncoming}
    <div class="overlay incoming">
      <div class="dialog">
        <h3>Incoming call</h3>
        <p>{incomingFrom}</p>
        <div class="dialog-actions">
          <button type="button" class="btn btn-danger" onclick={rejectIncoming}>Decline</button>
          <button type="button" class="btn btn-primary" onclick={() => void acceptIncoming()}
            >Accept</button
          >
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .video-call {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    min-height: 280px;
  }
  .header h2 {
    margin: 0;
    font-size: 16px;
  }
  .hint,
  .node-id {
    font-size: 11px;
    color: #9ca3af;
    margin: 2px 0;
  }
  .actions {
    margin-top: 6px;
  }
  .stage {
    flex: 1;
    background: #1a1a1a;
    border-radius: 8px;
    min-height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .idle,
  .connecting {
    text-align: center;
    color: #aaa;
  }
  .video-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    width: 100%;
    padding: 8px;
  }
  .tile {
    width: 100%;
    max-height: 160px;
    background: #000;
    border-radius: 6px;
    object-fit: cover;
  }
  .bar {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 8px;
    font-size: 12px;
  }
  .btn {
    padding: 6px 12px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
  }
  .btn-primary {
    background: #6366f1;
    color: #fff;
  }
  .btn-secondary {
    background: #444;
    color: #eee;
  }
  .btn-danger {
    background: #dc2626;
    color: #fff;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 900;
  }
  .dialog {
    background: #333;
    padding: 16px;
    border-radius: 8px;
    min-width: 280px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .dialog label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: #ccc;
  }
  .dialog input[type='text'] {
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    color: #eee;
    border-radius: 4px;
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }
  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #444;
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin: 0 auto 8px;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .error {
    color: #f87171;
  }
  .icon {
    background: #444;
    border: none;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
