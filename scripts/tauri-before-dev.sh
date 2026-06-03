#!/usr/bin/env sh
#
# Exodus Browser — Tauri beforeDevCommand hook (starts Vite; Tauri waits for devUrl).
#
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"

# shellcheck source=/dev/null
. "$SCRIPT_DIR/env-dev.sh"

cd "$ROOT_DIR"

if ! command -v bun >/dev/null 2>&1; then
  echo "ERROR: bun not found. Install from https://bun.sh and ensure ~/.bun/bin is on PATH." >&2
  exit 1
fi

exec bun run dev
