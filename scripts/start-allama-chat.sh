#!/usr/bin/env bash
# Exodus — start native Allama for sidebar AI chat (port 11435).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${ALLAMA_PORT:-11435}"
HOST="${ALLAMA_HOST:-127.0.0.1}"
LOG="${ALLAMA_LOG:-/tmp/exodus-allama-11435.log}"
PIDFILE="${ALLAMA_PIDFILE:-/tmp/exodus-allama-11435.pid}"

find_binary() {
  if [[ -n "${ALLAMA_BINARY:-}" && -x "${ALLAMA_BINARY}" ]]; then
    echo "${ALLAMA_BINARY}"
    return 0
  fi
  for candidate in \
    "$ROOT/../Allama/allama/target/release/allama" \
    "$HOME/Allama/allama/target/release/allama" \
    "$ROOT/allama/target/release/allama"; do
    if [[ -x "$candidate" ]]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

BINARY="$(find_binary)" || {
  echo "error: allama not found. Run: sh scripts/build-allama.sh" >&2
  exit 1
}

if [[ -f "$PIDFILE" ]]; then
  OLD_PID="$(cat "$PIDFILE")"
  if kill -0 "$OLD_PID" 2>/dev/null; then
    if curl -sf "http://${HOST}:${PORT}/api/tags" >/dev/null 2>&1; then
      echo "✓ Allama already running (pid $OLD_PID) http://${HOST}:${PORT}"
      exit 0
    fi
    kill "$OLD_PID" 2>/dev/null || true
  fi
fi

# Link Exodus bundle models into ~/.allama/models when present
if [[ -d "$ROOT/allama/models" ]]; then
  export ALLAMA_INFERENCE_MODELS_DIR="$ROOT/allama/models"
fi

echo "▶ Starting Allama: $BINARY"
echo "   http://${HOST}:${PORT}  (log: $LOG)"
nohup "$BINARY" serve --host "$HOST" --port "$PORT" >>"$LOG" 2>&1 &
echo $! >"$PIDFILE"

for ((i = 0; i < 60; i++)); do
  if curl -sf "http://${HOST}:${PORT}/api/tags" >/dev/null 2>&1; then
    echo "✓ Allama ready (pid $(cat "$PIDFILE"))"
    echo ""
    echo "Models:"
    curl -sf "http://${HOST}:${PORT}/api/tags" | python3 -c "
import sys, json
for m in json.load(sys.stdin).get('models', [])[:8]:
    sz = m.get('size', 0)
    mb = sz // 1024 // 1024 if sz else 0
    print(f\"  - {m['name']} ({mb} MB)\")
" 2>/dev/null || true
    echo ""
    echo "Chat test:  sh scripts/test-allama-chat.sh"
    echo "Exodus app: pnpm tauri dev"
    exit 0
  fi
  sleep 1
done

echo "error: Allama did not become ready. Tail log:" >&2
tail -30 "$LOG" >&2 || true
exit 1
