<script lang="ts">
  /**
   * Exodus Browser — group message body with clickable @mentions.
   */
  import { splitMentionContent, type MentionTarget } from '$lib/groupMentions';

  type Props = {
    content: string;
    onMentionAction?: (target: MentionTarget, action: 'chat' | 'voice' | 'video') => void;
  };

  let { content, onMentionAction }: Props = $props();

  const segments = $derived(splitMentionContent(content));

  function target(seg: { displayName: string; nodeId: string }): MentionTarget {
    return { nodeId: seg.nodeId, displayName: seg.displayName };
  }
</script>

<span class="mention-body">
  {#each segments as seg, i (i)}
    {#if seg.kind === 'text'}
      {seg.text}
    {:else}
      <span class="mention-inline">
        <button
          type="button"
          class="mention-name"
          title="DM @{seg.displayName}"
          onclick={() => onMentionAction?.(target(seg), 'chat')}>@{seg.displayName}</button
        >
        {#if onMentionAction}
          <button
            type="button"
            class="mention-act"
            title="Voice"
            onclick={() => onMentionAction?.(target(seg), 'voice')}>📞</button
          >
          <button
            type="button"
            class="mention-act"
            title="Video"
            onclick={() => onMentionAction?.(target(seg), 'video')}>📹</button
          >
        {/if}
      </span>
    {/if}
  {/each}
</span>

<style>
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
    background: #3f3f9a55;
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
    padding: 0 1px;
    font-size: 12px;
    line-height: 1;
  }
</style>
