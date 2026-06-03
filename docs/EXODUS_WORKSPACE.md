# ExodusWorkSpace

Per-node shared folder for P2P file exchange, integrated with the P2P CDN mesh and file transfer registry.

Transfers continue while the **Tauri desktop app** is running (not tied to a browser tab). Closing the main window **hides to the system tray**; use tray → Quit to exit. Failed or paused downloads auto-resume on a 45s reconnect loop when auto-reconnect is enabled.

## Layout

```
{app_data}/ExodusWorkSpace/
  shared/          # published files (peers can fetch via CDN hash)
  inbox/           # received exports / background downloads
  outbox/          # reserved for outbound queue
  workspace.json   # manifest (names, hashes, paths)

{app_data}/file_transfers/
  {transfer_id}/   # chunks, resume.json, checksum_report.json

{app_data}/transfer_engine_settings.json
{app_data}/wan_relay.json
```

## Network

- Gossip / CDN room id: **`ExodusWorkSpace`** (joined at startup with `lobby`).
- Each node runs an HTTP mesh server; tickets look like  
  `exodus-cdn://{node_id}@{lan_ip}:{port}/{blake3_hash}`.
- **WAN relay (embedded)**: default `http://127.0.0.1:8790/exodus-mesh/fetch?host=H&port=P&path=/blobs/...`  
  Proxies to the peer mesh HTTP server for NAT traversal.

## Download pipeline (background engine)

1. Local CDN blob cache hit → export to inbox  
2. **`fetch_from_mesh_peers`** (same path as `p2p_cdn_download` peer fetch)  
3. **`start_cdn_download`** full CDN orchestration (peers + HTTP fallback)  
4. Chunk-by-chunk mesh + relay URLs with resume + throttle  
5. BLAKE3 verify + `checksum_report.json`

## Reliability

| Feature | Behavior |
|---------|----------|
| Background jobs | Tokio tasks in `FileTransferEngine` while app is open |
| System tray | Close hides window; transfers continue until Quit |
| Resume | `resume.json` per transfer; chunk index checkpoint |
| Auto-reconnect | Every 45s, re-queue pending/failed/paused downloads |
| Checksum | BLAKE3 per chunk + file; `file_transfer_verify_checksum` |
| Throttle | `throttle_bytes_per_sec` in engine settings |
| Workspace watch | `notify` on `shared/` → auto-publish to CDN |

## Tauri commands

| Command | Purpose |
|---------|---------|
| `exodus_workspace_info` | Paths, node id, mesh host/port, file count |
| `exodus_workspace_list` | Manifest entries in `shared/` |
| `exodus_workspace_watch_start` / `_stop` | Auto-sync watcher on `shared/` |
| `file_transfer_service_start` | Initialize hub (workspace + engine + relay) |
| `file_transfer_pick_file` | Native system file picker |
| `file_transfer_initiate` | Publish + CDN seed + register upload |
| `file_transfer_dashboard` | All transfers + engine settings |
| `file_transfer_set_throttle` | Bandwidth cap (bytes/sec, 0 = unlimited) |
| `file_transfer_set_auto_reconnect` | Toggle reconnect loop |
| `file_transfer_set_relay_config` | WAN relay client base URL |
| `file_transfer_set_relay_serve` | Embedded relay server on/off + port |
| `wan_relay_server_info` | Local relay HTTP server status |
| `file_transfer_start_background_download` | Resume download into `inbox/` |
| `file_transfer_verify_checksum` | Build/verify checksum report |
| `p2p_cdn_download` | CDN job API (also used internally by transfer engine) |

Events: `file-transfer-progress`, `file-transfer-initiated`, `exodus-workspace-file-added`, `exodus-window-hidden`, `exodus-focus-workspace`.

## UI

Sidebar **P2P → WorkSpace** tab (`FileTransfer.svelte`): dashboard, throttle, relay, watch, pick file, live progress.

## Tests

```bash
sh scripts/test-file-transfer.sh
cargo test -p exodus-tauri wan_relay exodus_workspace file_transfer
pnpm test src/lib/fileTransfer.test.ts
```
