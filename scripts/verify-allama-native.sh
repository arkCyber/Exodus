#!/usr/bin/env bash
# Exodus Browser — verify native Allama HTTP inference (build optional).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${ALLAMA_VERIFY_PORT:-11436}"
HOST="127.0.0.1"
MODEL="${ALLAMA_VERIFY_MODEL:-gemma4-4b}"
EXODUS_MODELS="$ROOT/allama/models"
TIMEOUT_SEC="${ALLAMA_VERIFY_TIMEOUT:-120}"

find_binary() {
  if [[ -n "${ALLAMA_BINARY:-}" && -x "${ALLAMA_BINARY}" ]]; then
    echo "${ALLAMA_BINARY}"
    return 0
  fi
  for candidate in \
    "$ROOT/../Allama/allama/target/release/allama" \
    "$HOME/Allama/allama/target/release/allama" \
    "$ROOT/allama/target/release/allama" \
    "$ROOT/allama/build/bin/allama"; do
    if [[ -x "$candidate" ]]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

BINARY="$(find_binary)" || {
  echo "error: allama binary not found. Run: sh scripts/build-allama.sh" >&2
  exit 1
}

echo "▶ Using binary: $BINARY"
echo "▶ Port: $PORT  Model: $MODEL"

# Link Exodus bundle models into ~/.allama/models when useful
if [[ -d "$EXODUS_MODELS" ]]; then
  export ALLAMA_INFERENCE_MODELS_DIR="$EXODUS_MODELS"
fi

cleanup() {
  if [[ -n "${SRV_PID:-}" ]]; then
    kill "$SRV_PID" 2>/dev/null || true
    wait "$SRV_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

"$BINARY" serve --host "$HOST" --port "$PORT" >/tmp/exodus-allama-verify.log 2>&1 &
SRV_PID=$!

echo "▶ Waiting for /api/tags (max ${TIMEOUT_SEC}s)..."
for ((i = 0; i < TIMEOUT_SEC; i++)); do
  if curl -sf "http://${HOST}:${PORT}/api/tags" >/tmp/exodus-allama-tags.json 2>/dev/null; then
    break
  fi
  if ! kill -0 "$SRV_PID" 2>/dev/null; then
    echo "error: allama serve exited early. Log:" >&2
    tail -40 /tmp/exodus-allama-verify.log >&2 || true
    exit 1
  fi
  sleep 1
done

if [[ ! -s /tmp/exodus-allama-tags.json ]]; then
  echo "error: /api/tags did not respond" >&2
  tail -40 /tmp/exodus-allama-verify.log >&2 || true
  exit 1
fi

echo "✓ /api/tags OK"
if command -v jq >/dev/null 2>&1; then
  jq -r '.models[].name' /tmp/exodus-allama-tags.json | head -10
fi

if ! grep -q "\"name\".*\"$MODEL\"" /tmp/exodus-allama-tags.json 2>/dev/null; then
  FIRST="$(grep -o '"name":"[^"]*"' /tmp/exodus-allama-tags.json | head -1 | sed 's/"name":"//;s/"//')"
  if [[ -n "$FIRST" ]]; then
    echo "▶ Model '$MODEL' not listed; using '$FIRST'"
    MODEL="$FIRST"
  fi
fi

echo "▶ POST /api/generate (stream=false, may take a while on first load)..."
GEN_BODY=$(cat <<EOF
{"model":"$MODEL","prompt":"Say hello in one short sentence.","stream":false}
EOF
)

HTTP_CODE=$(curl -sf -o /tmp/exodus-allama-gen.json -w "%{http_code}" \
  -H "Content-Type: application/json" \
  -d "$GEN_BODY" \
  "http://${HOST}:${PORT}/api/generate" || echo "000")

if [[ "$HTTP_CODE" != "200" ]]; then
  echo "error: /api/generate HTTP $HTTP_CODE" >&2
  cat /tmp/exodus-allama-gen.json >&2 || true
  exit 1
fi

if grep -q '"response"' /tmp/exodus-allama-gen.json; then
  echo "✓ /api/generate OK"
  if command -v jq >/dev/null 2>&1; then
    jq -r '.response' /tmp/exodus-allama-gen.json | head -3
  else
    head -c 200 /tmp/exodus-allama-gen.json
    echo ""
  fi
else
  echo "error: unexpected generate response:" >&2
  cat /tmp/exodus-allama-gen.json >&2
  exit 1
fi

echo ""
echo "✓ Native Allama inference verified on http://${HOST}:${PORT}"
