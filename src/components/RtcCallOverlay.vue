<!--
  Exodus Browser — floating call UI (voice/video) over IM or contacts.
-->
<template>
  <div v-if="open" class="call-overlay" role="dialog" aria-label="Active call">
    <div v-if="phase === 'ringing' && onAccept && onReject" class="incoming-panel">
      <p class="ring-title">Incoming call</p>
      <p class="peer">{{ peerName }}</p>
      <div class="row">
        <button type="button" class="btn reject" @click="onReject?.()">Decline</button>
        <button type="button" class="btn accept" @click="onAccept?.()">Accept</button>
      </div>
    </div>
    <div v-else class="active-panel">
      <p class="peer">{{ peerName }}</p>
      <p class="phase">{{ phaseLabel }}</p>
      <div v-if="phase === 'connected' || phase === 'connecting'" class="videos">
        <video ref="localVideoEl" class="tile" autoplay muted playsinline />
        <video ref="remoteVideoEl" class="tile" autoplay playsinline />
      </div>
      <div class="controls">
        <button v-if="onToggleAudio" type="button" class="icon" @click="onToggleAudio">
          {{ useAudio ? '🎤' : '🔇' }}
        </button>
        <button v-if="onToggleVideo" type="button" class="icon" @click="onToggleVideo">
          {{ useVideo ? '📹' : '🚫' }}
        </button>
        <button type="button" class="btn hangup" @click="onHangup">End</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { CallUiPhase } from '$lib/webrtc/rtcCallSession';

const props = withDefaults(
  defineProps<{
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
  }>(),
  {
    durationSec: 0,
    localStream: null,
    remoteStream: null,
    useVideo: true,
    useAudio: true,
  },
);

const localVideoEl = ref<HTMLVideoElement | null>(null);
const remoteVideoEl = ref<HTMLVideoElement | null>(null);

watch(
  () => props.localStream,
  (stream) => {
    if (localVideoEl.value) localVideoEl.value.srcObject = stream ?? null;
  },
  { immediate: true },
);

watch(
  () => props.remoteStream,
  (stream) => {
    if (remoteVideoEl.value) remoteVideoEl.value.srcObject = stream ?? null;
  },
  { immediate: true },
);

function formatDuration(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

const phaseLabel = computed(() => {
  if (props.phase === 'connected') return formatDuration(props.durationSec);
  if (props.phase === 'ringing') return 'Ringing…';
  return props.phase;
});
</script>

<style scoped>
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
