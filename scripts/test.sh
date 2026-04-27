#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

echo "[1/3] cargo test"
cargo test

echo "[2/3] cargo test --all-features"
cargo test --all-features

echo "[3/3] cargo check --features verify-signature"
cargo check --features verify-signature

echo "All verification checks passed."
