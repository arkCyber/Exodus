<script lang="ts">
  /**
   * Exodus Browser — WeChat-style IM: contacts list + chat + quick voice/video call.
   */
  import { onMount } from 'svelte';
  import type { Contact } from '$lib/contactDirectory';
  import {
    buildHumanContact,
    contactAdd,
    contactAddFriendByDigit,
    contactDirectoryServiceStart,
    contactGetDigitForNode,
    contactGetLocalDigit,
    contactList,
    touchContactLastContacted,
  } from '$lib/contactDirectory';
  import {
    IM_OPEN_CONTACT_EVENT,
    dmRoomId,
    ensureDmGroup,
    loadDmMessages,
    sendDmText,
    startCallFromUi,
    type ImOpenContactDetail,
  } from '$lib/imChat';
  import type { GroupMessage } from '$lib/groupChat';
  import {
    extractMentionNodeIds,
    insertMentionToken,
    mergeMentionCandidates,
    mentionCandidatesForQueryFromTargets,
    contactsToMentionTargets,
    type MentionTarget,
  } from '$lib/groupMentions';
  import MentionMessageBody from '$lib/components/MentionMessageBody.svelte';
  import {
    getSavedDisplayName,
    isLikelyPeerNodeId,
    resolveLocalIdentity,
    setSavedDisplayName,
  } from '$lib/imSession';
  import {
    fetchOnlinePeers,
    isNodeOnline,
    startPresenceHeartbeat,
    stopPresenceHeartbeat,
    type PresenceEntry,
  } from '$lib/presence';
  import { loadCustomTurnUrls, saveCustomTurnUrls } from '$lib/webrtc/rtcConfig';
  import { getRtcCallManager } from '$lib/webrtc/rtcCallSession';

  type Props = {
    onStatus?: (message: string) => void;
  };

  let { onStatus = () => {} }: Props = $props();

  let localUserId = $state('exodus-local-user');
  let localName = $state('You');
  let contacts = $state<Contact[]>([]);
  let active = $state<Contact | null>(null);
  let messages = $state<GroupMessage[]>([]);
  let draft = $state('');
  let search = $state('');
  let loading = $state(false);
  let localNode = $state('');
  let peerDigit = $state('');
  let onlineMap = $state<Map<string, PresenceEntry>>(new Map());
  let showAdd = $state(false);
  let showTurn = $state(false);
  let turnText = $state(loadCustomTurnUrls().join('\n'));
  let addName = $state('');
  let addNode = $state('');
  let addDigit = $state('');
  let myDigit = $state('');
  let mentionMenu = $state<MentionTarget[]>([]);
  let showMentionMenu = $state(false);

  const callMgr = getRtcCallManager();

  const mentionTargets = $derived(
    active && isLikelyPeerNodeId(active.node_id)
      ? mergeMentionCandidates(contacts, [
          { nodeId: active.node_id, displayName: active.name },
        ])
      : contactsToMentionTargets(contacts)
  );

  const filtered = $derived(
    contacts.filter((c) => {
      if (!search.trim()) return true;
      const q = search.toLowerCase();
      return (
        c.name.toLowerCase().includes(q) ||
        c.node_id.toLowerCase().includes(q) ||
        c.notes.toLowerCase().includes(q)
      );
    })
  );

  const roomId = $derived(
    active && localNode ? dmRoomId(localNode, active.node_id) : ''
  );

  async function refreshPresence() {
    if (!localNode) return;
    onlineMap = await fetchOnlinePeers(localNode);
  }

  async function bootstrap() {
    await contactDirectoryServiceStart();
    const id = await resolveLocalIdentity();
    localUserId = id.userId;
    localName = id.displayName;
    localNode = id.nodeId;
    await callMgr.init();
    await startPresenceHeartbeat(localNode, localName);
    await refreshContacts();
    await refreshPresence();
    try {
      myDigit = await contactGetLocalDigit();
    } catch {
      myDigit = '';
    }
  }

  function updateMentionMenu() {
    const hit = mentionCandidatesForQueryFromTargets(draft, mentionTargets);
    if (!hit || hit.candidates.length === 0) {
      showMentionMenu = false;
      mentionMenu = [];
      return;
    }
    mentionMenu = hit.candidates;
    showMentionMenu = true;
  }

  function pickMention(target: MentionTarget) {
    draft = insertMentionToken(draft, target);
    showMentionMenu = false;
    mentionMenu = [];
  }

  function onDraftInput() {
    updateMentionMenu();
  }

  function onDraftKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && showMentionMenu) {
      showMentionMenu = false;
      e.preventDefault();
      return;
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      if (showMentionMenu && mentionMenu.length > 0) {
        pickMention(mentionMenu[0]);
        e.preventDefault();
        return;
      }
      void sendMessage();
      e.preventDefault();
    }
  }

  function onImMentionAction(t: MentionTarget, action: 'chat' | 'voice' | 'video') {
    if (action === 'voice') startCallFromUi({ nodeId: t.nodeId, name: t.displayName, video: false, audio: true });
    else if (action === 'video') startCallFromUi({ nodeId: t.nodeId, name: t.displayName, video: true, audio: true });
    else if (active?.node_id !== t.nodeId) {
      const c = contacts.find((x) => x.node_id === t.nodeId);
      if (c) void selectContact(c);
    }
  }

  function saveTurnSettings() {
    const urls = turnText
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean);
    saveCustomTurnUrls(urls);
    onStatus('TURN URLs saved (used on next call)');
  }

  function saveDisplayName() {
    setSavedDisplayName(localName);
    void startPresenceHeartbeat(localNode, localName);
    onStatus('Display name saved');
  }

  async function copyText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      onStatus(`${label} copied`);
    } catch {
      onStatus('Copy failed');
    }
  }

  async function refreshContacts() {
    try {
      contacts = await contactList();
    } catch (e) {
      onStatus(String(e));
      contacts = [];
    }
  }

  async function selectContact(c: Contact) {
    active = c;
    peerDigit = '';
    if (!localNode) return;
    const rid = dmRoomId(localNode, c.node_id);
    loading = true;
    try {
      await ensureDmGroup(rid, localUserId, localName, c.node_id, c.name);
      messages = await loadDmMessages(rid);
      try {
        peerDigit = await contactGetDigitForNode(c.node_id);
      } catch {
        peerDigit = '';
      }
    } catch (e) {
      onStatus(String(e));
    } finally {
      loading = false;
    }
  }

  async function sendMessage() {
    const text = draft.trim();
    if (!text || !active || !roomId) return;
    const mentions = extractMentionNodeIds(text, contacts);
    try {
      await sendDmText(roomId, localUserId, localName, text, mentions);
      draft = '';
      showMentionMenu = false;
      messages = await loadDmMessages(roomId);
      void touchContactLastContacted(active.node_id);
    } catch (e) {
      onStatus(String(e));
    }
  }

  function voiceCall() {
    if (!active) return;
    startCallFromUi({ nodeId: active.node_id, name: active.name, video: false, audio: true });
  }

  function videoCall() {
    if (!active) return;
    startCallFromUi({ nodeId: active.node_id, name: active.name, video: true, audio: true });
  }

  async function addContactManual() {
    if (!addName.trim() || !addNode.trim()) return;
    try {
      await contactAdd(buildHumanContact({ name: addName.trim(), nodeId: addNode.trim() }));
      showAdd = false;
      addName = '';
      addNode = '';
      await refreshContacts();
      onStatus('Contact added');
    } catch (e) {
      onStatus(String(e));
    }
  }

  async function addByDigit() {
    const digit = addDigit.replace(/\D/g, '');
    if (digit.length !== 12) {
      onStatus('Enter 12-digit Exodus ID');
      return;
    }
    try {
      const c = await contactAddFriendByDigit(digit, addName.trim() || `Friend ${digit}`, localUserId);
      showAdd = false;
      addDigit = '';
      await refreshContacts();
      await selectContact(c);
      onStatus('Friend added');
    } catch (e) {
      onStatus(String(e));
    }
  }

  function onOpenImEvent(ev: Event) {
    const detail = (ev as CustomEvent<ImOpenContactDetail>).detail;
    const c = contacts.find((x) => x.node_id === detail.nodeId || x.contact_id === detail.contactId);
    if (c) void selectContact(c);
    else {
      const stub: Contact = buildHumanContact({
        name: detail.name,
        nodeId: detail.nodeId,
      });
      stub.contact_id = detail.contactId;
      void selectContact(stub);
    }
  }

  onMount(() => {
    localName = getSavedDisplayName();
    void bootstrap();
    window.addEventListener(IM_OPEN_CONTACT_EVENT, onOpenImEvent);
    // Disabled message poll to prevent cursor spinning
    /*
    const poll = setInterval(() => {
      if (active && roomId) {
        void loadDmMessages(roomId).then((m) => (messages = m));
      }
      void refreshPresence();
    }, 5000);
    */
    return () => {
      // clearInterval(poll);
      stopPresenceHeartbeat();
      window.removeEventListener(IM_OPEN_CONTACT_EVENT, onOpenImEvent);
    };
  });
