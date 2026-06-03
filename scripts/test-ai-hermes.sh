#!/usr/bin/env bash
# Exodus Browser — AI inference + Hermes + Allama integration tests.
# Usage: sh scripts/test-ai-hermes.sh   or   pnpm test:ai-hermes

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " AI + Hermes test — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Allama + inference + Hermes (Rust)..."
sh scripts/test-allama.sh

echo ""
echo "▶ Hermes + inference clients (Vitest)..."
pnpm exec vitest run \
  src/lib/hermesClient.test.ts \
  src/lib/hermesStrategies.test.ts \
  src/lib/inferenceClient.test.ts \
  src/lib/agentActions.test.ts

echo ""
echo "▶ AI config (Vitest)..."
pnpm exec vitest run src/lib/aiConfig.test.ts

echo ""
echo "✓ AI + Hermes tests passed"
