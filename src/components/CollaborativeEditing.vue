<!--
  Exodus Browser — Collaborative editing panel (local docs + share list).
-->
<template>
  <div class="collaborative-editing">
    <div class="header">
      <h2>Collaborative Editing</h2>
      <button type="button" class="btn btn-primary" @click="showNewDocumentDialog = true">New Document</button>
    </div>

    <div class="editor-container">
      <div class="sidebar">
        <h3>Documents</h3>
        <div v-if="documents.length === 0" class="empty-state">
          <p>No documents</p>
        </div>
        <div
          v-for="doc in documents"
          :key="doc.id"
          class="document-item"
          :class="{ active: activeDocument?.id === doc.id }"
          @click="selectDocument(doc)"
        >
          <div class="doc-title">{{ doc.title }}</div>
          <div class="doc-meta">
            <span>Updated: {{ formatDate(doc.updated_at) }}</span>
            <span>{{ doc.collaborators.length }} collaborator(s)</span>
          </div>
          <button
            type="button"
            class="btn-icon delete"
            title="Delete"
            @click.stop="() => void deleteDocument(doc.id)"
          >
            ×
          </button>
        </div>
      </div>

      <div v-if="activeDocument" class="editor-main">
        <div class="editor-toolbar">
          <input v-model="activeDocument.title" type="text" class="title-input" @change="persistActive" />
          <button type="button" class="btn btn-secondary" @click="showShareDialog = true">Share</button>
          <button type="button" class="btn btn-secondary" @click="persistActive">Save</button>
        </div>
        <textarea
          v-model="currentContent"
          class="editor-textarea"
          placeholder="Start writing…"
          @input="onContentInput"
        />
        <div v-if="remoteCursors.length > 0" class="cursors-hint">
          <span v-for="c in remoteCursors" :key="c.user_id" class="cursor-chip" :style="{ color: c.color }">
            {{ c.user_name }} @ {{ c.position }}
          </span>
        </div>
        <div v-if="activeDocument.collaborators.length" class="collaborators">
          <span v-for="peer in activeDocument.collaborators" :key="peer" class="collab-chip">
            {{ peer }}
            <button type="button" class="btn-icon" @click="removeCollaborator(peer)">×</button>
          </span>
        </div>
      </div>
      <div v-else class="editor-placeholder">
        <p>Select or create a document</p>
      </div>
    </div>

    <div v-if="showNewDocumentDialog" class="dialog-overlay" @click.self="showNewDocumentDialog = false">
      <div class="dialog" role="dialog" @click.stop>
        <h3>New Document</h3>
        <label>
          Title
          <input v-model="newDocumentTitle" type="text" />
        </label>
        <label>
          Initial content
          <textarea v-model="newDocumentContent" rows="4" />
        </label>
        <div class="dialog-actions">
          <button type="button" class="btn btn-secondary" @click="showNewDocumentDialog = false">Cancel</button>
          <button type="button" class="btn btn-primary" @click="() => void createDocument()">Create</button>
        </div>
      </div>
    </div>

    <div v-if="showShareDialog" class="dialog-overlay" @click.self="showShareDialog = false">
      <div class="dialog" role="dialog" @click.stop>
        <h3>Share document</h3>
        <label>
          Peer ID
          <input v-model="sharePeer" type="text" placeholder="node id" />
        </label>
        <div class="dialog-actions">
          <button type="button" class="btn btn-secondary" @click="showShareDialog = false">Cancel</button>
          <button type="button" class="btn btn-primary" @click="shareDocument">Share</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, inject } from 'vue';
import { CONFIRM_DIALOG_KEY } from '$lib/confirm';
import {
  loadCollabDocuments,
  createCollabDocument,
  updateCollabDocument,
  deleteCollabDocument,
  addCollabPeer,
  removeCollabPeer,
  type CollabDocument,
} from '$lib/collaborativeDocs';

interface Cursor {
  user_id: string;
  user_name: string;
  position: number;
  color: string;
}

const emit = defineEmits<{ status: [message: string] }>();

const confirmDialog = inject(CONFIRM_DIALOG_KEY, null);

const documents = ref<CollabDocument[]>([]);
const activeDocument = ref<CollabDocument | null>(null);
const currentContent = ref('');
const showNewDocumentDialog = ref(false);
const showShareDialog = ref(false);
const remoteCursors = ref<Cursor[]>([]);
const newDocumentTitle = ref('');
const newDocumentContent = ref('');
const sharePeer = ref('');

