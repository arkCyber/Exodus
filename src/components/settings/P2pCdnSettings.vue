<!--
  Exodus Browser — P2P CDN room panel (settings section).
-->
<template>
  <section class="settings-section" data-testid="p2p-cdn-settings">
    <h3>P2P CDN</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <label>
        Room ID
        <input v-model="roomId" type="text" class="field" data-testid="p2p-room-id" />
      </label>
      <div class="toolbar">
        <button type="button" class="nav-button secondary" :disabled="loading" @click="() => void refresh()" data-testid="p2p-refresh">Refresh feed</button>
        <button type="button" class="nav-button secondary" @click="showAnnounce = !showAnnounce" data-testid="p2p-announce-asset">Announce asset</button>
      </div>
      <p v-if="nodeId" class="hint">Node {{ nodeId.slice(0, 12) }}… · {{ feed?.assets?.length ?? 0 }} assets</p>
      <ul v-if="feed?.assets?.length" class="list">
        <li v-for="a in feed.assets" :key="a.contentHash" class="row">
          <span>{{ a.title }}</span>
          <button type="button" class="nav-button secondary" @click="() => void download(a)" data-testid="p2p-download">Download</button>
        </li>
      </ul>
      <div v-if="showAnnounce" class="announce">
        <input v-model="announceTitle" placeholder="Title" class="field" data-testid="p2p-announce-title" />
        <input v-model="announceHash" placeholder="Content hash (BLAKE3)" class="field" data-testid="p2p-announce-hash" />
        <input v-model="announceUrl" placeholder="HTTP URL (optional)" class="field" data-testid="p2p-announce-url" />
        <button type="button" class="nav-button" @click="() => void announce()" data-testid="p2p-publish">Publish</button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  p2pCdnJoinRoom,
  p2pCdnRoomFeed,
  p2pCdnStartMesh,
  p2pCdnSyncGossip,
  p2pCdnAnnounceGroupHot,
  p2pCdnDownload,
  type CdnAsset,
  type CdnRoomFeed,
} from '$lib/p2p/cdn';

const props = defineProps<{ roomId?: string }>();
const emit = defineEmits<{ status: [message: string] }>();

const roomId = ref(props.roomId ?? 'lobby');
const nodeId = ref('');
const feed = ref<CdnRoomFeed | null>(null);
const loading = ref(false);
const showAnnounce = ref(false);
const announceTitle = ref('');
const announceHash = ref('');
const announceUrl = ref('');

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    await p2pCdnJoinRoom(roomId.value);
    const info = await p2pCdnStartMesh();
    nodeId.value = info.nodeId;
    await p2pCdnSyncGossip(roomId.value);
    feed.value = await p2pCdnRoomFeed(roomId.value);
  } catch (error) {
    emit('status', 'P2P CDN feed failed');
  } finally {
    loading.value = false;
  }
}

async function download(asset: CdnAsset): Promise<void> {
  try {
    await p2pCdnDownload({
      roomId: asset.roomId,
      contentHash: asset.contentHash,
      title: asset.title,
      kind: asset.kind,
      httpUrl: asset.sourceUrl,
    });
    emit('status', `Downloaded ${asset.title}`);
    await refresh();
  } catch (error) {
    emit('status', 'Download failed');
  }
}

async function announce(): Promise<void> {
  if (!announceTitle.value.trim() || !announceHash.value.trim()) {
    emit('status', 'Title and content hash required');
    return;
  }
  try {
    await p2pCdnAnnounceGroupHot({
      groupId: roomId.value,
      title: announceTitle.value.trim(),
      contentHash: announceHash.value.trim(),
      kind: 'article',
      sizeBytes: 0,
      sourceUrl: announceUrl.value.trim() || undefined,
    });
    emit('status', 'Asset announced');
    showAnnounce.value = false;
    await refresh();
  } catch (error) {
    emit('status', 'Announce failed');
  }
}

onMounted(() => void refresh());
</script>

<style scoped>
/* Field/button look is unified by ChromeSettingsPage :deep(.settings-section) styles */
.list { list-style: none; padding: 0; margin: 0; max-height: 160px; overflow-y: auto; }
.row { display: flex; justify-content: space-between; align-items: center; gap: 8px; padding: 6px 0; font-size: 12px; }
.announce { display: flex; flex-direction: column; gap: 8px; margin-top: 8px; padding: 12px; }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
</style>
