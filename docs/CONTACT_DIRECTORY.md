# Contact Directory (in-process hub)

The contact directory runs **inside the Exodus app process** (no Unix socket RPC). Data is stored under:

`{app_data}/contact_directory/directory.json`

## Features

- Contacts, groups, 12-digit ↔ node mappings
- `local_node_id` aligned with **P2P CDN** node id on startup
- Auto-generated stable **12-digit Exodus ID** per node (`contact_get_local_digit`)
- Survives app restart via JSON persistence

## Tauri commands

| Command | Purpose |
|---------|---------|
| `contact_directory_service_start` | Ensure hub + return hub info |
| `contact_directory_hub_info` | Storage path + node id |
| `contact_get_local_digit` | This device's 12-digit ID |
| `contact_list` / `contact_add` / … | CRUD (in-process) |

## Group chat @mentions

In the Group tab, type `@` to pick a contact or **group member** (roster = address book + `memberIds` + recent senders). Messages store `@[Name](node:…)` tokens and `mentions: [nodeId]`. Click **@Name** in the message for DM / voice / video.

## IM @mentions

In the IM tab, type `@` while chatting with a contact; sent messages support the same tokens. Incoming peer messages render clickable **@mentions** for quick call-back.

## Import / export

| Command | Purpose |
|---------|---------|
| `contact_export_json` | Full JSON backup (contacts + groups) |
| `contact_import_json` | Restore; `merge: true` upserts by node id |

Contacts UI: **Export** / **Import** buttons download or paste JSON.
