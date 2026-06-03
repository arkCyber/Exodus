#!/bin/bash
#
# Exodus Browser — kill stale dev ports, ensure sidecar, start Vite dev server.
#
set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"

sh "$SCRIPT_DIR/free-dev-ports.sh"

# Ensure sidecar is running
echo "Ensuring sidecar is running..."
sh "$SCRIPT_DIR/ensure-sidecar.sh"

# Start dev server
echo "Starting dev server..."
cd "$ROOT_DIR"
npm run dev &
DEV_PID=$!

VITE_PORT="${EXODUS_VITE_PORT:-1421}"

# Wait for dev server to start
echo "Waiting for dev server to start..."
for i in $(seq 1 15); do
  sleep 1
  if lsof -i :"$VITE_PORT" > /dev/null 2>&1; then
    echo "Dev server started on port ${VITE_PORT} after ${i} seconds"
    break
  fi
  echo "Waiting for dev server... (${i}/15)"
done

if ! lsof -i :"$VITE_PORT" > /dev/null 2>&1; then
  echo "ERROR: Dev server failed to start on port ${VITE_PORT}"
  exit 1
fi

echo "Dev server is running, keeping it alive..."
wait "$DEV_PID"
