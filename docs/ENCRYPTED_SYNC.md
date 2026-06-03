# Encrypted bookmark vault & cloud sync

Local vault uses **PBKDF2 + AES-256-GCM** (`encrypted_sync.rs`). Cloud sync stores only the encrypted blob; the server never sees your passphrase.

## Local workflow

1. Settings → Privacy → **Set passphrase** (min 8 characters).
2. **Encrypt bookmarks to vault** — writes `sync_vault.enc` under app data `encrypted_sync/`.
3. Optional: **Upload to cloud** after configuring a sync server URL.

## Reference sync server (development)

```bash
# Terminal 1 — start server (stores files in ./sync-vault-data)
pnpm sync-server

# Optional bearer token
SYNC_TOKEN=dev-secret pnpm sync-server
```

In Exodus Settings → Privacy:

| Field | Example |
|-------|---------|
| Sync server URL | `http://127.0.0.1:8787/api` |
| Sync token | (empty, or `dev-secret` if `SYNC_TOKEN` set) |

Then: encrypt bookmarks → **Upload to cloud** → **Download from cloud** on another machine (same passphrase).

## HTTP API

Base URL must start with `http` or `https`. Client calls:

| Method | Path | Body |
|--------|------|------|
| `PUT` | `{base}/vault/{deviceId}` | `{ "deviceId", "payload": "<base64>", "updatedAt": <unix> }` |
| `GET` | `{base}/vault/{deviceId}` | same JSON shape |

Optional header: `Authorization: Bearer <token>`.

`deviceId` is auto-generated per install (`sync_device_id.txt`).

## Production notes

- Run behind HTTPS with a real auth layer.
- The reference server (`scripts/sync-vault-server.mjs`) is **not** hardened for public internet use.
- Rotate tokens; consider per-user vault paths instead of device-only IDs.
