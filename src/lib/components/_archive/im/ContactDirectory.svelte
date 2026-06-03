<script lang="ts">
  /**
   * Exodus Browser — Contact Directory (full editor + quick chat/call).
   */
  import { onMount } from 'svelte';
  import type { Contact } from '$lib/contactDirectory';
  import {
    buildHumanContact,
    contactAdd,
    contactDirectoryServiceStart,
    contactExportJson,
    contactImportJson,
    contactList,
    contactRemove,
    contactAddFriendByDigit,
    downloadContactExport,
  } from '$lib/contactDirectory';
  import { openImChat, openP2pTab, startCallFromUi } from '$lib/imChat';
  import { resolveLocalIdentity } from '$lib/imSession';
  import { fetchOnlinePeers, isNodeOnline } from '$lib/presence';

  let contacts: Contact[] = [];
  let onlineMap = new Map<string, { nodeId: string; displayName: string; lastSeen: number }>();
  let localNode = '';
  let filteredContacts: Contact[] = [];
  let showAddContactDialog = false;
  let selectedContact: Contact | null = null;
  let searchQuery = '';
  let filterGroup = 'all';
  let statusMessage = '';

  let contactName = '';
  let contactNodeId = '';
  let contactNotes = '';
  let contactDigit = '';
  let contactGroups = '';

  async function loadContacts() {
    try {
      await contactDirectoryServiceStart();
      contacts = await contactList();
      if (localNode) {
        onlineMap = await fetchOnlinePeers(localNode);
      }
      statusMessage = `${contacts.length} contact(s)`;
    } catch (e) {
      statusMessage = String(e);
      contacts = [];
    }
    filterContacts();
  }

  function filterContacts() {
    filteredContacts = contacts.filter((c) => {
      if (c.is_blocked) return false;
      if (filterGroup !== 'all' && !c.groups.includes(filterGroup)) return false;
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        return (
          c.name.toLowerCase().includes(q) ||
          c.node_id.toLowerCase().includes(q) ||
          c.notes.toLowerCase().includes(q)
        );
      }
      return true;
    });
  }

  async function addContact() {
    if (!contactName.trim() || !contactNodeId.trim()) return;
    try {
      const groups = contactGroups
        .split(',')
        .map((g) => g.trim())
        .filter(Boolean);
      await contactAdd(
        buildHumanContact({
          name: contactName.trim(),
          nodeId: contactNodeId.trim(),
          notes: contactNotes.trim(),
          groups: groups.length ? groups : ['friends'],
        })
      );
      showAddContactDialog = false;
      contactName = '';
      contactNodeId = '';
      contactNotes = '';
      contactGroups = '';
      await loadContacts();
    } catch (error) {
      statusMessage = String(error);
    }
  }

  async function addByDigit() {
    const digit = contactDigit.replace(/\D/g, '');
    if (digit.length !== 12) {
      statusMessage = '12-digit ID required';
      return;
    }
    try {
      await contactAddFriendByDigit(digit, contactName.trim() || `Friend ${digit}`, 'exodus-local-user');
      showAddContactDialog = false;
      contactDigit = '';
      await loadContacts();
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function deleteContact(id: string) {
    if (!confirm('Delete this contact?')) return;
    try {
      await contactRemove(id);
      if (selectedContact?.contact_id === id) selectedContact = null;
      await loadContacts();
    } catch (error) {
      statusMessage = String(error);
    }
  }

  function openChat(c: Contact) {
    openP2pTab('im');
    openImChat({ contactId: c.contact_id, name: c.name, nodeId: c.node_id });
    statusMessage = `Chat with ${c.name}`;
  }

  function voiceCall(c: Contact) {
    openP2pTab('im');
    startCallFromUi({ nodeId: c.node_id, name: c.name, video: false, audio: true });
    statusMessage = `Calling ${c.name}…`;
  }

  function videoCall(c: Contact) {
    openP2pTab('im');
    startCallFromUi({ nodeId: c.node_id, name: c.name, video: true, audio: true });
    statusMessage = `Video call ${c.name}…`;
  }

  async function exportContacts() {
    try {
      const json = await contactExportJson();
      downloadContactExport(json);
      statusMessage = 'Contacts exported';
    } catch (e) {
      statusMessage = String(e);
    }
  }

  async function importContacts() {
    const text = window.prompt('Paste Exodus contacts JSON export:');
    if (!text?.trim()) return;
    const merge = window.confirm(
      'Merge with existing contacts?\n\nOK = merge by node id\nCancel = replace all contacts'
    );
    try {
      const n = await contactImportJson(text.trim(), merge);
      await loadContacts();
      statusMessage = `Imported ${n} contact(s)`;
    } catch (e) {
      statusMessage = String(e);
    }
  }

  function getAllGroups(): string[] {
    const groups = new Set<string>();
    contacts.forEach((c) => c.groups.forEach((g) => groups.add(g)));
    return Array.from(groups);
  }

  onMount(() => {
    void (async () => {
      try {
        const id = await resolveLocalIdentity();
        localNode = id.nodeId;
      } catch {
        localNode = '';
      }
      await loadContacts();
    })();
    // Disabled contact load timer to prevent cursor spinning
    /*
    const t = setInterval(() => void loadContacts(), 30_000);
    return () => clearInterval(t);
    */
  });

  $: searchQuery, filterContacts();
  $: filterGroup, filterContacts();
</script>

<div class="contact-directory">
  <div class="header">
    <h2>Contacts</h2>
    {#if statusMessage}
      <p class="hint">{statusMessage}</p>
    {/if}
    <div class="actions">
      <input type="text" placeholder="Search…" bind:value={searchQuery} class="search-input" />
      <select bind:value={filterGroup} class="filter-select">
        <option value="all">All</option>
        {#each getAllGroups() as group}
          <option value={group}>{group}</option>
        {/each}
      </select>
      <button type="button" class="btn btn-secondary" on:click={() => void exportContacts()}>
        Export
      </button>
      <button type="button" class="btn btn-secondary" on:click={() => void importContacts()}>
        Import
      </button>
      <button type="button" class="btn btn-primary" on:click={() => (showAddContactDialog = true)}>
        Add
      </button>
    </div>
  </div>

  <div class="contact-container">
    <div class="contact-list">
      {#if filteredContacts.length === 0}
        <div class="empty-state">
          <p>No contacts — add by node id or 12-digit Exodus ID</p>
        </div>
      {:else}
        {#each filteredContacts as contact (contact.contact_id)}
          <div
            class="contact-item"
            class:active={selectedContact?.contact_id === contact.contact_id}
            role="button"
            tabindex="0"
            on:click={() => (selectedContact = contact)}
            on:keydown={(e) => e.key === 'Enter' && (selectedContact = contact)}
          >
            <div class="avatar" class:online={isNodeOnline(onlineMap, contact.node_id)}>
              {contact.name[0]?.toUpperCase() ?? '?'}
            </div>
            <div class="contact-info">
              <div class="name">{contact.name}</div>
              <div class="peer-id">
                {isNodeOnline(onlineMap, contact.node_id) ? 'Online' : 'Offline'} · {contact.node_id.slice(
                  0,
                  24
                )}…
              </div>
            </div>
            <div class="quick-actions" role="group">
              <button type="button" title="Chat" on:click|stopPropagation={() => openChat(contact)}
                >💬</button
              >
              <button type="button" title="Voice" on:click|stopPropagation={() => voiceCall(contact)}
                >📞</button
              >
              <button type="button" title="Video" on:click|stopPropagation={() => videoCall(contact)}
                >📹</button
              >
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="contact-details">
      {#if selectedContact}
        <h3>{selectedContact.name}</h3>
        <p class="peer-full">{selectedContact.node_id}</p>
        {#if selectedContact.notes}
          <p>{selectedContact.notes}</p>
        {/if}
        <div class="detail-actions">
          <button type="button" class="btn btn-primary" on:click={() => openChat(selectedContact!)}>
            Open chat (WebChat tab)
          </button>
          <button type="button" class="btn btn-secondary" on:click={() => voiceCall(selectedContact!)}>
            Voice call
          </button>
          <button type="button" class="btn btn-secondary" on:click={() => videoCall(selectedContact!)}>
            Video call
          </button>
          <button
            type="button"
            class="btn btn-danger"
            on:click={() => deleteContact(selectedContact!.contact_id)}>Delete</button
          >
        </div>
      {:else}
        <p class="muted">Select a contact</p>
      {/if}
    </div>
  </div>

  {#if showAddContactDialog}
    <div
      class="dialog-overlay"
      role="button"
      tabindex="0"
      on:click={() => (showAddContactDialog = false)}
      on:keydown={(e) => e.key === 'Escape' && (showAddContactDialog = false)}
    >
      <div class="dialog" role="dialog" on:click|stopPropagation on:keydown|stopPropagation>
        <h3>Add contact</h3>
        <form on:submit|preventDefault={addContact}>
          <label>Name <input type="text" bind:value={contactName} required /></label>
          <label>Node ID <input type="text" bind:value={contactNodeId} required /></label>
          <label>Notes <input type="text" bind:value={contactNotes} /></label>
          <label>Groups <input type="text" bind:value={contactGroups} placeholder="friends, work" /></label>
          <label>12-digit ID (optional)
            <input type="text" bind:value={contactDigit} maxlength="14" placeholder="1234-5678-9012" />
          </label>
          <button type="button" class="btn btn-secondary" on:click={() => void addByDigit()}
            >Add friend by digit</button
          >
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showAddContactDialog = false)}
              >Cancel</button
            >
            <button type="submit" class="btn btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .contact-directory {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 400px;
  }
  .header h2 {
    margin: 0 0 4px;
    font-size: 15px;
  }
  .hint {
    font-size: 11px;
    color: #888;
    margin: 0 0 8px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .search-input,
  .filter-select {
    padding: 6px 8px;
    background: #333;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
    font-size: 12px;
  }
  .contact-container {
    display: flex;
    gap: 10px;
    min-height: 0;
    flex: 1;
  }
  .contact-list {
    flex: 1;
    overflow-y: auto;
    max-height: 280px;
  }
  .contact-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    border-radius: 6px;
    cursor: pointer;
    margin-bottom: 4px;
    background: #333;
  }
  .contact-item.active {
    background: #4f46e5;
  }
  .avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #6366f1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    color: #fff;
  }
  .avatar.online {
    box-shadow: 0 0 0 2px #22c55e;
  }
  .contact-info {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-size: 13px;
    font-weight: 600;
  }
  .peer-id {
    font-size: 10px;
    color: #aaa;
  }
  .quick-actions button {
    background: #444;
    border: none;
    border-radius: 4px;
    padding: 4px 6px;
    cursor: pointer;
  }
  .contact-details {
    width: 140px;
    font-size: 12px;
    padding: 8px;
    background: #2a2a2a;
    border-radius: 6px;
  }
  .peer-full {
    font-size: 10px;
    word-break: break-all;
    color: #888;
  }
  .detail-actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 10px;
  }
  .btn {
    padding: 6px 10px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
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
  .empty-state,
  .muted {
    color: #888;
    font-size: 12px;
  }
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1500;
  }
  .dialog {
    background: #333;
    padding: 16px;
    border-radius: 8px;
    min-width: 300px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .dialog label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: #ccc;
  }
  .dialog input {
    padding: 6px;
    background: #444;
    border: 1px solid #555;
    color: #eee;
    border-radius: 4px;
  }
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }
</style>