</script>

<div class="im-messenger">
  <aside class="sidebar">
    <div class="sidebar-head">
      <input type="search" placeholder="Search contacts…" bind:value={search} />
      <button type="button" class="icon-btn" title="Add contact" onclick={() => (showAdd = !showAdd)}
        >＋</button
      >
      <button type="button" class="icon-btn secondary" title="NAT / TURN" onclick={() => (showTurn = !showTurn)}
        >⚙</button
      >
    </div>
    {#if showTurn}
      <div class="add-box">
        <label class="mini-label">Your name</label>
        <input type="text" bind:value={localName} placeholder="Display name" />
        <button type="button" class="mini" onclick={saveDisplayName}>Save name</button>
        <label class="mini-label">TURN URLs (one per line)</label>
        <textarea bind:value={turnText} rows="3" placeholder="turn:…"></textarea>
        <button type="button" class="mini" onclick={saveTurnSettings}>Save TURN</button>
        <span class="mini-label">My 12-digit ID: {myDigit || '—'}</span>
        <span class="mini-label">Node: {localNode.slice(0, 20)}…</span>
      </div>
    {/if}
    {#if showAdd}
      <div class="add-box">
        <input type="text" placeholder="Name" bind:value={addName} />
        <input type="text" placeholder="Node id" bind:value={addNode} />
        <button type="button" class="mini" onclick={() => void addContactManual()}>Add</button>
        <input type="text" placeholder="12-digit ID" bind:value={addDigit} maxlength="12" />
        <button type="button" class="mini" onclick={() => void addByDigit()}>Add by ID</button>
      </div>
    {/if}
    <ul class="contact-list">
      {#each filtered as c (c.contact_id)}
        <li>
          <button
            type="button"
            class="contact-row"
            class:active={active?.contact_id === c.contact_id}
            onclick={() => void selectContact(c)}
          >
            <span class="avatar" class:online={isNodeOnline(onlineMap, c.node_id)}
              >{c.name.slice(0, 1)}</span
            >
            <span class="meta">
              <span class="name">{c.name}</span>
              <span class="sub"
                >{isNodeOnline(onlineMap, c.node_id) ? 'Online' : 'Offline'} · {c.node_id.slice(
                  0,
                  14
                )}…</span
              >
            </span>
          </button>
        </li>
      {/each}
    </ul>
  </aside>

  <main class="chat-pane">
    {#if !active}
      <div class="empty">Select a contact to chat</div>
    {:else}
      <header class="chat-header">
        <div>
          <h3>{active.name}</h3>
          <span class="sub"
            >{isNodeOnline(onlineMap, active.node_id) ? 'Online' : 'Offline'}
            {#if peerDigit} · ID {peerDigit}{/if}</span
          >
          <span class="sub mono">{active.node_id}</span>
        </div>
        <div class="call-actions">
          <button
            type="button"
            class="call-btn"
            title="Copy node id"
            onclick={() => void copyText(active!.node_id, 'Node id')}>⎘</button
          >
          <button type="button" class="call-btn" title="Voice call" onclick={() => void voiceCall()}
            >📞</button
          >
          <button type="button" class="call-btn" title="Video call" onclick={() => void videoCall()}
            >📹</button
          >
        </div>
      </header>
      <div class="messages" role="log">
        {#if loading}
          <p class="muted">Loading…</p>
        {:else if messages.length === 0}
          <p class="muted">Say hello — messages sync via P2P group room</p>
        {:else}
          {#each messages as msg (msg.messageId)}
            <div class="msg" class:mine={msg.senderId === localUserId}>
              <strong>{msg.senderName}</strong>
              <span class="msg-content">
                {#if msg.senderId !== localUserId && isLikelyPeerNodeId(msg.senderId)}
                  <MentionMessageBody
                    content={msg.content}
                    onMentionAction={onImMentionAction}
                  />
                {:else}
                  {msg.content}
                {/if}
              </span>
            </div>
          {/each}
        {/if}
      </div>
      <footer class="composer-wrap">
        {#if showMentionMenu}
          <ul class="mention-menu" role="listbox">
            {#each mentionMenu as item (item.nodeId)}
              <li>
                <button type="button" onclick={() => pickMention(item)}>
                  {item.displayName}
                  <span class="mn-sub">{item.nodeId.slice(0, 16)}…</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
        <div class="composer">
          <input
            type="text"
            bind:value={draft}
            placeholder="Message… @name to mention"
            oninput={onDraftInput}
            onkeydown={onDraftKeydown}
          />
          <button type="button" class="send" onclick={() => void sendMessage()}>Send</button>
        </div>
      </footer>
    {/if}
  </main>
</div>

<style>
  .im-messenger {
    display: flex;
    height: 360px;
    min-height: 280px;
    border: 1px solid #404040;
    border-radius: 8px;
    overflow: hidden;
    background: #1e1e1e;
  }
  .sidebar {
    width: 38%;
    min-width: 140px;
    border-right: 1px solid #333;
    display: flex;
    flex-direction: column;
  }
  .sidebar-head {
    display: flex;
    gap: 6px;
    padding: 8px;
  }
  .sidebar-head input {
    flex: 1;
    padding: 6px 8px;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 6px;
    color: #eee;
    font-size: 12px;
  }
  .icon-btn {
    background: #6366f1;
    border: none;
    color: #fff;
    width: 32px;
    border-radius: 6px;
    cursor: pointer;
  }
  .add-box {
    padding: 0 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .add-box input {
    padding: 6px;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    color: #eee;
    font-size: 11px;
  }
  .mini {
    font-size: 11px;
    padding: 4px 8px;
    background: #444;
    border: none;
    color: #eee;
    border-radius: 4px;
    cursor: pointer;
  }
  .contact-list {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
  }
  .contact-row {
    width: 100%;
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 10px 8px;
    border: none;
    background: transparent;
    color: #eee;
    cursor: pointer;
    text-align: left;
  }
  .contact-row:hover {
    background: #2d2d2d;
  }
  .contact-row.active {
    background: #3f3f9a;
  }
  .avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: #6366f1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
  }
  .avatar.online {
    box-shadow: 0 0 0 2px #22c55e;
  }
  .mini-label {
    font-size: 10px;
    color: #888;
  }
  .add-box textarea {
    width: 100%;
    font-size: 11px;
    background: #2a2a2a;
    border: 1px solid #444;
    color: #eee;
    border-radius: 4px;
  }
  .icon-btn.secondary {
    background: #444;
  }
  .sub.mono {
    font-family: ui-monospace, monospace;
    font-size: 10px;
    word-break: break-all;
  }
  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .name {
    font-size: 13px;
    font-weight: 600;
  }
  .sub {
    font-size: 10px;
    color: #888;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .chat-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
    font-size: 13px;
  }
  .chat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid #333;
  }
  .chat-header h3 {
    margin: 0;
    font-size: 15px;
  }
  .call-actions {
    display: flex;
    gap: 8px;
  }
  .call-btn {
    background: #333;
    border: 1px solid #555;
    border-radius: 50%;
    width: 36px;
    height: 36px;
    cursor: pointer;
    font-size: 16px;
  }
  .call-btn:hover {
    background: #22c55e;
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
  }
  .msg {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: 8px;
    font-size: 12px;
    max-width: 85%;
  }
  .msg.mine {
    margin-left: auto;
    text-align: right;
  }
  .muted {
    color: #888;
    font-size: 12px;
  }
  .composer-wrap {
    position: relative;
    border-top: 1px solid #333;
  }
  .mention-menu {
    position: absolute;
    bottom: 100%;
    left: 8px;
    right: 8px;
    margin: 0 0 4px;
    padding: 4px;
    list-style: none;
    background: #2a2a2a;
    border: 1px solid #6366f1;
    border-radius: 8px;
    max-height: 120px;
    overflow-y: auto;
    z-index: 4;
  }
  .mention-menu button {
    width: 100%;
    text-align: left;
    padding: 6px 8px;
    background: transparent;
    border: none;
    color: #eee;
    cursor: pointer;
    border-radius: 4px;
    font-size: 12px;
  }
  .mention-menu button:hover {
    background: #6366f1;
  }
  .mn-sub {
    display: block;
    font-size: 10px;
    color: #888;
  }
  .msg-content {
    display: block;
    margin-top: 2px;
  }
  .composer {
    display: flex;
    gap: 8px;
    padding: 8px;
  }
  .composer input {
    flex: 1;
    padding: 8px;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 6px;
    color: #eee;
  }
  .send {
    background: #6366f1;
    color: #fff;
    border: none;
    padding: 8px 14px;
    border-radius: 6px;
    cursor: pointer;
  }
</style>
