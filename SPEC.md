# tauri-latest-json Specification

This document defines the expected behavior of the CLI.

## Scope

- Generate a Tauri-compatible `latest.json` from built installer artifacts.
- Support multi-platform output in one file.
- Work as a standalone CLI binary.

## Functional Requirements

1. Version resolution

- Prefer `package.json` `version` when present and valid.
- Fallback to `Cargo.toml` `[package].version` when `package.json` is absent.
- Return an error when neither source yields a version.

2. Installer detection

- Detect installers recursively under bundle directory for:
  - `.msi`, `.exe`, `.dmg`, `.AppImage`, `.deb`, `.rpm`, `.tar.gz`
- Return an error when no installers are found.

3. Platform mapping

- Map `.msi` and `.exe` to `windows-x86_64`.
- Map `.dmg` to:
  - `darwin-aarch64` when filename includes `aarch64` or `arm64`
  - `darwin-x86_64` otherwise
- Map `.AppImage`, `.deb`, `.rpm`, `.tar.gz` to:
  - `linux-aarch64` when filename includes `aarch64` or `arm64`
  - `linux-x86_64` otherwise

4. Signature behavior

- Require a matching `.sig` for each detected installer platform.
- Return an error when a required platform signature is missing.

5. Output structure

- Write `latest.json` to the project current working directory.
- Include keys:
  - `version` (string)
  - `notes` (string)
  - `pub_date` (RFC3339 UTC, seconds precision)
  - `platforms` (object keyed by platform)
- For each platform, include:
  - `signature`
  - `url` using `<download_url_base>/<installer_filename>`

6. Auto mode behavior

- The default generate command must:
  - Detect bundle dir from known candidates.
  - Detect `tauri.conf.json` from known candidates.
  - Read updater public key from `plugins.updater.pubkey`.

7. Command behavior

- `help`, `-h`, and `--help` print CLI usage.
- `version`, `-V`, and `--version` print binary name and package version.
- Calling the CLI without valid generate args exits with non-zero status and prints help.

## Verification Matrix

- Unit tests in `src/bin/tauri-latest-json.rs` verify:
  - platform mapping variants
  - version precedence and fallback
  - generated JSON shape and values
  - errors on missing installers/signatures
  - generate behavior with current working directory
  - auto path detection and config usage
  - help/version argument parsing

## How To Validate

```bash
cargo test
cargo test --all-features
cargo check --features verify-signature
```
