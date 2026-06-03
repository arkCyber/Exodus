<script lang="ts">
  /**
   * Exodus Browser — sidebar P2P hub (group chat + CDN feed tabs).
   */
  import GroupChatPanel from '$lib/components/GroupChatPanel.svelte';
  import P2pCdnPanel from '$lib/components/P2pCdnPanel.svelte';
  import FileTransfer from '$lib/components/FileTransfer.svelte';
  import ImMessenger from './ImMessenger.svelte';
  import VideoCall from '$lib/components/VideoCall.svelte';
  import MeetingRoom from '$lib/components/MeetingRoom.svelte';
  import ContactDirectory from './ContactDirectory.svelte';
  import RtcCallHost from './RtcCallHost.svelte';
  import { listen } from '@tauri-apps/api/event';
  import { P2P_TAB_EVENT, type P2pSidebarTab } from '$lib/imChat';
  import { onMount } from 'svelte';

  type Props = {
    roomId?: string;
    onStatus: (message: string) => void;
  };

  let { roomId = $bindable('lobby'), onStatus }: Props = $props();

  let subTab = $state<'im' | 'contacts' | 'chat' | 'cdn' | 'workspace' | 'call' | 'meeting'>('im');

  onMount(() => {
    const handler = (ev: Event) => {
      const tab = (ev as CustomEvent<P2pSidebarTab>).detail;
      if (tab) subTab = tab;
    };
    window.addEventListener(P2P_TAB_EVENT, handler);
    const unsubs: Array<() => void> = [];
    void (async () => {
      unsubs.push(
        await listen('exodus-focus-workspace', () => {
          subTab = 'workspace';
        })
      );
      unsubs.push(
        await listen('exodus-focus-im', () => {
          subTab = 'im';
        }),
        await listen('exodus-open-webchat', () => {
          subTab = 'im';
        })
      );
    })();
    return () => {
      window.removeEventListener(P2P_TAB_EVENT, handler);
      for (const u of unsubs) u();
    };
  });
</script>

<div class="p2p-sidebar">
  <RtcCallHost {onStatus} />
  <div class="sub-tabs" role="tablist">
    <button
      type="button"
      role="tab"
      class:active={subTab === 'im'}
      onclick={() => {
        subTab = 'im';
      }}
    >
      WebChat
    </button>
    <button
      type="button"
      role="tab"
      class:active={subTab === 'contacts'}
      onclick={() => {
        subTab = 'contacts';
      }}
    >
      Contacts
    </button>
    <button
      type="button"
      role="tab"
      class:active={subTab === 'chat'}
      onclick={() => {
        subTab = 'chat';
      }}
    >
      Group
    </button>
    <button
      type="button"
      role="tab"
      class:active={subTab === 'cdn'}
      onclick={() => {
        subTab = 'cdn';
      }}
    >
      CDN feed
    </button>
    <button
      type="button"
      role="tab"
      class:active={subTab === 'workspace'}
      onclick={() => {
        subTab = 'workspace';
      }}
    >
      WorkSpace
    </button>
    <button
      type="button"
      role="tab"
      class:active={subTab === 'call'}
      onclick={() => {
        subTab = 'call';
      }}
    >
      Call
    </button>
    <button
      type="button"
      role="tab"
      class:active={subTab === 'meeting'}
      onclick={() => {
        subTab = 'meeting';
      }}
    >
      Meeting
    </button>
  </div>

  {#if subTab === 'im'}
    <ImMessenger {onStatus} />
  {:else if subTab === 'contacts'}
    <ContactDirectory />
  {:else if subTab === 'chat'}
    <GroupChatPanel bind:groupId={roomId} {onStatus} compact />
  {:else if subTab === 'cdn'}
    <P2pCdnPanel {roomId} {onStatus} compact />
  {:else if subTab === 'workspace'}
    <FileTransfer />
  {:else if subTab === 'call'}
    <VideoCall />
  {:else}
    <MeetingRoom />
  {/if}
</div>

<style>
  .p2p-sidebar {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }

  .sub-tabs {
    display: flex;
    gap: 6px;
  }

  .sub-tabs button {
    flex: 1;
    padding: 6px 8px;
    font-size: 12px;
    background: #333;
    border: 1px solid #444;
    border-radius: 6px;
    color: #ccc;
    cursor: pointer;
  }

  .sub-tabs button.active {
    background: #6366f1;
    border-color: #6366f1;
    color: #fff;
  }
</style>
