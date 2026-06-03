<!--
  Exodus Browser — 1:1 voice/video call (WebRTC + P2P gossip signaling).
-->
<template>
  <div class="video-call">
    <header class="header">
      <h2>Call</h2>
      <p class="hint">{{ statusMessage }}</p>
      <p v-if="nodeInfo" class="node-id" :title="nodeInfo.nodeId">
        My ID: {{ nodeInfo.nodeId.slice(0, 24) }}…
      </p>
      <div class="actions">
        <button v-if="callState === 'idle'" type="button" class="btn btn-primary" @click="showDial = true">
          New call
        </button>
        <button v-else type="button" class="btn btn-danger" @click="() => void endCall()">Hang up</button>
      </div>
    </header>

    <div class="stage">
      <div v-if="callState === 'idle' && !showIncoming" class="idle">
        Ready — share your node ID with peers on the same network
      </div>
      <div v-else-if="callState === 'connecting' || callState === 'ringing'" class="connecting">
        <p>{{ callState === 'ringing' ? 'Ringing…' : 'Connecting…' }}</p>
      </div>
      <template v-else-if="callState === 'connected'">
        <div class="video-grid">
          <video ref="localVideoEl" class="tile local" autoplay muted playsinline />
          <video ref="remoteVideoEl" class="tile remote" autoplay playsinline />
        </div>
        <div class="bar">
          <span>{{ remoteName }}</span>
          <span>{{ formatDuration(duration) }}</span>
          <button type="button" class="icon" :title="useAudio ? 'Mute' : 'Unmute'" @click="useAudio = !useAudio">
            {{ useAudio ? '🎤' : '🔇' }}
          </button>
          <button type="button" class="icon" :title="useVideo ? 'Video' : 'No video'" @click="useVideo = !useVideo">
            {{ useVideo ? '📹' : '🚫' }}
          </button>
        </div>
      </template>
      <p v-else-if="callState === 'error'" class="error">{{ statusMessage }}</p>
    </div>

    <div v-if="showDial" class="overlay" @click.self="showDial = false">
      <div class="dialog" role="dialog" @click.stop>
        <h3>Call peer</h3>
        <label>
          Remote node ID
          <input v-model="calleeInput" type="text" placeholder="exodus-…" />
        </label>
        <label><input v-model="useVideo" type="checkbox" /> Video</label>
        <label><input v-model="useAudio" type="checkbox" /> Audio</label>
        <div class="dialog-actions">
          <button type="button" class="btn btn-secondary" @click="showDial = false">Cancel</button>
          <button type="button" class="btn btn-primary" @click="() => void dialPeer()">Call</button>
        </div>
      </div>
    </div>

    <div v-if="showIncoming" class="overlay incoming">
      <div class="dialog">
        <h3>Incoming call</h3>
        <p>{{ incomingFrom }}</p>
        <div class="dialog-actions">
          <button type="button" class="btn btn-danger" @click="rejectIncoming">Decline</button>
          <button type="button" class="btn btn-primary" @click="() => void acceptIncoming()">Accept</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
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

const emit = defineEmits<{ status: [message: string] }>();

const nodeInfo = ref<VideoRtcNodeInfo | null>(null);
const callState = ref<CallUiState>('idle');
const sessionId = ref('');
const remoteNode = ref('');
const remoteName = ref('');
const statusMessage = ref('');
const duration = ref(0);
const useVideo = ref(true);
const useAudio = ref(true);
const calleeInput = ref('');
const showDial = ref(false);
const showIncoming = ref(false);
const incomingFrom = ref('');
const incomingSession = ref('');

const localVideoEl = ref<HTMLVideoElement | null>(null);
const remoteVideoEl = ref<HTMLVideoElement | null>(null);
let activeCall: RtcOneToOneCall | null = null;
let timerId: ReturnType<typeof setInterval> | null = null;
let unlistenIncoming: UnlistenFn | null = null;

function attachStream(el: HTMLVideoElement | null | undefined, stream: MediaStream | null): void {
  if (el) el.srcObject = stream;
}

async function initRtc(): Promise<void> {
  try {
    nodeInfo.value = await videoRtcServiceStart();
  } catch (e) {
    try {
      nodeInfo.value = await videoRtcNodeInfo();
    } catch (e2) {
      const err = e2 instanceof Error ? e2 : new Error(String(e2));
      if (err.message.includes('Tauri invoke not available')) {
        // Running in non-Tauri environment (dev/test)
        console.warn('[VideoCall] Tauri not available, using mock node info');
        nodeInfo.value = { nodeId: 'dev-mock-node', displayName: 'Dev User' };
      } else {
        throw e2;
      }
    }
  }
  statusMessage.value = `Node: ${nodeInfo.value?.nodeId?.slice(0, 20) ?? '?'}…`;
}

