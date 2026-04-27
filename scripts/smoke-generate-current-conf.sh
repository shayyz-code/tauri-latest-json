#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/target/release/bundle"

cat >"$TMP_DIR/package.json" <<'JSON'
{"name":"dummy","version":"1.4.0"}
JSON

if [[ -f "$ROOT_DIR/tauri.conf.json" ]]; then
  cp "$ROOT_DIR/tauri.conf.json" "$TMP_DIR/tauri.conf.json"
  echo "Using repository tauri.conf.json: $ROOT_DIR/tauri.conf.json"
else
  cat >"$TMP_DIR/tauri.conf.json" <<'JSON'
{
  "plugins": {
    "updater": {
      "pubkey": "test-pubkey"
    }
  }
}
JSON
  echo "Repository tauri.conf.json not found; using generated fallback config."
fi

echo "windows installer" >"$TMP_DIR/target/release/bundle/app_1.4.0_x64_en-US.msi"
echo "windows-signature" >"$TMP_DIR/target/release/bundle/app_1.4.0_x64_en-US.msi.sig"

echo "Running generator smoke check"
(cd "$TMP_DIR" && cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- https://example.com/downloads "current-conf smoke notes" >/dev/null)

if [[ ! -f "$TMP_DIR/latest.json" ]]; then
  echo "latest.json was not generated."
  exit 1
fi

grep -q '"version": "1.4.0"' "$TMP_DIR/latest.json"
grep -q '"windows-x86_64"' "$TMP_DIR/latest.json"
grep -q 'https://example.com/downloads/app_1.4.0_x64_en-US.msi' "$TMP_DIR/latest.json"

echo "Generation smoke test with current/fallback tauri.conf.json passed."
