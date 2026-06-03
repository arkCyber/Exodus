<script lang="ts">
  /**
   * Exodus Browser — floating call UI (voice/video) over IM or contacts.
   */
  import type { CallUiPhase } from '$lib/webrtc/rtcCallSession';

  type Props = {
    open: boolean;
    phase: CallUiPhase;
    peerName: string;
    durationSec?: number;
    localStream?: MediaStream | null;
    remoteStream?: MediaStream | null;
    useVideo?: boolean;
    useAudio?: boolean;
    onHangup: () => void;
    onToggleVideo?: () => void;
    onToggleAudio?: () => void;
    onAccept?: () => void;
    onReject?: () => void;
  };

  let {
    open,
    phase,
    peerName,
    durationSec = 0,
    localStream = null,
    remoteStream = null,
    useVideo = true,
    useAudio = true,
    onHangup,
    onToggleVideo,
    onToggleAudio,
    onAccept,
    onReject,
  }: Props = $props();

  let localVideoEl: HTMLVideoElement | undefined;
  let remoteVideoEl: HTMLVideoElement | undefined;

  $effect(() => {
    if (localVideoEl) localVideoEl.srcObject = localStream;
  });
  $effect(() => {
    if (remoteVideoEl) remoteVideoEl.srcObject = remoteStream;
  });

  function formatDuration(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

{#if open}
  <div class="call-overlay" role="dialog" aria-label="Active call">
    {#if phase === 'ringing' && onAccept && onReject}
      <div class="incoming-panel">
        <p class="ring-title">Incoming call</p>
        <p class="peer">{peerName}</p>
        <div class="row">
          <button type="button" class="btn reject" onclick={() => onReject?.()}>Decline</button>
          <button type="button" class="btn accept" onclick={() => onAccept?.()}>Accept</button>
        </div>
      </div>
    {:else}
      <div class="active-panel">
        <p class="peer">{peerName}</p>
        <p class="phase">
          {#if phase === 'connected'}
            {formatDuration(durationSec)}
          {:else if phase === 'ringing'}
            Ringing…
          {:else}
            {phase}
          {/if}
        </p>
        {#if phase === 'connected' || phase === 'connecting'}
          <div class="videos">
            <video bind:this={localVideoEl} class="tile" autoplay muted playsinline></video>
            <video bind:this={remoteVideoEl} class="tile" autoplay playsinline></video>
          </div>
        {/if}
        <div class="controls">
          {#if onToggleAudio}
            <button type="button" class="icon" onclick={onToggleAudio}>{useAudio ? '🎤' : '🔇'}</button>
          {/if}
          {#if onToggleVideo}
            <button type="button" class="icon" onclick={onToggleVideo}>{useVideo ? '📹' : '🚫'}</button>
          {/if}
          <button type="button" class="btn hangup" onclick={onHangup}>End</button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .call-overlay {
    position: fixed;
    inset: 0;
    z-index: 2000;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .incoming-panel,
  .active-panel {
    background: #2a2a2a;
    border-radius: 12px;
    padding: 24px;
    min-width: 280px;
    text-align: center;
  }
  .peer {
    font-size: 18px;
    font-weight: 600;
    margin: 8px 0;
  }
  .phase {
    color: #9ca3af;
    font-size: 14px;
  }
  .videos {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin: 16px 0;
  }
  .tile {
    width: 100%;
    height: 140px;
    background: #000;
    border-radius: 8px;
    object-fit: cover;
  }
  .row,
  .controls {
    display: flex;
    gap: 12px;
    justify-content: center;
    margin-top: 16px;
  }
  .btn {
    padding: 10px 20px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 600;
  }
  .accept {
    background: #22c55e;
    color: #fff;
  }
  .reject,
  .hangup {
    background: #dc2626;
    color: #fff;
  }
  .icon {
    background: #444;
    border: none;
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
  }
</style>
