#!/usr/bin/env sh
# Exodus Browser — production frontend build for Tauri (Bun).
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
# shellcheck source=/dev/null
. "$SCRIPT_DIR/env-dev.sh"
cd "$SCRIPT_DIR/.."
exec bun run build
