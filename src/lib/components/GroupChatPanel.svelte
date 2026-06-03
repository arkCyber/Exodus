<script lang="ts">
  /**
   * Exodus Browser — group chat with P2P CDN attachment seeding and @mention quick call.
   */
  import { onMount } from 'svelte';
  import {
    buildGroupPayload,
    groupChatServiceStart,
    groupCreate,
    groupGet,
    groupGetMembers,
    groupGetMessages,
    groupListUser,
    type GroupMessage,
  } from '$lib/groupChat';
  import {
    prepareGroupFileAttachment,
    sendGroupMessageWithCdn,
    type GroupChatMessage,
  } from '$lib/p2p/cdnIntegrations';
  import { p2pCdnJoinRoom } from '$lib/p2p/cdn';
  import { isLikelyPeerNodeId } from '$lib/imSession';
  import { openImChat, openP2pTab, startCallFromUi } from '$lib/imChat';
  import type { Contact } from '$lib/contactDirectory';
  import { contactDirectoryServiceStart, contactList } from '$lib/contactDirectory';
  import {
    extractMentionNodeIds,
    insertMentionToken,
    mergeMentionCandidates,
    mentionCandidatesForQueryFromTargets,
    resolveMentionNodeId,
    type MentionTarget,
  } from '$lib/groupMentions';
  import MentionMessageBody from '$lib/components/MentionMessageBody.svelte';

  type Props = {
    groupId?: string;
    onStatus: (message: string) => void;
    /** Sidebar layout: tighter spacing, shorter copy. */
    compact?: boolean;
  };

  let { groupId = $bindable('lobby'), onStatus, compact = false }: Props = $props();

  const userId = 'exodus-local-user';
  let serviceReady = $state(false);
  let messages = $state<GroupMessage[]>([]);
  let draft = $state('');
  let loading = $state(false);
  let groupName = $state('');
  let contacts = $state<Contact[]>([]);
  let mentionTargets = $state<MentionTarget[]>([]);
  let mentionMenu = $state<MentionTarget[]>([]);
  let showMentionMenu = $state(false);

  async function ensureService() {
    if (serviceReady) return;
    await groupChatServiceStart();
    serviceReady = true;
  }

  async function loadContacts() {
    try {
      await contactDirectoryServiceStart();
      contacts = await contactList();
    } catch {
      contacts = [];
    }
  }

  async function refreshMentionTargets() {
    const extras: Array<{ nodeId: string; displayName: string }> = [];
    if (groupId.trim()) {
      const g = await groupGet(groupId);
      if (g) {
        for (const id of g.memberIds) {
          if (id === userId) continue;
          const c = contacts.find((x) => x.node_id === id);
          extras.push({ nodeId: id, displayName: c?.name ?? id.slice(0, 14) });
        }
      }
      try {
        const members = await groupGetMembers(groupId);
        for (const m of members) {
          const id = m.agentId;
          if (!isLikelyPeerNodeId(id)) continue;
          extras.push({
            nodeId: id,
            displayName: m.nickname?.trim() || m.agentName || id.slice(0, 14),
          });
        }
      } catch {
        /* members optional */
      }
    }
    for (const msg of messages) {
      if (msg.senderId !== userId && isLikelyPeerNodeId(msg.senderId)) {
        extras.push({ nodeId: msg.senderId, displayName: msg.senderName });
      }
    }
    mentionTargets = mergeMentionCandidates(contacts, extras);
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
      void sendText();
      e.preventDefault();
    }
  }

  async function loadMessages() {
    if (!groupId.trim()) return;
    loading = true;
    try {
      await ensureService();
      await p2pCdnJoinRoom(groupId);
      messages = await groupGetMessages(groupId, 80);
      const groups = await groupListUser(userId);
      groupName = groups.find((g) => g.groupId === groupId)?.name ?? groupId;
      await refreshMentionTargets();
    } catch (error) {
      console.error('group_get_messages failed:', error);
      onStatus('Group chat unavailable — start service in devtools if needed');
    } finally {
      loading = false;
    }
  }

  async function createDemoGroup() {
    const id = `group-${Date.now()}`;
    const name = window.prompt('Group name:', 'Exodus P2P Room') ?? 'Exodus P2P Room';
    try {
      await ensureService();
      await groupCreate(
        buildGroupPayload({
          groupId: id,
          name,
          description: 'P2P CDN + group chat',
          ownerId: userId,
          memberIds: [userId],
        }),
      );
      groupId = id;
      onStatus(`Created group ${name}`);
      await loadMessages();
    } catch (error) {
      console.error('group_create failed:', error);
      onStatus(`Create group failed: ${error}`);
    }
  }

  async function sendText() {
    const text = draft.trim();
    if (!text || !groupId) return;
    const mentions = extractMentionNodeIds(text, contacts);
    try {
      await ensureService();
      const msg: GroupChatMessage = {
        messageId: `msg-${Date.now()}`,
        groupId,
        senderId: userId,
        senderName: 'You',
        content: text,
        messageType: 'text',
        attachments: [],
        mentions,
        timestamp: Date.now(),
        isEdited: false,
      };
      await sendGroupMessageWithCdn(msg);
      draft = '';
      showMentionMenu = false;
      await loadMessages();
      onStatus(mentions.length ? `Sent (mentioned ${mentions.length})` : 'Message sent');
    } catch (error) {
      console.error('send message failed:', error);
      onStatus(`Send failed: ${error}`);
    }
  }

  function chatWithTarget(t: MentionTarget) {
    openP2pTab('im');
    openImChat({
      contactId: t.contactId ?? t.nodeId,
      name: t.displayName,
      nodeId: t.nodeId,
    });
  }

  function callTarget(t: MentionTarget, video: boolean) {
    openP2pTab('im');
    startCallFromUi({ nodeId: t.nodeId, name: t.displayName, video, audio: true });
  }

  function onMentionAction(t: MentionTarget, action: 'chat' | 'voice' | 'video') {
    if (action === 'chat') chatWithTarget(t);
    else callTarget(t, action === 'video');
  }

  function chatWithSender(senderId: string, senderName: string) {
    if (!isLikelyPeerNodeId(senderId)) {
      onStatus('Sender is not a P2P node — cannot open DM');
      return;
    }
    openP2pTab('im');
    openImChat({ contactId: senderId, name: senderName, nodeId: senderId });
  }

  function callSender(senderId: string, senderName: string, video: boolean) {
    if (!isLikelyPeerNodeId(senderId)) {
      onStatus('Sender is not a P2P node — cannot call');
      return;
    }
    openP2pTab('im');
    startCallFromUi({ nodeId: senderId, name: senderName, video, audio: true });
  }

  /** Unique mention targets on a message (body + mentions[]). */
  function messageMentionTargets(msg: GroupMessage): MentionTarget[] {
    const ids = new Set<string>([
      ...extractMentionNodeIds(msg.content, contacts),
      ...msg.mentions.filter((id) => isLikelyPeerNodeId(id)),
    ]);
    const out: MentionTarget[] = [];
    for (const id of ids) {
      const t = resolveMentionNodeId(id, contacts);
      if (t) out.push(t);
    }
    return out;
  }

  async function attachFile() {
    const path = window.prompt('Path to file to attach (will seed P2P CDN):');
    if (!path?.trim() || !groupId) return;
    const caption = window.prompt('Message caption (optional):', path.split(/[/\\]/).pop() ?? '') ?? '';
    try {
      await ensureService();
      const att = await prepareGroupFileAttachment(groupId, path.trim());
      const msg: GroupChatMessage = {
        messageId: `msg-${Date.now()}`,
        groupId,
        senderId: userId,
        senderName: 'You',
        content: caption || `Shared ${att.fileName}`,
        messageType: 'file',
        attachments: [att],
        mentions: extractMentionNodeIds(caption, contacts),
        timestamp: Date.now(),
        isEdited: false,
      };
      await sendGroupMessageWithCdn(msg);
      await loadMessages();
      onStatus(`File ${att.fileName} seeded to CDN room ${groupId}`);
    } catch (error) {
      console.error('attach file failed:', error);
      onStatus(`Attach failed: ${error}`);
    }
  }

  onMount(() => {
    void loadContacts().then(() => refreshMentionTargets());
  });

  $effect(() => {
    if (typeof window === 'undefined' || !groupId) return;
    void loadMessages();
  });

  $effect(() => {
    if (contacts.length >= 0 && messages.length >= 0 && groupId) {
      void refreshMentionTargets();
    }
  });
