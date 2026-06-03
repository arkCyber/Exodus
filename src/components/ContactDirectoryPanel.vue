<!--
  Exodus Browser — contact directory panel (list, add, open IM / call).
-->
<template>
  <div class="contact-directory">
    <div class="toolbar">
      <input v-model="searchQuery" type="search" class="field" placeholder="Search contacts…" />
      <button type="button" class="btn btn-primary" @click="showAdd = true">Add</button>
    </div>
    <p v-if="statusMessage" class="hint">{{ statusMessage }}</p>
    <ul class="contact-list">
      <li v-for="c in filteredContacts" :key="c.node_id" class="contact-row">
        <div class="info" @click="openChat(c)">
          <strong>{{ c.name }}</strong>
          <span class="node">{{ c.node_id.slice(0, 16) }}…</span>
          <span v-if="isOnline(c.node_id)" class="online">online</span>
        </div>
        <div class="actions">
          <button type="button" class="btn btn-secondary" @click="openChat(c)">Chat</button>
          <button type="button" class="btn btn-secondary" @click="voiceCall(c)">Call</button>
        </div>
      </li>
      <li v-if="filteredContacts.length === 0" class="muted">No contacts</li>
    </ul>

    <div v-if="showAdd" class="dialog-overlay" @click.self="showAdd = false">
      <div class="dialog" @click.stop>
        <h3>Add contact</h3>
        <input v-model="addName" class="field" placeholder="Name" />
        <input v-model="addNode" class="field" placeholder="Node ID" />
        <div class="dialog-actions">
          <button type="button" class="btn btn-secondary" @click="showAdd = false">Cancel</button>
          <button type="button" class="btn btn-primary" @click="() => void addContact()">Save</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { Contact } from '$lib/contactDirectory';
import {
  buildHumanContact,
  contactAdd,
  contactDirectoryServiceStart,
  contactList,
} from '$lib/contactDirectory';
import { openImChat, openP2pTab, openWebChat, startCallFromUi } from '$lib/imChat';
import { resolveLocalIdentity } from '$lib/imSession';
import { fetchOnlinePeers, isNodeOnline, type PresenceEntry } from '$lib/presence';

const emit = defineEmits<{ status: [message: string] }>();

const contacts = ref<Contact[]>([]);
const onlineMap = ref<Map<string, PresenceEntry>>(new Map());
const localNode = ref('');
const searchQuery = ref('');
const statusMessage = ref('');
const showAdd = ref(false);
const addName = ref('');
const addNode = ref('');

const filteredContacts = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return contacts.value.filter((c) => {
    if (c.is_blocked) return false;
    if (!q) return true;
    return (
      c.name.toLowerCase().includes(q) ||
      c.node_id.toLowerCase().includes(q) ||
      c.notes.toLowerCase().includes(q)
    );
  });
});

function isOnline(nodeId: string): boolean {
  return isNodeOnline(onlineMap.value, nodeId);
}

async function loadContacts(): Promise<void> {
  try {
    await contactDirectoryServiceStart();
    contacts.value = await contactList();
    if (localNode.value) {
      onlineMap.value = await fetchOnlinePeers(localNode.value);
    }
    statusMessage.value = `${contacts.value.length} contact(s)`;
  } catch (e) {
    statusMessage.value = String(e);
    contacts.value = [];
    emit('status', statusMessage.value);
  }
}

function openChat(c: Contact): void {
  openP2pTab('im');
  openWebChat();
  openImChat({ contactId: c.contact_id, name: c.name, nodeId: c.node_id });
  emit('status', `Opening chat with ${c.name}`);
}

function voiceCall(c: Contact): void {
  openP2pTab('im');
  openWebChat();
  startCallFromUi({ nodeId: c.node_id, name: c.name, video: false, audio: true });
  emit('status', `Calling ${c.name}`);
}

async function addContact(): Promise<void> {
  if (!addName.value.trim() || !addNode.value.trim()) return;
  try {
    await contactAdd(buildHumanContact({ name: addName.value.trim(), nodeId: addNode.value.trim() }));
    showAdd.value = false;
    addName.value = '';
    addNode.value = '';
    await loadContacts();
    emit('status', 'Contact added');
  } catch (e) {
    emit('status', String(e));
  }
}

onMounted(() => {
  void (async () => {
    const id = await resolveLocalIdentity();
    localNode.value = id.nodeId;
    await loadContacts();
  })();
});
</script>

<style scoped>
.contact-directory {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 12px;
  min-height: 0;
  overflow-y: auto;
}

.toolbar {
  display: flex;
  gap: 6px;
}

.field {
  flex: 1;
  padding: 6px 8px;
  border-radius: 6px;
  border: 1px solid #404040;
  background: #1a1a1a;
  color: #e0e0e0;
}

.contact-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.contact-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  border-bottom: 1px solid #333;
}

.info {
  cursor: pointer;
  flex: 1;
}

.node {
  display: block;
  font-size: 10px;
  color: #888;
}

.online {
  color: #4ade80;
  font-size: 10px;
}

.actions {
  display: flex;
  gap: 4px;
}

.btn {
  padding: 4px 8px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 11px;
}

.btn-primary {
  background: #6366f1;
  color: #fff;
}

.btn-secondary {
  background: #444;
  color: #e0e0e0;
}

.hint,
.muted {
  color: #888;
  font-size: 11px;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.dialog {
  background: #292a2d;
  padding: 16px;
  border-radius: 8px;
  min-width: 240px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
