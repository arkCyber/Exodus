<!--
  Exodus Browser — multi-party meeting room (mesh WebRTC).
-->
<template>
  <div class="meeting-room">
    <header>
      <h2>Meeting</h2>
      <p class="hint">{{ statusMessage }}</p>
      <p v-if="nodeInfo" class="node-id" :title="nodeInfo.nodeId">
        Host ID: {{ nodeInfo.nodeId.slice(0, 20) }}…
      </p>
    </header>

    <template v-if="!inMeeting">
      <section class="create">
        <input v-model="meetingTitle" type="text" placeholder="Meeting title" />
        <button type="button" class="btn btn-primary" @click="() => void createMeeting()">Create room</button>
      </section>
      <section class="join">
        <input v-model="joinId" type="text" placeholder="Meeting ID (mtg-…)" />
        <button type="button" class="btn btn-secondary" @click="() => void joinMeeting()">Join</button>
      </section>
      <section class="list">
        <h3>Active rooms</h3>
        <p v-if="rooms.length === 0" class="empty">No active meetings</p>
        <div v-for="room in rooms" :key="room.meetingId" class="room-card">
          <strong>{{ room.title }}</strong>
          <span>{{ room.meetingId }}</span>
          <span>{{ room.participants.length }}/{{ room.maxParticipants }}</span>
          <button
            type="button"
            class="btn btn-secondary"
            @click="
              () => {
                joinId = room.meetingId;
                void joinMeeting();
              }
            "
          >
            Join
          </button>
        </div>
      </section>
    </template>

    <template v-else>
      <div class="conference">
        <video ref="localVideoEl" class="tile local" autoplay muted playsinline />
        <div v-for="nodeId in remoteNodeIds" :key="nodeId" class="remote-wrap">
          <video
            :ref="(el) => bindRemoteVideo(el as HTMLVideoElement | null, nodeId)"
            class="tile remote"
            autoplay
            playsinline
          />
          <span class="label">{{ nodeId.slice(0, 12) }}…</span>
        </div>
      </div>
      <button type="button" class="btn btn-danger" @click="() => void leaveMeeting()">Leave meeting</button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { RtcMeetingMesh } from '$lib/webrtc/rtcMeeting';
import {
  videoRtcMeetingCreate,
  videoRtcMeetingJoin,
  videoRtcMeetingLeave,
  videoRtcMeetingList,
  videoRtcNodeInfo,
  videoRtcServiceStart,
  type RtcMeetingRoom,
  type VideoRtcNodeInfo,
} from '$lib/videoRtc';

const emit = defineEmits<{ status: [message: string] }>();

const nodeInfo = ref<VideoRtcNodeInfo | null>(null);
const rooms = ref<RtcMeetingRoom[]>([]);
const activeRoom = ref<RtcMeetingRoom | null>(null);
const meetingTitle = ref('');
const joinId = ref('');
const statusMessage = ref('');
const inMeeting = ref(false);

const localVideoEl = ref<HTMLVideoElement | null>(null);
const remoteStreams = ref<Record<string, MediaStream>>({});
const remoteVideoEls = new Map<string, HTMLVideoElement>();

let mesh: RtcMeetingMesh | null = null;
let unlistenMeeting: UnlistenFn | null = null;

const remoteNodeIds = computed(() => Object.keys(remoteStreams.value));

function attachLocal(stream: MediaStream | null): void {
  if (localVideoEl.value) localVideoEl.value.srcObject = stream;
}

function bindRemoteVideo(el: HTMLVideoElement | null, nodeId: string): void {
  if (!el) {
    remoteVideoEls.delete(nodeId);
    return;
  }
  remoteVideoEls.set(nodeId, el);
  const stream = remoteStreams.value[nodeId];
  if (stream) el.srcObject = stream;
}

function syncRemoteVideos(): void {
  for (const [nodeId, stream] of Object.entries(remoteStreams.value)) {
    const el = remoteVideoEls.get(nodeId);
    if (el) el.srcObject = stream;
  }
}

