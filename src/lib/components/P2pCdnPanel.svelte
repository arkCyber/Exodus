<script lang="ts">
  /**
   * Exodus Browser — P2P CDN room panel (AI lobby / group hot content).
   */
  import {
    cdnSourceLabel,
    listenP2pCdnProgress,
    p2pCdnAnnounceGroupHot,
    p2pCdnDownload,
    p2pCdnHashFile,
    p2pCdnJoinRoom,
    p2pCdnStartMesh,
    p2pCdnRoomFeed,
    p2pCdnSyncGossip,
    type CdnAsset,
    type CdnContentKind,
    type CdnDownloadProgress,
    type CdnRoomFeed,
  } from '$lib/p2p/cdn';

  type Props = {
    roomId?: string;
    onStatus: (message: string) => void;
    compact?: boolean;
  };

  let { roomId = 'lobby', onStatus, compact = false }: Props = $props();

  let nodeId = $state('');
  let meshHost = $state<string | null>(null);
  let meshPort = $state<number | null>(null);
  let feed = $state<CdnRoomFeed | null>(null);
  let loading = $state(false);
  let progress = $state<CdnDownloadProgress | null>(null);
  let showAnnounce = $state(false);

  let announceTitle = $state('');
  let announceHash = $state('');
  let announceUrl = $state('');
  let announceKind = $state<CdnContentKind>('article');
  let announceSize = $state('0');

  /** Refresh room feed from gossip swarm index. */
  async function refresh() {
    loading = true;
    try {
      await p2pCdnJoinRoom(roomId);
      const info = await p2pCdnStartMesh();
      nodeId = info.nodeId;
      meshHost = info.meshHost ?? null;
      meshPort = info.meshPort ?? null;
      await p2pCdnSyncGossip(roomId);
      feed = await p2pCdnRoomFeed(roomId);
    } catch (error) {
      console.error('p2p_cdn_room_feed failed:', error);
      onStatus('P2P CDN feed failed');
    } finally {
      loading = false;
    }
  }

  /** Download asset with peers-first orchestration. */
  async function downloadAsset(asset: CdnAsset) {
    try {
      const peers = feed?.peerMap[asset.contentHash]?.length ?? 0;
      onStatus(
        peers > 0
          ? `Downloading ${asset.title} from ${peers} peer(s)…`
          : `Downloading ${asset.title} (HTTP fallback)…`,
      );
      const job = await p2pCdnDownload({
        roomId: asset.roomId,
        contentHash: asset.contentHash,
        title: asset.title,
        kind: asset.kind,
        httpUrl: asset.sourceUrl,
      });
      onStatus(`${asset.title}: ${cdnSourceLabel(job.source, job.peerCount)}`);
      await refresh();
    } catch (error) {
      console.error('p2p_cdn_download failed:', error);
      onStatus(`Download failed: ${error}`);
    }
  }

  /** Announce hot link by hash + optional HTTP URL. */
  async function submitAnnounce() {
    const title = announceTitle.trim();
    const contentHash = announceHash.trim();
    if (!title || !contentHash) {
      onStatus('Title and content hash are required');
      return;
    }
    const sizeBytes = Number.parseInt(announceSize, 10) || 0;
    try {
      await p2pCdnAnnounceGroupHot({
        groupId: roomId,
        title,
        contentHash,
        kind: announceKind,
        sizeBytes,
        sourceUrl: announceUrl.trim() || undefined,
      });
      onStatus(`Announced ${title} to room ${roomId}`);
      announceTitle = '';
      announceHash = '';
      announceUrl = '';
      await refresh();
    } catch (error) {
      console.error('p2p_cdn_announce_group_hot failed:', error);
      onStatus(`Announce failed: ${error}`);
    }
  }

  /** Hash a local file and fill the content hash field. */
  async function hashLocalFile() {
    const path = window.prompt('Path to local file to hash (BLAKE3):');
    if (!path?.trim()) return;
    try {
      const { contentHash, sizeBytes } = await p2pCdnHashFile(path.trim());
      announceHash = contentHash;
      announceSize = String(sizeBytes);
      if (!announceTitle.trim()) {
        const base = path.trim().split(/[/\\]/).pop() ?? 'Local file';
        announceTitle = base;
      }
      onStatus(`Hash: ${contentHash.slice(0, 16)}… (${formatSize(sizeBytes)})`);
    } catch (error) {
      console.error('p2p_cdn_hash_file failed:', error);
      onStatus(`Hash failed: ${error}`);
    }
  }

  /** Import local file into blob store and announce as seeder. */
  async function seedLocalFile() {
    const path = window.prompt('Path to local file to seed into the swarm:');
    if (!path?.trim()) return;
    const title =
      announceTitle.trim() ||
      window.prompt('Title for this asset:', path.trim().split(/[/\\]/).pop() ?? 'File') ||
      'Shared file';
    if (!title.trim()) return;
    try {
      onStatus('Importing and seeding…');
      await p2pCdnAnnounceGroupHot({
        groupId: roomId,
        title: title.trim(),
        contentHash: announceHash.trim() || 'pending',
        kind: announceKind,
        sizeBytes: Number.parseInt(announceSize, 10) || 0,
        localPath: path.trim(),
        sourceUrl: announceUrl.trim() || undefined,
      });
      onStatus(`Seeded ${title.trim()} in ${roomId}`);
      await refresh();
    } catch (error) {
      console.error('seed local failed:', error);
      onStatus(`Seed failed: ${error}`);
    }
  }

  function formatSize(bytes: number): string {
    if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
    if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
    if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(1)} KB`;
    return `${bytes} B`;
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void refresh();
    let unlisten: (() => void) | undefined;
    void listenP2pCdnProgress((p) => {
      progress = p;
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  });
</script>

<div class="p2p-cdn-panel" class:compact>
  {#if !compact}
    <h4 class="subsection-title">P2P CDN · {roomId}</h4>
    <p class="settings-hint">
      Content-addressed (BLAKE3 / iroh-blobs). If a peer in this room already has the file, download
      runs in parallel from their node — zero origin bandwidth after the first seed.
    </p>
  {:else}
    <p class="settings-hint muted">P2P CDN · {roomId}</p>
  {/if}
  {#if nodeId}
    <p class="settings-hint muted">Node: <code>{nodeId}</code></p>
  {/if}
  {#if meshHost && meshPort}
    <p class="settings-hint muted">
      Mesh: <code>http://{meshHost}:{meshPort}</code> (peers fetch <code>/blobs/&lt;hash&gt;</code>)
    </p>
  {/if}
  <div class="toolbar">
    <button type="button" class="nav-button secondary" disabled={loading} onclick={() => void refresh()}>
      Refresh feed
    </button>
    <button
      type="button"
      class="nav-button secondary"
      onclick={() => {
        showAnnounce = !showAnnounce;
      }}
    >
      {showAnnounce ? 'Hide announce' : 'Announce hot content'}
    </button>
  </div>

  {#if showAnnounce}
    <div class="announce-form">
      <label class="field">
        Title
        <input type="text" bind:value={announceTitle} placeholder="Llama 8B GGUF" />
      </label>
      <label class="field">
        Content hash (BLAKE3 hex)
        <input type="text" bind:value={announceHash} placeholder="64-char hex" />
      </label>
      <label class="field">
        Size (bytes)
        <input type="text" bind:value={announceSize} placeholder="5000000000" />
      </label>
      <label class="field">
        HTTP fallback URL
        <input type="text" bind:value={announceUrl} placeholder="https://…" />
      </label>
      <label class="field">
        Kind
        <select bind:value={announceKind}>
          <option value="article">article</option>
          <option value="ai_model">ai_model</option>
          <option value="video_model">video_model</option>
          <option value="dataset">dataset</option>
          <option value="generic_file">generic_file</option>
        </select>
      </label>
      <div class="announce-actions">
        <button type="button" class="nav-button secondary" onclick={() => void hashLocalFile()}>
          Hash local file
        </button>
        <button type="button" class="nav-button secondary" onclick={() => void seedLocalFile()}>
          Seed local file
        </button>
        <button type="button" class="nav-button" onclick={() => void submitAnnounce()}>
          Announce to room
        </button>
      </div>
    </div>
  {/if}

  {#if progress}
    <p class="progress-line">
      {cdnSourceLabel(progress.source, progress.peerCount)} · {progress.progressPercent.toFixed(0)}%
    </p>
  {/if}

  {#if loading}
    <p class="settings-hint">Loading room feed…</p>
  {:else if !feed || feed.assets.length === 0}
    <p class="settings-hint">No hot content in this room yet. Announce a link or seed a local file.</p>
  {:else}
    <ul class="cdn-asset-list">
      {#each feed.assets as asset (asset.contentHash)}
        {@const peers = feed.peerMap[asset.contentHash] ?? []}
        <li class="cdn-asset-item">
          <div class="cdn-asset-meta">
            <strong>{asset.title}</strong>
            <span class="muted">{asset.kind} · {formatSize(asset.sizeBytes)}</span>
            <span class="muted hash"><code>{asset.contentHash.slice(0, 16)}…</code></span>
            <span class="peer-badge" class:active={peers.length > 0}>
              {peers.length > 0 ? `${peers.length} peer(s) seeding` : 'HTTP origin only'}
            </span>
          </div>
          <button
            type="button"
            class="nav-button secondary"
            onclick={() => void downloadAsset(asset)}
          >
            {peers.length > 0 ? 'P2P download' : 'Download'}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .p2p-cdn-panel {
    margin-top: 12px;
  }

  .p2p-cdn-panel.compact {
    margin-top: 0;
  }

  .p2p-cdn-panel.compact .cdn-asset-list {
    max-height: 220px;
    overflow-y: auto;
  }

  .subsection-title {
    margin: 0 0 8px;
    font-size: 13px;
    color: #ccc;
  }

  .muted {
    color: #888;
    font-size: 12px;
  }

  .hash code {
    font-size: 11px;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 8px 0;
  }

  .announce-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 12px 0;
    padding: 12px;
    background: #252525;
    border: 1px solid #404040;
    border-radius: 8px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: #aaa;
  }

  .field input,
  .field select {
    padding: 6px 8px;
    background: #1a1a1a;
    border: 1px solid #404040;
    border-radius: 4px;
    color: #eee;
    font-size: 13px;
  }

  .announce-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }

  .progress-line {
    font-size: 12px;
    color: #7dd3fc;
    margin: 8px 0;
  }

  .cdn-asset-list {
    list-style: none;
    margin: 12px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .cdn-asset-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: #2a2a2a;
    border: 1px solid #404040;
    border-radius: 8px;
  }

  .cdn-asset-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .peer-badge {
    font-size: 11px;
    color: #888;
  }

  .peer-badge.active {
    color: #4ade80;
  }
</style>
