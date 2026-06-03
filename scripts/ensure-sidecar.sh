#!/usr/bin/env sh
# Build exodus-core sidecar only when the platform binary is missing.
set -e
cd "$(dirname "$0")/.."
TARGET="$(rustc -vV | sed -n 's/^host: //p')"
OUT="src-tauri/binaries/exodus-core-${TARGET}"
if [ ! -f "${OUT}" ]; then
  echo "[ensure-sidecar] Missing ${OUT}; building..."
  sh scripts/install-sidecar.sh
else
  echo "[ensure-sidecar] Found ${OUT}"
fi
