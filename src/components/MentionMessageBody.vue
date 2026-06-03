<!--
  Exodus Browser — group message body with clickable @mentions (Vue 3).
-->
<template>
  <span class="mention-body">
    <template v-for="(seg, index) in segments" :key="index">
      <template v-if="seg.kind === 'text'">{{ seg.text }}</template>
      <span v-else class="mention-inline">
        <button
          type="button"
          class="mention-name"
          :title="`DM @${seg.displayName}`"
          @click="emit('mentionAction', { nodeId: seg.nodeId, displayName: seg.displayName }, 'chat')"
        >
          @{{ seg.displayName }}
        </button>
        <button
          type="button"
          class="mention-act"
          title="Voice"
          @click="emit('mentionAction', { nodeId: seg.nodeId, displayName: seg.displayName }, 'voice')"
        >
          📞
        </button>
        <button
          type="button"
          class="mention-act"
          title="Video"
          @click="emit('mentionAction', { nodeId: seg.nodeId, displayName: seg.displayName }, 'video')"
        >
          📹
        </button>
      </span>
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { splitMentionContent, type MentionTarget } from '$lib/groupMentions';

const props = defineProps<{ content: string }>();

const emit = defineEmits<{
  mentionAction: [target: MentionTarget, action: 'chat' | 'voice' | 'video'];
}>();

const segments = computed(() => splitMentionContent(props.content));
</script>

<style scoped>
.mention-body {
  white-space: pre-wrap;
  word-break: break-word;
}

.mention-inline {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  margin: 0 2px;
  vertical-align: baseline;
}

.mention-name {
  background: rgba(63, 63, 154, 0.33);
  border: none;
  color: #a5b4fc;
  font-weight: 600;
  padding: 0 4px;
  border-radius: 4px;
  cursor: pointer;
  font-size: inherit;
}

.mention-name:hover {
  background: #6366f1;
  color: #fff;
}

.mention-act {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 0 2px;
  opacity: 0.85;
}

.mention-act:hover {
  opacity: 1;
}
</style>
