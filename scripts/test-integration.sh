#!/usr/bin/env bash
# Exodus Browser — focused integration tests (extensions + AI/Hermes + invoke sync).
# Usage: sh scripts/test-integration.sh   or   pnpm test:integration

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " Integration test — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Dev Web Extensions..."
sh scripts/test-dev-extensions.sh

echo ""
echo "▶ AI + Hermes..."
sh scripts/test-ai-hermes.sh

echo ""
echo "▶ Tauri invoke ↔ handler sync..."
sh scripts/check-invoke-commands.sh

echo ""
echo "✓ Integration tests passed"