let saveTimer: ReturnType<typeof setTimeout> | undefined;

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

function loadDocuments(): void {
  documents.value = loadCollabDocuments();
}

function selectDocument(doc: CollabDocument): void {
  activeDocument.value = { ...doc };
  currentContent.value = doc.content;
  simulateRemoteCursors();
}

function persistActive(): void {
  if (!activeDocument.value) return;
  activeDocument.value = updateCollabDocument({
    ...activeDocument.value,
    content: currentContent.value,
  });
  documents.value = loadCollabDocuments();
  emit('status', 'Document saved');
}

function onContentInput(): void {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => persistActive(), 800);
}

function createDocument(): void {
  if (!newDocumentTitle.value.trim()) return;
  const doc = createCollabDocument(newDocumentTitle.value, newDocumentContent.value);
  documents.value = loadCollabDocuments();
  showNewDocumentDialog.value = false;
  newDocumentTitle.value = '';
  newDocumentContent.value = '';
  selectDocument(doc);
  emit('status', `Created "${doc.title}"`);
}

function shareDocument(): void {
  if (!activeDocument.value || !sharePeer.value.trim()) return;
  activeDocument.value = addCollabPeer(activeDocument.value, sharePeer.value);
  documents.value = loadCollabDocuments();
  showShareDialog.value = false;
  sharePeer.value = '';
  emit('status', 'Collaborator added');
}

function removeCollaborator(peer: string): void {
  if (!activeDocument.value) return;
  activeDocument.value = removeCollabPeer(activeDocument.value, peer);
  documents.value = loadCollabDocuments();
}

/** Delete a document after shell confirmation (or window.confirm fallback). */
function deleteDocument(id: string): void {
  const runDelete = (): void => {
    deleteCollabDocument(id);
    documents.value = loadCollabDocuments();
    if (activeDocument.value?.id === id) {
      activeDocument.value = null;
      currentContent.value = '';
    }
    emit('status', 'Document deleted');
  };

  const offer = {
    title: 'Delete document?',
    message: 'Are you sure you want to delete this document? This cannot be undone.',
    confirmLabel: 'Delete',
    danger: true,
  };

  if (confirmDialog) {
    confirmDialog.openConfirmDialog(offer, async () => {
      runDelete();
    });
    return;
  }
  if (window.confirm(offer.message)) {
    runDelete();
  }
}

function simulateRemoteCursors(): void {
  if (!activeDocument.value) {
    remoteCursors.value = [];
    return;
  }
  const len = currentContent.value.length;
  remoteCursors.value = [
    { user_id: 'demo-alice', user_name: 'Alice', position: Math.floor(len * 0.3), color: '#ef4444' },
    { user_id: 'demo-bob', user_name: 'Bob', position: Math.floor(len * 0.6), color: '#3b82f6' },
  ];
}

onMounted(() => {
  loadDocuments();
});
</script>

<style scoped>
.collaborative-editing {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  min-height: 0;
  font-size: 12px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h2 {
  margin: 0;
  font-size: 14px;
}

.editor-container {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 8px;
  flex: 1;
  min-height: 0;
}

.sidebar {
  overflow-y: auto;
  border-right: 1px solid #333;
  padding-right: 6px;
}

.document-item {
  padding: 6px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  position: relative;
}

.document-item.active {
  background: rgba(99, 102, 241, 0.25);
}

.doc-title {
  font-weight: 500;
}

.doc-meta {
  font-size: 10px;
  color: #888;
  display: flex;
  flex-direction: column;
}

.editor-main {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
}

.editor-toolbar {
  display: flex;
  gap: 6px;
}

.title-input {
  flex: 1;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #e0e0e0;
}

.editor-textarea {
  flex: 1;
  min-height: 120px;
  padding: 8px;
  border-radius: 6px;
  border: 1px solid #444;
  background: #111;
  color: #e0e0e0;
  resize: vertical;
  font-family: ui-monospace, monospace;
  font-size: 12px;
}

.cursors-hint {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cursor-chip {
  font-size: 10px;
}

.collab-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: #333;
  padding: 2px 6px;
  border-radius: 4px;
  margin-right: 4px;
}

.editor-placeholder,
.empty-state {
  color: #888;
  padding: 16px;
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
  min-width: 260px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dialog label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dialog input,
.dialog textarea {
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
  padding: 4px 10px;
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

.btn-icon {
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
}
</style>
