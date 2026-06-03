<script lang="ts">
  /**
   * Exodus Browser — multi-party meeting room (mesh WebRTC, WeChat-style conference).
   */
  import { onDestroy, onMount } from 'svelte';
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

  let nodeInfo: VideoRtcNodeInfo | null = null;
  let rooms: RtcMeetingRoom[] = [];
  let activeRoom: RtcMeetingRoom | null = null;
  let meetingTitle = '';
  let joinId = '';
  let statusMessage = '';
  let inMeeting = false;

  let localVideoEl: HTMLVideoElement | undefined;
  let remoteStreams: Record<string, MediaStream> = {};
  let mesh: RtcMeetingMesh | null = null;

  function bindStream(node: HTMLVideoElement, stream: MediaStream) {
    node.srcObject = stream;
    return {
      update(s: MediaStream) {
        node.srcObject = s;
      },
    };
  }
  let unlistenMeeting: UnlistenFn | null = null;

  function attachLocal(stream: MediaStream | null) {
    if (localVideoEl) localVideoEl.srcObject = stream;
  }


  async function refresh() {
    try {
      rooms = await videoRtcMeetingList();
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function createMeeting() {
    if (!meetingTitle.trim()) return;
    try {
      activeRoom = await videoRtcMeetingCreate(meetingTitle.trim(), 6);
      statusMessage = `Created ${activeRoom.meetingId}`;
      await enterMeeting(activeRoom.meetingId, true);
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function joinMeeting() {
    if (!joinId.trim()) return;
    try {
      activeRoom = await videoRtcMeetingJoin(joinId.trim());
      await enterMeeting(activeRoom.meetingId);
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function enterMeeting(meetingId: string, skipJoin = false) {
    if (!nodeInfo) return;
    inMeeting = true;
    mesh = new RtcMeetingMesh(meetingId, nodeInfo.nodeId, {
      onLocalStream: attachLocal,
      onRemoteStream: (nodeId, stream) => {
        remoteStreams = { ...remoteStreams, [nodeId]: stream };
      },
      onParticipantLeft: (nodeId) => {
        const next = { ...remoteStreams };
        delete next[nodeId];
        remoteStreams = next;
      },
      onError: (e) => {
        statusMessage = e;
      },
    });
    await mesh.start(true, true);
    if (!skipJoin) {
      activeRoom = await videoRtcMeetingJoin(meetingId).catch(() => activeRoom);
    }
    statusMessage = `In meeting ${meetingId}`;
  }

  async function leaveMeeting() {
    await mesh?.leave();
    mesh = null;
    if (activeRoom) {
      await videoRtcMeetingLeave(activeRoom.meetingId).catch(() => {});
    }
    inMeeting = false;
    activeRoom = null;
    attachLocal(null);
    statusMessage = 'Left meeting';
    await refresh();
  }

  onMount(() => {
    void (async () => {
      try {
        nodeInfo = await videoRtcServiceStart();
      } catch {
        nodeInfo = await videoRtcNodeInfo();
      }
      await refresh();
      unlistenMeeting = await listen<RtcMeetingRoom>('exodus-rtc-meeting-update', async () => {
        if (activeRoom) {
          rooms = await videoRtcMeetingList();
        }
      });
    })();
  });

  onDestroy(() => {
    unlistenMeeting?.();
    void leaveMeeting();
  });
</script>

<div class="meeting-room">
  <header>
    <h2>Meeting</h2>
    <p class="hint">{statusMessage}</p>
    {#if nodeInfo}
      <p class="node-id" title={nodeInfo.nodeId}>Host ID: {nodeInfo.nodeId.slice(0, 20)}…</p>
    {/if}
  </header>

  {#if !inMeeting}
    <section class="create">
      <input type="text" bind:value={meetingTitle} placeholder="Meeting title" />
      <button type="button" class="btn btn-primary" onclick={() => void createMeeting()}
        >Create room</button
      >
    </section>
    <section class="join">
      <input type="text" bind:value={joinId} placeholder="Meeting ID (mtg-…)" />
      <button type="button" class="btn btn-secondary" onclick={() => void joinMeeting()}
        >Join</button
      >
    </section>
    <section class="list">
      <h3>Active rooms</h3>
      {#if rooms.length === 0}
        <p class="empty">No active meetings</p>
      {:else}
        {#each rooms as room (room.meetingId)}
          <div class="room-card">
            <strong>{room.title}</strong>
            <span>{room.meetingId}</span>
            <span>{room.participants.length}/{room.maxParticipants}</span>
            <button
              type="button"
              class="btn btn-secondary"
              onclick={() => {
                joinId = room.meetingId;
                void joinMeeting();
              }}>Join</button
            >
          </div>
        {/each}
      {/if}
    </section>
  {:else}
    <div class="conference">
      <video bind:this={localVideoEl} class="tile local" autoplay muted playsinline></video>
      {#each Object.entries(remoteStreams) as [nodeId, stream] (nodeId)}
        <div class="remote-wrap">
          <video class="tile remote" autoplay playsinline use:bindStream={stream}></video>
          <span class="label">{nodeId.slice(0, 12)}…</span>
        </div>
      {/each}
    </div>
    <button type="button" class="btn btn-danger" onclick={() => void leaveMeeting()}
      >Leave meeting</button
    >
  {/if}
</div>

<style>
  .meeting-room {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  h2 {
    margin: 0;
    font-size: 16px;
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
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    color: #eee;
    border-radius: 4px;
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
    background: #555;
    color: #eee;
  }
  .btn-danger {
    background: #dc2626;
    color: #fff;
  }
  .room-card {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    padding: 8px;
    background: #333;
    border-radius: 6px;
    margin-bottom: 6px;
    font-size: 12px;
  }
  .conference {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 8px;
    min-height: 180px;
  }
  .tile {
    width: 100%;
    height: 120px;
    background: #000;
    border-radius: 6px;
    object-fit: cover;
  }
  .remote-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .label {
    font-size: 10px;
    color: #aaa;
  }
  .empty {
    color: #888;
    font-size: 12px;
  }
</style>
