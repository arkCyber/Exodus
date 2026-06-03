<script lang="ts">
  /**
   * Exodus Browser — Collaborative Editing UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface Document {
    id: string;
    title: string;
    content: string;
    created_at: number;
    updated_at: number;
    owner: string;
    collaborators: string[];
  }

  interface Cursor {
    user_id: string;
    user_name: string;
    position: number;
    color: string;
  }

  let documents: Document[] = [];
  let activeDocument: Document | null = null;
  let showNewDocumentDialog = false;
  let showShareDialog = false;
  let remoteCursors: Cursor[] = [];
  let currentContent = '';

  // New document form
  let newDocumentTitle = '';
  let newDocumentContent = '';

  // Share form
  let sharePeer = '';

  async function loadDocuments() {
    // In a real implementation, this would load documents from the backend
    // For now, we'll use a placeholder
    documents = [];
  }

  async function createDocument() {
    if (!newDocumentTitle) return;

    try {
      const doc: Document = {
        id: crypto.randomUUID(),
        title: newDocumentTitle,
        content: newDocumentContent,
        created_at: Date.now() / 1000,
        updated_at: Date.now() / 1000,
        owner: 'local-user',
        collaborators: [],
      };

      // In a real implementation, this would save via the backend
      console.log('Creating document:', doc);
      
      documents.push(doc);
      showNewDocumentDialog = false;
      newDocumentTitle = '';
      newDocumentContent = '';
    } catch (error) {
      console.error('Failed to create document:', error);
    }
  }

  async function saveDocument() {
    if (!activeDocument) return;

    try {
      activeDocument.content = currentContent;
      activeDocument.updated_at = Date.now() / 1000;

      // In a real implementation, this would save via the backend
      console.log('Saving document:', activeDocument);
    } catch (error) {
      console.error('Failed to save document:', error);
    }
  }

  function selectDocument(doc: Document) {
    activeDocument = doc;
    currentContent = doc.content;
  }

  async function shareDocument() {
    if (!activeDocument || !sharePeer) return;

    try {
      // In a real implementation, this would share via the backend
      console.log('Sharing document with:', sharePeer);
      
      if (!activeDocument.collaborators.includes(sharePeer)) {
        activeDocument.collaborators.push(sharePeer);
      }
      
      showShareDialog = false;
      sharePeer = '';
    } catch (error) {
      console.error('Failed to share document:', error);
    }
  }

  function removeCollaborator(peer: string) {
    if (!activeDocument) return;

    const index = activeDocument.collaborators.indexOf(peer);
    if (index > -1) {
      activeDocument.collaborators.splice(index, 1);
    }
  }

  async function deleteDocument(id: string) {
    if (!confirm('Are you sure you want to delete this document?')) return;

    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting document:', id);
      
      documents = documents.filter((d) => d.id !== id);
      
      if (activeDocument && activeDocument.id === id) {
        activeDocument = null;
        currentContent = '';
      }
    } catch (error) {
      console.error('Failed to delete document:', error);
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  // Simulate remote cursor updates
  function simulateRemoteCursors() {
    if (!activeDocument) return;

    // In a real implementation, this would come from the backend via WebSocket
    remoteCursors = [
      {
        user_id: 'user1',
        user_name: 'Alice',
        position: Math.floor(Math.random() * currentContent.length),
        color: '#ef4444',
      },
      {
        user_id: 'user2',
        user_name: 'Bob',
        position: Math.floor(Math.random() * currentContent.length),
        color: '#3b82f6',
      },
    ];
  }

  // Load documents on mount
  loadDocuments();

  // Simulate cursor updates
  setInterval(() => {
    if (activeDocument) {
      simulateRemoteCursors();
    }
  }, 2000);

  // Auto-save on content change
  $: currentContent, activeDocument, saveDocument();
</script>

<div class="collaborative-editing">
  <div class="header">
    <h2>Collaborative Editing</h2>
    <div class="actions">
      <button class="btn btn-primary" on:click={() => (showNewDocumentDialog = true)}>
        New Document
      </button>
    </div>
  </div>

  <div class="editor-container">
    <div class="sidebar">
      <div class="sidebar-header">
        <h3>Documents</h3>
      </div>
      <div class="document-list">
        {#if documents.length === 0}
          <div class="empty-state">
            <p>No documents</p>
          </div>
        {:else}
          {#each documents as doc (doc.id)}
            <div
              class="document-item {activeDocument?.id === doc.id ? 'active' : ''}"
              on:click={() => selectDocument(doc)}
            >
              <div class="doc-title">{doc.title}</div>
              <div class="doc-meta">
                <div class="updated">Updated: {formatDate(doc.updated_at)}</div>
                <div class="collaborators">
                  {doc.collaborators.length} collaborator{doc.collaborators.length !== 1 ? 's' : ''}
                </div>
              </div>
              <button
                class="btn-icon delete"
                title="Delete"
                on:click|stopPropagation={() => deleteDocument(doc.id)}
              >
                🗑️
              </button>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <div class="editor">
      {#if activeDocument}
        <div class="editor-header">
          <div class="document-info">
            <h3>{activeDocument.title}</h3>
            <div class="meta">
              <span>Owner: {activeDocument.owner}</span>
              <span>Last updated: {formatDate(activeDocument.updated_at)}</span>
            </div>
          </div>
          <div class="editor-actions">
            <button class="btn btn-secondary" on:click={() => (showShareDialog = true)}>
              Share
            </button>
            <button class="btn btn-primary" on:click={saveDocument}>
              Save
            </button>
          </div>
        </div>

        <div class="collaborators-bar">
          <div class="collaborators-list">
            <div class="collaborator you">
              <div class="avatar" style="background: #6366f1">Y</div>
              <span>You</span>
            </div>
            {#each activeDocument.collaborators as peer}
              <div class="collaborator">
                <div class="avatar" style="background: #059669">{peer[0].toUpperCase()}</div>
                <span>{peer}</span>
              </div>
            {/each}
          </div>
        </div>

        <div class="editor-content">
          <textarea
            bind:value={currentContent}
            placeholder="Start typing..."
            class="document-editor"
          ></textarea>
          {#each remoteCursors as cursor (cursor.user_id)}
            <div
              class="remote-cursor"
              style="left: {cursor.position * 8}px; background: {cursor.color}"
              title="{cursor.user_name}"
            >
              <div class="cursor-label" style="background: {cursor.color}">
                {cursor.user_name}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="no-document">
          <div class="no-doc-icon">📄</div>
          <p>Select a document or create a new one</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- New Document Dialog -->
  {#if showNewDocumentDialog}
    <div class="dialog-overlay" on:click={() => (showNewDocumentDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>New Document</h3>
        <form on:submit|preventDefault={createDocument}>
          <div class="form-group">
            <label>Title</label>
            <input type="text" bind:value={newDocumentTitle} required />
          </div>
          <div class="form-group">
            <label>Initial Content (optional)</label>
            <textarea
              bind:value={newDocumentContent}
              placeholder="Start with some content..."
              rows="5"
            ></textarea>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showNewDocumentDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Create</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Share Dialog -->
  {#if showShareDialog && activeDocument}
    <div class="dialog-overlay" on:click={() => (showShareDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Share Document</h3>
        <div class="form-group">
          <label>Add Collaborator</label>
          <input
            type="text"
            bind:value={sharePeer}
            placeholder="e.g., peer@example.com"
          />
        </div>
        <div class="form-actions">
          <button class="btn btn-secondary" on:click={() => (showShareDialog = false)}>
            Cancel
          </button>
          <button class="btn btn-primary" on:click={shareDocument}>Share</button>
        </div>

        <div class="current-collaborators">
          <h4>Current Collaborators</h4>
          <div class="collaborator-list">
            {#each activeDocument.collaborators as peer}
              <div class="collaborator-item">
                <span>{peer}</span>
                <button class="btn-icon" on:click={() => removeCollaborator(peer)}>✕</button>
              </div>
            {/each}
            {#if activeDocument.collaborators.length === 0}
              <p class="no-collaborators">No collaborators yet</p>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .collaborative-editing {
    padding: 20px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
  }

  .editor-container {
    flex: 1;
    display: flex;
    gap: 20px;
    min-height: 0;
  }

  .sidebar {
    width: 300px;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .sidebar-header {
    padding: 15px;
    border-bottom: 1px solid #444;
  }

  .sidebar-header h3 {
    margin: 0;
  }

  .document-list {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #888;
  }

  .document-item {
    padding: 15px;
    background: #444;
    border-radius: 6px;
    margin-bottom: 10px;
    cursor: pointer;
    position: relative;
    transition: background 0.2s;
  }

  .document-item:hover {
    background: #555;
  }

  .document-item.active {
    background: #6366f1;
    border-color: #4f46e5;
  }

  .doc-title {
    font-weight: bold;
    color: #eee;
    margin-bottom: 5px;
  }

  .doc-meta {
    font-size: 12px;
    color: #aaa;
  }

  .doc-meta div {
    margin-bottom: 3px;
  }

  .document-item .btn-icon {
    position: absolute;
    top: 10px;
    right: 10px;
    background: transparent;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 4px;
  }

  .document-item .btn-icon:hover {
    color: #dc2626;
  }

  .editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
    min-height: 0;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    border-bottom: 1px solid #444;
  }

  .document-info h3 {
    margin: 0 0 5px 0;
  }

  .document-info .meta {
    font-size: 12px;
    color: #aaa;
  }

  .document-info .meta span {
    margin-right: 15px;
  }

  .editor-actions {
    display: flex;
    gap: 10px;
  }

  .collaborators-bar {
    padding: 10px 15px;
    border-bottom: 1px solid #444;
    background: #444;
  }

  .collaborators-list {
    display: flex;
    gap: 10px;
  }

  .collaborator {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .collaborator.you {
    font-weight: bold;
  }

  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    color: white;
  }

  .editor-content {
    flex: 1;
    position: relative;
    padding: 20px;
    overflow: hidden;
  }

  .document-editor {
    width: 100%;
    height: 100%;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
    padding: 15px;
    font-family: monospace;
    font-size: 14px;
    resize: none;
  }

  .remote-cursor {
    position: absolute;
    top: 20px;
    width: 2px;
    height: 20px;
    border-radius: 1px;
  }

  .cursor-label {
    position: absolute;
    top: -20px;
    left: 0;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 10px;
    color: white;
    white-space: nowrap;
  }

  .no-document {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    color: #888;
  }

  .no-doc-icon {
    font-size: 64px;
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #333;
    border: 1px solid #555;
    border-radius: 8px;
    padding: 20px;
    min-width: 400px;
    max-width: 500px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    color: #aaa;
  }

  .form-group input[type='text'],
  .form-group textarea {
    width: 100%;
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .current-collaborators {
    margin-top: 20px;
    padding-top: 20px;
    border-top: 1px solid #444;
  }

  .current-collaborators h4 {
    margin: 0 0 15px 0;
  }

  .collaborator-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .collaborator-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px;
    background: #444;
    border-radius: 4px;
  }

  .collaborator-item .btn-icon {
    background: transparent;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 4px;
  }

  .collaborator-item .btn-icon:hover {
    color: #dc2626;
  }

  .no-collaborators {
    color: #888;
    font-style: italic;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    border: none;
  }

  .btn-primary {
    background: #6366f1;
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }

  .btn-secondary {
    background: #444;
    color: #eee;
  }

  .btn-secondary:hover {
    background: #555;
  }

  .btn-icon {
    background: #444;
    border: 1px solid #555;
    color: #eee;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-icon:hover {
    background: #555;
  }
</style>
