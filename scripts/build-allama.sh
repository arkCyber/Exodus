#!/usr/bin/env bash
# Exodus Browser — build native Allama binary (Rust cargo preferred, CMake fallback).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

build_cargo_allama() {
  local dir="$1"
  if [[ ! -f "$dir/Cargo.toml" ]]; then
    return 1
  fi
  echo "▶ cargo build --release in $dir"
  (cd "$dir" && cargo build --release)
  if [[ -x "$dir/target/release/allama" ]]; then
    echo ""
    echo "✓ Allama binary (Rust): $dir/target/release/allama"
    echo "  export ALLAMA_BINARY=$dir/target/release/allama"
    return 0
  fi
  return 1
}

# 1) Sibling Allama repo (common layout: ~/Allama/allama next to ~/Exodus)
for dir in \
  "$ROOT/../Allama/allama" \
  "$HOME/Allama/allama" \
  "$ROOT/allama"; do
  if build_cargo_allama "$dir"; then
    exit 0
  fi
done

# 2) CMake llama.cpp tree under Exodus/allama (requires ggml submodule)
ALLAMA_DIR="$ROOT/allama"
BUILD_DIR="$ALLAMA_DIR/build"

if [[ ! -f "$ALLAMA_DIR/CMakeLists.txt" ]]; then
  echo "error: no Cargo.toml (Rust allama) and no $ALLAMA_DIR/CMakeLists.txt" >&2
  echo "hint: clone https://github.com/arkCyber/allama next to Exodus as ../Allama/allama" >&2
  exit 1
fi

if [[ ! -d "$ALLAMA_DIR/ggml" ]]; then
  echo "error: $ALLAMA_DIR/ggml missing — CMake build needs llama.cpp ggml submodule" >&2
  echo "hint: use Rust allama: cd ../Allama/allama && cargo build --release" >&2
  exit 1
fi

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "▶ cmake configure..."
cmake ..

echo "▶ cmake build..."
cmake --build . -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"

for candidate in \
  "$BUILD_DIR/bin/allama" \
  "$BUILD_DIR/bin/llama-server" \
  "$ALLAMA_DIR/target/release/allama"; do
  if [[ -x "$candidate" ]]; then
    echo ""
    echo "✓ Allama binary: $candidate"
    echo "  export ALLAMA_BINARY=$candidate"
    exit 0
  fi
done

echo "warning: build finished but no allama binary found" >&2
exit 1
