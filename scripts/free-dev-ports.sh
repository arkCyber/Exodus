#!/usr/bin/env sh
#
# Exodus Browser — free dev TCP ports before starting Vite / Tauri.
# Stops stale exodus-tauri + Vite/HMR listeners, then waits until ports are free.
#
# Usage:
#   sh scripts/free-dev-ports.sh
#   EXODUS_DEV_PORTS="1421 24678 8790" sh scripts/free-dev-ports.sh
#
set -eu

VITE_PORT="${EXODUS_VITE_PORT:-1421}"
HMR_PORT="${EXODUS_HMR_PORT:-24678}"
MAX_WAIT="${EXODUS_PORT_WAIT_SECS:-10}"

# Space-separated list; defaults to Vite + HMR. Set EXODUS_DEV_PORTS to override entirely.
if [ -n "${EXODUS_DEV_PORTS:-}" ]; then
  PORT_LIST="$EXODUS_DEV_PORTS"
else
  PORT_LIST="$VITE_PORT $HMR_PORT"
fi

# Stop leftover Exodus dev binaries so ports and WebView hosts are not orphaned.
stop_stale_exodus() {
  if pgrep -f "/target/debug/exodus-tauri" >/dev/null 2>&1; then
    echo "[free-dev-ports] stopping stale exodus-tauri (debug) processes..."
    pkill -f "/target/debug/exodus-tauri" 2>/dev/null || true
    sleep 1
  fi
}

# Kill any process listening on the given TCP port and wait until it is released.
free_port() {
  port="$1"

  if lsof -ti :"$port" >/dev/null 2>&1; then
    echo "[free-dev-ports] port ${port} in use — killing listeners..."
    lsof -ti :"$port" | xargs kill -9 2>/dev/null || true
  else
    echo "[free-dev-ports] port ${port} is free"
    return 0
  fi

  waited=0
  while lsof -ti :"$port" >/dev/null 2>&1; do
    if [ "$waited" -ge "$MAX_WAIT" ]; then
      echo "[free-dev-ports] ERROR: port ${port} still in use after ${MAX_WAIT}s" >&2
      lsof -i :"$port" >&2 || true
      exit 1
    fi
    sleep 1
    waited=$((waited + 1))
  done

  echo "[free-dev-ports] port ${port} is ready"
}

stop_stale_exodus
echo "[free-dev-ports] checking ports: ${PORT_LIST}"
for port in $PORT_LIST; do
  free_port "$port"
done
echo "[free-dev-ports] all dev ports ready"
