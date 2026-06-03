<!--
  Exodus Browser — global 1:1 call overlay (incoming/outgoing from any WebChat surface).
-->
<template>
  <RtcCallOverlay
    :open="callOpen"
    :phase="callPhase"
    :peer-name="callPeerName"
    :duration-sec="callDuration"
    :local-stream="localCallStream"
    :remote-stream="remoteCallStream"
    :use-video="useVideo"
    :use-audio="useAudio"
    :on-hangup="() => void hangup()"
    :on-accept="callPhase === 'ringing' && callMgr.sessionId ? () => void acceptCall() : undefined"
    :on-reject="callPhase === 'ringing' ? () => void rejectCall() : undefined"
  />
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import RtcCallOverlay from '@/components/RtcCallOverlay.vue';
import { IM_START_CALL_EVENT, openP2pTab, openWebChat, type ImStartCallDetail } from '$lib/imChat';
import {
  getRtcCallManager,
  type CallSessionCallbacks,
  type CallUiPhase,
} from '$lib/webrtc/rtcCallSession';

const emit = defineEmits<{ status: [message: string] }>();

const callMgr = getRtcCallManager();
const callPhase = ref<CallUiPhase>('idle');
const callPeerName = ref('');
const callDuration = ref(0);
const localCallStream = ref<MediaStream | null>(null);
const remoteCallStream = ref<MediaStream | null>(null);
const useVideo = ref(true);
const useAudio = ref(true);

let timerId: ReturnType<typeof setInterval> | null = null;
let unsubPhase: (() => void) | null = null;

const callOpen = computed(() => callPhase.value !== 'idle');

function callCallbacks(): CallSessionCallbacks {
  return {
    onPhase: (phase) => {
      callPhase.value = phase;
      if (phase === 'connected') {
        callDuration.value = 0;
        if (timerId) clearInterval(timerId);
        timerId = setInterval(() => {
          callDuration.value += 1;
        }, 1000);
      }
      if (phase === 'idle') {
        if (timerId) clearInterval(timerId);
        timerId = null;
        localCallStream.value = null;
        remoteCallStream.value = null;
      }
    },
    onLocalStream: (stream) => {
      localCallStream.value = stream;
    },
    onRemoteStream: (stream) => {
      remoteCallStream.value = stream;
    },
    onIncoming: (name) => {
      callPeerName.value = name;
      openP2pTab('im');
      openWebChat();
      emit('status', `Incoming call from ${name}`);
    },
    onError: (message) => {
      emit('status', message);
    },
  };
}

async function acceptCall(): Promise<void> {
  await callMgr.acceptIncoming(useVideo.value, useAudio.value, callCallbacks());
}

async function rejectCall(): Promise<void> {
  await callMgr.hangup(callCallbacks());
}

async function hangup(): Promise<void> {
  await callMgr.hangup(callCallbacks());
}

function onStartCallEvent(ev: Event): void {
  const detail = (ev as CustomEvent<ImStartCallDetail>).detail;
  if (!detail?.nodeId) return;
  callPeerName.value = detail.name;
  useVideo.value = detail.video;
  useAudio.value = detail.audio;
  openP2pTab('im');
  openWebChat();
  void callMgr.startOutgoing(detail.nodeId, detail.name, detail.video, detail.audio, callCallbacks());
}

onMounted(() => {
  void (async () => {
    try {
      await callMgr.init();
    } catch (e) {
      console.error('[RtcCallHost] Failed to initialize RTC manager:', e);
      // Continue anyway - call features will be disabled
    }
    try {
      await callMgr.listenIncoming(callCallbacks());
    } catch (e) {
      console.error('[RtcCallHost] Failed to listen for incoming calls:', e);
      // Continue anyway - call features will be disabled
    }
  })();
  unsubPhase = callMgr.subscribePhase((phase) => {
    callPhase.value = phase;
  });
  window.addEventListener(IM_START_CALL_EVENT, onStartCallEvent);
});

onUnmounted(() => {
  window.removeEventListener(IM_START_CALL_EVENT, onStartCallEvent);
  unsubPhase?.();
  if (timerId) clearInterval(timerId);
});
</script>
