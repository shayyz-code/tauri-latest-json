# tauri-latest-json

[![Crates.io](https://img.shields.io/crates/v/tauri-latest-json.svg)](https://crates.io/crates/tauri-latest-json)
[![docs.rs](https://docs.rs/tauri-latest-json/badge.svg)](https://docs.rs/tauri-latest-json)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/d/tauri-latest-json.svg)](https://crates.io/crates/tauri-latest-json)

Generate a `latest.json` file for [Tauri](https://v2.tauri.app/) auto-updates, supporting multi-platform builds (Windows, macOS Intel/ARM, Linux).

This CLI scans your Tauri `bundle` directory for installers and outputs a valid `latest.json` for the [Tauri Updater](https://v2.tauri.app/plugin/updater/).

## Features

- **Multi-platform detection**: Automatically finds `.msi`, `.exe`, `.dmg` (Intel/ARM), `.AppImage`, `.deb`, `.rpm`, and `.tar.gz` artifacts.
- **Smart platform mapping**: Maps artifacts to their respective Tauri platform keys (`windows-x86_64`, `darwin-aarch64`, etc.).
- **Flexible Versioning**: Reads version from `package.json`, `Cargo.toml`, or `tauri.conf.json` (supports both Tauri 1.0 and 2.0 structures).
- **Root-run friendly**: Can be run from your project root or `src-tauri` directory.
- **Graceful Signature Handling**: Automatically skips artifacts without `.sig` files (like `.dmg` which Tauri doesn't sign for updates) with a helpful warning.
- **Verification Support**: Optional built-in signature verification against your public key.

## Quick Start

### 1. Install

```bash
cargo install tauri-latest-json
```

### 2. Run from your project root

Navigate to your Tauri project root and run:

```bash
tauri-latest-json <download_url_base> <notes...>
```

**Example:**

```bash
tauri-latest-json https://github.com/user/repo/releases/download/v0.3.1 "Improved updater support"
```

This will:

1. Detect your app version from your project files.
2. Locate built artifacts in your `target/` directory.
3. Match installers with their `.sig` files.
4. Generate a `latest.json` in your current directory.

## CLI Commands

```bash
tauri-latest-json help       # Show usage help
tauri-latest-json version    # Show version
```

## Signature Verification (Optional)

To enable compile-time verification of signatures (requires `tauri-cli` installed):

```bash
cargo install tauri-cli
cargo run --features verify-signature -- <download_url_base> <notes>
```

## Platform Detection Logic

The tool prioritizes updater-compatible artifacts:

| Platform          | Priority Artifact | Extension Fallbacks          |
| ----------------- | ----------------- | ---------------------------- |
| **Windows**       | `.msi`            | `.exe`                       |
| **macOS (Intel)** | `.app.tar.gz`     | `.dmg` (skipped for updates) |
| **macOS (ARM)**   | `.app.tar.gz`     | `.dmg` (skipped for updates) |
| **Linux (x64)**   | `.AppImage`       | `.deb`, `.rpm`, `.tar.gz`    |
| **Linux (ARM)**   | `.AppImage`       | `.deb`, `.rpm`, `.tar.gz`    |

> **Note**: Tauri doesn't generate `.sig` files for `.dmg`. This tool will warn you and skip them for the updater JSON, as they aren't used for auto-updates.

## Requirements

- A valid Tauri project with `tauri.conf.json`.
- A configured `updater` plugin with a `pubkey`.
- Built artifacts in `target/release/bundle` or `src-tauri/target/release/bundle`.

## Specs

Behavior specs and acceptance criteria are documented in [SPEC.md](SPEC.md).

## Testing

Run the verification suite:

```bash
make verify
make clippy
```

Optional real-app validation:

```bash
make smoke-real-app
```

By default this looks for a local `real-tauri-app/` directory (gitignored). You can point to another app path with `REAL_APP_DIR=/path/to/your-app`.
The smoke script supports `tauri.conf.json` at either app root or `src-tauri/`, requires `plugins.updater.pubkey`, and bootstraps temporary test artifacts if your real app has no built installers yet.

Manual equivalent:

```bash
cargo test
cargo test --all-features
cargo check --features verify-signature
./scripts/smoke-cli.sh
./scripts/smoke-generate.sh
./scripts/smoke-generate-current-conf.sh
REAL_APP_DIR=/path/to/your-app ./scripts/smoke-real-tauri-app.sh
```

See release notes in [CHANGELOG.md](CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Please also read our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT — see [LICENSE](LICENSE).
