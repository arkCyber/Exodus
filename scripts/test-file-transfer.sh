#!/usr/bin/env bash
# Exodus Browser — ExodusWorkSpace + file transfer tests.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " File transfer / WorkSpace tests"
echo "════════════════════════════════════════"

echo ""
echo "▶ Rust (exodus_workspace, file_transfer, integration)..."
cargo test -p exodus-tauri exodus_workspace:: file_transfer_service:: file_transfer_integration -- --nocapture

echo ""
echo "▶ Vitest fileTransfer..."
pnpm exec vitest run src/lib/fileTransfer.test.ts

echo ""
echo "✓ File transfer tests passed"
