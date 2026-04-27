#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/5] --help"
cargo run -- --help >/dev/null

echo "[2/5] help"
cargo run -- help >/dev/null

echo "[3/5] --version"
cargo run -- --version >/dev/null

echo "[4/5] version"
cargo run -- version >/dev/null

echo "[5/5] invalid args should fail"
if cargo run -- only-download-url >/dev/null 2>&1; then
  echo "Expected failure for invalid args, but command succeeded."
  exit 1
fi

echo "CLI smoke checks passed."
