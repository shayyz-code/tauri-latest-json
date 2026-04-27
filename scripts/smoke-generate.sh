#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/target/release/bundle"

cat >"$TMP_DIR/package.json" <<'JSON'
{"name":"dummy","version":"9.9.9"}
JSON

cat >"$TMP_DIR/tauri.conf.json" <<'JSON'
{"plugins":{"updater":{"pubkey":"test-pubkey"}}}
JSON

echo "windows installer" >"$TMP_DIR/target/release/bundle/app_9.9.9_x64_en-US.msi"
echo "windows-signature" >"$TMP_DIR/target/release/bundle/app_9.9.9_x64_en-US.msi.sig"

echo "Running generator in temp project: $TMP_DIR"
(cd "$TMP_DIR" && cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- https://example.com/downloads "smoke notes" >/dev/null)

if [[ ! -f "$TMP_DIR/latest.json" ]]; then
  echo "latest.json was not generated."
  exit 1
fi

grep -q '"version": "9.9.9"' "$TMP_DIR/latest.json"
grep -q '"windows-x86_64"' "$TMP_DIR/latest.json"
grep -q 'https://example.com/downloads/app_9.9.9_x64_en-US.msi' "$TMP_DIR/latest.json"

echo "Generation smoke test passed."
