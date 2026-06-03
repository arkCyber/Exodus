# Exodus P2P CDN (iroh-blobs compatible)

AI-driven decentralized content distribution for the Exodus browser: when a room or lobby
discusses a large article, video model, or local LLM weights, peers share bytes via
**content addressing (BLAKE3)** instead of repeated origin downloads.

## Architecture

```
AI recommendation / group chat
        │
        ▼
  gossip topic `exodus-cdn-{roomId}`  ──►  swarm index (hash → peers)
        │
        ▼
  download orchestrator
    1. local blob store hit
    2. parallel peer fetch (iroh-blobs when `iroh-cdn` feature enabled)
    3. HTTP fallback → import → re-announce as seed
```

| Component | Path |
|-----------|------|
| Blob store (16 KiB chunks) | `src-tauri/src/p2p_cdn/store.rs` |
| Gossip bridge | `src-tauri/src/p2p_cdn/gossip_bridge.rs` |
| Swarm + rooms | `src-tauri/src/p2p_cdn/swarm.rs` |
| Download | `src-tauri/src/p2p_cdn/download.rs` |
| iroh adapter | `src-tauri/src/p2p_cdn/iroh_adapter.rs` |
| Tauri API | `src-tauri/src/p2p_cdn/commands.rs` |
| Frontend | `src/lib/p2p/cdn.ts`, `P2pCdnPanel.svelte` |

## Tauri commands

- `p2p_cdn_join_room` / `p2p_cdn_leave_room` — also subscribes to `exodus-cdn-{room}` on gossip service
- `p2p_cdn_room_feed` — assets + peer map (syncs external gossip first)
- `p2p_cdn_sync_gossip` — pull from in-process bus + `p2p_gossip` microservice
- `p2p_cdn_hash_file` — BLAKE3 hash + size for a local path
- `p2p_cdn_announce_asset` — AI hot-content payload
- `p2p_cdn_announce_group_hot` — group hot link or `localPath` seed
- `p2p_cdn_download` — peers-first download
- `p2p_cdn_register_local_seed` — seed after manual import

Events: `exodus-p2p-cdn-progress`

## Exodus Mesh HTTP (default)

Each node runs an HTTP server on a random port:

- `GET /health` — liveness
- `GET /blobs/{blake3_hash}` — full blob (small files)
- `GET /blobs/{hash}/meta` — `{ sizeBytes, chunkCount }`
- `GET /blobs/{hash}/chunks/{index}` — parallel chunk fetch (16 KiB aligned store)

Gossip tickets use:

`exodus-cdn://{node_id}@{lan_ip}:{port}/{content_hash}`

Compatible with iroh-blobs **content addressing** (BLAKE3); transport is Exodus mesh until
`iroh-cdn` feature wires real `iroh-blobs::Downloader`.

## Enable real iroh-blobs (optional)

```bash
cd src-tauri
cargo build --features iroh-cdn
```

Note: iroh 0.90 may not compile on all platforms (netwatch); mesh HTTP works everywhere.

## Group / AI integration

- `p2p_cdn_announce_group_hot` — group chat hot file or link
- `p2p_cdn_announce_url_hot` — RAG / large page URL (discovery hash = BLAKE3(url))
- `p2p_cdn_group_send_message` — group message + auto CDN announce per attachment
- `p2p_cdn_announce_asset` — AI recommendation JSON payload

**Settings UI**: Group chat panel + P2P CDN panel share the same room id. Attach a file in
group chat to hash, seed, and announce to the swarm.

**Sidebar (toolbar P2P icon)**:
- Tab **Group chat** / **CDN feed** in the right sidebar
- AI panel: **Share page to P2P CDN** and per-URL chips on assistant replies
- `/ask` search results: **P2P** button per hit
- **Omnibox badge** (`p2p_cdn_url_status`): `P2P · N` peers, `P2P · cached`, or `P2P · listed`
- AI stream: highlight **P2P seed** chips for `.gguf` / video / archive URLs while typing
- After AI reply: banner **Announce all to room** when large-file links detected

Command: `p2p_cdn_url_status` — query swarm status for the current page URL.

**Auto-announce**: When auto-index captures a page ≥ 500 KB text, it is announced to the `lobby`
room (see `src/lib/p2p/cdnIntegrations.ts`).

## Automated tests

```bash
# Focused P2P CDN only (~30s)
sh scripts/test-p2p-cdn.sh

# Full project gate (includes P2P CDN)
pnpm verify
```

Rust coverage includes: store chunking, mesh HTTP server, parallel mesh fetch,
swarm seed/fetch pipeline, gossip JSON roundtrip, and integration tests in
`src-tauri/src/p2p_cdn/integration_tests.rs`.

## Gossip microservice bridge

When `p2p_gossip_service_start` is running, CDN announcements are also published to
`exodus-cdn-{roomId}` topics. Other Exodus processes (or a second app instance) can subscribe
via the same gossip socket and see hot content without sharing in-process memory.

Start gossip from devtools (optional):

```js
await invoke('p2p_gossip_service_start');
```

## ExodusWorkSpace

Shared per-node folder `{app_data}/ExodusWorkSpace/` (room id `ExodusWorkSpace`) for file publish/receive via CDN + transfer registry. See [`docs/EXODUS_WORKSPACE.md`](EXODUS_WORKSPACE.md).

## Hand-test

1. `pnpm tauri dev` → Settings → **P2P CDN · lobby**
2. Use **Announce hot content** to hash/seed a local file or post a hash + HTTP URL
2. From devtools console (example):

```js
import { invoke } from '@tauri-apps/api/core';
await invoke('p2p_cdn_announce_asset', {
  roomId: 'lobby',
  payload: {
    contentHash: 'abc123',
    title: 'Demo 5GB Model',
    kind: 'ai_model',
    sizeBytes: 5000000000,
    sourceUrl: 'https://example.com/model.bin'
  }
});
```

3. Refresh feed → see asset; first download uses HTTP; second client (future: second machine with iroh) pulls from peers.
