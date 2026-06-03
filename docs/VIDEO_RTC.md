# Exodus Video RTC (WeChat-style)

Real-time **1:1 calls** and **multi-party meetings** over WebRTC, with signaling via the P2P gossip bus and an in-process RTC hub.

## Architecture

```
Frontend (WebRTC)
  RTCPeerConnection + getUserMedia
        │
        ▼
  video_rtc_publish_signal / poll_signals
        │
        ▼
  Gossip topic + local signal store
        │
        ▼
  Remote peer (same topic)
```

| Mode | Gossip topic | WebRTC topology |
|------|----------------|-----------------|
| 1:1 Call | `exodus-rtc-peer-{nodeA}-{nodeB}` | Single peer connection |
| Meeting | `exodus-rtc-meeting-{meetingId}` | Full mesh (≤12 participants) |

## UI

| Tab | Component | Purpose |
|-----|-----------|---------|
| **P2P → IM** | `ImMessenger.svelte` | WeChat-style: contacts + chat + 📞/📹 in header |
| **P2P → Contacts** | `ContactDirectory.svelte` | Address book; 💬📞📹 quick actions → IM |
| **P2P → Call** | `VideoCall.svelte` | Direct dial by node id |
| **P2P → Meeting** | `MeetingRoom.svelte` | Multi-party room |

1:1 chat uses a stable DM room: `dm-{nodeA}-{nodeB}` (group-chat backend).

From **Contacts**, tap 💬 to switch to **IM** and open the thread; 📞/📹 start a call overlay on **IM**.

Add friends via **12-digit Exodus ID** (`contact_add_friend_by_digit`) in IM or Contacts.

## Tauri commands

| Command | Purpose |
|---------|---------|
| `video_rtc_service_start` | Init hub + gossip subscription |
| `video_rtc_node_info` | Local node id |
| `video_rtc_call_start` | Start 1:1 call + ring signal |
| `video_rtc_call_update` | Update call status |
| `video_rtc_meeting_create` | Create meeting room |
| `video_rtc_meeting_join` / `_leave` | Join or leave |
| `video_rtc_publish_signal` | Publish WebRTC SDP/ICE |
| `video_rtc_poll_signals` | Poll signaling messages |

Events: `exodus-rtc-incoming-call`, `exodus-rtc-signal`, `exodus-rtc-meeting-update`

## NAT / production

Default STUN: Google public STUN. For strict NAT or WAN, configure TURN in `src/lib/webrtc/rtcConfig.ts`.

## Tests

```bash
cargo test -p exodus-tauri video_rtc
pnpm test src/lib/videoRtc.test.ts
```
