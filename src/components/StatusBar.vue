<template>
  <div v-if="showBar" class="status-bar" role="status" aria-live="polite">
    <div class="badges">
      <span class="badge badge-network" :class="{ 'badge-network--online': isOnline, 'badge-network--offline': !isOnline }" :title="isOnline ? 'Browser is online' : 'Browser is offline'">
        {{ isOnline ? 'Online' : 'Offline' }}
      </span>
      <span v-if="aiModel" class="badge badge-model" :title="`Using AI model: ${aiModel}`">
        AI-Model: {{ aiModel }}
      </span>
      <span v-if="privateMode" class="badge badge-private" title="Private browsing — visits are not recorded">
        Private
      </span>
      <span v-if="httpsOnly" class="badge badge-https" title="HTTPS-only mode">
        HTTPS only
      </span>
      <span v-if="blockPopups" class="badge badge-popup" title="Popup windows are blocked">
        Popups blocked
      </span>
      <span v-if="privacyStats && privacyStats.trackers_blocked > 0" class="badge badge-trackers" title="Tracker requests blocked this session">
        {{ privacyStats.trackers_blocked }} trackers blocked
      </span>
      <span v-if="isAgentExecuting" class="badge badge-agent" :title="agentCommand ? `Agent executing: ${agentCommand}` : 'AI agent is executing'">
        {{ agentCommand ? `Agent: ${agentCommand.substring(0, 30)}${agentCommand.length > 30 ? '...' : ''}` : 'Agent running' }}
      </span>
      <span v-if="agentDomSummary && isAgentExecuting" class="badge badge-dom" :title="`DOM: ${agentDomSummary}`">
        {{ agentDomSummary.substring(0, 25) }}{{ agentDomSummary.length > 25 ? '...' : '' }}
      </span>
      <span v-if="agentLog.length > 0 && isAgentExecuting" class="badge badge-log" :title="`Last log: ${agentLog[agentLog.length - 1]}`">
        {{ agentLog[agentLog.length - 1].substring(0, 30) }}{{ agentLog[agentLog.length - 1].length > 30 ? '...' : '' }}
      </span>
    </div>
    <span v-if="message" class="message">{{ message }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface PrivacyStatsSummary {
  trackers_blocked: number;
  trackers_allowed: number;
  fingerprinting_blocked: number;
  fingerprinting_allowed: number;
}

interface Props {
  message?: string;
  privateMode?: boolean;
  httpsOnly?: boolean;
  blockPopups?: boolean;
  privacyStats?: PrivacyStatsSummary | null;
  isAgentExecuting?: boolean;
  aiModel?: string;
  isOnline?: boolean;
  agentCommand?: string;
  agentLog?: string[];
  agentDomSummary?: string;
}

const props = withDefaults(defineProps<Props>(), {
  message: '',
  privateMode: false,
  httpsOnly: false,
  blockPopups: false,
  privacyStats: null,
  isAgentExecuting: false,
  aiModel: '',
  isOnline: true,
  agentCommand: '',
  agentLog: () => [],
  agentDomSummary: '',
});

const showBar = computed(() => {
  return Boolean(props.message) ||
    props.privateMode ||
    props.httpsOnly ||
    props.blockPopups ||
    (props.privacyStats !== null && props.privacyStats.trackers_blocked > 0) ||
    props.isAgentExecuting ||
    props.aiModel;
});
</script>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 16px;
  font-size: 12px;
  color: #ffffff;
  background: var(--chrome-toolbar-bg, #dee1e6);
  border-top: 1px solid var(--chrome-divider, #dadce0);
  min-height: 24px;
}

@media (prefers-color-scheme: dark) {
  .status-bar {
    background: #2d2e30;
    border-color: #5f6368;
    color: #ffffff;
  }
}

.badges {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.badge {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.01em;
  border: 1px solid transparent;
  transition: background-color 0.15s ease;
  color: rgba(255, 255, 255, 0.9);
}

.badge-private {
  background: rgba(95, 99, 104, 0.12);
}

@media (prefers-color-scheme: dark) {
  .badge-private {
    background: rgba(255, 255, 255, 0.1);
  }
}

.badge-https {
  background: rgba(19, 115, 51, 0.12);
}

.badge-popup {
  background: rgba(176, 96, 0, 0.12);
}

.badge-trackers {
  background: rgba(19, 115, 51, 0.12);
}

.badge-agent {
  background: rgba(9, 105, 218, 0.12);
}

.badge-model {
  background: rgba(138, 43, 226, 0.12);
}

.badge-network {
  background: rgba(95, 99, 104, 0.12);
}

.badge-network--online {
  background: rgba(19, 115, 51, 0.12);
}

.badge-network--offline {
  background: rgba(217, 48, 37, 0.12);
}

.badge-dom {
  background: rgba(95, 99, 104, 0.08);
}

.badge-log {
  background: rgba(95, 99, 104, 0.08);
}

.message {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: rgba(255, 255, 255, 0.9);
}

@media (prefers-color-scheme: dark) {
  .message {
    color: rgba(255, 255, 255, 0.9);
  }
}
</style>
