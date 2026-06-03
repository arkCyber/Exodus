#!/usr/bin/env bash
# Exodus Browser — WebChat / IM Vue 3 regression suite (no Svelte).
# Usage: ./scripts/test-webchat.sh   or   pnpm test:webchat

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm vitest run \
  src/lib/imChat.test.ts \
  src/lib/imStore.test.ts \
  src/lib/imSession.test.ts \
  src/lib/imMessengerWechat.test.ts \
  src/lib/groupMentions.test.ts \
  src/lib/chatCollections.test.ts \
  src/lib/contactDirectory.test.ts \
  src/lib/presence.test.ts \
  src/lib/socialTimeline.test.ts \
  src/lib/p2p/cdnIntegrations.test.ts \
  src/components/ImMessenger.test.ts \
  src/components/ImMessenger.wechat.test.ts \
  src/components/ImMessenger.favorites.test.ts \
  src/components/ImMessenger.collections.test.ts \
  src/components/ImMessenger.contactManagement.test.ts \
  src/components/ImMessengerIcon.test.ts \
  src/components/MentionMessageBody.test.ts \
  src/components/RtcCallHost.test.ts \
  src/components/ContactDirectoryPanel.test.ts \
  src/components/P2pSidebarPanel.test.ts

echo "✓ WebChat Vue 3 regression tests passed"