</script>

<div class="group-chat-panel" class:compact>
  {#if !compact}
    <h4 class="subsection-title">Group chat · P2P CDN</h4>
    <p class="settings-hint">
      Type <code>@name</code> to mention a contact — then use 💬 📞 📹 on their message. Room
      <code>{groupId}</code> shares the CDN panel below.
    </p>
  {:else}
    <p class="settings-hint muted">
      <code>{groupId}</code> · <code>@contact</code> for mention &amp; call
    </p>
  {/if}
  {#if groupName}
    <p class="settings-hint muted">{groupName}</p>
  {/if}
  <div class="toolbar">
    <button type="button" class="nav-button secondary" onclick={() => void createDemoGroup()}>
      New group
    </button>
    <button type="button" class="nav-button secondary" disabled={loading} onclick={() => void loadMessages()}>
      Refresh
    </button>
    <label class="field inline">
      Room id
      <input type="text" bind:value={groupId} placeholder="lobby or group-…" />
    </label>
  </div>

  <div class="message-list" role="log">
    {#if loading}
      <p class="settings-hint">Loading messages…</p>
    {:else if messages.length === 0}
      <p class="settings-hint">No messages yet. Send text or attach a file.</p>
    {:else}
      {#each messages as msg (msg.messageId)}
        {@const targets = messageMentionTargets(msg)}
        <div class="msg-row">
          <div class="msg-head">
            <strong>{msg.senderName}</strong>
            {#if isLikelyPeerNodeId(msg.senderId)}
              <span class="peer-actions">
                <button
                  type="button"
                  class="link-btn"
                  title="DM"
                  onclick={() => chatWithSender(msg.senderId, msg.senderName)}>💬</button
                >
                <button
                  type="button"
                  class="link-btn"
                  title="Voice"
                  onclick={() => callSender(msg.senderId, msg.senderName, false)}>📞</button
                >
                <button
                  type="button"
                  class="link-btn"
                  title="Video"
                  onclick={() => callSender(msg.senderId, msg.senderName, true)}>📹</button
                >
              </span>
            {/if}
          </div>
          <span class="muted msg-content">
            <MentionMessageBody content={msg.content} onMentionAction={onMentionAction} />
          </span>
          {#if targets.length > 0}
            <div class="mention-bar" role="group" aria-label="Mentioned — quick actions">
              {#each targets as t (t.nodeId)}
                <span class="mention-chip">
                  @{t.displayName}
                  <button type="button" class="link-btn" title="DM" onclick={() => chatWithTarget(t)}
                    >💬</button
                  >
                  <button
                    type="button"
                    class="link-btn"
                    title="Voice"
                    onclick={() => callTarget(t, false)}>📞</button
                  >
                  <button
                    type="button"
                    class="link-btn"
                    title="Video"
                    onclick={() => callTarget(t, true)}>📹</button
                  >
                </span>
              {/each}
            </div>
          {/if}
          {#if msg.attachments.length > 0}
            <span class="att-badge">
              {msg.attachments.length} attachment(s) · {msg.attachments[0].blobHash.slice(0, 12)}…
            </span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  <div class="composer-wrap">
    {#if showMentionMenu}
      <ul class="mention-menu" role="listbox">
        {#each mentionMenu as item (item.nodeId)}
          <li>
            <button type="button" onclick={() => pickMention(item)}>
              <span class="mn-name">{item.displayName}</span>
              <span class="mn-sub">{item.nodeId.slice(0, 18)}…</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
    <div class="composer">
      <input
        type="text"
        bind:value={draft}
        placeholder="Message… @contact to mention"
        oninput={onDraftInput}
        onkeydown={onDraftKeydown}
      />
      <button type="button" class="nav-button secondary" onclick={() => void attachFile()}>Attach file</button>
      <button type="button" class="nav-button" onclick={() => void sendText()}>Send</button>
    </div>
  </div>
</div>

<style>
  .group-chat-panel {
    margin-top: 16px;
  }

  .group-chat-panel.compact {
    margin-top: 0;
  }

  .group-chat-panel.compact .message-list {
    max-height: 160px;
  }

  .subsection-title {
    margin: 0 0 8px;
    font-size: 13px;
    color: #ccc;
  }

  .settings-hint {
    font-size: 12px;
    color: #aaa;
    margin: 4px 0;
  }

  .muted {
    color: #888;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: flex-end;
    margin: 8px 0;
  }

  .field.inline {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: #aaa;
    flex: 1;
    min-width: 140px;
  }

  .field.inline input {
    padding: 6px 8px;
    background: #1a1a1a;
    border: 1px solid #404040;
    border-radius: 4px;
    color: #eee;
  }

  .message-list {
    max-height: 200px;
    overflow-y: auto;
    padding: 8px;
    background: #252525;
    border: 1px solid #404040;
    border-radius: 8px;
    margin-bottom: 8px;
  }

  .msg-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .peer-actions {
    display: inline-flex;
    gap: 4px;
  }
  .link-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0 2px;
    font-size: 14px;
  }
  .msg-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 0;
    border-bottom: 1px solid #333;
    font-size: 12px;
  }

  .mention-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }

  .mention-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: #3f3f9a44;
    border: 1px solid #6366f1;
    border-radius: 12px;
    font-size: 11px;
    color: #c7d2fe;
  }

  .att-badge {
    font-size: 11px;
    color: #4ade80;
  }

  .composer-wrap {
    position: relative;
  }

  .mention-menu {
    position: absolute;
    bottom: 100%;
    left: 0;
    right: 0;
    margin: 0 0 4px;
    padding: 4px;
    list-style: none;
    background: #2a2a2a;
    border: 1px solid #6366f1;
    border-radius: 8px;
    max-height: 140px;
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
  }

  .mention-menu button:hover {
    background: #6366f1;
  }

  .mn-name {
    display: block;
    font-weight: 600;
    font-size: 12px;
  }

  .mn-sub {
    font-size: 10px;
    color: #888;
  }

  .composer {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .composer input {
    flex: 1;
    min-width: 120px;
    padding: 6px 8px;
    background: #1a1a1a;
    border: 1px solid #404040;
    border-radius: 4px;
    color: #eee;
  }
</style>
