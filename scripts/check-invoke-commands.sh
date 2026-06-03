#!/usr/bin/env bash
# Exodus Browser — ensure frontend invoke() names match Tauri generate_handler! list.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LIB_RS="$ROOT/src-tauri/src/lib.rs"
TMP_DIR="${TMPDIR:-/tmp}/exodus-invoke-check-$$"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -f "$LIB_RS" ]]; then
  echo "Missing $LIB_RS" >&2
  exit 1
fi

# Commands registered in invoke_handler (last segment after ::)
awk '/invoke_handler\(tauri::generate_handler!\[/,/\]\)/' "$LIB_RS" \
  | tr ',' '\n' \
  | sed -E 's/.*::([a-z_0-9]+).*/\1/; s/^[[:space:]]+//; s/[[:space:]]+$//' \
  | grep -E '^[a-z0-9_]+$' \
  | grep -v '^generate_handler$' \
  | sort -u > "$TMP_DIR/handler.txt"

# invoke('cmd') including multiline TypeScript generics (slurp per file)
find "$ROOT/src" \( -name '*.ts' -o -name '*.svelte' \) -print0 \
  | xargs -0 perl -0777 -ne '
    while (/invoke\s*(?:<[^>]*>)?\s*\(\s*[\x27\x22]([a-z0-9_]+)[\x27\x22]/gs) { print "$1\n" }
  ' 2>/dev/null \
  | sort -u > "$TMP_DIR/frontend.txt"

missing=0
while IFS= read -r cmd; do
  [[ -z "$cmd" ]] && continue
  if ! grep -qx "$cmd" "$TMP_DIR/handler.txt"; then
    echo "Frontend invoke missing from invoke_handler: $cmd" >&2
    missing=1
  fi
done < "$TMP_DIR/frontend.txt"

while IFS= read -r cmd; do
  [[ -z "$cmd" ]] && continue
  if ! grep -qx "$cmd" "$TMP_DIR/frontend.txt"; then
    echo "Note: handler not referenced in src/: $cmd" >&2
  fi
done < "$TMP_DIR/handler.txt"

if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

FE_COUNT=$(wc -l < "$TMP_DIR/frontend.txt" | tr -d ' ')
HD_COUNT=$(wc -l < "$TMP_DIR/handler.txt" | tr -d ' ')
echo "invoke command check OK (${FE_COUNT} frontend, ${HD_COUNT} registered)"
