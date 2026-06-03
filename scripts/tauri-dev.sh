#!/usr/bin/env sh
#
# Exodus Browser — start Tauri dev (Bun + Vite on :1421).
# Usage: bun run tauri:dev
#
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"

# shellcheck source=/dev/null
. "$SCRIPT_DIR/env-dev.sh"

sh "$SCRIPT_DIR/free-dev-ports.sh"
cd "$ROOT_DIR"

if ! command -v bun >/dev/null 2>&1; then
  echo "ERROR: bun not found. Install from https://bun.sh" >&2
  exit 1
fi

exec bun run tauri dev "$@"
