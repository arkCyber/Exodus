#!/usr/bin/env sh
#
# Exodus Browser — tail startup.log from app_data (macOS default path).
# Usage:
#   sh scripts/show-startup-log.sh [lines]
#   sh scripts/show-startup-log.sh 80 --geom
#   EXODUS_LOG_GEOM=1 sh scripts/show-startup-log.sh
#
set -eu

LINES=80
GEOM_ONLY=0

for arg in "$@"; do
  case "$arg" in
    --geom)
      GEOM_ONLY=1
      ;;
    [0-9]*)
      LINES="$arg"
      ;;
    *)
      echo "Unknown argument: $arg (use: [lines] [--geom])" >&2
      exit 1
      ;;
  esac
done

LOG_DIR="${EXODUS_LOG_DIR:-$HOME/Library/Application Support/com.exodus.browser/logs}"
LOG_FILE="$LOG_DIR/startup.log"

if [ ! -f "$LOG_FILE" ]; then
  echo "startup.log not found at: $LOG_FILE" >&2
  echo "Start the app once with: bun run tauri:dev" >&2
  exit 1
fi

if [ "$GEOM_ONLY" = "1" ]; then
  echo "=== [WINDOW_GEOM] (last ${LINES} lines) ==="
  echo "Path: $LOG_FILE"
  echo "---"
  grep "\[WINDOW_GEOM\]" "$LOG_FILE" | tail -n "$LINES" || true
  exit 0
fi

echo "=== startup.log (last ${LINES} lines) ==="
echo "Path: $LOG_FILE"
echo "---"
tail -n "$LINES" "$LOG_FILE"

echo "---"
echo "=== [WINDOW_GEOM] (last ${LINES} lines) ==="
grep "\[WINDOW_GEOM\]" "$LOG_FILE" | tail -n "$LINES" || true
