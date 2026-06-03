#!/usr/bin/env bash
# Exodus — quick real chat test against Allama on port 11435.
set -euo pipefail

PORT="${ALLAMA_PORT:-11435}"
HOST="${ALLAMA_HOST:-127.0.0.1}"
MODEL="${ALLAMA_CHAT_MODEL:-gemma4-e2b}"
BASE="http://${HOST}:${PORT}"

if ! curl -sf "${BASE}/api/tags" >/tmp/exodus-chat-tags.json; then
  echo "error: Allama offline. Run: sh scripts/start-allama-chat.sh" >&2
  exit 1
fi

if ! grep -q "\"name\".*\"${MODEL}\"" /tmp/exodus-chat-tags.json 2>/dev/null; then
  MODEL="$(python3 -c "
import json
models=[m['name'] for m in json.load(open('/tmp/exodus-chat-tags.json')).get('models',[])]
# prefer small real models
for pref in ('gemma4-e2b','gemma4-4b','qwen','llama'):
  for n in models:
    if pref in n.lower() and 'smoke' not in n:
      print(n); raise SystemExit
print(models[0] if models else 'gemma4-e2b')
" 2>/dev/null || echo gemma4-e2b)"
  echo "▶ Using model: $MODEL"
fi

echo "▶ Streaming chat (Exodus sidebar uses /v1/chat/completions)..."
PROMPT="${1:-用一句话说你好，并说明你是本地 AI 助手。}"
STREAM_FILE="$(mktemp /tmp/exodus-chat-stream.XXXXXX)"
BODY_FILE="$(mktemp /tmp/exodus-chat-body.XXXXXX)"
python3 -c "
import json, sys
prompt = sys.argv[1]
body = {
  'model': sys.argv[2],
  'messages': [
    {'role': 'system', 'content': 'You are Exodus, a helpful local AI assistant. Reply concisely.'},
    {'role': 'user', 'content': prompt},
  ],
  'stream': True,
}
open(sys.argv[3], 'w').write(json.dumps(body))
" "$PROMPT" "$MODEL" "$BODY_FILE"

curl -sS --max-time 300 -N -H 'Content-Type: application/json' \
  -d @"$BODY_FILE" \
  "${BASE}/v1/chat/completions" >"$STREAM_FILE" || {
  echo "error: stream request failed" >&2
  rm -f "$STREAM_FILE" "$BODY_FILE"
  exit 1
}

FULL="$(python3 - "$STREAM_FILE" <<'PY'
import json, sys
path = sys.argv[1]
out = []
for line in open(path):
    line = line.strip()
    if not line.startswith("data: "):
        continue
    data = line[6:].strip()
    if data == "[DONE]":
        break
    try:
        d = json.loads(data)
        c = d.get("choices", [{}])[0].get("delta", {}).get("content", "")
        if c:
            out.append(c)
    except json.JSONDecodeError:
        pass
print("".join(out))
PY
)"
printf '%s' "$FULL"
rm -f "$STREAM_FILE" "$BODY_FILE"

echo ""
echo ""
if [[ -n "$FULL" ]]; then
  echo "✓ Real inference OK (${#FULL} chars)"
else
  echo "error: empty stream" >&2
  exit 1
fi