async function refresh(): Promise<void> {
  try {
    rooms.value = await videoRtcMeetingList();
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function createMeeting(): Promise<void> {
  if (!meetingTitle.value.trim()) return;
  try {
    activeRoom.value = await videoRtcMeetingCreate(meetingTitle.value.trim(), 6);
    statusMessage.value = `Created ${activeRoom.value.meetingId}`;
    emit('status', statusMessage.value);
    await enterMeeting(activeRoom.value.meetingId, true);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function joinMeeting(): Promise<void> {
  if (!joinId.value.trim()) return;
  try {
    activeRoom.value = await videoRtcMeetingJoin(joinId.value.trim());
    await enterMeeting(activeRoom.value.meetingId);
  } catch (e) {
    statusMessage.value = String(e);
    emit('status', statusMessage.value);
  }
}

async function enterMeeting(meetingId: string, skipJoin = false): Promise<void> {
  if (!nodeInfo.value) return;
  inMeeting.value = true;
  mesh = new RtcMeetingMesh(meetingId, nodeInfo.value.nodeId, {
    onLocalStream: attachLocal,
    onRemoteStream: (nodeId, stream) => {
      remoteStreams.value = { ...remoteStreams.value, [nodeId]: stream };
      syncRemoteVideos();
    },
    onParticipantLeft: (nodeId) => {
      const next = { ...remoteStreams.value };
      delete next[nodeId];
      remoteStreams.value = next;
      remoteVideoEls.delete(nodeId);
    },
    onError: (e) => {
      statusMessage.value = e;
      emit('status', e);
    },
  });
  await mesh.start(true, true);
  if (!skipJoin) {
    activeRoom.value = await videoRtcMeetingJoin(meetingId).catch(() => activeRoom.value);
  }
  statusMessage.value = `In meeting ${meetingId}`;
  emit('status', statusMessage.value);
}

async function leaveMeeting(): Promise<void> {
  await mesh?.leave();
  mesh = null;
  if (activeRoom.value) {
    await videoRtcMeetingLeave(activeRoom.value.meetingId).catch(() => {});
  }
  inMeeting.value = false;
  activeRoom.value = null;
  remoteStreams.value = {};
  remoteVideoEls.clear();
  attachLocal(null);
  statusMessage.value = 'Left meeting';
  emit('status', statusMessage.value);
  await refresh();
}

onMounted(() => {
  void (async () => {
    try {
      nodeInfo.value = await videoRtcServiceStart();
    } catch (e) {
      try {
        nodeInfo.value = await videoRtcNodeInfo();
      } catch (e2) {
        const err = e2 instanceof Error ? e2 : new Error(String(e2));
        if (err.message.includes('Tauri invoke not available')) {
          // Running in non-Tauri environment (dev/test)
          console.warn('[MeetingRoom] Tauri not available, using mock node info');
          nodeInfo.value = { nodeId: 'dev-mock-node', displayName: 'Dev User' };
        } else {
          throw e2;
        }
      }
    }
    await refresh();
    unlistenMeeting = await listen<RtcMeetingRoom>('exodus-rtc-meeting-update', async () => {
      if (activeRoom.value) rooms.value = await videoRtcMeetingList();
    });
  })();
});

onUnmounted(() => {
  unlistenMeeting?.();
  void leaveMeeting();
});
</script>

<style scoped>
.meeting-room {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 12px;
  min-height: 0;
  overflow-y: auto;
}

h2 {
  margin: 0;
  font-size: 15px;
}

.hint,
.node-id {
  font-size: 11px;
  color: #9ca3af;
  margin: 2px 0;
}

.create,
.join {
  display: flex;
  gap: 8px;
}

input[type='text'] {
  flex: 1;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #e0e0e0;
}

.room-card {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  padding: 8px;
  border: 1px solid #333;
  border-radius: 6px;
  margin-bottom: 6px;
}

.conference {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
}

.tile {
  width: 100%;
  height: 90px;
  background: #000;
  border-radius: 6px;
  object-fit: cover;
}

.remote-wrap {
  position: relative;
}

.label {
  font-size: 10px;
  color: #888;
}

.empty {
  color: #888;
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
</style>
