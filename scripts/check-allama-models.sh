#!/usr/bin/env bash
# Exodus — audit ~/.allama/models layout vs Ollama/Allama rules.
set -euo pipefail

MODELS_DIR="${ALLAMA_MODELS_DIR:-$HOME/.allama/models}"
PORT="${ALLAMA_PORT:-11435}"

echo "=== Allama model audit ==="
echo "Models dir: $MODELS_DIR"
echo ""

if [[ ! -d "$MODELS_DIR" ]]; then
  echo "error: directory does not exist" >&2
  exit 1
fi

echo "--- Valid (metadata.json + GGUF) ---"
valid=0
for meta in "$MODELS_DIR"/*/metadata.json; do
  [[ -f "$meta" ]] || continue
  dir="$(dirname "$meta")"
  name="$(basename "$dir")"
  [[ "$name" == .* ]] && continue
  gguf_exact="${dir}/${name}.gguf"
  gguf_any="$(find "$dir" -maxdepth 1 -name '*.gguf' 2>/dev/null | head -1)"
  if [[ -f "$gguf_exact" ]]; then
    sz=$(du -sh "$gguf_exact" 2>/dev/null | awk '{print $1}')
    echo "  OK  $name  ($sz, standard path)"
    valid=$((valid + 1))
  elif [[ -n "$gguf_any" && -f "$gguf_any" ]]; then
    sz=$(du -sh "$gguf_any" 2>/dev/null | awk '{print $1}')
    echo "  OK* $name  ($sz, non-standard: $(basename "$gguf_any"))"
    valid=$((valid + 1))
  else
    echo "  BAD $name  (metadata.json but NO .gguf)"
  fi
done

echo ""
echo "--- Listed in API but worth checking ---"
if curl -sf "http://127.0.0.1:${PORT}/api/tags" >/tmp/exodus-audit-tags.json 2>/dev/null; then
  python3 <<'PY'
import json
with open("/tmp/exodus-audit-tags.json") as f:
    data = json.load(f)
for m in data.get("models", []):
    name = m["name"]
    sz = m.get("size") or 0
    flag = "STUB?" if sz < 1_000_000 else "OK"
    print(f"  {flag:5} {name:25} api_size={sz//1024//1024}MB")
PY
else
  echo "  (Allama not running on port $PORT — start with: sh scripts/start-allama-chat.sh)"
fi

echo ""
echo "--- Dirs missing metadata.json (invisible to /api/tags) ---"
missing=0
for dir in "$MODELS_DIR"/*/; do
  name="$(basename "$dir")"
  [[ "$name" == .* ]] && continue
  [[ -f "${dir}metadata.json" ]] && continue
  has_gguf="$(find "$dir" -maxdepth 2 -name '*.gguf' -print -quit 2>/dev/null || true)"
  if [[ -n "$has_gguf" ]]; then
    echo "  HAS_GGUF  $name  → run: allama create <name> -f Modelfile"
  else
    echo "  empty?    $name"
  fi
  missing=$((missing + 1))
done
echo "  ($missing directories skipped by API)"

echo ""
echo "--- Loose .gguf in models root (not registered) ---"
loose=0
for f in "$MODELS_DIR"/*.gguf; do
  [[ -f "$f" ]] || continue
  echo "  $f"
  loose=$((loose + 1))
done
[[ "$loose" -eq 0 ]] && echo "  (none)"

echo ""
echo "Summary: $valid models with metadata+weights; use API names in Exodus Chat model field."
echo "Ollama alignment: see docs/ALLAMA_MODELS.md"
