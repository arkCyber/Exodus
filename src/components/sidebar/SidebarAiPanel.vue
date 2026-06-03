<!--
  Exodus Browser — sidebar AI chat panel.
-->
<template>
  <div class="ai-chat-panel">
    <div v-if="aiChatHistory.length > 0 || chatStreamBuffer" class="ai-chat-messages">
      <div
        v-for="(msg, i) in aiChatHistory"
        :key="i"
        class="chat-bubble"
        :class="msg.role"
      >
        {{ msg.content }}
      </div>
      <div v-if="isLoading && aiStreamMode === 'chat' && chatStreamBuffer" class="chat-bubble assistant streaming">
        {{ chatStreamBuffer }}
      </div>
    </div>
    <div v-else-if="!isLoading" class="sidebar-placeholder">
      <p>Ask Exodus anything, or select text on the page for AI Summary.</p>
      <button type="button" class="nav-button secondary full" @click="emit('toggle-agent')">Open Agent</button>
      <button
        v-if="canAnnouncePage"
        type="button"
        class="nav-button secondary full"
        @click="emit('open-p2p')"
      >
        Open P2P room
      </button>
    </div>

    <form class="ai-chat-form" @submit.prevent="emit('send-chat')">
      <input
        type="text"
        class="ai-chat-input"
        :value="aiChatInput"
        :placeholder="aiOnline ? 'Ask Exodus…' : 'AI offline — check settings'"
        :disabled="isLoading"
        @input="(e) => emit('chat-input', (e.target as HTMLInputElement).value)"
      />
      <button v-if="isLoading" type="button" class="nav-button secondary" @click="emit('cancel-chat')">
        Stop
      </button>
      <button v-else type="submit" class="nav-button" :disabled="isLoading || !aiChatInput.trim()">
        Send
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
import type { AiChatMessage } from '$lib/browserTypes';

defineProps<{
  aiChatHistory: AiChatMessage[];
  chatStreamBuffer: string;
  aiStreamMode: 'none' | 'chat' | 'summary';
  isLoading: boolean;
  aiOnline: boolean;
  aiChatInput: string;
  canAnnouncePage: boolean;
}>();

const emit = defineEmits<{
  'send-chat': [];
  'cancel-chat': [];
  'toggle-agent': [];
  'open-p2p': [];
  'chat-input': [value: string];
  navigate: [url: string];
}>();
</script>

<style scoped>
.ai-chat-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: 100%;
  min-height: 200px;
}

.ai-chat-messages {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chat-bubble {
  padding: 8px 12px;
  border-radius: 12px;
  font-size: 13px;
  max-width: 95%;
}

.chat-bubble.user {
  align-self: flex-end;
  background: #6366f1;
  color: #fff;
}

.chat-bubble.assistant {
  align-self: flex-start;
  background: #2d2d30;
  color: #e8eaed;
}

.ai-chat-form {
  display: flex;
  gap: 6px;
}

.ai-chat-input {
  flex: 1;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid #404040;
  background: #1a1a1a;
  color: #e0e0e0;
}

.nav-button {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid #6366f1;
  background: rgba(99, 102, 241, 0.35);
  color: #fff;
  cursor: pointer;
}

.nav-button.secondary {
  border-color: #555;
  background: #333;
}

.sidebar-placeholder {
  flex: 1;
  color: #9aa0a6;
  font-size: 13px;
}

.full {
  width: 100%;
  margin-top: 6px;
}
</style>
