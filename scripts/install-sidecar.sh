#!/usr/bin/env sh
# Build exodus-core and copy into src-tauri/binaries/ with the Tauri platform suffix.
set -e
cd "$(dirname "$0")/.."
TARGET="$(rustc -vV | sed -n 's/^host: //p')"
OUT="src-tauri/binaries/exodus-core-${TARGET}"
echo "Building exodus-core for ${TARGET}..."
cargo build --release -p exodus-core
cp "target/release/exodus-core" "${OUT}"
chmod +x "${OUT}"
echo "Installed ${OUT}"