async function dialPeer(): Promise<void> {
  if (!calleeInput.value.trim() || !nodeInfo.value) return;
  showDial.value = false;
  callState.value = 'connecting';
  try {
    const session = await videoRtcCallStart(
      calleeInput.value.trim(),
      calleeInput.value.trim(),
      useVideo.value,
      useAudio.value,
    );
    sessionId.value = session.sessionId;
    remoteNode.value = session.calleeNode;
    remoteName.value = session.calleeName ?? remoteNode.value;
    activeCall = new RtcOneToOneCall(sessionId.value, nodeInfo.value.nodeId, remoteNode.value, true, {
      onLocalStream: (s) => attachStream(localVideoEl.value, s),
      onRemoteStream: (s) => attachStream(remoteVideoEl.value, s),
      onStateChange: (st) => {
        if (st === 'connected') {
          callState.value = 'connected';
          void videoRtcCallUpdate(sessionId.value, 'connected');
          startTimer();
        }
      },
      onError: (e) => {
        statusMessage.value = e;
        callState.value = 'error';
        emit('status', e);
      },
    });
    await activeCall.start(useVideo.value, useAudio.value);
    callState.value = 'ringing';
    statusMessage.value = `Calling ${remoteName.value}…`;
    emit('status', statusMessage.value);
  } catch (e) {
    callState.value = 'error';
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function acceptIncoming(): Promise<void> {
  if (!nodeInfo.value || !incomingSession.value) return;
  showIncoming.value = false;
  callState.value = 'connecting';
  sessionId.value = incomingSession.value;
  remoteNode.value = incomingFrom.value;
  remoteName.value = incomingFrom.value;
  try {
    activeCall = new RtcOneToOneCall(sessionId.value, nodeInfo.value.nodeId, remoteNode.value, false, {
      onLocalStream: (s) => attachStream(localVideoEl.value, s),
      onRemoteStream: (s) => attachStream(remoteVideoEl.value, s),
      onStateChange: (st) => {
        if (st === 'connected') {
          callState.value = 'connected';
          void videoRtcCallUpdate(sessionId.value, 'connected');
          startTimer();
        }
      },
      onError: (e) => {
        statusMessage.value = e;
        callState.value = 'error';
        emit('status', e);
      },
    });
    await activeCall.accept();
    await activeCall.start(useVideo.value, useAudio.value);
    statusMessage.value = `In call with ${remoteName.value}`;
    emit('status', statusMessage.value);
  } catch (e) {
    callState.value = 'error';
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

function rejectIncoming(): void {
  showIncoming.value = false;
  callState.value = 'idle';
}

async function endCall(): Promise<void> {
  await activeCall?.stop(false);
  activeCall = null;
  if (sessionId.value) await videoRtcCallUpdate(sessionId.value, 'ended').catch(() => {});
  stopTimer();
  attachStream(localVideoEl.value, null);
  attachStream(remoteVideoEl.value, null);
  callState.value = 'idle';
  sessionId.value = '';
  statusMessage.value = 'Call ended';
  emit('status', statusMessage.value);
}

function startTimer(): void {
  stopTimer();
  duration.value = 0;
  timerId = setInterval(() => {
    duration.value += 1;
  }, 1000);
}

function stopTimer(): void {
  if (timerId) clearInterval(timerId);
  timerId = null;
}

function formatDuration(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

onMounted(() => {
  void initRtc();
  void listen<{ sessionId: string; fromNode: string; displayName?: string }>(
    'exodus-rtc-incoming-call',
    (ev) => {
      if (callState.value !== 'idle') return;
      incomingSession.value = ev.payload.sessionId;
      incomingFrom.value = ev.payload.displayName ?? ev.payload.fromNode;
      showIncoming.value = true;
    },
  ).then((fn) => {
    unlistenIncoming = fn;
  });
});

onUnmounted(() => {
  unlistenIncoming?.();
  void endCall();
});
</script>

<style scoped>
.video-call {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 8px;
  min-height: 240px;
}

.header h2 {
  margin: 0;
  font-size: 15px;
}

.hint,
.node-id {
  font-size: 11px;
  color: #9ca3af;
  margin: 2px 0;
}

.stage {
  flex: 1;
  background: #1a1a1a;
  border-radius: 8px;
  min-height: 160px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.idle,
.connecting {
  color: #9ca3af;
  font-size: 13px;
}

.video-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  width: 100%;
  padding: 8px;
}

.tile {
  width: 100%;
  max-height: 120px;
  background: #000;
  border-radius: 6px;
  object-fit: cover;
}

.bar {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 6px;
  font-size: 12px;
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.dialog {
  background: #292a2d;
  padding: 16px;
  border-radius: 8px;
  min-width: 260px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dialog label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.dialog input[type='text'] {
  padding: 6px 8px;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #e0e0e0;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn {
  padding: 6px 12px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 12px;
}

.btn-primary {
  background: #6366f1;
  color: #fff;
}

.btn-secondary {
  background: #444;
  color: #e0e0e0;
}

.btn-danger {
  background: #dc2626;
  color: #fff;
}

.error {
  color: #f87171;
}
</style>
