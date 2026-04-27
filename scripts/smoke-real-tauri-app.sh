#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REAL_APP_DIR="${REAL_APP_DIR:-$ROOT_DIR/real-tauri-app}"
NOTES="real app smoke notes"
DOWNLOAD_BASE="https://example.com/downloads"

if [[ ! -d "$REAL_APP_DIR" ]]; then
  echo "Missing real app directory: $REAL_APP_DIR"
  echo "Place your real Tauri app there, or set REAL_APP_DIR to a different path."
  exit 1
fi

if [[ -f "$REAL_APP_DIR/tauri.conf.json" ]]; then
  TAURI_CONF_PATH="$REAL_APP_DIR/tauri.conf.json"
elif [[ -f "$REAL_APP_DIR/src-tauri/tauri.conf.json" ]]; then
  TAURI_CONF_PATH="$REAL_APP_DIR/src-tauri/tauri.conf.json"
else
  echo "Missing tauri.conf.json in real app (checked root and src-tauri)."
  exit 1
fi

if ! grep -q '"updater"' "$TAURI_CONF_PATH" || ! grep -q '"pubkey"' "$TAURI_CONF_PATH"; then
  echo "Missing plugins.updater.pubkey in $TAURI_CONF_PATH"
  echo "Add updater config so generated latest.json includes signatures correctly."
  exit 1
fi

TMP_FILES=()
cleanup() {
  for f in "${TMP_FILES[@]}"; do
    rm -f "$f"
  done
}
trap cleanup EXIT

if [[ -f "$REAL_APP_DIR/package.json" ]]; then
  VERSION="$(grep -Eo '"version"\s*:\s*"[^"]+"' "$REAL_APP_DIR/package.json" | head -n1 | sed -E 's/.*"([^"]+)"/\1/')"
elif [[ -f "$REAL_APP_DIR/Cargo.toml" ]]; then
  VERSION="$(grep -E '^\s*version\s*=\s*".+"' "$REAL_APP_DIR/Cargo.toml" | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')"
elif [[ -f "$REAL_APP_DIR/src-tauri/Cargo.toml" ]]; then
  VERSION="$(grep -E '^\s*version\s*=\s*".+"' "$REAL_APP_DIR/src-tauri/Cargo.toml" | head -n1 | sed -E 's/.*"([^"]+)".*/\1/')"
  cat >"$REAL_APP_DIR/package.json" <<JSON
{"name":"real-tauri-app","version":"$VERSION"}
JSON
  TMP_FILES+=("$REAL_APP_DIR/package.json")
else
  echo "Missing version source in real app (need package.json, Cargo.toml, or src-tauri/Cargo.toml)."
  exit 1
fi

if [[ -z "${VERSION:-}" ]]; then
  echo "Could not resolve app version from real app files."
  exit 1
fi

BUNDLE_DIR="$REAL_APP_DIR/target/release/bundle"
if [[ ! -d "$BUNDLE_DIR" && -d "$REAL_APP_DIR/src-tauri" ]]; then
  BUNDLE_DIR="$REAL_APP_DIR/src-tauri/target/release/bundle"
fi
mkdir -p "$BUNDLE_DIR"

for arch in x64 arm64; do
  TEST_INSTALLER="$BUNDLE_DIR/app_${VERSION}_${arch}.app.tar.gz"
  TEST_SIGNATURE="${TEST_INSTALLER}.sig"
  if [[ ! -f "$TEST_INSTALLER" ]]; then
    echo "mac updater archive (${arch})" >"$TEST_INSTALLER"
    TMP_FILES+=("$TEST_INSTALLER")
  fi
  if [[ ! -f "$TEST_SIGNATURE" ]]; then
    echo "test-signature-${arch}" >"$TEST_SIGNATURE"
    TMP_FILES+=("$TEST_SIGNATURE")
  fi
done

echo "Running generator against real app: $REAL_APP_DIR"
(cd "$REAL_APP_DIR" && cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- "$DOWNLOAD_BASE" "$NOTES" >/dev/null)

if [[ ! -f "$REAL_APP_DIR/latest.json" ]]; then
  echo "latest.json was not generated in real app directory."
  exit 1
fi

grep -q '"platforms"' "$REAL_APP_DIR/latest.json"
grep -q "\"notes\": \"$NOTES\"" "$REAL_APP_DIR/latest.json"

echo "Real app smoke test passed."
