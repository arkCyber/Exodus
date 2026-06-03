#!/usr/bin/env bash
# Exodus — archive broken/duplicate models; add Modelfile + template + params (Ollama-aligned).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODELS="${ALLAMA_MODELS_DIR:-$HOME/.allama/models}"
PRESETS="$ROOT/allama/model-presets"
ARCHIVE="$MODELS/.archive-$(date +%Y%m%d-%H%M%S)"
EXODUS_GGUF="$ROOT/allama/models/gemma-4-E4B/gemma-4-E4B-it-Q4_K_M.gguf"

log() { echo "▶ $*"; }

mkdir -p "$ARCHIVE"

archive_dir() {
  local name="$1"
  local src="$MODELS/$name"
  [[ -d "$src" ]] || return 0
  log "Archive: $name"
  mv "$src" "$ARCHIVE/"
}

# --- 1) Archive stubs and duplicates ---
for name in \
  cli-smoke-model \
  llama3-api-copy \
  llama3-copy \
  llama3-new-copy \
  gemma4-26b-moe-new \
  nonexistent-model \
  test-model \
  interactive; do
  archive_dir "$name"
done

# Stray root Modelfile
if [[ -f "$MODELS/Modelfile" ]]; then
  log "Archive: root Modelfile"
  mv "$MODELS/Modelfile" "$ARCHIVE/Modelfile.root-stray"
fi

# --- 2) Loose GGUF → .incoming-loose ---
INCOMING="$MODELS/.incoming-loose"
mkdir -p "$INCOMING"
for f in "$MODELS"/*.gguf; do
  [[ -f "$f" ]] || continue
  log "Move loose GGUF: $(basename "$f")"
  mv "$f" "$INCOMING/"
done

# --- 3) Archive empty dirs (no metadata, no gguf) ---
for dir in "$MODELS"/*/; do
  name="$(basename "$dir")"
  [[ "$name" == .* ]] && continue
  [[ "$name" == ".incoming-loose" ]] && continue
  if [[ ! -f "${dir}metadata.json" ]] && ! find "$dir" -name '*.gguf' -print -quit 2>/dev/null | grep -q .; then
    log "Archive empty: $name"
    mv "$dir" "$ARCHIVE/"
  fi
done

# --- 4) Fix qwen3.6-35b-a3b GGUF naming ---
QWEN_DIR="$MODELS/qwen3.6-35b-a3b"
if [[ -d "$QWEN_DIR" && -f "$QWEN_DIR/metadata.json" ]]; then
  if [[ ! -f "$QWEN_DIR/qwen3.6-35b-a3b.gguf" ]]; then
    src="$(find "$QWEN_DIR" -maxdepth 1 -name '*.gguf' | head -1)"
    if [[ -n "$src" && -f "$src" ]]; then
      log "Link GGUF for qwen3.6-35b-a3b"
      ln -sf "$(basename "$src")" "$QWEN_DIR/qwen3.6-35b-a3b.gguf"
    fi
  fi
fi

# --- 5) Enrich primary models (Modelfile + template + system + parameters) ---
enrich_model() {
  local name="$1"
  local preset="$2"
  local dir="$MODELS/$name"
  [[ -d "$dir" ]] || return 0
  log "Enrich: $name"
  cp "$PRESETS/Modelfile.$preset" "$dir/Modelfile"
  cp "$PRESETS/gemma-turns.template.jinja" "$dir/template.jinja"
  cp "$PRESETS/gemma.system.txt" "$dir/system.txt"
  cp "$PRESETS/gemma.parameters.json" "$dir/parameters.json"
}

enrich_model gemma4-e2b gemma4-e2b
enrich_model gemma4-4b gemma4-4b
enrich_model gemma4-26b-moe gemma4-26b-moe
if [[ -d "$MODELS/gemma4-31b" ]]; then
  enrich_model gemma4-31b gemma4-4b
fi
if [[ -d "$QWEN_DIR" ]]; then
  enrich_model qwen3.6-35b-a3b gemma4-e2b
fi

# --- 6) Register Exodus bundle as gemma-4-e4b ---
E4B_DIR="$MODELS/gemma-4-e4b"
if [[ -f "$EXODUS_GGUF" ]]; then
  log "Register Exodus bundle → gemma-4-e4b"
  mkdir -p "$E4B_DIR"
  if [[ ! -e "$E4B_DIR/gemma-4-e4b.gguf" ]]; then
    ln -sf "$EXODUS_GGUF" "$E4B_DIR/gemma-4-e4b.gguf"
  fi
  cp "$PRESETS/Modelfile.gemma-4-e4b" "$E4B_DIR/Modelfile"
  cp "$PRESETS/gemma-turns.template.jinja" "$E4B_DIR/template.jinja"
  cp "$PRESETS/gemma.system.txt" "$E4B_DIR/system.txt"
  cp "$PRESETS/gemma.parameters.json" "$E4B_DIR/parameters.json"
  SZ=$(stat -f%z "$EXODUS_GGUF" 2>/dev/null || stat -c%s "$EXODUS_GGUF")
  cat >"$E4B_DIR/metadata.json" <<EOF
{
  "name": "gemma-4-e4b",
  "tag": "latest",
  "size": $SZ,
  "modified": "$(date -u +%Y-%m-%dT%H:%M:%S+00:00)",
  "digest": "sha256:gemma-4-e4b",
  "details": {
    "architecture": "gemma4",
    "parameters": "4B",
    "context_length": 8192,
    "quantization": "Q4_K_M"
  }
}
EOF
fi

# --- 7) Refresh metadata sizes for enriched dirs ---
for dir in "$MODELS"/*/; do
  name="$(basename "$dir")"
  [[ "$name" == .* ]] && continue
  meta="${dir}metadata.json"
  gguf="${dir}/${name}.gguf"
  if [[ ! -f "$gguf" ]]; then
    gguf="$(find "$dir" -maxdepth 1 -name '*.gguf' ! -type l | head -1)"
  fi
  [[ -f "$meta" && -f "$gguf" ]] || continue
  SZ=$(python3 -c "import pathlib; print(pathlib.Path('$gguf').resolve().stat().st_size)")
  python3 -c "
import json, pathlib, sys
p = pathlib.Path(sys.argv[1])
m = json.loads(p.read_text())
m['size'] = int(sys.argv[2])
m['name'] = sys.argv[3]
p.write_text(json.dumps(m, indent=2) + '\n')
" "$meta" "$SZ" "$name"
done

echo ""
echo "✓ Cleanup complete"
echo "  Archive: $ARCHIVE"
echo "  Loose weights: $INCOMING"
echo ""
echo "Recommended Exodus Chat model names:"
echo "  gemma4-e2b  |  gemma4-4b  |  gemma-4-e4b  |  gemma4-26b-moe"
echo ""
echo "Restart Allama: kill \$(cat /tmp/exodus-allama-11435.pid 2>/dev/null) 2>/dev/null; sh scripts/start-allama-chat.sh"
sh "$ROOT/scripts/check-allama-models.sh"
