<!--
  Exodus Browser — sidebar P2P hub (IM, contacts, group, CDN, workspace, calls, collab).
-->
<template>
  <div class="p2p-sidebar">
    <div class="sub-tabs" role="tablist">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        type="button"
        role="tab"
        :class="{ active: subTab === tab.id }"
        @click="subTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="panel-body">
      <ImMessenger v-if="subTab === 'im'" @status="forwardStatus" />
      <ContactDirectoryPanel v-else-if="subTab === 'contacts'" @status="forwardStatus" />
      <GroupChatSettings
        v-else-if="subTab === 'chat'"
        :group-id="roomId"
        @status="forwardStatus"
      />
      <P2pCdnSettings v-else-if="subTab === 'cdn'" />
      <FileTransfer v-else-if="subTab === 'workspace'" @status="forwardStatus" />
      <CollaborativeEditing v-else-if="subTab === 'collab'" @status="forwardStatus" />
      <VideoCall v-else-if="subTab === 'call'" @status="forwardStatus" />
      <MeetingRoom v-else-if="subTab === 'meeting'" @status="forwardStatus" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { P2P_TAB_EVENT, type P2pSidebarTab } from '$lib/imChat';
import { logInfo, logDebug } from '@/lib/logger';
import ImMessenger from '@/components/ImMessenger.vue';
import ContactDirectoryPanel from '@/components/ContactDirectoryPanel.vue';
import GroupChatSettings from '@/components/settings/GroupChatSettings.vue';
import P2pCdnSettings from '@/components/settings/P2pCdnSettings.vue';
import FileTransfer from '@/components/FileTransfer.vue';
import CollaborativeEditing from '@/components/CollaborativeEditing.vue';
import VideoCall from '@/components/VideoCall.vue';
import MeetingRoom from '@/components/MeetingRoom.vue';

const props = defineProps<{ roomId?: string }>();
const emit = defineEmits<{ status: [message: string] }>();

const roomId = ref(props.roomId ?? 'lobby');
const subTab = ref<P2pSidebarTab>('im');

const tabs: { id: P2pSidebarTab; label: string }[] = [
  { id: 'im', label: 'WebChat' },
  { id: 'contacts', label: 'Contacts' },
  { id: 'chat', label: 'Group' },
  { id: 'cdn', label: 'CDN' },
  { id: 'workspace', label: 'WorkSpace' },
  { id: 'collab', label: 'Collab' },
  { id: 'call', label: 'Call' },
  { id: 'meeting', label: 'Meeting' },
];

let unsubs: UnlistenFn[] = [];

function forwardStatus(msg: string): void {
  emit('status', msg);
}

function onP2pTabEvent(ev: Event): void {
  const tab = (ev as CustomEvent<P2pSidebarTab>).detail;
  if (tab) subTab.value = tab;
}

onMounted(() => {
  window.addEventListener(P2P_TAB_EVENT, onP2pTabEvent);
  logDebug('P2pSidebarPanel', 'Component mounted, setting up event listeners');
  void (async () => {
    unsubs.push(
      await listen('exodus-focus-im', () => {
        logInfo('P2pSidebarPanel', 'exodus-focus-im event received');
        subTab.value = 'im';
      }),
      await listen('exodus-open-webchat', () => {
        logInfo('P2pSidebarPanel', 'exodus-open-webchat event received');
        subTab.value = 'im';
      }),
    );
    unsubs.push(
      await listen('exodus-focus-workspace', () => {
        logInfo('P2pSidebarPanel', 'exodus-focus-workspace event received');
        subTab.value = 'workspace';
      }),
    );
  })();
});

onUnmounted(() => {
  window.removeEventListener(P2P_TAB_EVENT, onP2pTabEvent);
  for (const u of unsubs) u();
  unsubs = [];
  logDebug('P2pSidebarPanel', 'Component unmounted, event listeners cleaned up');
});
</script>

<style scoped>
.p2p-sidebar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
  flex: 1;
  height: 100%;
}

.sub-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.sub-tabs button {
  flex: 1 1 auto;
  min-width: 52px;
  padding: 6px 6px;
  font-size: 11px;
  background: #333;
  border: 1px solid #444;
  border-radius: 6px;
  color: #ccc;
  cursor: pointer;
}

.sub-tabs button.active {
  background: #6366f1;
  border-color: #6366f1;
  color: #fff;
}

.panel-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
