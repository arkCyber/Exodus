#!/usr/bin/env bash
# Exodus Browser — focused P2P CDN test suite (Rust + Vitest).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " P2P CDN tests — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Rust p2p_cdn module tests..."
cargo test -p exodus-tauri p2p_cdn:: -- --nocapture

echo ""
echo "▶ Vitest p2p/cdn..."
pnpm exec vitest run src/lib/p2p/cdn.test.ts src/lib/p2p/cdnIntegrations.test.ts src/lib/p2p/cdnPageStatus.test.ts

echo ""
echo "▶ Invoke sync (p2p_cdn commands)..."
sh scripts/check-invoke-commands.sh

echo ""
echo "════════════════════════════════════════"
echo " ✓ P2P CDN tests passed"
echo "════════════════════════════════════════"
