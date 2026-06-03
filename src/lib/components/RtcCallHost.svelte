<script lang="ts">
  /**
   * Exodus Browser — global 1:1 call overlay (incoming calls on any P2P tab).
   */
  import { onDestroy, onMount } from 'svelte';
  import { IM_START_CALL_EVENT, openP2pTab, type ImStartCallDetail } from '$lib/imChat';
  import {
    getRtcCallManager,
    type CallUiPhase,
  } from '$lib/webrtc/rtcCallSession';
  import RtcCallOverlay from '$lib/components/RtcCallOverlay.svelte';

  type Props = {
    onStatus?: (message: string) => void;
  };

  let { onStatus = () => {} }: Props = $props();

  const callMgr = getRtcCallManager();
  let callPhase = $state<CallUiPhase>('idle');
  let callPeerName = $state('');
  let callDuration = $state(0);
  let localCallStream = $state<MediaStream | null>(null);
  let remoteCallStream = $state<MediaStream | null>(null);
  let useVideo = $state(true);
  let useAudio = $state(true);
  let timerId: ReturnType<typeof setInterval> | null = null;
  let unsubPhase: (() => void) | null = null;

  const callOpen = $derived(callPhase !== 'idle');

  function callCallbacks() {
    return {
      onPhase: (p: CallUiPhase) => {
        callPhase = p;
        if (p === 'connected') {
          callDuration = 0;
          if (timerId) clearInterval(timerId);
          timerId = setInterval(() => {
            callDuration += 1;
          }, 1000);
        }
        if (p === 'idle') {
          if (timerId) clearInterval(timerId);
          timerId = null;
          localCallStream = null;
          remoteCallStream = null;
        }
      },
      onLocalStream: (s: MediaStream | null) => {
        localCallStream = s;
      },
      onRemoteStream: (s: MediaStream | null) => {
        remoteCallStream = s;
      },
      onIncoming: (name: string) => {
        callPeerName = name;
        openP2pTab('im');
        onStatus(`Incoming call from ${name}`);
      },
      onError: (e: string) => onStatus(e),
    };
  }

  async function acceptCall() {
    await callMgr.acceptIncoming(useVideo, useAudio, callCallbacks());
  }

  async function rejectCall() {
    await callMgr.hangup(callCallbacks());
  }

  async function hangup() {
    await callMgr.hangup(callCallbacks());
  }

  function onStartCallEvent(ev: Event) {
    const d = (ev as CustomEvent<ImStartCallDetail>).detail;
    callPeerName = d.name;
    openP2pTab('im');
    void callMgr.startOutgoing(d.nodeId, d.name, d.video, d.audio, callCallbacks());
  }

  onMount(() => {
    void (async () => {
      await callMgr.init();
      await callMgr.listenIncoming(callCallbacks());
    })();
    unsubPhase = callMgr.subscribePhase((p) => {
      callPhase = p;
    });
    window.addEventListener(IM_START_CALL_EVENT, onStartCallEvent);
    return () => {
      window.removeEventListener(IM_START_CALL_EVENT, onStartCallEvent);
    };
  });

  onDestroy(() => {
    unsubPhase?.();
    if (timerId) clearInterval(timerId);
  });
</script>

<RtcCallOverlay
  open={callOpen}
  phase={callPhase}
  peerName={callPeerName}
  durationSec={callDuration}
  localStream={localCallStream}
  remoteStream={remoteCallStream}
  {useVideo}
  {useAudio}
  onHangup={() => void hangup()}
  onAccept={callPhase === 'ringing' && callMgr.sessionId ? () => void acceptCall() : undefined}
  onReject={callPhase === 'ringing' ? () => void rejectCall() : undefined}
/>
