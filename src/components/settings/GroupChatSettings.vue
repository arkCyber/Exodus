<!--
  Exodus Browser — group chat settings section (P2P room messaging).
-->
<template>
  <section class="settings-section" data-testid="group-chat-settings">
    <h3>Group chat</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <label>
        Group ID
        <input v-model="groupId" type="text" class="field" @change="() => void loadMessages()" data-testid="group-chat-id" />
      </label>
      <div class="messages">
        <p v-for="m in messages" :key="m.messageId" class="msg">
          <strong>{{ m.senderId.slice(0, 8) }}</strong>: {{ m.content }}
        </p>
        <p v-if="messages.length === 0" class="hint">No messages yet.</p>
      </div>
      <div class="toolbar">
        <input v-model="draft" type="text" class="field" placeholder="Message…" @keydown.enter="() => void send()" data-testid="group-chat-draft" />
        <button type="button" class="nav-button" :disabled="!draft.trim()" @click="() => void send()" data-testid="group-chat-send">Send</button>
        <button type="button" class="nav-button secondary" @click="() => void loadMessages()" data-testid="group-chat-refresh">Refresh</button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { groupChatServiceStart, groupGetMessages, type GroupMessage } from '$lib/groupChat';
import { sendGroupMessageWithCdn } from '$lib/p2p/cdnIntegrations';

const props = defineProps<{ groupId?: string }>();
const emit = defineEmits<{ status: [message: string] }>();

const groupId = ref(props.groupId ?? 'lobby');
const messages = ref<GroupMessage[]>([]);
const draft = ref('');
const userId = 'exodus-local-user';
const loading = ref(true);

async function ensureService(): Promise<void> {
  await groupChatServiceStart();
}

async function loadMessages(): Promise<void> {
  loading.value = true;
  try {
    await ensureService();
    messages.value = await groupGetMessages(groupId.value);
  } catch (error) {
    console.error('groupGetMessages failed:', error);
  } finally {
    loading.value = false;
  }
}

async function send(): Promise<void> {
  const text = draft.value.trim();
  if (!text) return;
  try {
    await ensureService();
    await sendGroupMessageWithCdn({
      messageId: crypto.randomUUID(),
      groupId: groupId.value,
      senderId: userId,
      senderName: 'Local user',
      content: text,
      messageType: 'text',
      attachments: [],
      mentions: [],
      timestamp: Math.floor(Date.now() / 1000),
      isEdited: false,
    });
    draft.value = '';
    await loadMessages();
    emit('status', 'Message sent');
  } catch (error) {
    emit('status', 'Send failed');
  }
}

onMounted(() => void loadMessages());
</script>

<style scoped>
/* Field/button look is unified by ChromeSettingsPage :deep(.settings-section) styles */
.msg { font-size: 12px; margin: 4px 0; }
.msg strong { color: var(--cs-title, #9aa0a6); font-weight: 500; }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
</style>
